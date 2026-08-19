use std::collections::HashMap;
use std::path::PathBuf;

use api_types::auth::{KableAccount, KableAccountsJson, MicrosoftToken};
use api_types::Timestamp;
use chrono::Utc;

use crate::constants::KABLE_ACCOUNTS_FILE;
use crate::features::accounts::kable_account;
use crate::features::accounts::secure_token;
use crate::system::fs::{create_dir, launcher_dir, read_str, write_str};
use crate::Logger;

/// Convert a Microsoft authentication token into a Kable account and add it
/// to the local account store.
pub async fn add_microsoft_account(token: MicrosoftToken) -> Result<KableAccount, String> {
    let account = kable_account::from_microsoft_token(token).await?;

    add_account(account.clone()).await?;

    Ok(account)
}

/// Add an already constructed Kable account to the local account store.
pub async fn add_account(account: KableAccount) -> Result<(), String> {
    let mut accounts_json = ensure_accounts_file().await?;

    if accounts_json.accounts.contains_key(&account.local_id) {
        return Err(format!("Account ID {} already exists", account.local_id));
    }

    let local_id = account.local_id.clone();
    let should_activate = accounts_json.active_account_local_id.is_empty();

    accounts_json.accounts.insert(local_id.clone(), account);

    if should_activate {
        accounts_json.active_account_local_id = local_id;
    }

    write_accounts(&accounts_json).await?;

    Ok(())
}

pub async fn remove_account(account: KableAccount) -> Result<Vec<KableAccount>, String> {
    let mut accounts_json = ensure_accounts_file().await?;

    if !accounts_json.accounts.contains_key(&account.local_id) {
        return Err(format!("Account ID {} does not exist", account.local_id));
    }

    accounts_json.accounts.remove(&account.local_id);

    if accounts_json.active_account_local_id == account.local_id {
        accounts_json.active_account_local_id = accounts_json.accounts.keys().next().cloned().unwrap_or_default();
    }

    write_accounts(&accounts_json).await?;

    Ok(accounts_json.accounts.values().cloned().collect())
}

pub async fn set_active_account(account: KableAccount) -> Result<(), String> {
    let accounts_json = ensure_accounts_file().await?;

    if !accounts_json.accounts.contains_key(&account.local_id) {
        return Err(format!("Account ID {} does not exist", account.local_id));
    }

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

pub async fn get_active_account() -> Result<Option<KableAccount>, String> {
    let accounts_json = ensure_accounts_file().await?;

    Ok(accounts_json.accounts.get(&accounts_json.active_account_local_id).cloned())
}

/// List all accounts. This also attempts to refresh all stored accounts before
/// returning them. If refreshing an individual account fails, that account is
/// still returned with its existing credentials.
pub async fn list_accounts() -> Result<Vec<KableAccount>, String> {
    let accounts_json = ensure_accounts_file().await?;

    Logger::debug_global(format!("Listing {} accounts", accounts_json.accounts.len()).as_str(), None);

    Ok(accounts_json.accounts.values().cloned().collect())
}

/**********************************************************************************
 * Above are the API appropriate functions to manage accounts.
 * Below are filesystem utility functions to read/write accounts from disk.
 **********************************************************************************/

async fn get_kable_accounts_path() -> Result<PathBuf, String> {
    let launcher_dir = launcher_dir()?;
    let accounts_path = launcher_dir.join(KABLE_ACCOUNTS_FILE);

    if !accounts_path.exists() {
        if let Some(parent_dir) = accounts_path.parent() {
            create_dir(parent_dir).await.map_err(|e| format!("Failed to create Kable launcher directory: {}", e))?;
        }

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

/// Load the accounts file and refresh all stored accounts before returning.
async fn ensure_accounts_file() -> Result<KableAccountsJson, String> {
    let accounts_path = get_kable_accounts_path().await?;

    if !accounts_path.exists() {
        write_str(&accounts_path, "{}", false).await?;
    }

    let content = read_str(&accounts_path).await?;

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
