use crate::features::accounts::skins;

use api_types::auth::KableAccount;
use api_types::skins::{AccountCape, AccountSkin, CurrentSkin, PlayerProfile, SkinModel, SkinUploadConfig, SkinUploadResponse};

#[tauri::command]
#[specta::specta]
pub async fn get_player_profile(account: KableAccount) -> Result<PlayerProfile, String> {
    skins::get_player_profile(account).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_current_skin_info(account: KableAccount) -> Result<CurrentSkin, String> {
    skins::get_current_skin_info(account).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_all_account_skins(account: KableAccount) -> Result<Vec<AccountSkin>, String> {
    skins::get_all_account_skins(account).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_local_skins() -> Result<Vec<AccountSkin>, String> {
    skins::get_local_skins().await
}

#[tauri::command]
#[specta::specta]
pub async fn upload_skin_to_account(config: SkinUploadConfig) -> Result<SkinUploadResponse, String> {
    skins::upload_skin_to_account(config).await
}

#[tauri::command]
#[specta::specta]
pub async fn modify_skin_by_id(
    skin_id: String,
    new_name: Option<String>,
    new_cape_id: Option<String>,
    new_slim: Option<bool>,
) -> Result<(), String> {
    skins::modify_skin_by_id(skin_id, new_name, new_cape_id, new_slim).await
}

#[tauri::command]
#[specta::specta]
pub async fn remove_skin_by_id(skin_id: String) -> Result<(), String> {
    skins::remove_skin_by_id(skin_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn apply_account_skin(skin_id: String) -> Result<(), String> {
    skins::apply_account_skin(skin_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_skin_model(new_model: SkinModel) -> Result<(), String> {
    skins::change_skin_model(new_model).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_active_cape() -> Result<Option<AccountCape>, String> {
    skins::get_active_cape().await
}

#[tauri::command]
#[specta::specta]
pub async fn apply_cape(cape_id: Option<String>) -> Result<(), String> {
    skins::apply_cape(cape_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn download_skin_from_url(url: String) -> Result<Vec<u8>, String> {
    skins::download_skin_from_url(url).await
}
