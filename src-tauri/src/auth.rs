/**
 * Authentication module - public API that provides
 * 1. get_minecraft_account()
 * 2. authenticate()
 */
pub mod auth_util;
pub mod code_flow;
pub mod device_code_flow;
pub mod secure_token;

// Re-export the auth_util functions and types for convenience
pub use auth_util::{
    get_active_launcher_account, get_all_launcher_accounts, get_launcher_accounts_path_string,
    read_launcher_accounts, remove_launcher_account, set_active_launcher_account,
    write_launcher_account, write_launcher_accounts, LauncherAccount, LauncherAccountsJson,
    MinecraftProfile,
};

use crate::logging::{LogLevel, Logger};
use serde::{Deserialize, Serialize};

/// Authentication method to use for getting accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    /// Use the device_code_flow.rs implementation (Device Code Flow)
    DeviceCodeFlow,
    /// Use the code_flow.rs implementation (Authorization Code Flow)
    AuthCodeFlow,
    /// Use the custom_auth.rs implementation (custom OAuth2 flow) - TODO: implement
    Custom,
    /// Use offline/mock authentication for testing
    Offline,
}

/// Get the current authenticated Minecraft account for launching games
/// This is the main function that other backend files should use
#[tauri::command]
pub async fn get_minecraft_account(
    auth_method: Option<AuthMethod>,
) -> Result<LauncherAccount, String> {
    let method = auth_method.unwrap_or(AuthMethod::DeviceCodeFlow);

    Logger::console_log(
        LogLevel::Info,
        &format!("🔍 Getting Minecraft account using method: {:?}", method),
        None,
    );

    match method {
        AuthMethod::DeviceCodeFlow => get_minecraft_account_device_code().await,
        AuthMethod::AuthCodeFlow => get_minecraft_account_auth_code().await,
        AuthMethod::Custom => {
            // TODO: Implement custom authentication
            Err("Custom authentication not yet implemented".to_string())
        }
        AuthMethod::Offline => get_minecraft_account_offline().await,
    }
}

/// Get account using the device_code_flow.rs implementation
async fn get_minecraft_account_device_code() -> Result<LauncherAccount, String> {
    Logger::console_log(
        LogLevel::Debug,
        "� Using device code flow authentication",
        None,
    );

    // Try to get the active launcher account
    match get_active_launcher_account().await? {
        Some(launcher_account) => {
            // Sanitize access token before logging
            let mut sanitized_account = launcher_account.clone();
            sanitized_account.access_token = String::new();
            Logger::console_log(
                LogLevel::Info,
                &format!("✅ Found active account: {:?}", sanitized_account),
                None,
            );

            // Check if access token is still valid
            if is_access_token_valid(&launcher_account) {
                Logger::console_log(LogLevel::Info, "🔑 Access token is still valid", None);
                Ok(launcher_account)
            } else {
                Logger::console_log(
                    LogLevel::Warning,
                    "⚠️ Access token expired, need to re-authenticate",
                    None,
                );
                Err("Access token expired. Please authenticate again.".to_string())
            }
        }
        None => {
            Logger::console_log(LogLevel::Warning, "❌ No active account found", None);
            Err("No authenticated account found. Please sign in first.".to_string())
        }
    }
}

/// Get account using the code_flow.rs implementation
async fn get_minecraft_account_auth_code() -> Result<LauncherAccount, String> {
    Logger::console_log(
        LogLevel::Debug,
        "🌐 Using authorization code flow authentication",
        None,
    );

    // Try to get the active launcher account
    match get_active_launcher_account().await? {
        Some(launcher_account) => {
            // Sanitize access token before logging
            let mut sanitized_account = launcher_account.clone();
            sanitized_account.access_token = String::new();
            Logger::console_log(
                LogLevel::Info,
                &format!("✅ Found active account: {:?}", sanitized_account),
                None,
            );
            // Check if access token is still valid
            if is_access_token_valid(&launcher_account) {
                Logger::console_log(LogLevel::Info, "🔑 Access token is still valid", None);
                Ok(launcher_account)
            } else {
                Logger::console_log(
                    LogLevel::Warning,
                    "⚠️ Access token expired, attempting re-authentication",
                    None,
                );
                // Attempt to re-authenticate using code flow
                match crate::auth::code_flow::start_microsoft_auth_code().await {
                    Ok(_) => {
                        // After successful re-auth, try to get the account again
                        match get_active_launcher_account().await? {
                            Some(new_account) => Ok(new_account),
                            None => Err("Re-authentication failed, no account found.".to_string()),
                        }
                    }
                    Err(e) => Err(format!("Re-authentication failed: {}", e)),
                }
            }
        }
        None => {
            Logger::console_log(LogLevel::Warning, "❌ No active account found", None);
            // Attempt to authenticate if no account is found
            match crate::auth::code_flow::start_microsoft_auth_code().await {
                Ok(_) => match get_active_launcher_account().await? {
                    Some(new_account) => Ok(new_account),
                    None => Err("Authentication failed, no account found.".to_string()),
                },
                Err(e) => Err(format!("Authentication failed: {}", e)),
            }
        }
    }
}

