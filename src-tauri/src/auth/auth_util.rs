use crate::auth::secure_token::{decrypt_token, encrypt_token};
use crate::logging::{LogLevel, Logger};
/**
 * This file contains authentication related utility functions.
 * E.g. functions to read/write to/from launcher_accounts.json to work with stored accounts,
 *      opening URLs in the browser (independent of the OS), etc.
 */
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;
use tokio::fs as async_fs;

// In-memory cache for accounts to avoid redundant disk reads
static ACCOUNTS_CACHE: RwLock<Option<LauncherAccountsJson>> = RwLock::new(None);

// ...existing code...
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LauncherAccount {
    pub access_token: String,
    pub access_token_expires_at: String,
    pub encrypted_refresh_token: Option<String>, // AES-encrypted refresh token
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

/// Get the path to the kable_accounts.json file
pub fn get_kable_accounts_path() -> Result<PathBuf, String> {
    // Use the launcher directory for kable_accounts.json
    let launcher_dir = crate::get_kable_launcher_dir()?;
    let accounts_path = launcher_dir.join("kable_accounts.json");
    // If file does not exist, create it with an empty structure
    if !accounts_path.exists() {
        // Ensure parent directory exists and atomically create the file (sync helper)
        if let Some(parent_dir) = accounts_path.parent() {
            crate::ensure_folder_sync(parent_dir)
                .map_err(|e| format!("Failed to create Kable launcher directory: {}", e))?;
        }
        // Write empty structure
        let empty = serde_json::json!({
            "accounts": {},
            "active_account_local_id": "",
            "mojang_client_token": ""
        });
        let content = serde_json::to_string_pretty(&empty)
            .map_err(|e| format!("Failed to serialize empty accounts: {}", e))?;
        crate::write_file_atomic_sync(&accounts_path, content.as_bytes())?;
    }
    Ok(accounts_path)
}

/// Invalidate the accounts cache (should be called after write operations)
fn invalidate_accounts_cache() {
    if let Ok(mut cache) = ACCOUNTS_CACHE.write() {
        *cache = None;
        Logger::console_log(LogLevel::Debug, "🔄 Accounts cache invalidated", None);
    }
}

/// Read all accounts from kable_accounts.json with caching
pub async fn read_launcher_accounts() -> Result<LauncherAccountsJson, String> {
    // Try to use cached data first
    if let Ok(cache) = ACCOUNTS_CACHE.read() {
        if let Some(cached_accounts) = cache.as_ref() {
            Logger::console_log(LogLevel::Debug, "💾 Using cached accounts data", None);
            return Ok(cached_accounts.clone());
        }
    }

    // Cache miss - read from disk
    Logger::console_log(
        LogLevel::Info,
        "📖 Reading Kable accounts from file...",
        None,
    );
    let accounts_path = get_kable_accounts_path()
        .map_err(|e| format!("Failed to get Kable accounts path: {}", e))?;
    Logger::console_log(
        LogLevel::Debug,
        &format!("📁 Accounts file path: {:?}", accounts_path),
        None,
    );
    if !accounts_path.exists() {
        Logger::console_log(
            LogLevel::Warning,
            "⚠️ kable_accounts.json not found, returning empty structure",
            None,
        );
        let empty_accounts = LauncherAccountsJson {
            accounts: HashMap::new(),
            active_account_local_id: String::new(),
            mojang_client_token: String::new(),
        };
        // Cache the empty structure
        if let Ok(mut cache) = ACCOUNTS_CACHE.write() {
            *cache = Some(empty_accounts.clone());
        }
        return Ok(empty_accounts);
    }
    let content = async_fs::read_to_string(&accounts_path)
        .await
        .map_err(|e| format!("Failed to read kable_accounts.json: {}", e))?;
    let mut accounts: LauncherAccountsJson = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse kable_accounts.json: {}", e))?;
    // Decrypt access tokens after reading. If decryption fails for any token,
    // clear it and persist the cleaned accounts back to disk so we don't keep
    // attempting to decrypt a possibly stale/corrupted token on each startup.
    let mut changed = false;
    for account in accounts.accounts.values_mut() {
        if !account.access_token.is_empty() {
            match decrypt_token(&account.access_token) {
                Ok(decrypted) => {
                    account.access_token = decrypted;
                }
                Err(e) => {
                    Logger::console_log(
                        LogLevel::Warning,
                        &format!(
                            "Failed to decrypt access token for account {}: {}. Clearing stored token.",
                            account.local_id, e
                        ),
                        None,
                    );
                    account.access_token.clear();
                    changed = true;
                }
            }
        }
    }

    // Persist cleaned accounts if any token was cleared due to decryption failure.
    if changed {
        // Fire-and-forget: try to write cleaned accounts back to disk. If this fails
        // we don't want to prevent the caller from continuing to use the app.
        let accounts_copy = accounts.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::auth::write_launcher_accounts(accounts_copy).await {
                crate::logging::Logger::console_log(
                    LogLevel::Warning,
                    &format!(
                        "Failed to persist cleaned accounts after decryption errors: {}",
                        e
                    ),
                    None,
                );
            }
        });
    }
    Logger::console_log(
        LogLevel::Info,
        &format!("✅ Successfully read {} accounts", accounts.accounts.len()),
        None,
    );

    // Cache the accounts before returning
    if let Ok(mut cache) = ACCOUNTS_CACHE.write() {
        *cache = Some(accounts.clone());
    }

    Ok(accounts)
}

