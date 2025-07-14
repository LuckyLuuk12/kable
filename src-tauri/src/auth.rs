use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;
use std::fs;
use crate::AppError;
use std::collections::HashMap;
use std::env;
use serde_json::json;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

// PKCE state storage
static PKCE_VERIFIER: once_cell::sync::Lazy<std::sync::Mutex<Option<String>>> = 
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(None));

// Load environment variables
fn get_client_id() -> String {
    env::var("AZURE_CLIENT_ID").expect("AZURE_CLIENT_ID must be set in .env file")
}

fn get_redirect_uri() -> String {
    env::var("AZURE_REDIRECT_URI").expect("AZURE_REDIRECT_URI must be set in .env file")
}

fn get_oauth_port() -> u16 {
    env::var("OAUTH_PORT")
        .unwrap_or_else(|_| "43110".to_string())
        .parse()
        .unwrap_or(43110)
}

// Enhanced Microsoft OAuth2 structures based on the gist
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MicrosoftAccount {
    pub id: String,
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub skin_url: Option<String>,
    pub is_active: bool,
    pub last_used: i64,
    pub minecraft_access_token: Option<String>, // Minecraft-specific token (optional for non-partner apps)
    pub minecraft_expires_at: Option<i64>,
    pub xbox_user_hash: String,
}

