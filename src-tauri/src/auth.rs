use serde::{Deserialize, Serialize};
use chrono::{Utc, DateTime};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;
use std::fs;
use crate::AppError;
use crate::logging::{Logger, LogLevel};
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

impl MinecraftSession {
    /// Convert MinecraftSession to MicrosoftAccount for launcher compatibility
    pub fn to_microsoft_account(&self) -> MicrosoftAccount {
        let now = Utc::now().timestamp();
        MicrosoftAccount {
            id: self.uuid.clone(),
            username: self.username.clone(),
            uuid: self.uuid.clone(),
            access_token: self.access_token.clone(),
            refresh_token: String::new(), // Not available in MinecraftSession
            expires_at: now + 3600, // Assume 1 hour expiry
            skin_url: None,
            is_active: true,
            last_used: now,
            minecraft_access_token: Some(self.access_token.clone()),
            minecraft_expires_at: Some(now + 3600),
            xbox_user_hash: String::new(), // Not available in MinecraftSession
        }
    }
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
    
    Logger::console_log(LogLevel::Debug, &format!("Generated auth URL with PKCE: {}", auth_url), None);
    Logger::console_log(LogLevel::Debug, &format!("Code challenge: {}", code_challenge), None);
    Ok(auth_url)
}

async fn start_oauth_callback_server() {
    let callback_result = Arc::clone(&OAUTH_CALLBACK_RESULT);
    let port = get_oauth_port();
    
    tokio::spawn(async move {
        Logger::console_log(LogLevel::Info, &format!("Starting OAuth callback server on port {}", port), None);
        
        let server = tiny_http::Server::http(format!("localhost:{}", port)).unwrap();
        
        for request in server.incoming_requests() {
            let url = request.url();
            Logger::console_log(LogLevel::Debug, &format!("Received callback request: {}", url), None);
            
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
                        Logger::console_log(LogLevel::Info, &format!("Authorization code received: {}", code), None);
                        Ok(code.to_string())
                    } else if let Some(error) = query_pairs.get("error") {
                        Logger::console_log(LogLevel::Error, &format!("OAuth error: {}", error), None);
                        Err(format!("OAuth error: {}", error))
                    } else {
                        Logger::console_log(LogLevel::Warning, "No authorization code received", None);
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
        
        Logger::console_log(LogLevel::Info, "OAuth callback server stopped", None);
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
    
    Logger::console_log(LogLevel::Debug, "Token exchange parameters:", None);
    Logger::console_log(LogLevel::Debug, &format!("  client_id: {}", client_id), None);
    Logger::console_log(LogLevel::Debug, &format!("  redirect_uri: {}", redirect_uri), None);
    Logger::console_log(LogLevel::Debug, &format!("  auth_code: {}", auth_code), None);
    Logger::console_log(LogLevel::Debug, &format!("  code_verifier: {}", code_verifier), None);
    Logger::console_log(LogLevel::Debug, "  using PKCE (no client_secret)", None);
    
    // Build form data for PKCE token exchange
    let form_data = [
        ("client_id", client_id.as_str()),
        ("code", &auth_code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", redirect_uri.as_str()),
        ("code_verifier", &code_verifier),
    ];
    
    Logger::console_log(LogLevel::Debug, &format!("Form data being sent: {:?}", form_data), None);
    
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
        Logger::console_log(LogLevel::Error, &format!("Token exchange failed with status {}: {}", status, error_body), None);
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
    
    Logger::console_log(LogLevel::Debug, &format!("Xbox user hash: {}", user_hash), None);
    Logger::console_log(LogLevel::Debug, &format!("XSTS token (first 50 chars): {}", &xsts_response.token[..std::cmp::min(50, xsts_response.token.len())]), None);
    
    // Step 4: Authenticate with Minecraft (or fall back to Xbox profile)
    let (minecraft_access_token, minecraft_expires_at, username, uuid) = 
        match authenticate_minecraft(&xsts_response.token, user_hash).await {
            Ok(minecraft_response) => {
                // Try to get Minecraft profile
                match get_minecraft_profile(&minecraft_response.access_token).await {
                    Ok(profile) => {
                        Logger::console_log(LogLevel::Info, "Full Minecraft authentication successful!", None);
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
    
    // Write to launcher_accounts.json
    if let Err(e) = write_launcher_account(account.clone()).await {
        eprintln!("Warning: Failed to write launcher account: {}", e);
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
            return Err(AppError::OAuth("XSTS authorization failed: 401 Unauthorized - Account may not have Xbox Live access".to_string()));
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
            return Err(AppError::OAuth("Minecraft authentication failed: 403 Forbidden - Account may not own Minecraft or lacks permissions".to_string()));
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
    // Read Kable's own auth sessions, not the Minecraft launcher profiles
    let kable_dir = crate::installations::get_kable_directory().map_err(|e| e.to_string())?;
    let sessions_path = kable_dir.join("minecraft_sessions.json");
    
    if !sessions_path.exists() {
        // Create default auth sessions if it doesn't exist
        let default_profiles = LauncherProfiles {
            authentication_database: HashMap::new(),
            launcher_version: "3.0.0".to_string(),
            selected_user: None,
        };
        return Ok(default_profiles);
    }
    
    let content = fs::read_to_string(&sessions_path)
        .map_err(|e| format!("Failed to read minecraft_sessions.json: {}", e))?;
    
    let profiles: LauncherProfiles = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse minecraft_sessions.json: {}", e))?;
    
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

    let kable_dir = crate::installations::get_kable_directory().map_err(|e| e.to_string())?;
    let sessions_path = kable_dir.join("minecraft_sessions.json");
    
    // Ensure kable directory exists
    fs::create_dir_all(&kable_dir)
        .map_err(|e| format!("Failed to create kable directory: {}", e))?;
    
    // Load existing profiles or create new
    let mut profiles = if sessions_path.exists() {
        let content = fs::read_to_string(&sessions_path)
            .map_err(|e| format!("Failed to read minecraft_sessions.json: {}", e))?;
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
        .map_err(|e| format!("Failed to serialize minecraft sessions: {}", e))?;
    
    fs::write(&sessions_path, content)
        .map_err(|e| format!("Failed to write minecraft_sessions.json: {}", e))?;
    
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

// Launcher Accounts JSON structure (matches .minecraft/launcher_accounts.json)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LauncherAccount {
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub access_token_expires_at: String, // ISO date string
    #[serde(default)]
    pub avatar: String, // base64 encoded image
    #[serde(default)]
    pub eligible_for_free_trials: bool,
    #[serde(default)]
    pub eligible_for_migration: bool,
    #[serde(default)]
    pub franchise_inventory_id: String,
    #[serde(default)]
    pub has_multiple_profiles: bool,
    #[serde(default)]
    pub in_forced_migration: bool,
    #[serde(default)]
    pub legacy: bool,
    #[serde(default)]
    pub license_product_ids: Vec<String>,
    pub local_id: String, // This is required for identification
    #[serde(default)]
    pub minecraft_profile: Option<MinecraftProfile>,
    #[serde(default)]
    pub persistent: bool,
    #[serde(default)]
    pub remote_id: String,
    #[serde(rename = "type", default)]
    pub account_type: String, // Usually "Xbox"
    // Handle both "userProperties" and "userProperites" (typo in some files)
    #[serde(alias = "userProperites", default)]
    pub user_properties: Vec<serde_json::Value>,
    #[serde(default)]
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftProfile {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub requires_profile_name_change: bool,
    #[serde(default)]
    pub requires_skin_change: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherAccountsJson {
    #[serde(default)]
    pub accounts: HashMap<String, LauncherAccount>,
    #[serde(default)]
    pub active_account_local_id: String,
    #[serde(default)]
    pub mojang_client_token: String,
}

/// Read all accounts from launcher_accounts.json
#[tauri::command]
pub async fn read_launcher_accounts() -> Result<LauncherAccountsJson, String> {
    let minecraft_dir = get_minecraft_directory().map_err(|e| e.to_string())?;
    let launcher_accounts_path = minecraft_dir.join("launcher_accounts.json");
    
    if !launcher_accounts_path.exists() {
        // Create empty accounts file if it doesn't exist
        let default_accounts = LauncherAccountsJson {
            accounts: HashMap::new(),
            active_account_local_id: String::new(),
            mojang_client_token: String::new(),
        };
        return Ok(default_accounts);
    }
    
    let content = fs::read_to_string(&launcher_accounts_path)
        .map_err(|e| format!("Failed to read launcher_accounts.json: {}", e))?;
    
    let accounts: LauncherAccountsJson = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse launcher_accounts.json: {}", e))?;
    
    Ok(accounts)
}

/// Write account to launcher_accounts.json
#[tauri::command]
pub async fn write_launcher_account(account: MicrosoftAccount) -> Result<(), String> {
    let minecraft_dir = get_minecraft_directory().map_err(|e| e.to_string())?;
    let launcher_accounts_path = minecraft_dir.join("launcher_accounts.json");
    
    // Ensure minecraft directory exists
    fs::create_dir_all(&minecraft_dir)
        .map_err(|e| format!("Failed to create minecraft directory: {}", e))?;
    
    // Load existing accounts or create new
    let mut accounts_data = if launcher_accounts_path.exists() {
        let content = fs::read_to_string(&launcher_accounts_path)
            .map_err(|e| format!("Failed to read launcher_accounts.json: {}", e))?;
        serde_json::from_str(&content)
            .unwrap_or_else(|_| LauncherAccountsJson {
                accounts: HashMap::new(),
                active_account_local_id: String::new(),
                mojang_client_token: String::new(),
            })
    } else {
        LauncherAccountsJson {
            accounts: HashMap::new(),
            active_account_local_id: String::new(),
            mojang_client_token: String::new(),
        }
    };
    
    // Convert MicrosoftAccount to LauncherAccount format
    let expires_at = if let Some(minecraft_expires_at) = account.minecraft_expires_at {
        DateTime::from_timestamp(minecraft_expires_at, 0)
            .unwrap_or_else(Utc::now)
            .to_rfc3339()
    } else {
        DateTime::from_timestamp(account.expires_at, 0)
            .unwrap_or_else(Utc::now)
            .to_rfc3339()
    };
    
    // Generate a default avatar if none provided
    let default_avatar = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsMAAA7DAcdvqGQAAATFSURBVHhe7Vo7jxxFEK55z97u2nt3KUj8A8syQiAkAidGxpIlIEHiByATkOCIjJSMwEIkkDgDAss6icCJAxvJQpbJcEoM99jdeT+o6u7Tze50T9/c+MDrme/0Xd3MbNdUVU9XVc+ecfPKdgk9hilkbzEEQMjeYgiAkL3FEAAhe4shAEL2FkMAhOwthgAI2Vtod4NFudmbRdMwxF9yNAaAho58DwyNkpcW6FkQx1A2TKIyADTIc1249dEHsD2dQlEU4spmgCYtTlK488t92D+ag2nKV3tjAHwMwO1PP4adC5sZgChJ4Ju7P8M/h0fKAGiTIDlOeWBTqcNQBoXsLYYAsDUuYU6y3KzEJwP3BfNBxbcqzZ2LF0DGXSSVP5499cnkZYSBP7PpBH2ZSn0kGn98/0Wjd65tb24jhEjSDKdP7aLx/MfbjQFo6qI2AbrJM8nBJm46ZD5VOVQBIXuLIQBCqoG9QEn9wBqPz5mYY5rYFTKdVVZtkVID488fvmzIdAZY3ohSqTheRYkNxmIeoQHixBpo1MgrVMO1ILVhbKB+tYLJ1AfTUlxHBXkSoFS7qA4AZUjHg9evfw7OZBudXY0m3fRwP4Cvbt2Fw4MAG6ZVI+ienlPCZzcOYDahrktcOCUoaHFqwHf3Z3CwsFD/qpmk33Es+PrbT+C1N3Yhz9ZugAqKNIK/9u5AutzHQ/nD3rwEUInlj8F0R0xWaThbYHoTWMQWzAMT5mGdi8hs7XgV5OQSdch0zwODScMdA9h1+yxvC2x/yiMpn2KGU+SAhtG4xvhaxKeFyTrZOugArkeiHy3n7zjU9rHcoIE+AK84hgAI2VvoA9BUwzCzUorAQsdklVghmaQtKWmgTUk78jGoAnXV9R+T7qAEKdFAG4AijaHMUyarxF9QYpnxnAJGSN+lt8gnHCGpDGZ5BmmGxG1pW2Y4zrVRv1vXz4j6ywxtKev2FVnCJF7kjijQ3AhhBO0RlZJ6nCi2BU7z/t9L9mZlfSJwDpnzT5/9DlEUsdlsA9qp2bYNly9dAd/32fEK8JB0znbH+Dn+JNZRQh7M8Zq6Gmg6QTIEBys+QT6xBkjiHJ2hmd978BsslmGtUdKBHHYcG65ffQcm45HyFXeRUyssDiQwFN8HHEO7BKiDIiUy4i+2PukBqJGd5+uZnOfruj0pl6CPdf2CZIPMtmPqoP/EK44hAEL2Fp0DIFu3J+TJjBIYyfbk+uk7/rpuzq7QVoEmkIFJgv2ApEyQcWmWw6MnzyAMz1AGkbZlwbtvXWL/o0ABqcMAz+322v7MAaCbRlEC9x48YXLdCDKY9uvX3rsMky1fWcZUIHVpmsOvD5/CMojr+vHHwix/4+qbMLs4xopwJje6LQEygpwPohhCCaMoBdd1sJFxwffa0gPPcyCOUwhCif4wQYndXsvAruOF5AD1GuVPAr06O1nXbcj1Gw19RFcMVUDI3mIIgJDnBtm6PR1xrNDBauI54dwDQPt6+oq6NRMume/dc50SnfoAKkc/7T1mko7XQaeoidH9t6YKZBiVQKoI66BTlmXCh++/DTuzyf/TB+hARi6DCI4WIRwtWxLHzFHKnH+R+E9yAL0PoKegFcWY88ZQBYTsLYYACNlb9DwAAP8CwzL5HoQDEDYAAAAASUVORK5CYII=";
    
    let launcher_account = LauncherAccount {
        access_token: account.minecraft_access_token.clone().unwrap_or_default(),
        access_token_expires_at: expires_at,
        avatar: account.skin_url.clone().unwrap_or_else(|| default_avatar.to_string()),
        eligible_for_free_trials: true,
        eligible_for_migration: false,
        franchise_inventory_id: "1/OQ==".to_string(),
        has_multiple_profiles: false,
        in_forced_migration: false,
        legacy: false,
        license_product_ids: vec![],
        local_id: account.uuid.clone(),
        minecraft_profile: Some(MinecraftProfile {
            id: account.uuid.replace("-", ""), // Minecraft profile IDs don't have dashes
            name: account.username.clone(),
            requires_profile_name_change: false,
            requires_skin_change: false,
        }),
        persistent: true,
        remote_id: account.id.clone(),
        account_type: "Xbox".to_string(),
        user_properties: vec![],
        username: account.username.clone(),
    };
    
    // Add/update the account
    accounts_data.accounts.insert(account.uuid.clone(), launcher_account);
    accounts_data.active_account_local_id = account.uuid;
    
    // Generate mojang client token if empty
    if accounts_data.mojang_client_token.is_empty() {
        accounts_data.mojang_client_token = uuid::Uuid::new_v4().to_string();
    }
    
    // Write back to file
    let content = serde_json::to_string_pretty(&accounts_data)
        .map_err(|e| format!("Failed to serialize launcher accounts: {}", e))?;
    
    fs::write(&launcher_accounts_path, content)
        .map_err(|e| format!("Failed to write launcher_accounts.json: {}", e))?;
    
    Ok(())
}

/// Remove account from launcher_accounts.json
#[tauri::command]
pub async fn remove_launcher_account(account_id: String) -> Result<(), String> {
    let minecraft_dir = get_minecraft_directory().map_err(|e| e.to_string())?;
    let launcher_accounts_path = minecraft_dir.join("launcher_accounts.json");
    
    if !launcher_accounts_path.exists() {
        return Ok(()); // Nothing to remove
    }
    
    let content = fs::read_to_string(&launcher_accounts_path)
        .map_err(|e| format!("Failed to read launcher_accounts.json: {}", e))?;
    
    let mut accounts_data: LauncherAccountsJson = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse launcher_accounts.json: {}", e))?;
    
    // Remove the account
    accounts_data.accounts.remove(&account_id);
    
    // Update active account if it was the removed one
    if accounts_data.active_account_local_id == account_id {
        accounts_data.active_account_local_id = accounts_data.accounts.keys().next().unwrap_or(&String::new()).clone();
    }
    
    // Write back to file
    let content = serde_json::to_string_pretty(&accounts_data)
        .map_err(|e| format!("Failed to serialize launcher accounts: {}", e))?;
    
    fs::write(&launcher_accounts_path, content)
        .map_err(|e| format!("Failed to write launcher_accounts.json: {}", e))?;
    
    Ok(())
}

/// Set the active account in launcher_accounts.json
#[tauri::command]
pub async fn set_active_launcher_account(account_id: String) -> Result<(), String> {
    let minecraft_dir = get_minecraft_directory().map_err(|e| e.to_string())?;
    let launcher_accounts_path = minecraft_dir.join("launcher_accounts.json");
    
    if !launcher_accounts_path.exists() {
        return Err("launcher_accounts.json not found".to_string());
    }
    
    let content = fs::read_to_string(&launcher_accounts_path)
        .map_err(|e| format!("Failed to read launcher_accounts.json: {}", e))?;
    
    let mut accounts_data: LauncherAccountsJson = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse launcher_accounts.json: {}", e))?;
    
    // Check if account exists
    if !accounts_data.accounts.contains_key(&account_id) {
        return Err("Account not found".to_string());
    }
    
    // Set as active
    accounts_data.active_account_local_id = account_id;
    
    // Write back to file
    let content = serde_json::to_string_pretty(&accounts_data)
        .map_err(|e| format!("Failed to serialize launcher accounts: {}", e))?;
    
    fs::write(&launcher_accounts_path, content)
        .map_err(|e| format!("Failed to write launcher_accounts.json: {}", e))?;
    
    Ok(())
}

/// Convert LauncherAccount to MicrosoftAccount for internal use
fn launcher_account_to_microsoft_account(launcher_account: &LauncherAccount) -> MicrosoftAccount {
    let expires_at = if !launcher_account.access_token_expires_at.is_empty() {
        DateTime::parse_from_rfc3339(&launcher_account.access_token_expires_at)
            .map(|dt| dt.timestamp())
            .unwrap_or_else(|_| Utc::now().timestamp() + 3600)
    } else {
        Utc::now().timestamp() + 3600 // Default 1 hour from now
    };
    
    // Get username from minecraft profile if available, otherwise use account username
    let username = launcher_account.minecraft_profile
        .as_ref()
        .map(|profile| profile.name.clone())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| launcher_account.username.clone());
    
    // Get UUID from minecraft profile if available, otherwise use local_id
    let uuid = launcher_account.minecraft_profile
        .as_ref()
        .map(|profile| profile.id.clone())
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| launcher_account.local_id.clone());
    
    MicrosoftAccount {
        id: if !launcher_account.remote_id.is_empty() { launcher_account.remote_id.clone() } else { launcher_account.local_id.clone() },
        username,
        uuid,
        access_token: launcher_account.access_token.clone(),
        refresh_token: String::new(), // Not stored in launcher_accounts.json
        expires_at,
        skin_url: if !launcher_account.avatar.is_empty() { Some(launcher_account.avatar.clone()) } else { None },
        is_active: true,
        last_used: Utc::now().timestamp(),
        minecraft_access_token: if !launcher_account.access_token.is_empty() { Some(launcher_account.access_token.clone()) } else { None },
        minecraft_expires_at: Some(expires_at),
        xbox_user_hash: String::new(), // Not stored in launcher_accounts.json
    }
}

/// Get the currently active account from launcher_accounts.json
#[tauri::command]
pub async fn get_active_launcher_account() -> Result<Option<MicrosoftAccount>, String> {
    let accounts_data = read_launcher_accounts().await?;
    
    if accounts_data.active_account_local_id.is_empty() {
        return Ok(None);
    }
    
    if let Some(launcher_account) = accounts_data.accounts.get(&accounts_data.active_account_local_id) {
        Ok(Some(launcher_account_to_microsoft_account(launcher_account)))
    } else {
        Ok(None)
    }
}

/// Get all accounts from launcher_accounts.json as MicrosoftAccount list
#[tauri::command]
pub async fn get_all_launcher_accounts() -> Result<Vec<MicrosoftAccount>, String> {
    let accounts_data = read_launcher_accounts().await?;
    
    let accounts: Vec<MicrosoftAccount> = accounts_data.accounts.values()
        .map(launcher_account_to_microsoft_account)
        .collect();
    
    Ok(accounts)
}

// Simplified auth status for testing
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthStatus {
    pub authenticated: bool,
    pub username: Option<String>,
    pub uuid: Option<String>,
}

// Simple auth status check - for now, return mock data for testing
// TODO: Replace with proper Microsoft OAuth implementation
#[tauri::command]
pub async fn check_auth_status() -> Result<AuthStatus, String> {
    // For testing purposes, always return authenticated
    // In production, this would check for valid stored tokens
    Ok(AuthStatus {
        authenticated: true,
        username: Some("TestUser".to_string()),
        uuid: Some("test-uuid-1234".to_string()),
    })
}

// Mock function to get access token - replace with real Microsoft OAuth
#[tauri::command]
pub async fn get_access_token() -> Result<String, String> {
    // TODO: Implement proper Microsoft OAuth flow
    // For now, return a mock token so we can test the launcher
    Ok("mock_access_token_for_testing".to_string())
}

// Mock function for Microsoft login - replace with real OAuth
#[tauri::command]
pub async fn microsoft_login() -> Result<MicrosoftAccount, String> {
    // TODO: Open browser for Microsoft OAuth
    // For now, return mock data
    Ok(MicrosoftAccount {
        id: "test-id".to_string(),
        username: "TestUser".to_string(),
        uuid: "test-uuid-1234".to_string(),
        access_token: "mock_access_token".to_string(),
        refresh_token: "mock_refresh_token".to_string(),
        expires_at: chrono::Utc::now().timestamp() + 3600,
        skin_url: None,
        is_active: true,
        last_used: chrono::Utc::now().timestamp(),
        minecraft_access_token: Some("mock_minecraft_token".to_string()),
        minecraft_expires_at: Some(chrono::Utc::now().timestamp() + 3600),
        xbox_user_hash: "mock_xbox_hash".to_string(),
    })
}
