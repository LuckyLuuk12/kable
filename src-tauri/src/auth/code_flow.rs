/**
 * This file contains Microsoft Authentication backend logic using the minecraft-msa-auth crate.
 * @see https://github.com/minecraft-rs/minecraft-msa-auth/blob/826a6846d4e1109a7acfa1a989aa77533aa01fc9/examples/auth_code_flow.rs
 */

use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, ClientId, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenUrl, AuthorizationCode, TokenResponse
};
use minecraft_msa_auth::MinecraftAuthorizationFlow;
use reqwest::Client;
use tauri;
use serde::{Deserialize, Serialize};
use crate::logging::{Logger, LogLevel};
use crate::auth::auth_util::{open_url, write_launcher_account, LauncherAccount, MinecraftProfile, get_client_id, get_redirect_uri, get_oauth_port};
use chrono::{Utc, DateTime};
use uuid;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use once_cell::sync::Lazy;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use url::Url;

const MSA_AUTHORIZE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const MSA_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";

// Global state to store PKCE verifiers for the authorization code flow
static AUTH_STATE_STORAGE: Lazy<Arc<Mutex<HashMap<String, PkceCodeVerifier>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthCodeResponse {
    pub auth_url: String,
    pub state: String,
    pub local_server_port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MicrosoftToken {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
}

/// Start the Microsoft authorization code authentication flow
#[tauri::command]
pub async fn start_microsoft_auth_code() -> Result<AuthCodeResponse, String> {
    Logger::console_log(LogLevel::Info, "🔐 Starting Microsoft authorization code authentication...", None);
    
    // Load environment variables
    let client_id = get_client_id()?;
    let redirect_uri = get_redirect_uri()?;
    let oauth_port = get_oauth_port();
    
    Logger::console_log(
        LogLevel::Debug, 
        &format!("🔧 Using client ID: {}, redirect URI: {}, port: {}", 
                client_id, redirect_uri, oauth_port), 
        None
    );

    let client = BasicClient::new(
        ClientId::new(client_id),
        None,
        AuthUrl::new(MSA_AUTHORIZE_URL.to_string())
            .map_err(|e| format!("Failed to create auth URL: {}", e))?,
        Some(TokenUrl::new(MSA_TOKEN_URL.to_string())
            .map_err(|e| format!("Failed to create token URL: {}", e))?),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri.clone())
        .map_err(|e| format!("Failed to create redirect URI: {}", e))?);

    // Generate PKCE challenge and verifier
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate authorization URL
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("XboxLive.signin offline_access".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    let state = csrf_token.secret().to_string();

    // Store the PKCE verifier for later use
    {
        let mut storage = AUTH_STATE_STORAGE.lock().unwrap();
        storage.insert(state.clone(), pkce_verifier);
    }

    Logger::console_log(
        LogLevel::Info, 
        &format!("🌐 Authorization URL generated. State: {}", state), 
        None
    );

    // Start local server to listen for callback
    let bind_address = format!("127.0.0.1:{}", oauth_port);
    let listener = TcpListener::bind(&bind_address).await
        .map_err(|e| format!("Failed to start local server on {}: {}", bind_address, e))?;
    
    let local_port = listener.local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?
        .port();

    Logger::console_log(
        LogLevel::Info, 
        &format!("👂 Local callback server started on port: {}", local_port), 
        None
    );

    // Spawn background task to handle the callback
    let state_for_task = state.clone();
    let client_id_for_task = get_client_id()?;
    let redirect_uri_for_task = redirect_uri.clone();
    tokio::spawn(async move {
        if let Err(e) = handle_auth_callback(listener, state_for_task, client_id_for_task, redirect_uri_for_task).await {
            Logger::console_log(LogLevel::Error, &format!("❌ Callback handler error: {}", e), None);
        }
    });

    // Automatically open the authorization URL
    if let Err(e) = open_url(auth_url.to_string()).await {
        Logger::console_log(
            LogLevel::Warning, 
            &format!("⚠️ Failed to open browser automatically: {}", e), 
            None
        );
    }

    Ok(AuthCodeResponse {
        auth_url: auth_url.to_string(),
        state,
        local_server_port: local_port,
    })
}

/// Handle the OAuth callback from the local server
async fn handle_auth_callback(
    listener: TcpListener, 
    expected_state: String,
    client_id: String,
    redirect_uri: String
) -> Result<(), String> {
    Logger::console_log(LogLevel::Info, "👂 Listening for OAuth callback...", None);

    // Accept one connection
    let (mut stream, _) = listener.accept().await
        .map_err(|e| format!("Failed to accept connection: {}", e))?;

    // Read the HTTP request
    let mut buffer = [0; 4096];
    let bytes_read = stream.read(&mut buffer).await
        .map_err(|e| format!("Failed to read request: {}", e))?;

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    
    // Parse the request line to get the URL
    let request_line = request.lines().next()
        .ok_or("Invalid HTTP request")?;
    
    let url_part = request_line.split_whitespace().nth(1)
        .ok_or("Invalid HTTP request format")?;

    // Send success response using callback.html from the SvelteKit app directory
    use std::fs;
    let callback_html_path = "src/callback.html"; // SvelteKit's callback.html location
    let html_content = fs::read_to_string(callback_html_path)
        .unwrap_or_else(|_| "<html><body><h1>✅ Authentication successful!</h1><p>You can close this window and return to the application.</p><script>window.close();</script></body></html>".to_string());
    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}", html_content);
    stream.write_all(response.as_bytes()).await
        .map_err(|e| format!("Failed to send response: {}", e))?;

    // Parse the callback URL
    let parsed_url = if url_part.starts_with("http") {
        Url::parse(url_part)
    } else {
        Url::parse(&format!("http://127.0.0.1{}", url_part))
    }.map_err(|e| format!("Failed to parse callback URL: {}", e))?;

    // Extract parameters
    let mut auth_code = None;
    let mut state = None;
    let mut error = None;

    for (key, value) in parsed_url.query_pairs() {
        match key.as_ref() {
            "code" => auth_code = Some(value.to_string()),
            "state" => state = Some(value.to_string()),
            "error" => error = Some(value.to_string()),
            _ => {}
        }
    }

    if let Some(error_msg) = error {
        return Err(format!("OAuth error: {}", error_msg));
    }

    let auth_code = auth_code.ok_or("No authorization code received")?;
    let received_state = state.ok_or("No state parameter received")?;

    // Verify state parameter
    if received_state != expected_state {
        return Err("State parameter mismatch - possible CSRF attack".to_string());
    }

    Logger::console_log(LogLevel::Info, "✅ Authorization code received successfully", None);

    // Exchange authorization code for tokens
    if let Err(e) = exchange_auth_code_for_tokens(auth_code, received_state, client_id, redirect_uri).await {
        Logger::console_log(LogLevel::Error, &format!("❌ Token exchange failed: {}", e), None);
        return Err(e);
    }

    Ok(())
}