// Simplified response structures matching the working example
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizationTokenResponse {
    pub token_type: String,
    pub scope: String,
    pub expires_in: u32,
    pub ext_expires_in: u32,
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: Option<String>, // Optional for device code flow
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XboxLiveAuthenticationResponse {
    pub issue_instant: String,
    pub not_after: String,
    pub token: String,
    pub display_claims: HashMap<String, Vec<HashMap<String, String>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftAuthenticationResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,
    pub username: Option<String>, // Optional field
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftProfileResponse {
    pub id: String,
    pub name: String,
}

// Minecraft launcher session data structures
#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftSession {
    pub access_token: String,
    pub client_token: String,
    pub uuid: String,
    pub username: String,
    pub user_type: String,
    pub user_properties: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LauncherProfiles {
    pub authentication_database: HashMap<String, MinecraftSession>,
    pub launcher_version: String,
    pub selected_user: Option<String>,
}

// Using Azure Application credentials from environment
// Set AZURE_CLIENT_ID and AZURE_REDIRECT_URI in .env file

// Global state for OAuth callback
static OAUTH_CALLBACK_RESULT: once_cell::sync::Lazy<Arc<Mutex<Option<Result<String, String>>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

#[tauri::command]
pub async fn start_microsoft_auth() -> Result<String, String> {
    // Clear any previous callback result
    *OAUTH_CALLBACK_RESULT.lock().await = None;
    
    // Generate PKCE verifier and challenge
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);
    
    // Store the verifier for later use
    {
        let mut verifier_guard = PKCE_VERIFIER.lock().unwrap();
        *verifier_guard = Some(code_verifier);
    }
    
    // Start the callback server
    start_oauth_callback_server().await;
    
    let client_id = get_client_id();
    let redirect_uri = get_redirect_uri();
    
    // Generate OAuth URL with PKCE parameters for public client
    let auth_url = format!(
        "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256&prompt=select_account",
        client_id,
        urlencoding::encode(&redirect_uri),
        urlencoding::encode("XboxLive.signin offline_access"),
        urlencoding::encode(&code_challenge)
    );
    
    println!("Generated auth URL with PKCE: {}", auth_url);
    println!("Code challenge: {}", code_challenge);
    Ok(auth_url)
}

async fn start_oauth_callback_server() {
    let callback_result = Arc::clone(&OAUTH_CALLBACK_RESULT);
    let port = get_oauth_port();
    
    tokio::spawn(async move {
        println!("Starting OAuth callback server on port {}", port);
        
        let server = tiny_http::Server::http(format!("localhost:{}", port)).unwrap();
        
        for request in server.incoming_requests() {
            let url = request.url();
            println!("Received callback request: {}", url);
            
            if url.starts_with("/callback") {
                let response_html = r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <title>Authentication Complete</title>
                        <style>
                            body { font-family: Arial, sans-serif; text-align: center; padding: 50px; }
                            .success { color: #28a745; }
                        </style>
                    </head>
                    <body>
                        <h1 class="success">Authentication Complete!</h1>
                        <p>You can close this window and return to the application.</p>
                        <script>
                            setTimeout(() => window.close(), 2000);
                        </script>
                    </body>
                    </html>
                "#;
                
                // Parse the callback URL - need to construct full URL for parsing
                if let Ok(parsed_url) = url::Url::parse(&format!("http://localhost:{}{}", port, url)) {
                    let query_pairs: HashMap<_, _> = parsed_url.query_pairs().collect();
                    
                    let result = if let Some(code) = query_pairs.get("code") {
                        println!("Authorization code received: {}", code);
                        Ok(code.to_string())
                    } else if let Some(error) = query_pairs.get("error") {
                        println!("OAuth error: {}", error);
                        Err(format!("OAuth error: {}", error))
                    } else {
                        println!("No authorization code received");
                        Err("No authorization code received".to_string())
                    };
                    
                    // Store the result
                    *callback_result.lock().await = Some(result);
                }
                
                let response = tiny_http::Response::from_string(response_html)
                    .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
                
                let _ = request.respond(response);
                break; // Stop the server after handling the callback
            } else {
                // Return 404 for other requests
                let response = tiny_http::Response::from_string("Not Found").with_status_code(404);
                let _ = request.respond(response);
            }
        }
        
        println!("OAuth callback server stopped");
    });
}

#[tauri::command]
pub async fn complete_microsoft_auth(auth_code: String) -> Result<MicrosoftAccount, String> {
    // Step 1: Exchange auth code for tokens
    let client = reqwest::Client::new();
    let client_id = get_client_id();
    let redirect_uri = get_redirect_uri();
    
    // Get the stored PKCE verifier
    let code_verifier = {
        let verifier_guard = PKCE_VERIFIER.lock().unwrap();
        verifier_guard.as_ref().ok_or("No PKCE verifier found")?.clone()
    };
    
    println!("Token exchange parameters:");
    println!("  client_id: {}", client_id);
    println!("  redirect_uri: {}", redirect_uri);
    println!("  auth_code: {}", auth_code);
    println!("  code_verifier: {}", code_verifier);
    println!("  using PKCE (no client_secret)");
    
    // Build form data for PKCE token exchange
    let form_data = [
        ("client_id", client_id.as_str()),
        ("code", &auth_code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", redirect_uri.as_str()),
        ("code_verifier", &code_verifier),
    ];
    
    println!("Form data being sent: {:?}", form_data);
    
    let token_response = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
        .form(&form_data)
        .send()
        .await
        .map_err(|e| format!("Failed to exchange auth code: {}", e))?;

    // Debug: Check the response status and body if it failed
    if !token_response.status().is_success() {
        let status = token_response.status();
        let error_body = token_response.text().await.unwrap_or_else(|_| "Failed to read error response".to_string());
        println!("Token exchange failed with status {}: {}", status, error_body);
        return Err(format!("Token exchange failed with status {}: {}", status, error_body));
    }

    let token_response = token_response
        .json::<AuthorizationTokenResponse>()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    // Step 2: Authenticate with Xbox Live
    let xbox_auth_response = authenticate_xbox_live(&token_response.access_token)
        .await
        .map_err(|e| format!("Xbox Live authentication failed: {}", e))?;
    
    // Step 3: Get XSTS token
    let xsts_response = get_xsts_token(&xbox_auth_response.token)
        .await
        .map_err(|e| format!("XSTS token request failed: {}", e))?;
    
    let user_hash = xbox_auth_response.display_claims
        .get("xui")
        .and_then(|xui| xui.first())
        .and_then(|user| user.get("uhs"))
        .map(|s| s.as_str())
        .ok_or("Failed to extract user hash")?;
    
    println!("Xbox user hash: {}", user_hash);
    println!("XSTS token (first 50 chars): {}", &xsts_response.token[..std::cmp::min(50, xsts_response.token.len())]);
    
    // Step 4: Authenticate with Minecraft (or fall back to Xbox profile)
    let (minecraft_access_token, minecraft_expires_at, username, uuid) = 
        match authenticate_minecraft(&xsts_response.token, user_hash).await {
            Ok(minecraft_response) => {
                // Try to get Minecraft profile
                match get_minecraft_profile(&minecraft_response.access_token).await {
                    Ok(profile) => {
                        println!("Full Minecraft authentication successful!");
                        (
                            Some(minecraft_response.access_token),
                            Some(Utc::now().timestamp() + minecraft_response.expires_in as i64),
                            profile.name,
                            profile.id
                        )
                    }
                    Err(e) => {
                        println!("Minecraft profile retrieval failed: {}", e);
                        // Fall back to Xbox profile data
                        let xbox_username = format!("User-{}", &user_hash[..8]);
                        let xbox_uuid = uuid::Uuid::new_v4().to_string();
                        (None, None, xbox_username, xbox_uuid)
                    }
                }
            }
            Err(e) => {
                println!("Minecraft authentication failed (this is expected for non-partner apps): {}", e);
                println!("Falling back to Xbox Live profile information...");
                
                // Use Xbox Live user information as fallback
                let xbox_username = format!("User-{}", &user_hash[..8]);
                let xbox_uuid = uuid::Uuid::new_v4().to_string();
                
                (None, None, xbox_username, xbox_uuid)
            }
        };

    let account = MicrosoftAccount {
        id: uuid::Uuid::new_v4().to_string(),
        username,
        uuid,
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        expires_at: Utc::now().timestamp() + token_response.expires_in as i64,
        skin_url: None, // Simplified - no skins for now
        is_active: true,
        last_used: Utc::now().timestamp(),
        minecraft_access_token,
        minecraft_expires_at,
        xbox_user_hash: user_hash.to_string(),
    };
    
    // Write session to Minecraft launcher profiles
    if let Err(e) = write_minecraft_session(account.clone()).await {
        eprintln!("Warning: Failed to write Minecraft session: {}", e);
    }
    
    Ok(account)
}

// Simplified authentication functions based on working example
async fn authenticate_xbox_live(access_token: &str) -> Result<XboxLiveAuthenticationResponse, AppError> {
    let client = reqwest::Client::new();
    
    println!("Authenticating with Xbox Live...");
    
    let response = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .json(&json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={}", access_token)
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        }))
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        println!("Xbox Live authentication failed with status {}: {}", status, error_text);
        return Err(AppError::OAuth(format!("Xbox Live authentication failed: {} - {}", status, error_text)));
    }
    
    let response_text = response.text().await?;
    println!("Xbox Live authentication response: {}", response_text);
    
    let xbox_response: XboxLiveAuthenticationResponse = serde_json::from_str(&response_text)
        .map_err(|e| {
            println!("Failed to parse Xbox Live response: {}", e);
            AppError::Json(e)
        })?;
    
    println!("Xbox Live authentication successful!");
    println!("Xbox token length: {}", xbox_response.token.len());
    println!("Display claims: {:?}", xbox_response.display_claims);
    
    Ok(xbox_response)
}