/// Write accounts to launcher_accounts.json
pub async fn write_launcher_accounts(mut accounts: LauncherAccountsJson) -> Result<(), String> {
    Logger::console_log(LogLevel::Info, "💾 Writing Kable accounts to file...", None);
    let accounts_path = get_kable_accounts_path()
        .map_err(|e| format!("Failed to get Kable accounts path: {}", e))?;
    // Encrypt access tokens before writing
    for account in accounts.accounts.values_mut() {
        if !account.access_token.is_empty() {
            match encrypt_token(&account.access_token) {
                Ok(encrypted) => account.access_token = encrypted,
                Err(e) => {
                    Logger::console_log(
                        LogLevel::Warning,
                        &format!(
                            "⚠️ Failed to encrypt access token for account {}: {}",
                            account.local_id, e
                        ),
                        None,
                    );
                    // Optionally clear or leave as plaintext if encryption fails
                }
            }
        }
    }
    let content = serde_json::to_string_pretty(&accounts)
        .map_err(|e| format!("Failed to serialize Kable accounts: {}", e))?;
    if let Some(parent_dir) = accounts_path.parent() {
        crate::ensure_parent_dir_exists_async(parent_dir)
            .await
            .map_err(|e| format!("Failed to create Kable directory: {}", e))?;
    }
    crate::write_file_atomic_async(&accounts_path, content.as_bytes())
        .await
        .map_err(|e| format!("Failed to write kable_accounts.json: {}", e))?;
    Logger::console_log(
        LogLevel::Info,
        &format!(
            "✅ Successfully wrote {} accounts to file",
            accounts.accounts.len()
        ),
        None,
    );

    // Invalidate cache after write
    invalidate_accounts_cache();

    Ok(())
}

/// Add or update a single account in launcher_accounts.json
pub async fn write_launcher_account(account: LauncherAccount) -> Result<(), String> {
    Logger::console_log(
        LogLevel::Info,
        &format!("📝 Adding/updating account: {}", account.username),
        None,
    );

    let mut accounts = read_launcher_accounts().await?;

    // Add or update the account
    accounts
        .accounts
        .insert(account.local_id.clone(), account.clone());

    // If this is the first account or no active account is set, make it active
    if accounts.active_account_local_id.is_empty() || accounts.accounts.len() == 1 {
        accounts.active_account_local_id = account.local_id.clone();
        Logger::console_log(
            LogLevel::Info,
            &format!("🎯 Set {} as active account", account.username),
            None,
        );
    }

    write_launcher_accounts(accounts).await?;

    Ok(())
}

/// Remove an account from launcher_accounts.json
pub async fn remove_launcher_account(account_id: String) -> Result<(), String> {
    // codeql[rs/clear-text-logging] - account_id is Minecraft UUID (public identifier)
    Logger::console_log(
        LogLevel::Info,
        &format!("🗑️ Removing account: {}", account_id),
        None,
    );

    let mut accounts = read_launcher_accounts().await?;

    // Remove the account
    if accounts.accounts.remove(&account_id).is_some() {
        Logger::console_log(LogLevel::Info, "✅ Account removed successfully", None);

        // If this was the active account, clear the active account or set a new one
        if accounts.active_account_local_id == account_id {
            accounts.active_account_local_id = accounts
                .accounts
                .keys()
                .next()
                .unwrap_or(&String::new())
                .clone();
            Logger::console_log(
                LogLevel::Info,
                &format!(
                    "🎯 New active account: {}",
                    accounts.active_account_local_id
                ),
                None,
            );
        }

        write_launcher_accounts(accounts).await?;
    } else {
        Logger::console_log(LogLevel::Warning, "⚠️ Account not found", None);
        return Err("Account not found".to_string());
    }

    Ok(())
}

