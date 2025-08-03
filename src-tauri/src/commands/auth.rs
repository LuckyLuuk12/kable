pub use crate::auth::*;

// Tauri command wrappers for all public async functions in the auth module

#[tauri::command]
pub async fn get_minecraft_account(auth_method: Option<AuthMethod>) -> Result<LauncherAccount, String> {
    crate::auth::get_minecraft_account(auth_method).await
}

#[tauri::command]
pub async fn get_launch_auth_account() -> Result<LauncherAccount, String> {
    crate::auth::get_launch_auth_account().await
}

#[tauri::command]
pub async fn refresh_minecraft_account() -> Result<LauncherAccount, String> {
    crate::auth::refresh_minecraft_account().await
}

#[tauri::command]
pub async fn read_launcher_accounts() -> Result<crate::auth::LauncherAccountsJson, String> {
    crate::auth::auth_util::read_launcher_accounts().await
}

#[tauri::command]
pub async fn write_launcher_accounts(accounts: crate::auth::LauncherAccountsJson) -> Result<(), String> {
    crate::auth::auth_util::write_launcher_accounts(accounts).await
}

#[tauri::command]
pub async fn write_launcher_account(account: crate::auth::LauncherAccount) -> Result<(), String> {
    crate::auth::auth_util::write_launcher_account(account).await
}

#[tauri::command]
pub async fn remove_launcher_account(account_id: String) -> Result<(), String> {
    crate::auth::auth_util::remove_launcher_account(account_id).await
}

#[tauri::command]
pub async fn set_active_launcher_account(account_id: String) -> Result<(), String> {
    crate::auth::auth_util::set_active_launcher_account(account_id).await
}

#[tauri::command]
pub async fn get_active_launcher_account() -> Result<Option<crate::auth::LauncherAccount>, String> {
    crate::auth::auth_util::get_active_launcher_account().await
}

#[tauri::command]
pub async fn get_all_launcher_accounts() -> Result<Vec<crate::auth::LauncherAccount>, String> {
    crate::auth::auth_util::get_all_launcher_accounts().await
}

#[tauri::command]
pub async fn get_launcher_accounts_path_string() -> Result<String, String> {
    crate::auth::auth_util::get_launcher_accounts_path_string().await
}

#[tauri::command]
pub async fn validate_and_cleanup_accounts() -> Result<String, String> {
    crate::auth::auth_util::validate_and_cleanup_accounts().await
}

#[tauri::command]
pub async fn start_microsoft_auth_code() -> Result<crate::auth::code_flow::AuthCodeResponse, String> {
    crate::auth::code_flow::start_microsoft_auth_code().await
}

#[tauri::command]
pub async fn complete_minecraft_auth_code(microsoft_token: crate::auth::code_flow::MicrosoftToken) -> Result<crate::auth::LauncherAccount, String> {
    crate::auth::code_flow::complete_minecraft_auth_code(microsoft_token).await
}

#[tauri::command]
pub async fn poll_microsoft_auth_code(state: String) -> Result<Option<crate::auth::code_flow::MicrosoftToken>, String> {
    crate::auth::code_flow::poll_microsoft_auth_code(state).await
}

#[tauri::command]
pub async fn start_microsoft_device_auth() -> Result<crate::auth::device_code_flow::DeviceCodeResponse, String> {
    crate::auth::device_code_flow::start_microsoft_device_auth().await
}

#[tauri::command]
pub async fn poll_microsoft_device_auth(device_code: String) -> Result<Option<crate::auth::code_flow::MicrosoftToken>, String> {
    crate::auth::device_code_flow::poll_microsoft_device_auth(device_code).await
}