/// Exchange authorization code for Microsoft access token and then Minecraft token
async fn exchange_auth_code_for_tokens(
    auth_code: String, 
    state: String,
    client_id: String,
    redirect_uri: String
) -> Result<(), String> {
    Logger::console_log(LogLevel::Info, "🔄 Exchanging authorization code for tokens...", None);
    Logger::console_log(LogLevel::Debug, &format!("📋 Client ID: {}", client_id), None);
    Logger::console_log(LogLevel::Debug, &format!("🔄 Redirect URI: {}", redirect_uri), None);
    Logger::console_log(LogLevel::Debug, &format!("📝 Auth code length: {}", auth_code.len()), None);

    // Retrieve the PKCE verifier
    let pkce_verifier = {
        let mut storage = AUTH_STATE_STORAGE.lock().unwrap();
        storage.remove(&state)
            .ok_or("PKCE verifier not found for state")?
    };

    Logger::console_log(LogLevel::Debug, "🔐 PKCE verifier retrieved successfully", None);

    // Create OAuth2 client configuration
    Logger::console_log(LogLevel::Debug, &format!("🔧 Creating OAuth2 client with ID: {}", client_id), None);
    Logger::console_log(LogLevel::Debug, &format!("🔗 Redirect URI: {}", redirect_uri), None);
    Logger::console_log(LogLevel::Debug, &format!("🌐 Auth URL: {}", MSA_AUTHORIZE_URL), None);
    Logger::console_log(LogLevel::Debug, &format!("🔑 Token URL: {}", MSA_TOKEN_URL), None);

    let client = BasicClient::new(
        ClientId::new(client_id.clone()),
        None, // No client secret for PKCE flow
        AuthUrl::new(MSA_AUTHORIZE_URL.to_string())
            .map_err(|e| format!("Failed to create auth URL: {}", e))?,
        Some(TokenUrl::new(MSA_TOKEN_URL.to_string())
            .map_err(|e| format!("Failed to create token URL: {}", e))?),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri.clone())
        .map_err(|e| format!("Failed to create redirect URI: {}", e))?);

    Logger::console_log(LogLevel::Debug, "🔧 OAuth2 client configured successfully", None);

    // Exchange authorization code for access token
    Logger::console_log(LogLevel::Debug, "📤 Starting token exchange request...", None);
    Logger::console_log(LogLevel::Debug, &format!("📋 Using client_id: {}", client_id), None);
    Logger::console_log(LogLevel::Debug, &format!("🔗 Using redirect_uri: {}", redirect_uri), None);
    Logger::console_log(LogLevel::Debug, &format!("🔑 Auth code: {}", auth_code), None);
    
    let token_result = client
        .exchange_code(AuthorizationCode::new(auth_code))
        .set_pkce_verifier(pkce_verifier)
        .add_extra_param("client_id", &client_id)  // Explicitly add client_id for Microsoft
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            Logger::console_log(LogLevel::Error, &format!("❌ Token exchange failed: {}", e), None);
            Logger::console_log(LogLevel::Error, &format!("❌ Client configuration - ID: {}, Redirect: {}", client_id, redirect_uri), None);
            format!("Failed to exchange authorization code: {}", e)
        })?;

    let microsoft_token = MicrosoftToken {
        access_token: token_result.access_token().secret().to_string(),
        expires_at: Utc::now() + chrono::Duration::seconds(
            token_result.expires_in()
                .map(|d| d.as_secs() as i64)
                .unwrap_or(3600)
        ),
    };

    Logger::console_log(LogLevel::Info, "✅ Microsoft access token obtained", None);

    // Now exchange for Minecraft token
    complete_minecraft_auth_code(microsoft_token).await?;

    Ok(())
}

