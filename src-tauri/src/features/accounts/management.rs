use std::path::PathBuf;

use api_types::auth::{KableAccount, KableAccountsJson, MicrosoftToken};
use api_types::Timestamp;
use chrono::Utc;
use minecraft_msa_auth::MinecraftAuthorizationFlow;
use reqwest::Client;

use crate::constants::KABLE_ACCOUNTS_FILE;
use crate::features::accounts::kable_account;
use crate::features::accounts::secure_token;
use crate::system::fs::{create_dir, launcher_dir, read_str, write_str};
use crate::Logger;

/// Refresh the Minecraft access token when it expires within this window.
const TOKEN_REFRESH_WINDOW_MINUTES: i64 = 5;

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

    write_accounts(&accounts_json).await
}

/// Remove an account from the local account store.
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

/// Set the active account without refreshing credentials.
pub async fn set_active_account(account: KableAccount) -> Result<(), String> {
    let mut accounts_json = ensure_accounts_file().await?;

    if !accounts_json.accounts.contains_key(&account.local_id) {
        return Err(format!("Account ID {} does not exist", account.local_id));
    }

    accounts_json.active_account_local_id = account.local_id;

    write_accounts(&accounts_json).await
}

/// Return the active account without making network requests.
///
/// Use `get_active_account_for_launch` when preparing to launch Minecraft.
pub async fn get_active_account() -> Result<Option<KableAccount>, String> {
    let accounts_json = ensure_accounts_file().await?;
    Ok(accounts_json.accounts.get(&accounts_json.active_account_local_id).cloned())
}

/// Return the active account with a sufficiently fresh Minecraft access token.
///
/// This should be called once at the beginning of a launch, not for every
/// individual argument or placeholder.
pub async fn get_active_account_for_launch() -> Result<Option<KableAccount>, String> {
    let mut accounts_json = ensure_accounts_file().await?;

    let active_id = accounts_json.active_account_local_id.clone();

    if active_id.is_empty() {
        return Ok(None);
    }

    let Some(account) = accounts_json.accounts.get_mut(&active_id) else {
        return Err(format!("Active account ID {} does not exist in the account store", active_id));
    };

    if !minecraft_token_needs_refresh(account) {
        return Ok(Some(account.clone()));
    }

    // Save any rotated refresh token even if the subsequent Minecraft
    // token exchange fails.
    let refresh_result = refresh_account(account).await;

    write_accounts(&accounts_json).await?;

    refresh_result?;

    Ok(accounts_json.accounts.get(&active_id).cloned())
}

/// List stored accounts without refreshing them.
pub async fn list_accounts() -> Result<Vec<KableAccount>, String> {
    let accounts_json = ensure_accounts_file().await?;

    Logger::debug_global(format!("Listing {} accounts", accounts_json.accounts.len()).as_str(), None);

    Ok(accounts_json.accounts.values().cloned().collect())
}

/**********************************************************************************
 * Filesystem utilities
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

/// Load stored accounts without making network requests.
///
/// Invalid JSON is reported rather than silently treating the account store
/// as empty and potentially overwriting the user's saved accounts.
async fn ensure_accounts_file() -> Result<KableAccountsJson, String> {
    let accounts_path = get_kable_accounts_path().await?;
    let content = read_str(&accounts_path).await?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse accounts file: {}", e))
}

async fn write_accounts(accounts_json: &KableAccountsJson) -> Result<(), String> {
    let accounts_path = get_kable_accounts_path().await?;

    let content = serde_json::to_string_pretty(accounts_json).map_err(|e| format!("Failed to serialize accounts: {}", e))?;

    write_str(&accounts_path, &content, false).await?;
    Ok(())
}

/// Determine whether the stored Minecraft access token is expired, close to
/// expiry, or has an unreadable expiry timestamp.
fn minecraft_token_needs_refresh(account: &KableAccount) -> bool {
    let expires_at = match chrono::DateTime::parse_from_rfc3339(&account.access_token_expires_at) {
        Ok(timestamp) => timestamp.with_timezone(&Utc),
        Err(_) => return true,
    };

    expires_at <= Utc::now() + chrono::Duration::minutes(TOKEN_REFRESH_WINDOW_MINUTES)
}

/// Refresh Microsoft credentials and exchange the Microsoft access token for
/// a Minecraft access token.
///
/// `account.access_token` must always contain the Minecraft access token,
/// never the Microsoft OAuth access token.
async fn refresh_account(account: &mut KableAccount) -> Result<(), String> {
    let encrypted_refresh_token = account
        .encrypted_refresh_token
        .as_ref()
        .ok_or_else(|| format!("Account {} needs refreshing but has no stored Microsoft refresh token", account.local_id))?
        .clone();

    let refresh_token = secure_token::decrypt_token(&encrypted_refresh_token)
        .await
        .map_err(|e| format!("Failed to decrypt Microsoft refresh token: {}", e))?;

    let token = MicrosoftToken { access_token: String::new(), expires_at: Timestamp(Utc::now()), refresh_token: Some(refresh_token) };

    let new_microsoft_token = crate::integrations::mojang_api::auth::refresh_microsoft_token(token)
        .await
        .map_err(|e| format!("Failed to refresh Microsoft token: {}", e))?;

    // Store a rotated refresh token immediately in the in-memory account.
    // The caller persists the account even if the following exchange fails.
    if let Some(refreshed_token) = new_microsoft_token.refresh_token.as_ref() {
        account.encrypted_refresh_token = Some(
            secure_token::encrypt_token(refreshed_token)
                .await
                .map_err(|e| format!("Failed to encrypt refreshed Microsoft token: {}", e))?,
        );
    }

    let minecraft_flow = MinecraftAuthorizationFlow::new(Client::new());

    let minecraft_token = minecraft_flow
        .exchange_microsoft_token(&new_microsoft_token.access_token)
        .await
        .map_err(|e| format!("Failed to exchange refreshed Microsoft token for Minecraft token: {}", e))?;

    let minecraft_access_token = minecraft_token.access_token().as_ref().to_string();

    let minecraft_expires_at = Utc::now() + chrono::Duration::seconds(minecraft_token.expires_in() as i64);

    // Update the Minecraft credentials only after the exchange succeeds.
    account.access_token = minecraft_access_token;
    account.access_token_expires_at = minecraft_expires_at.to_rfc3339();

    Ok(())
}
