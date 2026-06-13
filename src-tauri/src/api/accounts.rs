use api_types::auth::{DeviceCodeResponse, LauncherAccount, MicrosoftToken};

// start_authentication, poll_authentication, list, add, remove, set active, get active accounts
// use crate::features::accounts::{
//     add_account, get_active_account, list_accounts, remove_account, set_active_account,
// };
use crate::features::accounts::management;
use crate::integrations::mojang_api::auth;

#[tauri::command]
pub async fn start_authentication() -> Result<DeviceCodeResponse, String> {
    auth::start_authentication().await
}

#[tauri::command]
pub async fn poll_authentication(device_code: String) -> Result<MicrosoftToken, String> {
    auth::poll_authentication(device_code).await
}

#[tauri::command]
pub async fn list_accounts() -> Result<Vec<LauncherAccount>, String> {
    management::list_accounts().await
}

#[tauri::command]
pub async fn add_account(account: LauncherAccount) -> Result<(), String> {
    management::add_account(account).await
}

#[tauri::command]
pub async fn remove_account(account: LauncherAccount) -> Result<(), String> {
    management::remove_account(account).await
}

#[tauri::command]
pub async fn set_active_account(account: LauncherAccount) -> Result<(), String> {
    management::set_active_account(account).await
}

#[tauri::command]
pub async fn get_active_account() -> Result<Option<LauncherAccount>, String> {
    management::get_active_account().await
}