async fn get_xsts_token(xbox_token: &str) -> Result<XboxLiveAuthenticationResponse, AppError> {
    let client = reqwest::Client::new();
    
    println!("Getting XSTS token...");
    
    let response = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbox_token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        }))
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        println!("XSTS token request failed with status {}: {}", status, error_text);
        
        // Check for specific XSTS errors
        if status == 401 {
            println!("401 Unauthorized - This usually means:");
            println!("1. The Xbox Live token is invalid or expired");
            println!("2. The account doesn't have Xbox Live access");
            return Err(AppError::OAuth(format!("XSTS authorization failed: 401 Unauthorized - Account may not have Xbox Live access")));
        }
        
        return Err(AppError::OAuth(format!("XSTS token request failed: {} - {}", status, error_text)));
    }
    
    let response_text = response.text().await?;
    println!("XSTS token response: {}", response_text);
    
    let xsts_response: XboxLiveAuthenticationResponse = serde_json::from_str(&response_text)
        .map_err(|e| {
            println!("Failed to parse XSTS response: {}", e);
            AppError::Json(e)
        })?;
    
    println!("XSTS token obtained successfully!");
    println!("XSTS token length: {}", xsts_response.token.len());
    
    Ok(xsts_response)
}

async fn authenticate_minecraft(xsts_token: &str, user_hash: &str) -> Result<MinecraftAuthenticationResponse, AppError> {
    let client = reqwest::Client::new();
    
    println!("Authenticating with Minecraft Services...");
    println!("User hash: {}", user_hash);
    println!("XSTS token length: {}", xsts_token.len());
    
    let identity_token = format!("XBL3.0 x={};{}", user_hash, xsts_token);
    println!("Identity token format: XBL3.0 x=<user_hash>;<xsts_token>");
    println!("Identity token length: {}", identity_token.len());
    
    let request_body = json!({
        "identityToken": identity_token
    });
    
    println!("Sending request to Minecraft Services API...");
    println!("Request body: {}", serde_json::to_string_pretty(&request_body).unwrap_or_default());
    
    let response = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&request_body)
        .send()
        .await?;
    
    let status = response.status();
    let headers = response.headers().clone();
    
    println!("Minecraft API response status: {}", status);
    println!("Response headers: {:?}", headers);
    
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        println!("Minecraft authentication failed with status {}: {}", status, error_text);
        
        // Check for specific error types
        if status == 403 {
            println!("403 Forbidden - This usually means:");
            println!("1. The Xbox Live account doesn't own Minecraft");
            println!("2. The XSTS token doesn't have the right permissions");
            println!("3. The identity token format is incorrect");
            return Err(AppError::OAuth(format!("Minecraft authentication failed: 403 Forbidden - Account may not own Minecraft or lacks permissions")));
        }
        
        return Err(AppError::OAuth(format!("Minecraft authentication failed: {} - {}", status, error_text)));
    }
    
    let response_text = response.text().await?;
    println!("Minecraft authentication response: {}", response_text);
    
    let minecraft_response: MinecraftAuthenticationResponse = serde_json::from_str(&response_text)
        .map_err(|e| {
            println!("Failed to parse Minecraft authentication response: {}", e);
            println!("Response was: {}", response_text);
            AppError::Json(e)
        })?;
    
    println!("Minecraft authentication successful!");
    Ok(minecraft_response)
}

