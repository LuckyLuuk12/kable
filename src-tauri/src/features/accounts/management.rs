use std::collections::HashMap;
use std::path::PathBuf;

use api_types::auth::{KableAccount, KableAccountsJson, MicrosoftToken};
use api_types::Timestamp;
use chrono::Utc;

use crate::constants::KABLE_ACCOUNTS_FILE;
use crate::features::accounts::secure_token;
use crate::system::fs::{create_dir, launcher_dir, read_str, write_str};
use crate::Logger;

pub async fn add_account(account: KableAccount) -> Result<(), String> {
    let mut accounts_json = ensure_accounts_file().await?;

    if accounts_json.accounts.contains_key(&account.local_id) {
        return Err(format!("Account ID {} already exists", account.local_id));
    }

    accounts_json.accounts.insert(account.local_id.clone(), account);
    write_accounts(&accounts_json).await?;
    Ok(())
}

pub async fn remove_account(account: KableAccount) -> Result<Vec<KableAccount>, String> {
    let mut accounts_json = ensure_accounts_file().await?;
    if !accounts_json.accounts.contains_key(&account.local_id) {
        return Err(format!("Account ID {} does not exist", account.local_id));
    }
    accounts_json.accounts.remove(&account.local_id);
    // If the removed account was active, clear active account
    if accounts_json.active_account_local_id == account.local_id {
        accounts_json.active_account_local_id = String::new();
    }
    write_accounts(&accounts_json).await?;
    Ok(accounts_json.accounts.values().cloned().collect())
}

pub async fn set_active_account(account: KableAccount) -> Result<(), String> {
    // Check if the id exists in hashmap key of KableAccountsJson.
    let accounts_json = ensure_accounts_file().await?;
    if !accounts_json.accounts.contains_key(&account.local_id) {
        return Err(format!("Account ID {} does not exist", account.local_id));
    }
    // Set the active account
    let updated_json = KableAccountsJson {
        accounts: accounts_json.accounts,
        active_account_local_id: account.local_id,
        mojang_client_token: accounts_json.mojang_client_token,
    };
    write_str(
        &get_kable_accounts_path().await?,
        &serde_json::to_string_pretty(&updated_json).map_err(|e| format!("Failed to serialize updated accounts: {}", e))?,
        false,
    )
    .await?;
    Ok(())
}

pub async fn get_active_account() -> Result<KableAccount, String> {
    let accounts_json = ensure_accounts_file().await?;
    accounts_json
        .accounts
        .get(&accounts_json.active_account_local_id)
        .cloned()
        .ok_or_else(|| "No active account found".into())
}

/// List all accounts, this will also attempt to refresh all accounts before returning, if refreshing fails it will just return the accounts without refreshing. This way we ensure the file is always in a valid state and we attempt to keep tokens fresh without risking failure to list accounts at all.
pub async fn list_accounts() -> Result<Vec<KableAccount>, String> {
    let accounts_json = ensure_accounts_file().await?; // this attempts refreshing already
    Logger::debug_global(format!("Listing {} accounts", accounts_json.accounts.len()).as_str(), None);
    Ok(accounts_json.accounts.values().cloned().collect())
}

/**********************************************************************************
 * Above are the "API" appropriate functions to manage accounts.                  *
 * Below are filesystem utility functions to read/write accounts from disk.       *
 **********************************************************************************/

async fn get_kable_accounts_path() -> Result<PathBuf, String> {
    // Use the launcher directory for kable_accounts.json
    let launcher_dir = launcher_dir()?;
    let accounts_path = launcher_dir.join(KABLE_ACCOUNTS_FILE);
    // If file does not exist, create it with an empty structure
    if !accounts_path.exists() {
        // Ensure parent directory exists and atomically create the file (sync helper)
        if let Some(parent_dir) = accounts_path.parent() {
            create_dir(parent_dir).await.map_err(|e| format!("Failed to create Kable launcher directory: {}", e))?;
        }
        // Write empty structure
        let empty = serde_json::json!({
            "accounts": {},
            "active_account_local_id": "",
            "mojang_client_token": ""
        });
        let content = serde_json::to_string_pretty(&empty).map_err(|e| format!("Failed to serialize empty accounts: {}", e))?;
        write_str(&accounts_path, &content, false).await?;
    }
    Ok(accounts_path)
}
/// Load the file, try to parse as KableAccountsJson, on failure make file and/or fill with empty default and return that.
/// SIDE EFFECT: This will also attempt to refresh all accounts by calling refresh_accounts, if that fails it will just return the accounts without refreshing. This way we ensure the file is always in a valid state and we attempt to keep tokens fresh without risking failure to load accounts at all.
async fn ensure_accounts_file() -> Result<KableAccountsJson, String> {
    let accounts_path = get_kable_accounts_path().await?;
    if !accounts_path.exists() {
        // This should be handled by get_kable_accounts_path, but just in case, create an empty file
        write_str(&accounts_path, "{}", false).await?;
    }
    // Try to read and parse the file, if it fails (corrupted) overwrite with empty structure
    let content = read_str(&accounts_path).await?;
    // Ternary-style set the accounts_json to either the parsed content or a new empty structure if parsing fails
    let accounts_json = serde_json::from_str(&content).unwrap_or(KableAccountsJson {
        accounts: HashMap::new(),
        active_account_local_id: String::new(),
        mojang_client_token: String::new(),
    });
    refresh_accounts(accounts_json).await
}

async fn write_accounts(accounts_json: &KableAccountsJson) -> Result<(), String> {
    let accounts_path = get_kable_accounts_path().await?;
    let content = serde_json::to_string_pretty(accounts_json).map_err(|e| format!("Failed to serialize accounts: {}", e))?;
    write_str(&accounts_path, &content, false).await?;
    Ok(())
}

async fn refresh_account(account: &mut KableAccount) -> Result<(), String> {
    let encrypted_refresh_token = match &account.encrypted_refresh_token {
        Some(token) => token,
        None => return Ok(()),
    };

    let refresh_token = secure_token::decrypt_token(encrypted_refresh_token).await?;

    let token = MicrosoftToken { access_token: String::new(), expires_at: Timestamp(Utc::now()), refresh_token: Some(refresh_token) };

    let new_token = crate::integrations::mojang_api::auth::refresh_microsoft_token(token).await?;

    if let Some(refresh_token) = new_token.refresh_token {
        account.encrypted_refresh_token = Some(secure_token::encrypt_token(&refresh_token).await?);
    }

    account.access_token = new_token.access_token;
    account.access_token_expires_at = new_token.expires_at.0.to_rfc3339();

    Ok(())
}

async fn refresh_accounts(mut accounts_json: KableAccountsJson) -> Result<KableAccountsJson, String> {
    for account in accounts_json.accounts.values_mut() {
        if let Err(e) = refresh_account(account).await {
            eprintln!("Failed to refresh account {}: {}", account.local_id, e);
        }
    }

    write_accounts(&accounts_json).await?;

    Ok(accounts_json)
}
