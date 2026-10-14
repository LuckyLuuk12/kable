// start_authentication, poll_authentication, list, add, remove, set active, get active accounts

use crate::features::accounts::{kable_account, management};
use crate::integrations::mojang_api::auth;

use api_types::auth::{DeviceCodeResponse, KableAccount, MicrosoftToken};

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
pub async fn authenticate(token: MicrosoftToken) -> Result<KableAccount, String> {
    kable_account::authenticate(token).await
}

#[tauri::command]
#[specta::specta]
pub async fn list_accounts() -> Result<Vec<KableAccount>, String> {
    management::list_accounts().await
}

#[tauri::command]
#[specta::specta]
pub async fn add_account(account: KableAccount) -> Result<(), String> {
    management::add_account(account).await
}

#[tauri::command]
#[specta::specta]
pub async fn remove_account(account: KableAccount) -> Result<Vec<KableAccount>, String> {
    management::remove_account(account).await
}

#[tauri::command]
#[specta::specta]
pub async fn set_active_account(account: KableAccount) -> Result<(), String> {
    management::set_active_account(account).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_active_account() -> Result<Option<KableAccount>, String> {
    management::get_active_account().await
}
