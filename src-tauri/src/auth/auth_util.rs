/**
 * This file contains authentication related utility functions.
 * E.g. functions to read/write to/from launcher_accounts.json to work with stored accounts, 
 *      opening URLs in the browser (independent of the OS), etc.
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use tauri;
use crate::AppError;
use crate::logging::{Logger, LogLevel};
use crate::auth::secure_token::{encrypt_token, decrypt_token};


// ...existing code...
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LauncherAccount {
    pub access_token: String,
    pub access_token_expires_at: String,
    pub avatar: String,
    pub eligible_for_free_trials: bool,
    pub eligible_for_migration: bool,
    pub franchise_inventory_id: String,
    pub has_multiple_profiles: bool,
    pub in_forced_migration: bool,
    pub legacy: bool,
    pub license_product_ids: Vec<String>,
    pub local_id: String,
    pub minecraft_profile: MinecraftProfile,
    pub persistent: bool,
    pub remote_id: String,
    #[serde(rename = "type")]
    pub account_type: String,
    pub user_properties: Vec<serde_json::Value>, // Note: keeping the typo from the JSON structure
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
    pub requires_profile_name_change: bool,
    pub requires_skin_change: bool,
}
// ...existing code...
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct LauncherAccountsJson {
    pub accounts: HashMap<String, LauncherAccount>,
    pub active_account_local_id: String,
    pub mojang_client_token: String,
}

/// Get the path to the Minecraft directory
fn get_minecraft_directory() -> Result<PathBuf, AppError> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound, 
            "Could not find home directory"
        )))?;
    
    // Instead of using the Minecraft directory, use the Kable app directory
    #[cfg(target_os = "windows")]
    let kable_dir = home_dir.join("AppData").join("Roaming").join("kable-launcher");

    #[cfg(target_os = "macos")]
    let kable_dir = home_dir.join("Library").join("Application Support").join("kable-launcher");

    #[cfg(target_os = "linux")]
    let kable_dir = home_dir.join(".kable-launcher");

    Ok(kable_dir)
}

/// Get the path to the kable_accounts.json file
pub fn get_kable_accounts_path() -> Result<PathBuf, AppError> {
    let kable_dir = get_minecraft_directory()?;
    Ok(kable_dir.join("kable_accounts.json"))
}

/// Read all accounts from kable_accounts.json
#[tauri::command]
pub async fn read_launcher_accounts() -> Result<LauncherAccountsJson, String> {
    Logger::console_log(LogLevel::Info, "📖 Reading Kable accounts from file...", None);
    let accounts_path = get_kable_accounts_path()
        .map_err(|e| format!("Failed to get Kable accounts path: {}", e))?;
    Logger::console_log(LogLevel::Debug, &format!("📁 Accounts file path: {:?}", accounts_path), None);
    if !accounts_path.exists() {
        Logger::console_log(LogLevel::Warning, "⚠️ kable_accounts.json not found, returning empty structure", None);
        return Ok(LauncherAccountsJson {
            accounts: HashMap::new(),
            active_account_local_id: String::new(),
            mojang_client_token: String::new(),
        });
    }
    let content = fs::read_to_string(&accounts_path)
        .map_err(|e| format!("Failed to read kable_accounts.json: {}", e))?;
    let mut accounts: LauncherAccountsJson = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse kable_accounts.json: {}", e))?;
    // Decrypt access tokens after reading
    for account in accounts.accounts.values_mut() {
        if !account.access_token.is_empty() {
            match decrypt_token(&account.access_token) {
                Ok(decrypted) => account.access_token = decrypted,
                Err(e) => {
                    Logger::console_log(LogLevel::Warning, &format!("⚠️ Failed to decrypt access token for account {}: {}", account.local_id, e), None);
                    account.access_token.clear();
                }
            }
        }
    }
    Logger::console_log(LogLevel::Info, &format!("✅ Successfully read {} accounts", accounts.accounts.len()), None);
    Ok(accounts)
}

/// Write accounts to launcher_accounts.json
#[tauri::command]
pub async fn write_launcher_accounts(mut accounts: LauncherAccountsJson) -> Result<(), String> {
    Logger::console_log(LogLevel::Info, "💾 Writing Kable accounts to file...", None);
    let accounts_path = get_kable_accounts_path()
        .map_err(|e| format!("Failed to get Kable accounts path: {}", e))?;
    // Ensure the parent directory exists
    if let Some(parent_dir) = accounts_path.parent() {
        fs::create_dir_all(parent_dir)
            .map_err(|e| format!("Failed to create Kable directory: {}", e))?;
    }
    // Encrypt access tokens before writing
    for account in accounts.accounts.values_mut() {
        if !account.access_token.is_empty() {
            match encrypt_token(&account.access_token) {
                Ok(encrypted) => account.access_token = encrypted,
                Err(e) => {
                    Logger::console_log(LogLevel::Warning, &format!("⚠️ Failed to encrypt access token for account {}: {}", account.local_id, e), None);
                    // Optionally clear or leave as plaintext if encryption fails
                }
            }
        }
    }
    let content = serde_json::to_string_pretty(&accounts)
        .map_err(|e| format!("Failed to serialize Kable accounts: {}", e))?;
    fs::write(&accounts_path, content)
        .map_err(|e| format!("Failed to write kable_accounts.json: {}", e))?;
    Logger::console_log(LogLevel::Info, &format!("✅ Successfully wrote {} accounts to file", accounts.accounts.len()), None);
    Ok(())
}

/// Add or update a single account in launcher_accounts.json
#[tauri::command]
pub async fn write_launcher_account(account: LauncherAccount) -> Result<(), String> {
    Logger::console_log(LogLevel::Info, &format!("📝 Adding/updating account: {}", account.username), None);
    
    let mut accounts = read_launcher_accounts().await?;
    
    // Add or update the account
    accounts.accounts.insert(account.local_id.clone(), account.clone());
    
    // If this is the first account or no active account is set, make it active
    if accounts.active_account_local_id.is_empty() || accounts.accounts.len() == 1 {
        accounts.active_account_local_id = account.local_id.clone();
        Logger::console_log(LogLevel::Info, &format!("🎯 Set {} as active account", account.username), None);
    }
    
    write_launcher_accounts(accounts).await?;
    
    Ok(())
}

/// Remove an account from launcher_accounts.json
#[tauri::command]
pub async fn remove_launcher_account(account_id: String) -> Result<(), String> {
    Logger::console_log(LogLevel::Info, &format!("🗑️ Removing account: {}", account_id), None);
    
    let mut accounts = read_launcher_accounts().await?;
    
    // Remove the account
    if accounts.accounts.remove(&account_id).is_some() {
        Logger::console_log(LogLevel::Info, "✅ Account removed successfully", None);
        
        // If this was the active account, clear the active account or set a new one
        if accounts.active_account_local_id == account_id {
            accounts.active_account_local_id = accounts.accounts.keys().next().unwrap_or(&String::new()).clone();
            Logger::console_log(LogLevel::Info, &format!("🎯 New active account: {}", accounts.active_account_local_id), None);
        }
        
        write_launcher_accounts(accounts).await?;
    } else {
        Logger::console_log(LogLevel::Warning, "⚠️ Account not found", None);
        return Err("Account not found".to_string());
    }
    
    Ok(())
}

/// Set the active account in launcher_accounts.json
#[tauri::command]
pub async fn set_active_launcher_account(account_id: String) -> Result<(), String> {
    Logger::console_log(LogLevel::Info, &format!("🎯 Setting active account: {}", account_id), None);
    
    let mut accounts = read_launcher_accounts().await?;
    
    // Check if the account exists
    if !accounts.accounts.contains_key(&account_id) {
        return Err("Account not found".to_string());
    }
    
    accounts.active_account_local_id = account_id.clone();
    write_launcher_accounts(accounts).await?;
    
    Logger::console_log(LogLevel::Info, "✅ Active account updated successfully", None);
    
    Ok(())
}

/// Get the currently active account from launcher_accounts.json
#[tauri::command]
pub async fn get_active_launcher_account() -> Result<Option<LauncherAccount>, String> {
    Logger::console_log(LogLevel::Debug, "🔍 Getting active launcher account...", None);
    
    let accounts = read_launcher_accounts().await?;
    
    if accounts.active_account_local_id.is_empty() {
        Logger::console_log(LogLevel::Info, "ℹ️ No active account set", None);
        return Ok(None);
    }
    
    let active_account = accounts.accounts.get(&accounts.active_account_local_id).cloned();
    
    if active_account.is_some() {
        Logger::console_log(LogLevel::Info, "✅ Found active account", None);
    } else {
        Logger::console_log(LogLevel::Warning, "⚠️ Active account ID set but account not found", None);
    }
    
    Ok(active_account)
}

/// Get all accounts from launcher_accounts.json
#[tauri::command]
pub async fn get_all_launcher_accounts() -> Result<Vec<LauncherAccount>, String> {
    Logger::console_log(LogLevel::Debug, "📋 Getting all launcher accounts...", None);
    
    let accounts = read_launcher_accounts().await?;
    let account_list: Vec<LauncherAccount> = accounts.accounts.into_values().collect();
    
    Logger::console_log(LogLevel::Info, &format!("✅ Retrieved {} accounts", account_list.len()), None);
    
    Ok(account_list)
}

/// Open a URL in the default browser
#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    Logger::console_log(LogLevel::Info, &format!("🌐 Opening URL: {}", url), None);
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", &url])
            .spawn()
            .map_err(|e| format!("Failed to open URL on Windows: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL on macOS: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL on Linux: {}", e))?;
    }
    
    Logger::console_log(LogLevel::Info, "✅ URL opened successfully", None);
    
    Ok(())
}

/// Get the path to launcher_accounts.json as a string (useful for debugging)
#[tauri::command]
pub async fn get_launcher_accounts_path_string() -> Result<String, String> {
    let path = get_kable_accounts_path()
        .map_err(|e| format!("Failed to get Kable accounts path: {}", e))?;
    Ok(path.to_string_lossy().to_string())
}

/// Get client ID from environment variable
pub fn get_client_id() -> Result<String, String> {
    std::env::var("AZURE_CLIENT_ID")
        .or_else(|_| std::env::var("CLIENT_ID"))
        .map_err(|_| "AZURE_CLIENT_ID environment variable not set".to_string())
}

/// Get redirect URI from environment variable
pub fn get_redirect_uri() -> Result<String, String> {
    std::env::var("AZURE_REDIRECT_URI")
        .or_else(|_| std::env::var("REDIRECT_URI"))
        .map_err(|_| "AZURE_REDIRECT_URI environment variable not set".to_string())
}

/// Get OAuth port from environment variable, defaults to 5713
pub fn get_oauth_port() -> u16 {
    std::env::var("OAUTH_PORT")
        .unwrap_or_else(|_| "5713".to_string())
        .parse()
        .unwrap_or(5713)
}

/// Validate and clean up malformed accounts in launcher_accounts.json
#[tauri::command]
pub async fn validate_and_cleanup_accounts() -> Result<String, String> {
    Logger::console_log(LogLevel::Info, "🧹 Starting account validation and cleanup...", None);
    
    let mut accounts = read_launcher_accounts().await?;
    let original_count = accounts.accounts.len();
    let mut removed_count = 0;
    let mut invalid_accounts = Vec::new();
    
    // Find accounts with invalid data
    let mut valid_accounts = HashMap::new();
    
    for (local_id, account) in accounts.accounts.iter() {
        let mut is_valid = true;
        let mut issues = Vec::new();
        
        // Check if UUID is valid (not empty, not a username)
        if account.username.is_empty() {
            is_valid = false;
            issues.push("empty username");
        }
        
        // Check if username is actually a UUID (malformed data)
        if account.username.len() == 36 && account.username.contains('-') {
            // This looks like a UUID, likely malformed
            is_valid = false;
            issues.push("username appears to be UUID");
        }
        
        // Check if local_id is present
        if local_id.is_empty() {
            is_valid = false;
            issues.push("empty local_id");
        }
        
        if is_valid {
            valid_accounts.insert(local_id.clone(), account.clone());
        } else {
            invalid_accounts.push(format!("{}: {}", local_id, issues.join(", ")));
            removed_count += 1;
        }
    }
    
    // Update accounts with only valid ones
    accounts.accounts = valid_accounts;
    
    // Check if active account is still valid
    if !accounts.active_account_local_id.is_empty() && 
       !accounts.accounts.contains_key(&accounts.active_account_local_id) {
        Logger::console_log(LogLevel::Warning, "⚠️ Active account was invalid, clearing", None);
        accounts.active_account_local_id = accounts.accounts.keys().next().unwrap_or(&String::new()).clone();
    }
    
    // Write cleaned accounts back
    write_launcher_accounts(accounts).await?;
    
    let summary = if removed_count > 0 {
        format!(
            "✅ Cleanup complete! Removed {} invalid accounts out of {}. Invalid accounts: {}",
            removed_count, original_count, invalid_accounts.join("; ")
        )
    } else {
        format!("✅ All {} accounts are valid! No cleanup needed.", original_count)
    };
    
    Logger::console_log(LogLevel::Info, &summary, None);
    
    Ok(summary)
}



