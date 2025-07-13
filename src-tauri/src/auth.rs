use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::AppError;

// Microsoft OAuth2 structures
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct XboxLiveAuth {
    pub token: String,
    pub user_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
    pub skins: Option<Vec<MinecraftSkin>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftSkin {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
}

// Microsoft OAuth2 implementation
// Using the official Minecraft Launcher client ID for compatibility
const CLIENT_ID: &str = "00000000402b5328"; // Official Minecraft Launcher client ID
const REDIRECT_URI: &str = "http://localhost:5713/callback";

// Global state for OAuth callback
static OAUTH_CALLBACK_RESULT: once_cell::sync::Lazy<Arc<Mutex<Option<Result<String, String>>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

#[tauri::command]
pub async fn start_microsoft_auth() -> Result<String, String> {
    // Clear any previous callback result
    *OAUTH_CALLBACK_RESULT.lock().await = None;
    
    // Start the callback server
    start_oauth_callback_server().await;
    
    let auth_url = format!(
        "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri={}&scope=XboxLive.signin%20offline_access&prompt=consent",
        CLIENT_ID,
        urlencoding::encode(REDIRECT_URI)
    );
    
    Ok(auth_url)
}

async fn start_oauth_callback_server() {
    let callback_result = Arc::clone(&OAUTH_CALLBACK_RESULT);
    
    tokio::spawn(async move {
        let server = tiny_http::Server::http("localhost:5713").unwrap();
        
        for request in server.incoming_requests() {
            let url = request.url();
            
            if url.starts_with("/callback") {
                let response_html = r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <title>Authentication Complete</title>
                        <style>
                            body { font-family: Arial, sans-serif; text-align: center; padding: 50px; }
                            .success { color: #28a745; }
                            .error { color: #dc3545; }
                        </style>
                    </head>
                    <body>
                        <h1>Authentication Complete</h1>
                        <p>You can close this window and return to the application.</p>
                        <script>
                            // Emit event to main window
                            if (window.opener) {
                                window.opener.postMessage({ type: 'oauth-complete' }, '*');
                            }
                            setTimeout(() => window.close(), 2000);
                        </script>
                    </body>
                    </html>
                "#;
                
                // Parse the callback URL
                if let Ok(parsed_url) = url::Url::parse(&format!("http://localhost{}", url)) {
                    let query_pairs: std::collections::HashMap<_, _> = parsed_url.query_pairs().collect();
                    
                    let result = if let Some(code) = query_pairs.get("code") {
                        Ok(code.to_string())
                    } else if let Some(error) = query_pairs.get("error") {
                        Err(format!("OAuth error: {}", error))
                    } else {
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
    });
}

#[tauri::command]
pub async fn complete_microsoft_auth(auth_code: String) -> Result<MicrosoftAccount, String> {
    // Step 1: Exchange auth code for tokens
    let client = reqwest::Client::new();
    
    let token_response = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
        .form(&[
            ("client_id", CLIENT_ID),
            ("code", &auth_code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", REDIRECT_URI),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<OAuthTokenResponse>()
        .await
        .map_err(|e| e.to_string())?;

    // Step 2: Authenticate with Xbox Live
    let xbox_auth = authenticate_xbox_live(&token_response.access_token).await.map_err(|e| e.to_string())?;
    
    // Step 3: Get XSTS token
    let xsts_token = get_xsts_token(&xbox_auth.token).await.map_err(|e| e.to_string())?;
    
    // Step 4: Authenticate with Minecraft
    let minecraft_token = authenticate_minecraft(&xsts_token, &xbox_auth.user_hash).await.map_err(|e| e.to_string())?;
    
    // Step 5: Get Minecraft profile
    let profile = get_minecraft_profile(&minecraft_token).await.map_err(|e| e.to_string())?;
    
    let account = MicrosoftAccount {
        id: uuid::Uuid::new_v4().to_string(),
        username: profile.name,
        uuid: profile.id,
        access_token: minecraft_token,
        refresh_token: token_response.refresh_token,
        expires_at: Utc::now().timestamp() + token_response.expires_in,
        skin_url: profile.skins.and_then(|skins| skins.first().map(|s| s.url.clone())),
        is_active: true,
        last_used: Utc::now().timestamp(),
    };
    
    Ok(account)
}

async fn authenticate_xbox_live(access_token: &str) -> Result<XboxLiveAuth, AppError> {
    let client = reqwest::Client::new();
    
    let response = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
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
    
    let xbox_response: serde_json::Value = response.json().await?;
    
    let token = xbox_response["Token"]
        .as_str()
        .ok_or_else(|| AppError::OAuth("Failed to get Xbox Live token".to_string()))?
        .to_string();
    
    let user_hash = xbox_response["DisplayClaims"]["xui"][0]["uhs"]
        .as_str()
        .ok_or_else(|| AppError::OAuth("Failed to get user hash".to_string()))?
        .to_string();
    
    Ok(XboxLiveAuth { token, user_hash })
}

async fn get_xsts_token(xbox_token: &str) -> Result<String, AppError> {
    let client = reqwest::Client::new();
    
    let response = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbox_token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        }))
        .send()
        .await?;
    
    let xsts_response: serde_json::Value = response.json().await?;
    
    let token = xsts_response["Token"]
        .as_str()
        .ok_or_else(|| AppError::OAuth("Failed to get XSTS token".to_string()))?
        .to_string();
    
    Ok(token)
}

async fn authenticate_minecraft(xsts_token: &str, user_hash: &str) -> Result<String, AppError> {
    let client = reqwest::Client::new();
    
    let response = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "identityToken": format!("XBL3.0 x={};{}", user_hash, xsts_token)
        }))
        .send()
        .await?;
    
    let minecraft_response: serde_json::Value = response.json().await?;
    
    let access_token = minecraft_response["access_token"]
        .as_str()
        .ok_or_else(|| AppError::OAuth("Failed to get Minecraft access token".to_string()))?
        .to_string();
    
    Ok(access_token)
}

async fn get_minecraft_profile(access_token: &str) -> Result<MinecraftProfile, AppError> {
    let client = reqwest::Client::new();
    
    let response = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    
    let profile: MinecraftProfile = response.json().await?;
    Ok(profile)
}

#[tauri::command]
pub async fn refresh_minecraft_token(account_id: String) -> Result<MicrosoftAccount, String> {
    use crate::settings::{load_settings, save_settings};
    
    let mut settings = load_settings().await?;
    
    let account = settings.accounts
        .iter_mut()
        .find(|acc| acc.id == account_id)
        .ok_or("Account not found")?;
    
    // Use refresh token to get new access token
    let client = reqwest::Client::new();
    
    let token_response = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
        .form(&[
            ("client_id", CLIENT_ID),
            ("refresh_token", &account.refresh_token),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<OAuthTokenResponse>()
        .await
        .map_err(|e| e.to_string())?;

    // Re-authenticate with Xbox Live and Minecraft
    let xbox_auth = authenticate_xbox_live(&token_response.access_token).await.map_err(|e| e.to_string())?;
    let xsts_token = get_xsts_token(&xbox_auth.token).await.map_err(|e| e.to_string())?;
    let minecraft_token = authenticate_minecraft(&xsts_token, &xbox_auth.user_hash).await.map_err(|e| e.to_string())?;
    
    account.access_token = minecraft_token;
    account.refresh_token = token_response.refresh_token;
    account.expires_at = Utc::now().timestamp() + token_response.expires_in;
    account.last_used = Utc::now().timestamp();
    
    let account_clone = account.clone();
    save_settings(settings).await?;
    
    Ok(account_clone)
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
