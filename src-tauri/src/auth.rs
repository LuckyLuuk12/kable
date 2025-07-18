/**
 * Authentication module - public API that provides
 * 1. get_minecraft_account()
 * 2. authenticate()
 */

pub mod auth_util;
pub mod device_code_flow;
pub mod code_flow;

// Re-export the auth_util functions and types for convenience
pub use auth_util::{
    LauncherAccount, 
    MinecraftProfile, 
    LauncherAccountsJson,
    read_launcher_accounts,
    write_launcher_accounts,
    write_launcher_account,
    remove_launcher_account,
    set_active_launcher_account,
    get_active_launcher_account,
    get_all_launcher_accounts,
    get_launcher_accounts_path_string,
    open_url,
};

use serde::{Deserialize, Serialize};
use crate::logging::{Logger, LogLevel};

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

/// Account with access token ready for Minecraft launching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftAccount {
    pub username: String,
    pub uuid: String,
    pub access_token: Option<String>,
    pub expires_at: Option<String>,
    pub account_type: String, // "Xbox", "Mojang", "Offline"
    pub profile: MinecraftProfile,
}

impl From<LauncherAccount> for MinecraftAccount {
    fn from(launcher_account: LauncherAccount) -> Self {
        Self {
            username: launcher_account.username,
            uuid: launcher_account.minecraft_profile.id.clone(),
            access_token: Some(launcher_account.access_token),
            expires_at: Some(launcher_account.access_token_expires_at),
            account_type: launcher_account.account_type,
            profile: launcher_account.minecraft_profile,
        }
    }
}

/// Get the current authenticated Minecraft account for launching games
/// This is the main function that other backend files should use
#[tauri::command]
pub async fn get_minecraft_account(auth_method: Option<AuthMethod>) -> Result<MinecraftAccount, String> {
    let method = auth_method.unwrap_or(AuthMethod::DeviceCodeFlow);
    
    Logger::console_log(
        LogLevel::Info, 
        &format!("🔍 Getting Minecraft account using method: {:?}", method), 
        None
    );
    
    match method {
        AuthMethod::DeviceCodeFlow => {
            get_minecraft_account_device_code().await
        }
        AuthMethod::AuthCodeFlow => {
            get_minecraft_account_auth_code().await
        }
        AuthMethod::Custom => {
            // TODO: Implement custom authentication
            Err("Custom authentication not yet implemented".to_string())
        }
        AuthMethod::Offline => {
            get_minecraft_account_offline().await
        }
    }
}

/// Get account using the device_code_flow.rs implementation
async fn get_minecraft_account_device_code() -> Result<MinecraftAccount, String> {
    Logger::console_log(LogLevel::Debug, "� Using device code flow authentication", None);
    
    // Try to get the active launcher account
    match get_active_launcher_account().await? {
        Some(launcher_account) => {
            Logger::console_log(
                LogLevel::Info, 
                &format!("✅ Found active account: {:?}", launcher_account), 
                None
            );
            
            // Check if access token is still valid
            if is_access_token_valid(&launcher_account) {
                Logger::console_log(LogLevel::Info, "🔑 Access token is still valid", None);
                Ok(launcher_account.into())
            } else {
                Logger::console_log(LogLevel::Warning, "⚠️ Access token expired, need to re-authenticate", None);
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
async fn get_minecraft_account_auth_code() -> Result<MinecraftAccount, String> {
    Logger::console_log(LogLevel::Debug, "🌐 Using authorization code flow authentication", None);
    
    // Try to get the active launcher account
    match get_active_launcher_account().await? {
        Some(launcher_account) => {
            Logger::console_log(
                LogLevel::Info, 
                &format!("✅ Found active account: {:?}", launcher_account), 
                None
            );
            
            // Check if access token is still valid
            if is_access_token_valid(&launcher_account) {
                Logger::console_log(LogLevel::Info, "🔑 Access token is still valid", None);
                Ok(launcher_account.into())
            } else {
                Logger::console_log(LogLevel::Warning, "⚠️ Access token expired, need to re-authenticate", None);
                Err("Access token expired. Please authenticate again.".to_string())
            }
        }
        None => {
            Logger::console_log(LogLevel::Warning, "❌ No active account found", None);
            Err("No authenticated account found. Please sign in first.".to_string())
        }
    }
}

/// Get account using offline/mock authentication (for testing)
async fn get_minecraft_account_offline() -> Result<MinecraftAccount, String> {
    Logger::console_log(LogLevel::Warning, "📴 Using offline authentication mode", None);
    
    Ok(MinecraftAccount {
        username: "OfflinePlayer".to_string(),
        uuid: "00000000-0000-0000-0000-000000000000".to_string(),
        access_token: None,
        expires_at: None,
        account_type: "Offline".to_string(),
        profile: MinecraftProfile {
            id: "00000000-0000-0000-0000-000000000000".to_string(),
            name: "OfflinePlayer".to_string(),
            requires_profile_name_change: false,
            requires_skin_change: false,
        },
    })
}

/// Check if an access token is still valid (not expired)
fn is_access_token_valid(launcher_account: &LauncherAccount) -> bool {
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
                None
            );
            false
        }
    }
}

/// Helper function for installations.rs and other files that need account data
/// This provides the same interface as the old get_launch_auth_account function
#[tauri::command]
pub async fn get_launch_auth_account() -> Result<MinecraftAccount, String> {
    match get_minecraft_account(Some(AuthMethod::DeviceCodeFlow)).await {
        Ok(account) => Ok(account),
        Err(_) => {
            Logger::console_log(
                LogLevel::Warning, 
                "⚠️ Device code flow auth failed, falling back to offline mode", 
                None
            );
            get_minecraft_account(Some(AuthMethod::Offline)).await
        }
    }
}

/// Force refresh the current account's access token
#[tauri::command]
pub async fn refresh_minecraft_account() -> Result<MinecraftAccount, String> {
    Logger::console_log(LogLevel::Info, "🔄 Refreshing Minecraft account access token...", None);
    
    // For now, this will require the user to re-authenticate
    // In the future, we could implement refresh token logic
    Err("Token refresh not yet implemented. Please sign in again.".to_string())
}