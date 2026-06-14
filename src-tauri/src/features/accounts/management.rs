use std::collections::HashMap;
use std::path::PathBuf;

use api_types::auth::{LauncherAccount, LauncherAccountsJson, MicrosoftToken};
use chrono::Utc;

use crate::constants::KABLE_ACCOUNTS_FILE;
use crate::features::accounts::secure_token;
use crate::system::fs::{
    get_kable_launcher_dir, read_to_string, write_file, write_file_atomic_async,
};

pub async fn add_account(account: LauncherAccount) -> Result<(), String> {
    let mut accounts_json = ensure_accounts_file().await?;

    if accounts_json.accounts.contains_key(&account.local_id) {
        return Err(format!("Account ID {} already exists", account.local_id));
    }

    accounts_json
        .accounts
        .insert(account.local_id.clone(), account);
    write_accounts(&accounts_json).await?;
    Ok(())
}

pub async fn remove_account(account: LauncherAccount) -> Result<Vec<LauncherAccount>, String> {
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

pub async fn set_active_account(account: LauncherAccount) -> Result<(), String> {
    // Check if the id exists in hashmap key of LauncherAccountsJson.
    let accounts_json = ensure_accounts_file().await?;
    if !accounts_json.accounts.contains_key(&account.local_id) {
        return Err(format!("Account ID {} does not exist", account.local_id));
    }
    // Set the active account
    let updated_json = LauncherAccountsJson {
        accounts: accounts_json.accounts,
        active_account_local_id: account.local_id,
        mojang_client_token: accounts_json.mojang_client_token,
    };
    write_file(
        &get_kable_accounts_path().await?,
        &serde_json::to_string_pretty(&updated_json)
            .map_err(|e| format!("Failed to serialize updated accounts: {}", e))?,
    )
    .await?;
    Ok(())
}

pub async fn get_active_account() -> Result<Option<LauncherAccount>, String> {
    let accounts_json = ensure_accounts_file().await?;
    Ok(accounts_json
        .accounts
        .get(&accounts_json.active_account_local_id)
        .cloned())
}

pub async fn list_accounts() -> Result<Vec<LauncherAccount>, String> {
    let accounts_json = ensure_accounts_file().await?;
    Ok(accounts_json.accounts.values().cloned().collect())
}

/**********************************************************************************
 * Above are the "API" appropriate functions to manage accounts.                  *
 * Below are filesystem utility functions to read/write accounts from disk.       *
 **********************************************************************************/

async fn get_kable_accounts_path() -> Result<PathBuf, String> {
    // Use the launcher directory for kable_accounts.json
    let launcher_dir = get_kable_launcher_dir()?;
    let accounts_path = launcher_dir.join(KABLE_ACCOUNTS_FILE);
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
        write_file_atomic_async(&accounts_path, content.as_bytes()).await?;
    }
    Ok(accounts_path)
}
/// Load the file, try to parse as LauncherAccountsJson, on failure make file and/or fill with empty default and return that.
async fn ensure_accounts_file() -> Result<LauncherAccountsJson, String> {
    let accounts_path = get_kable_accounts_path().await?;
    if !accounts_path.exists() {
        // This should be handled by get_kable_accounts_path, but just in case, create an empty file
        write_file_atomic_async(&accounts_path, b"{}").await?;
    }
    // Try to read and parse the file, if it fails (corrupted) overwrite with empty structure
    let content = read_to_string(&accounts_path).await?;
    // Ternary-style set the accounts_json to either the parsed content or a new empty structure if parsing fails
    let accounts_json = serde_json::from_str(&content).unwrap_or(LauncherAccountsJson {
        accounts: HashMap::new(),
        active_account_local_id: String::new(),
        mojang_client_token: String::new(),
    });
    refresh_accounts(accounts_json).await
}

async fn write_accounts(accounts_json: &LauncherAccountsJson) -> Result<(), String> {
    let accounts_path = get_kable_accounts_path().await?;
    let content = serde_json::to_string_pretty(accounts_json)
        .map_err(|e| format!("Failed to serialize accounts: {}", e))?;
    write_file(&accounts_path, &content).await?;
    Ok(())
}

async fn refresh_account(account: &mut LauncherAccount) -> Result<(), String> {
    let encrypted_refresh_token = match &account.encrypted_refresh_token {
        Some(token) => token,
        None => return Ok(()),
    };

    let refresh_token = secure_token::decrypt_token(encrypted_refresh_token)?;

    let token = MicrosoftToken {
        access_token: String::new(),
        expires_at: Utc::now(),
        refresh_token: Some(refresh_token),
    };

    let new_token = crate::integrations::mojang_api::auth::refresh_microsoft_token(token).await?;

    if let Some(refresh_token) = new_token.refresh_token {
        account.encrypted_refresh_token = Some(secure_token::encrypt_token(&refresh_token)?);
    }

    account.access_token = new_token.access_token;
    account.access_token_expires_at = new_token.expires_at.to_rfc3339();

    Ok(())
}

async fn refresh_accounts(
    mut accounts_json: LauncherAccountsJson,
) -> Result<LauncherAccountsJson, String> {
    for account in accounts_json.accounts.values_mut() {
        if let Err(e) = refresh_account(account).await {
            eprintln!("Failed to refresh account {}: {}", account.local_id, e);
        }
    }

    write_accounts(&accounts_json).await?;

    Ok(accounts_json)
}