async fn get_minecraft_profile(access_token: &str) -> Result<MinecraftProfileResponse, AppError> {
    let client = reqwest::Client::new();
    
    let response = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    
    let profile: MinecraftProfileResponse = response.json().await?;
    Ok(profile)
}

// Minecraft session file management
fn get_minecraft_directory() -> Result<PathBuf, AppError> {
    let home_dir = dirs::home_dir().ok_or_else(|| AppError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot find home directory")))?;
    
    #[cfg(target_os = "windows")]
    let minecraft_dir = home_dir.join("AppData").join("Roaming").join(".minecraft");
    
    #[cfg(target_os = "macos")]
    let minecraft_dir = home_dir.join("Library").join("Application Support").join("minecraft");
    
    #[cfg(target_os = "linux")]
    let minecraft_dir = home_dir.join(".minecraft");
    
    Ok(minecraft_dir)
}

#[tauri::command]
pub async fn get_minecraft_session_path() -> Result<String, String> {
    let minecraft_dir = get_minecraft_directory().map_err(|e| e.to_string())?;
    let launcher_profiles_path = minecraft_dir.join("launcher_profiles.json");
    Ok(launcher_profiles_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn read_minecraft_sessions() -> Result<LauncherProfiles, String> {
    let minecraft_dir = get_minecraft_directory().map_err(|e| e.to_string())?;
    let launcher_profiles_path = minecraft_dir.join("launcher_profiles.json");
    
    if !launcher_profiles_path.exists() {
        // Create default launcher profiles if it doesn't exist
        let default_profiles = LauncherProfiles {
            authentication_database: HashMap::new(),
            launcher_version: "3.0.0".to_string(),
            selected_user: None,
        };
        return Ok(default_profiles);
    }
    
    let content = fs::read_to_string(&launcher_profiles_path)
        .map_err(|e| format!("Failed to read launcher_profiles.json: {}", e))?;
    
    let profiles: LauncherProfiles = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse launcher_profiles.json: {}", e))?;
    
    Ok(profiles)
}

#[tauri::command]
pub async fn write_minecraft_session(account: MicrosoftAccount) -> Result<(), String> {
    // Only write session if we have a Minecraft access token
    let minecraft_access_token = match account.minecraft_access_token {
        Some(token) => token,
        None => {
            println!("No Minecraft access token available - skipping session file creation");
            return Ok(());
        }
    };

    let minecraft_dir = get_minecraft_directory().map_err(|e| e.to_string())?;
    let launcher_profiles_path = minecraft_dir.join("launcher_profiles.json");
    
    // Ensure minecraft directory exists
    fs::create_dir_all(&minecraft_dir)
        .map_err(|e| format!("Failed to create minecraft directory: {}", e))?;
    
    // Load existing profiles or create new
    let mut profiles = if launcher_profiles_path.exists() {
        let content = fs::read_to_string(&launcher_profiles_path)
            .map_err(|e| format!("Failed to read launcher_profiles.json: {}", e))?;
        serde_json::from_str(&content)
            .unwrap_or_else(|_| LauncherProfiles {
                authentication_database: HashMap::new(),
                launcher_version: "3.0.0".to_string(),
                selected_user: None,
            })
    } else {
        LauncherProfiles {
            authentication_database: HashMap::new(),
            launcher_version: "3.0.0".to_string(),
            selected_user: None,
        }
    };
    
    // Create session entry
    let session = MinecraftSession {
        access_token: minecraft_access_token,
        client_token: uuid::Uuid::new_v4().to_string(),
        uuid: account.uuid.clone(),
        username: account.username.clone(),
        user_type: "microsoft".to_string(),
        user_properties: serde_json::json!({}),
    };
    
    // Add/update the session
    profiles.authentication_database.insert(account.uuid.clone(), session);
    profiles.selected_user = Some(account.uuid);
    
    // Write back to file
    let content = serde_json::to_string_pretty(&profiles)
        .map_err(|e| format!("Failed to serialize launcher profiles: {}", e))?;
    
    fs::write(&launcher_profiles_path, content)
        .map_err(|e| format!("Failed to write launcher_profiles.json: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn refresh_minecraft_token(_account_id: String) -> Result<MicrosoftAccount, String> {
    // For now, we'll need to implement account storage management
    // This is a placeholder that would need integration with your settings system
    Err("Token refresh not yet implemented - requires account storage".to_string())
}

#[tauri::command]
pub async fn get_oauth_callback_result() -> Result<Option<String>, String> {
    let mut result = OAUTH_CALLBACK_RESULT.lock().await;
    
    if let Some(callback_result) = result.take() {
        match callback_result {
            Ok(code) => Ok(Some(code)),
            Err(error) => Err(error),
        }
    } else {
        Ok(None)
    }
}

// Additional utility functions for session management
#[tauri::command]
pub async fn get_minecraft_launch_args(account: MicrosoftAccount) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    
    // Standard Minecraft launch arguments with authentication
    let access_token = account.minecraft_access_token.clone().unwrap_or_else(|| "offline".to_string());
    let user_type = if account.minecraft_access_token.is_some() { "microsoft" } else { "offline" };
    
    args.extend_from_slice(&[
        "--username".to_string(),
        account.username,
        "--uuid".to_string(),
        account.uuid,
        "--accessToken".to_string(),
        access_token,
        "--userType".to_string(),
        user_type.to_string(),
        "--versionType".to_string(),
        "release".to_string(),
    ]);
    
    Ok(args)
}

#[tauri::command]
pub async fn validate_minecraft_token(access_token: String) -> Result<bool, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(response.status().is_success())
}

#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    println!("Opening URL: {}", url);
    
    #[cfg(target_os = "windows")]
    {
        // Use rundll32 with url.dll to properly open URLs in the default browser
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", &url])
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    
    Ok(())
}

// PKCE helper functions
fn generate_code_verifier() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};
    
    // Generate a pseudo-random string using current time and system info
    let mut hasher = DefaultHasher::new();
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
    std::process::id().hash(&mut hasher);
    
    let hash = hasher.finish();
    let verifier = format!("{:x}{:x}", hash, hash.wrapping_mul(17));
    
    // Ensure it's the right length (43-128 characters)
    if verifier.len() < 43 {
        format!("{}{}", verifier, "a".repeat(43 - verifier.len()))
    } else if verifier.len() > 128 {
        verifier[..128].to_string()
    } else {
        verifier
    }
}

fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    general_purpose::URL_SAFE_NO_PAD.encode(hash)
}