/// Get account using offline/mock authentication (for testing)
async fn get_minecraft_account_offline() -> Result<LauncherAccount, String> {
    Logger::console_log(
        LogLevel::Warning,
        "📴 Using offline authentication mode",
        None,
    );
    // Try to get the active launcher account from launcher_accounts.json
    match get_active_launcher_account().await? {
        Some(launcher_account) => {
            // Sanitize access token before logging
            let mut sanitized_account = launcher_account.clone();
            sanitized_account.access_token = String::new();
            Logger::console_log(
                LogLevel::Info,
                &format!(
                    "✅ Found active account (offline mode): {:?}",
                    sanitized_account
                ),
                None,
            );
            // Return the account, even if it has no access token
            Ok(launcher_account)
        }
        None => {
            Logger::console_log(
                LogLevel::Warning,
                "❌ No active account found, returning fallback offline account",
                None,
            );
            Ok(LauncherAccount {
                access_token: "".to_string(),
                access_token_expires_at: "1970-01-01T00:00:00Z".to_string(),
                avatar: String::new(),
                eligible_for_free_trials: false,
                eligible_for_migration: false,
                franchise_inventory_id: String::new(),
                has_multiple_profiles: false,
                in_forced_migration: false,
                legacy: false,
                license_product_ids: vec![],
                local_id: String::new(),
                minecraft_profile: MinecraftProfile {
                    id: "00000000-0000-0000-0000-000000000000".to_string(),
                    name: "OfflinePlayer".to_string(),
                    requires_profile_name_change: false,
                    requires_skin_change: false,
                },
                persistent: false,
                remote_id: String::new(),
                account_type: "Offline".to_string(),
                user_properties: vec![],
                username: "OfflinePlayer".to_string(),
            })
        }
    }
}

/// Check if an access token is still valid (not expired)
pub fn is_access_token_valid(launcher_account: &LauncherAccount) -> bool {
    use chrono::{DateTime, Utc};

    if launcher_account.access_token.is_empty() {
        return false;
    }

    // Parse the expiry time
    match DateTime::parse_from_rfc3339(&launcher_account.access_token_expires_at) {
        Ok(expires_at) => {
            let now = Utc::now();
            let expires_at_utc = expires_at.with_timezone(&Utc);

            // Consider token invalid if it expires within the next 5 minutes
            let buffer_time = chrono::Duration::minutes(5);
            expires_at_utc > (now + buffer_time)
        }
        Err(_) => {
            Logger::console_log(
                LogLevel::Warning,
                "⚠️ Could not parse access token expiry time",
                None,
            );
            false
        }
    }
}

/// Helper function for installations.rs and other files that need account data
/// This provides the same interface as the old get_launch_auth_account function
#[tauri::command]
pub async fn get_launch_auth_account() -> Result<LauncherAccount, String> {
    match get_minecraft_account(Some(AuthMethod::DeviceCodeFlow)).await {
        Ok(account) => Ok(account),
        Err(_) => {
            Logger::console_log(
                LogLevel::Warning,
                "⚠️ Device code flow auth failed, falling back to offline mode",
                None,
            );
            get_minecraft_account(Some(AuthMethod::Offline)).await
        }
    }
}

/// Force refresh the current account's access token
#[tauri::command]
pub async fn refresh_minecraft_account() -> Result<LauncherAccount, String> {
    Logger::console_log(
        LogLevel::Info,
        "🔄 Refreshing Minecraft account access token...",
        None,
    );

    // For now, this will require the user to re-authenticate
    // In the future, we could implement refresh token logic
    Err("Token refresh not yet implemented. Please sign in again.".to_string())
}