/// Set the active account in launcher_accounts.json
pub async fn set_active_launcher_account(account_id: String) -> Result<(), String> {
    // codeql[rs/clear-text-logging] - account_id is Minecraft UUID (public identifier)
    Logger::console_log(
        LogLevel::Info,
        &format!("🎯 Setting active account: {}", account_id),
        None,
    );

    let mut accounts = read_launcher_accounts().await?;

    // Check if the account exists
    if !accounts.accounts.contains_key(&account_id) {
        return Err("Account not found".to_string());
    }

    accounts.active_account_local_id = account_id.clone();
    write_launcher_accounts(accounts).await?;

    Logger::console_log(
        LogLevel::Info,
        "✅ Active account updated successfully",
        None,
    );

    Ok(())
}

/// Get the currently active account from launcher_accounts.json
pub async fn get_active_launcher_account() -> Result<Option<LauncherAccount>, String> {
    Logger::console_log(
        LogLevel::Debug,
        "🔍 Getting active launcher account...",
        None,
    );

    let accounts = read_launcher_accounts().await?;

    if accounts.active_account_local_id.is_empty() {
        Logger::console_log(LogLevel::Info, "ℹ️ No active account set", None);
        return Ok(None);
    }

    let active_account = accounts
        .accounts
        .get(&accounts.active_account_local_id)
        .cloned();

    if active_account.is_some() {
        Logger::console_log(LogLevel::Info, "✅ Found active account", None);
    } else {
        Logger::console_log(
            LogLevel::Warning,
            "⚠️ Active account ID set but account not found",
            None,
        );
    }

    Ok(active_account)
}

/// Get all accounts from launcher_accounts.json
pub async fn get_all_launcher_accounts() -> Result<Vec<LauncherAccount>, String> {
    Logger::console_log(LogLevel::Debug, "📋 Getting all launcher accounts...", None);

    let accounts = read_launcher_accounts().await?;
    let account_list: Vec<LauncherAccount> = accounts.accounts.into_values().collect();

    Logger::console_log(
        LogLevel::Info,
        &format!("✅ Retrieved {} accounts", account_list.len()),
        None,
    );

    Ok(account_list)
}

/// Get the path to launcher_accounts.json as a string (useful for debugging)
pub async fn get_launcher_accounts_path_string() -> Result<String, String> {
    let path = get_kable_accounts_path()
        .map_err(|e| format!("Failed to get Kable accounts path: {}", e))?;
    Ok(path.to_string_lossy().to_string())
}

/// Hardcoded fallback Azure client ID
pub const DEFAULT_AZURE_CLIENT_ID: &str = "4c27a19f-a3d0-4cd2-8e05-9fd961f905df";

pub fn get_client_id() -> Result<String, String> {
    std::env::var("AZURE_CLIENT_ID")
        .or(std::env::var("CLIENT_ID"))
        .or(Ok(DEFAULT_AZURE_CLIENT_ID.to_string()))
        .map_err(|_: String| {
            "AZURE_CLIENT_ID / CLIENT_ID not set and no fallback available".to_string()
        })
}

/// Hardcoded fallback Azure redirect URI
pub const DEFAULT_AZURE_REDIRECT_URI: &str = "http://localhost:43110/callback";

pub fn get_redirect_uri() -> Result<String, String> {
    std::env::var("AZURE_REDIRECT_URI")
        .or(std::env::var("REDIRECT_URI"))
        .or(Ok(DEFAULT_AZURE_REDIRECT_URI.to_string()))
        .map_err(|_: String| {
            "AZURE_REDIRECT_URI / REDIRECT_URI not set and no fallback available".to_string()
        })
}

/// Hardcoded fallback OAuth port
pub const DEFAULT_OAUTH_PORT: u16 = 43110;

