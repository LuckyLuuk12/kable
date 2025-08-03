use crate::auth::auth_util::{
    get_client_id, write_launcher_account, LauncherAccount, MinecraftProfile,
};
use crate::commands::system::open_url;
use crate::logging::{LogLevel, Logger};
use chrono::{DateTime, Utc};
use minecraft_msa_auth::MinecraftAuthorizationFlow;
/**
 * This file contains Microsoft Authentication backend logic using the minecraft-msa-auth crate.
 * @see https://github.com/minecraft-rs/minecraft-msa-auth/blob/826a6846d4e1109a7acfa1a989aa77533aa01fc9/examples/device_flow.rs
 */
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, ClientId, DeviceAuthorizationUrl,
    Scope, StandardDeviceAuthorizationResponse, TokenResponse, TokenUrl,
};
use once_cell;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri;

const DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const MSA_AUTHORIZE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const MSA_TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";

// Global state to store device authorization responses for polling
static DEVICE_AUTH_STORAGE: once_cell::sync::Lazy<
    Arc<Mutex<HashMap<String, StandardDeviceAuthorizationResponse>>>,
> = once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MicrosoftToken {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinecraftToken {
    pub access_token: String,
    pub username: String,
    pub uuid: String,
}

/// Start the Microsoft device code authentication flow
#[tauri::command]
pub async fn start_microsoft_device_auth() -> Result<DeviceCodeResponse, String> {
    Logger::console_log(
        LogLevel::Info,
        "🔐 Starting Microsoft device code authentication...",
        None,
    );

    // Load client ID from environment
    let client_id = get_client_id()?;

    Logger::console_log(
        LogLevel::Debug,
        &format!("🔧 Using client ID: {}", client_id),
        None,
    );

    Logger::console_log(
        LogLevel::Debug,
        &format!("🌐 Device code URL: {}", DEVICE_CODE_URL),
        None,
    );
    Logger::console_log(
        LogLevel::Debug,
        &format!("🌐 Auth URL: {}", MSA_AUTHORIZE_URL),
        None,
    );
    Logger::console_log(
        LogLevel::Debug,
        &format!("🌐 Token URL: {}", MSA_TOKEN_URL),
        None,
    );

    let client = BasicClient::new(
        ClientId::new(client_id),
        None,
        AuthUrl::new(MSA_AUTHORIZE_URL.to_string()).map_err(|e| {
            Logger::console_log(
                LogLevel::Error,
                &format!("❌ Failed to create auth URL: {}", e),
                None,
            );
            format!("Failed to create auth URL: {}", e)
        })?,
        Some(TokenUrl::new(MSA_TOKEN_URL.to_string()).map_err(|e| {
            Logger::console_log(
                LogLevel::Error,
                &format!("❌ Failed to create token URL: {}", e),
                None,
            );
            format!("Failed to create token URL: {}", e)
        })?),
    )
    .set_device_authorization_url(
        DeviceAuthorizationUrl::new(DEVICE_CODE_URL.to_string()).map_err(|e| {
            Logger::console_log(
                LogLevel::Error,
                &format!("❌ Failed to create device code URL: {}", e),
                None,
            );
            format!("Failed to create device code URL: {}", e)
        })?,
    );

    Logger::console_log(
        LogLevel::Debug,
        "🔄 Requesting device code from Microsoft...",
        None,
    );

    let details: StandardDeviceAuthorizationResponse = client
        .exchange_device_code()
        .map_err(|e| {
            Logger::console_log(
                LogLevel::Error,
                &format!("❌ Failed to initiate device code flow: {}", e),
                None,
            );
            format!("Failed to initiate device code flow: {}", e)
        })?
        .add_scope(Scope::new("XboxLive.signin offline_access".to_string()))
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            Logger::console_log(
                LogLevel::Error,
                &format!("❌ Failed to request device code: {}", e),
                None,
            );
            format!("Failed to request device code: {}", e)
        })?;

    let response = DeviceCodeResponse {
        device_code: details.device_code().secret().to_string(),
        user_code: details.user_code().secret().to_string(),
        verification_uri: details.verification_uri().to_string(),
        expires_in: details.expires_in().as_secs(),
        interval: details.interval().as_secs(),
    };

    // Debug: Log device code response details
    Logger::console_log(LogLevel::Debug, "📋 Device code response:", None);
    Logger::console_log(
        LogLevel::Debug,
        &format!("  - User code: {}", response.user_code),
        None,
    );
    Logger::console_log(
        LogLevel::Debug,
        &format!("  - Verification URI: {}", response.verification_uri),
        None,
    );
    Logger::console_log(
        LogLevel::Debug,
        &format!("  - Expires in: {} seconds", response.expires_in),
        None,
    );
    Logger::console_log(
        LogLevel::Debug,
        &format!("  - Polling interval: {} seconds", response.interval),
        None,
    );
    Logger::console_log(
        LogLevel::Debug,
        &format!(
            "  - Device code length: {} chars",
            response.device_code.len()
        ),
        None,
    );

    // Store the device authorization response for polling
    {
        let mut storage = DEVICE_AUTH_STORAGE.lock().unwrap();
        storage.insert(response.device_code.clone(), details);
        Logger::console_log(
            LogLevel::Debug,
            &format!(
                "💾 Stored device auth response for device code: {}...",
                &response.device_code[..std::cmp::min(8, response.device_code.len())]
            ),
            None,
        );
    }

    Logger::console_log(
        LogLevel::Info,
        &format!(
            "🌐 Device code generated. User code: {}",
            response.user_code
        ),
        None,
    );

    // Automatically open the verification URL
    Logger::console_log(
        LogLevel::Debug,
        &format!(
            "🌐 Attempting to open verification URL: {}",
            response.verification_uri
        ),
        None,
    );

    if let Err(e) = open_url(response.verification_uri.clone()).await {
        Logger::console_log(
            LogLevel::Warning,
            &format!("⚠️ Failed to open browser automatically: {}", e),
            None,
        );
    } else {
        Logger::console_log(LogLevel::Debug, "✅ Browser opened successfully", None);
    }

    Ok(response)
}