/// Complete the Minecraft authentication using the Microsoft token (Authorization Code Flow)
#[tauri::command]
pub async fn complete_minecraft_auth_code(microsoft_token: MicrosoftToken) -> Result<LauncherAccount, String> {
    Logger::console_log(LogLevel::Info, "🎮 Completing Minecraft authentication...", None);

    let mc_flow = MinecraftAuthorizationFlow::new(Client::new());
    let mc_token = mc_flow
        .exchange_microsoft_token(&microsoft_token.access_token)
        .await
        .map_err(|e| format!("Failed to exchange Microsoft token for Minecraft token: {}", e))?;

    Logger::console_log(
        LogLevel::Info, 
        &format!("✅ Minecraft token obtained for user: {}", mc_token.username()), 
        None
    );

    // Get the profile UUID from the minecraft token - using a simple UUID generation for now
    let profile_uuid = uuid::Uuid::new_v4().to_string().replace("-", "");

    // Create LauncherAccount with proper structure
    let launcher_account = LauncherAccount {
        access_token: mc_token.access_token().clone().into_inner(),
        access_token_expires_at: microsoft_token.expires_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        avatar: format!("https://crafatar.com/avatars/{}?size=64&default=MHF_Steve&overlay", profile_uuid),
        eligible_for_free_trials: true,
        eligible_for_migration: false,
        franchise_inventory_id: "1/OQ==".to_string(),
        has_multiple_profiles: false,
        in_forced_migration: false,
        legacy: false,
        license_product_ids: vec![],
        local_id: profile_uuid.clone(),
        minecraft_profile: MinecraftProfile {
            id: profile_uuid.clone(),
            name: mc_token.username().clone(),
            requires_profile_name_change: false,
            requires_skin_change: false,
        },
        persistent: true,
        remote_id: profile_uuid.clone(),
        account_type: "Xbox".to_string(),
        user_properties: vec![],
        username: mc_token.username().clone(),
    };

    // Save the account using auth_util
    write_launcher_account(launcher_account.clone()).await
        .map_err(|e| format!("Failed to save launcher account: {}", e))?;

    Logger::console_log(
        LogLevel::Info, 
        &format!("✅ Account saved successfully: {}", launcher_account.username), 
        None
    );

    Ok(launcher_account)
}

/// Poll for authentication completion (not needed for auth code flow, but kept for compatibility)
#[tauri::command]
pub async fn poll_microsoft_auth_code(_state: String) -> Result<Option<MicrosoftToken>, String> {
    // In auth code flow, polling is not needed as the callback handles everything
    // This function is kept for compatibility with the frontend
    Logger::console_log(LogLevel::Debug, "ℹ️ Auth code flow doesn't require polling", None);
    Ok(None)
}
