use crate::constants::LAUNCHER_ACCOUNTS_FILE;
use crate::system::fs::{mc_dir, read_str, write_str};
use api_types::auth::{LauncherAccount, LauncherAccountsJson};

pub async fn list_accounts() -> Result<Vec<LauncherAccount>, String> {
    let path = mc_dir()?.join(LAUNCHER_ACCOUNTS_FILE);
    let contents = read_str(&path).await?;
    let accounts_json: LauncherAccountsJson =
        serde_json::from_str(&contents).map_err(|e| format!("Failed to parse accounts JSON: {}", e))?;
    Ok(accounts_json.accounts.values().cloned().collect())
}

pub async fn remove_account(account: LauncherAccount) -> Result<Vec<LauncherAccount>, String> {
    let path = mc_dir()?.join(LAUNCHER_ACCOUNTS_FILE);
    let contents = read_str(&path).await?;
    let mut accounts_json: LauncherAccountsJson =
        serde_json::from_str(&contents).map_err(|e| format!("Failed to parse accounts JSON: {}", e))?;

    if !accounts_json.accounts.contains_key(&account.local_id) {
        return Err(format!("Account ID {} does not exist", account.local_id));
    }

    accounts_json.accounts.remove(&account.local_id);
    write_str(
        &path,
        &serde_json::to_string_pretty(&accounts_json).map_err(|e| format!("Failed to serialize updated accounts: {}", e))?,
        false,
    )
    .await?;
    Ok(accounts_json.accounts.values().cloned().collect())
}

pub async fn add_account(account: LauncherAccount) -> Result<(), String> {
    let path = mc_dir()?.join(LAUNCHER_ACCOUNTS_FILE);
    let contents = read_str(&path).await.unwrap_or_else(|_| String::from("{\"accounts\": {}, \"mojangClientToken\": \"\"}"));
    let mut accounts_json: LauncherAccountsJson =
        serde_json::from_str(&contents).map_err(|e| format!("Failed to parse accounts JSON: {}", e))?;
    accounts_json.accounts.insert(account.local_id.clone(), account);
    write_str(
        &path,
        &serde_json::to_string_pretty(&accounts_json).map_err(|e| format!("Failed to serialize updated accounts: {}", e))?,
        false,
    )
    .await?;
    Ok(())
}