/// Poll for the completion of Microsoft device code authentication
#[tauri::command]
pub async fn poll_microsoft_device_auth(
    device_code: String,
) -> Result<Option<MicrosoftToken>, String> {
    Logger::console_log(
        LogLevel::Debug,
        "🔄 Polling Microsoft device authentication...",
        None,
    );

    // Debug: Log device code info
    Logger::console_log(
        LogLevel::Debug,
        &format!(
            "🔍 Polling with device code: {}...",
            &device_code[..std::cmp::min(8, device_code.len())]
        ),
        None,
    );

    // Retrieve the stored device authorization response
    let device_auth_response = {
        let storage = DEVICE_AUTH_STORAGE.lock().unwrap();
        storage.get(&device_code).cloned()
    };

    let details = match device_auth_response {
        Some(details) => {
            Logger::console_log(
                LogLevel::Debug,
                "✅ Found stored device authorization response",
                None,
            );
            details
        }
        None => {
            Logger::console_log(LogLevel::Error, "❌ Device code not found in storage", None);
            return Err(
                "Device code not found. Please start authentication flow first.".to_string(),
            );
        }
    };

    // Load client ID from environment
    let client_id = get_client_id()?;

    Logger::console_log(
        LogLevel::Debug,
        &format!("🔧 Using client ID for polling: {}", client_id),
        None,
    );

    let client = BasicClient::new(
        ClientId::new(client_id),
        None,
        AuthUrl::new(MSA_AUTHORIZE_URL.to_string())
            .map_err(|e| format!("Failed to create auth URL: {}", e))?,
        Some(
            TokenUrl::new(MSA_TOKEN_URL.to_string())
                .map_err(|e| format!("Failed to create token URL: {}", e))?,
        ),
    )
    .set_device_authorization_url(
        DeviceAuthorizationUrl::new(DEVICE_CODE_URL.to_string())
            .map_err(|e| format!("Failed to create device code URL: {}", e))?,
    );

    Logger::console_log(LogLevel::Debug, "🌐 Making token exchange request...", None);

    // Try to exchange the device code for a token (non-blocking)
    match client
        .exchange_device_access_token(&details)
        .request_async(async_http_client, tokio::time::sleep, None)
        .await
    {
        Ok(token) => {
            // Authentication successful!
            Logger::console_log(LogLevel::Info, "🎉 OAuth2 token exchange successful!", None);

            let expires_in_secs = token.expires_in().map_or(3600, |d| d.as_secs() as i64);
            let expires_at = Utc::now() + chrono::Duration::seconds(expires_in_secs);

            Logger::console_log(
                LogLevel::Debug,
                &format!(
                    "🕒 Token expires in {} seconds (at {})",
                    expires_in_secs, expires_at
                ),
                None,
            );

            let ms_token = MicrosoftToken {
                access_token: token.access_token().secret().to_string(),
                expires_at,
            };

            // Debug: Log token info (without sensitive data)
            Logger::console_log(
                LogLevel::Debug,
                &format!(
                    "✅ Microsoft token obtained - length: {} chars",
                    ms_token.access_token.len()
                ),
                None,
            );

            // Clean up stored device auth response
            {
                let mut storage = DEVICE_AUTH_STORAGE.lock().unwrap();
                storage.remove(&device_code);
                Logger::console_log(LogLevel::Debug, "🧹 Cleaned up device auth storage", None);
            }

            Logger::console_log(
                LogLevel::Info,
                "✅ Microsoft authentication successful!",
                None,
            );
            Ok(Some(ms_token))
        }
        Err(oauth2::RequestTokenError::ServerResponse(ref err)) => {
            // Check for "authorization_pending" which means user hasn't completed auth yet
            if err.error().as_ref() == "authorization_pending" {
                Logger::console_log(LogLevel::Debug, "⏳ Authorization still pending...", None);
                Ok(None) // Continue polling
            } else if err.error().as_ref() == "slow_down" {
                Logger::console_log(
                    LogLevel::Debug,
                    "🐌 Polling too fast, slowing down...",
                    None,
                );
                Ok(None) // Continue polling but slower
            } else if err.error().as_ref() == "expired_token" {
                Logger::console_log(LogLevel::Warning, "⏰ Device code has expired", None);
                // Clean up expired token
                {
                    let mut storage = DEVICE_AUTH_STORAGE.lock().unwrap();
                    storage.remove(&device_code);
                }
                Err("Device code has expired. Please start authentication flow again.".to_string())
            } else {
                let error_desc = err
                    .error_description()
                    .map(|s| s.as_str())
                    .unwrap_or("No description");
                Logger::console_log(
                    LogLevel::Error,
                    &format!("❌ OAuth2 server error: {} - {}", err.error(), error_desc),
                    None,
                );
                Err(format!("Authentication failed: {}", err.error()))
            }
        }
        Err(e) => {
            Logger::console_log(
                LogLevel::Error,
                &format!("❌ Network or other error during polling: {}", e),
                None,
            );
            Err(format!("Polling failed: {}", e))
        }
    }
}

