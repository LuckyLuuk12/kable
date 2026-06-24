// start_authentication, poll_authentication, list, add, remove, set active, get active accounts
use crate::features::accounts::management;
use crate::integrations::mojang_api::auth;
use api_types::auth::{DeviceCodeResponse, LauncherAccount, MicrosoftToken};

#[tauri::command]
#[specta::specta]
pub async fn start_authentication() -> Result<DeviceCodeResponse, String> {
    auth::start_authentication().await
}

#[tauri::command]
#[specta::specta]
pub async fn poll_authentication(device_code: String) -> Result<MicrosoftToken, String> {
    auth::poll_authentication(device_code).await
}

#[tauri::command]
#[specta::specta]
pub async fn list_accounts() -> Result<Vec<LauncherAccount>, String> {
    management::list_accounts().await
}

#[tauri::command]
#[specta::specta]
pub async fn add_account(account: LauncherAccount) -> Result<(), String> {
    management::add_account(account).await
}

#[tauri::command]
#[specta::specta]
pub async fn remove_account(account: LauncherAccount) -> Result<Vec<LauncherAccount>, String> {
    management::remove_account(account).await
}

#[tauri::command]
#[specta::specta]
pub async fn set_active_account(account: LauncherAccount) -> Result<(), String> {
    management::set_active_account(account).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_active_account() -> Result<Option<LauncherAccount>, String> {
    management::get_active_account().await
}