pub fn get_oauth_port() -> u16 {
    std::env::var("OAUTH_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_OAUTH_PORT)
}

/// Validate and clean up malformed accounts in launcher_accounts.json
pub async fn validate_and_cleanup_accounts() -> Result<String, String> {
    Logger::console_log(
        LogLevel::Info,
        "🧹 Starting account validation and cleanup...",
        None,
    );

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
    if !accounts.active_account_local_id.is_empty()
        && !accounts
            .accounts
            .contains_key(&accounts.active_account_local_id)
    {
        Logger::console_log(
            LogLevel::Warning,
            "⚠️ Active account was invalid, clearing",
            None,
        );
        accounts.active_account_local_id = accounts
            .accounts
            .keys()
            .next()
            .unwrap_or(&String::new())
            .clone();
    }

    // Write cleaned accounts back
    write_launcher_accounts(accounts).await?;

    let summary = if removed_count > 0 {
        format!(
            "✅ Cleanup complete! Removed {} invalid accounts out of {}. Invalid accounts: {}",
            removed_count,
            original_count,
            invalid_accounts.join("; ")
        )
    } else {
        format!(
            "✅ All {} accounts are valid! No cleanup needed.",
            original_count
        )
    };

    Logger::console_log(LogLevel::Info, &summary, None);

    Ok(summary)
}

/// Refresh Microsoft token for a specific account
pub async fn refresh_microsoft_token(local_id: String) -> Result<LauncherAccount, String> {
    use chrono::Utc;
    use minecraft_msa_auth::MinecraftAuthorizationFlow;
    use oauth2::{basic::BasicClient, AuthUrl, ClientId, TokenResponse, TokenUrl};
    use reqwest::Client;

    // Load accounts and find the one to refresh
    let accounts_json = crate::read_launcher_accounts().await?;
    let account = accounts_json
        .accounts
        .get(&local_id)
        .cloned()
        .ok_or_else(|| format!("Account not found for local_id: {}", local_id))?;

    let encrypted_refresh_token = account
        .encrypted_refresh_token
        .clone()
        .ok_or_else(|| "No refresh token available".to_string())?;
    let refresh_token = decrypt_token(&encrypted_refresh_token)?;

    let client_id = get_client_id()?;
    let client = BasicClient::new(ClientId::new(client_id))
        .set_auth_uri(
            AuthUrl::new(
                "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize".to_string(),
            )
            .map_err(|e| format!("Failed to create auth URL: {}", e))?,
        )
        .set_token_uri(
            TokenUrl::new(
                "https://login.microsoftonline.com/consumers/oauth2/v2.0/token".to_string(),
            )
            .map_err(|e| format!("Failed to create token URL: {}", e))?,
        );

    let token_result = client
        .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token))
        .request_async(&crate::auth::oauth_helpers::async_http_client)
        .await
        .map_err(|e| format!("Failed to refresh token: {}", e))?;

    // Compute new Microsoft token expiry
    let new_expires_at = Utc::now()
        + chrono::Duration::seconds(
            token_result
                .expires_in()
                .map(|d| d.as_secs() as i64)
                .unwrap_or(3600),
        );

    // Exchange refreshed Microsoft access token for a Minecraft token
    let ms_access = token_result.access_token().secret().to_string();
    let mc_flow = MinecraftAuthorizationFlow::new(Client::new());
    let mc_token = mc_flow
        .exchange_microsoft_token(&ms_access)
        .await
        .map_err(|e| {
            format!(
                "Failed to exchange Microsoft token for Minecraft token: {}",
                e
            )
        })?;

    // Extract Minecraft access token string
    let mc_access = mc_token.access_token().clone().into_inner();

    // Handle refresh token: encrypt any new refresh token; fail rather than silently storing plaintext
    let new_encrypted_refresh_token = if let Some(rt) = token_result.refresh_token() {
        match encrypt_token(rt.secret()) {
            Ok(enc) => Some(enc),
            Err(e) => {
                Logger::console_log(
                    LogLevel::Error,
                    &format!("❌ Failed to encrypt refreshed refresh token: {}", e),
                    None,
                );
                return Err(format!("Failed to encrypt refreshed refresh token: {}", e));
            }
        }
    } else {
        account.encrypted_refresh_token.clone()
    };

    // Persist the updated account using the Minecraft token and the Microsoft expiry
    let mut updated_account = account.clone();
    updated_account.access_token = mc_access;
    updated_account.access_token_expires_at = new_expires_at.to_rfc3339();
    updated_account.encrypted_refresh_token = new_encrypted_refresh_token;

    write_launcher_account(updated_account.clone()).await?;
    Ok(updated_account)
}