#[tauri::command]
pub async fn start_device_code_auth() -> Result<String, String> {
    let client_id = get_client_id();
    
    let client = reqwest::Client::new();
    
    // Start device code flow
    let device_response = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
        .form(&[
            ("client_id", client_id.as_str()),
            ("scope", "XboxLive.signin offline_access"),
        ])
        .send()
        .await
        .map_err(|e| format!("Failed to start device code flow: {}", e))?;
    
    let device_data: serde_json::Value = device_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse device code response: {}", e))?;
    
    let user_code = device_data["user_code"].as_str().ok_or("No user code received")?;
    let verification_uri = device_data["verification_uri"].as_str().ok_or("No verification URI received")?;
    let device_code = device_data["device_code"].as_str().ok_or("No device code received")?;
    let expires_in = device_data["expires_in"].as_u64().unwrap_or(900);
    
    println!("Device Code Flow started:");
    println!("  User code: {}", user_code);
    println!("  Verification URI: {}", verification_uri);
    println!("  Expires in: {} seconds", expires_in);
    
    // Store device code for polling
    {
        let mut verifier_guard = PKCE_VERIFIER.lock().unwrap();
        *verifier_guard = Some(device_code.to_string());
    }
    
    // Automatically open the verification URL
    let _ = std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", verification_uri])
        .spawn();
    
    // Return structured data for better UX
    Ok(format!("{}|{}", user_code, verification_uri))
}