/// Exchange Microsoft token for Minecraft token and save account
#[tauri::command]
pub async fn complete_minecraft_auth(
    microsoft_token: MicrosoftToken,
) -> Result<LauncherAccount, String> {
    Logger::console_log(
        LogLevel::Info,
        "🎮 Exchanging Microsoft token for Minecraft access...",
        None,
    );

    // Debug: Log the Microsoft token details (without sensitive data)
    Logger::console_log(
        LogLevel::Debug,
        &format!(
            "🔍 Microsoft token expires at: {}",
            microsoft_token.expires_at
        ),
        None,
    );
    Logger::console_log(
        LogLevel::Debug,
        &format!(
            "🔍 Microsoft token length: {} chars",
            microsoft_token.access_token.len()
        ),
        None,
    );

    let mc_flow = MinecraftAuthorizationFlow::new(Client::new());

    // Debug: Log before exchange
    Logger::console_log(
        LogLevel::Debug,
        "🔄 Starting Microsoft to Minecraft token exchange...",
        None,
    );

    let mc_token = mc_flow
        .exchange_microsoft_token(&microsoft_token.access_token)
        .await
        .map_err(|e| {
            Logger::console_log(
                LogLevel::Error,
                &format!("❌ Failed to exchange Microsoft token: {}", e),
                None,
            );
            format!("Failed to exchange for Minecraft token: {}", e)
        })?;

    // Debug: Log detailed Minecraft token information
    Logger::console_log(LogLevel::Debug, "🎮 Minecraft token details:", None);
    Logger::console_log(
        LogLevel::Debug,
        &format!("  - Username (actually UUID): '{}'", mc_token.username()),
        None,
    );
    Logger::console_log(
        LogLevel::Debug,
        &format!(
            "  - Access token length: {} chars",
            mc_token.access_token().as_ref().len()
        ),
        None,
    );

    // Get the UUID from the minecraft auth response (XUID, not MC UUID)
    let xuid = mc_token.username().clone();
    let access_token = mc_token.access_token().as_ref().to_string();

    Logger::console_log(LogLevel::Debug, &format!("🆔 XUID: {}", xuid), None);

    // Make API call to get the actual Minecraft profile information
    Logger::console_log(
        LogLevel::Debug,
        "🌐 Fetching Minecraft profile from API...",
        None,
    );
    let profile_url = "https://api.minecraftservices.com/minecraft/profile";

    let profile_response = reqwest::Client::new()
        .get(profile_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| {
            Logger::console_log(
                LogLevel::Error,
                &format!("❌ Failed to fetch Minecraft profile: {}", e),
                None,
            );
            format!("Failed to fetch Minecraft profile: {}", e)
        })?;

    Logger::console_log(
        LogLevel::Debug,
        &format!(
            "📡 Profile API response status: {}",
            profile_response.status()
        ),
        None,
    );

    if !profile_response.status().is_success() {
        let status = profile_response.status();
        let error_text = profile_response.text().await.unwrap_or_default();
        Logger::console_log(
            LogLevel::Error,
            &format!("❌ Profile API error {}: {}", status, error_text),
            None,
        );
        return Err(format!("Profile API error {}: {}", status, error_text));
    }

    let profile_json: serde_json::Value = profile_response.json().await.map_err(|e| {
        Logger::console_log(
            LogLevel::Error,
            &format!("❌ Failed to parse profile JSON: {}", e),
            None,
        );
        format!("Failed to parse profile JSON: {}", e)
    })?;

    Logger::console_log(
        LogLevel::Debug,
        &format!("📋 Profile JSON: {}", profile_json),
        None,
    );

    // Extract the actual username and UUID from the profile response
    let actual_username = profile_json["name"].as_str().unwrap_or("").to_string();
    let final_uuid = profile_json["id"].as_str().unwrap_or("").to_string();

    Logger::console_log(
        LogLevel::Info,
        &format!(
            "✅ Got Minecraft profile - Username: {}, UUID: {}",
            actual_username, final_uuid
        ),
        None,
    );

    Logger::console_log(
        LogLevel::Info,
        &format!(
            "✅ Minecraft authentication successful - User: '{}', UUID: '{}'",
            actual_username, final_uuid
        ),
        None,
    );

    // Create a LauncherAccount from the Minecraft token with proper profile data
    let account = LauncherAccount {
        access_token: access_token.clone(),
        access_token_expires_at: microsoft_token.expires_at.to_rfc3339(),
        avatar: format!("https://crafatar.com/avatars/{}?size=64", final_uuid),
        eligible_for_free_trials: true,
        eligible_for_migration: false,
        franchise_inventory_id: "1/Mg==".to_string(),
        has_multiple_profiles: false,
        in_forced_migration: false,
        legacy: false,
        license_product_ids: vec![],
        local_id: final_uuid.clone(), // MC UUID
        minecraft_profile: MinecraftProfile {
            id: final_uuid.clone(),        // MC UUID
            name: actual_username.clone(), // MC username
            requires_profile_name_change: false,
            requires_skin_change: false,
        },
        persistent: true,
        remote_id: xuid.clone(), // XUID from Microsoft token
        account_type: "Xbox".to_string(),
        user_properties: vec![],
        username: actual_username.clone(), // MC username
    };

    // Debug: Log the account structure before saving
    Logger::console_log(LogLevel::Debug, "💾 About to save account:", None);
    Logger::console_log(
        LogLevel::Debug,
        &format!("  - Username: '{}'", account.username),
        None,
    );
    Logger::console_log(
        LogLevel::Debug,
        &format!("  - Local ID: '{}'", account.local_id),
        None,
    );
    Logger::console_log(
        LogLevel::Debug,
        &format!("  - Profile Name: '{}'", account.minecraft_profile.name),
        None,
    );
    Logger::console_log(
        LogLevel::Debug,
        &format!("  - Profile ID: '{}'", account.minecraft_profile.id),
        None,
    );
    Logger::console_log(
        LogLevel::Debug,
        &format!("  - Account Type: '{}'", account.account_type),
        None,
    );

    // Save the account using auth_util
    write_launcher_account(account.clone()).await.map_err(|e| {
        Logger::console_log(
            LogLevel::Error,
            &format!("❌ Failed to save account: {}", e),
            None,
        );
        format!("Failed to save account: {}", e)
    })?;

    Logger::console_log(
        LogLevel::Info,
        &format!("💾 Account saved successfully: {}", account.username),
        None,
    );

    Ok(account)
}