#[tauri::command] 
pub async fn poll_device_code_auth() -> Result<Option<MicrosoftAccount>, String> {
    let client_id = get_client_id();
    
    // Get stored device code
    let device_code = {
        let verifier_guard = PKCE_VERIFIER.lock().unwrap();
        verifier_guard.as_ref().ok_or("No device code found")?.clone()
    };
    
    let client = reqwest::Client::new();
    
    // Poll for token
    let token_response = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("client_id", client_id.as_str()),
            ("device_code", &device_code),
        ])
        .send()
        .await
        .map_err(|e| format!("Failed to poll device code: {}", e))?;
    
    if !token_response.status().is_success() {
        let error_body = token_response.text().await.unwrap_or_default();
        
        // Check if it's still pending
        if error_body.contains("authorization_pending") {
            return Ok(None); // Still waiting
        } else if error_body.contains("authorization_declined") {
            return Err("User declined authorization".to_string());
        } else if error_body.contains("expired_token") {
            return Err("Device code expired".to_string());
        } else {
            return Err(format!("Token exchange failed: {}", error_body));
        }
    }
    
    let token_response: AuthorizationTokenResponse = token_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;
    
    // Continue with Xbox Live → XSTS → Minecraft flow same as before
    // Step 2: Authenticate with Xbox Live
    let xbox_auth_response = authenticate_xbox_live(&token_response.access_token)
        .await
        .map_err(|e| format!("Xbox Live authentication failed: {}", e))?;
    
    // Step 3: Get XSTS token  
    let xsts_response = get_xsts_token(&xbox_auth_response.token)
        .await
        .map_err(|e| format!("XSTS token request failed: {}", e))?;
    
    let user_hash = xbox_auth_response.display_claims
        .get("xui")
        .and_then(|xui| xui.first())
        .and_then(|user| user.get("uhs"))
        .ok_or("Failed to extract user hash")?;
    
    // Step 4: Authenticate with Minecraft (or fall back to Xbox profile)
    let (minecraft_access_token, minecraft_expires_at, username, uuid) = 
        match authenticate_minecraft(&xsts_response.token, user_hash).await {
            Ok(minecraft_response) => {
                // Try to get Minecraft profile
                match get_minecraft_profile(&minecraft_response.access_token).await {
                    Ok(profile) => {
                        println!("Full Minecraft authentication successful!");
                        (
                            Some(minecraft_response.access_token),
                            Some(Utc::now().timestamp() + minecraft_response.expires_in as i64),
                            profile.name,
                            profile.id
                        )
                    }
                    Err(e) => {
                        println!("Minecraft profile retrieval failed: {}", e);
                        // Fall back to Xbox profile data
                        let xbox_username = format!("User-{}", &user_hash[..8]);
                        let xbox_uuid = uuid::Uuid::new_v4().to_string();
                        (None, None, xbox_username, xbox_uuid)
                    }
                }
            }
            Err(e) => {
                println!("Minecraft authentication failed (this is expected for non-partner apps): {}", e);
                println!("Falling back to Xbox Live profile information...");
                
                // Use Xbox Live user information as fallback
                let xbox_username = format!("User-{}", &user_hash[..8]);
                let xbox_uuid = uuid::Uuid::new_v4().to_string();
                
                (None, None, xbox_username, xbox_uuid)
            }
        };

    let account = MicrosoftAccount {
        id: uuid::Uuid::new_v4().to_string(),
        username,
        uuid,
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        expires_at: Utc::now().timestamp() + token_response.expires_in as i64,
        skin_url: None,
        is_active: true,
        last_used: Utc::now().timestamp(),
        minecraft_access_token,
        minecraft_expires_at,
        xbox_user_hash: user_hash.to_string(),
    };
    
    // Write session to Minecraft launcher profiles
    if let Err(e) = write_minecraft_session(account.clone()).await {
        eprintln!("Warning: Failed to write Minecraft session: {}", e);
    }
    
    Ok(Some(account))
}

#[tauri::command]
pub async fn copy_to_clipboard(text: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let mut cmd = Command::new("powershell")
            .args(["-Command", &format!("Set-Clipboard -Value '{}'", text)])
            .spawn()
            .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;
        
        let _ = cmd.wait();
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let mut cmd = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;
        
        if let Some(stdin) = cmd.stdin.as_mut() {
            use std::io::Write;
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = cmd.wait();
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let mut cmd = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;
        
        if let Some(stdin) = cmd.stdin.as_mut() {
            use std::io::Write;
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = cmd.wait();
    }
    
    Ok(())
}
