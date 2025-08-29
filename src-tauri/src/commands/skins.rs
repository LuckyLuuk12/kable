/// Get all local skins from the Minecraft directory
#[tauri::command]
pub async fn get_local_skins() -> Result<Vec<AccountSkin>, String> {
    crate::skins::get_skins::get_local_skins().await
}
// Skin commands - Tauri command wrappers for skin functionality
// These commands call the functions in the skins module
use crate::skins::*;

/// Get the full player profile (id, name, skins, capes) from Mojang API
#[tauri::command]
pub async fn get_player_profile() -> Result<crate::skins::types::PlayerProfile, String> {
    crate::skins::get_player_profile().await
}

/// Remove a skin entry by its id from launcher_custom_skins.json
#[tauri::command]
pub fn remove_skin_by_id(skin_id: String) -> Result<(), String> {
    crate::skins::modify_skins::remove_skin_by_id(&skin_id)
}

/// Modify a skin entry by its id in launcher_custom_skins.json
#[tauri::command]
pub fn modify_skin_by_id(
    skin_id: String,
    new_name: Option<String>,
    new_cape_id: Option<String>,
    new_slim: Option<bool>,
) -> Result<(), String> {
    crate::skins::modify_skins::modify_skin_by_id(
        &skin_id,
        new_name.as_deref(),
        new_cape_id.as_deref(),
        new_slim,
    )
}

/// Upload a skin file to the authenticated Microsoft/Mojang account
#[tauri::command]
pub async fn upload_skin_to_account(
    config: SkinUploadConfig,
) -> Result<SkinUploadResponse, String> {
    crate::skins::upload_skin_to_account(config).await
}

/// Change the skin model (slim/classic) for the current skin
#[tauri::command]
pub async fn change_skin_model(new_model: SkinModel) -> Result<SkinUploadResponse, String> {
    crate::skins::change_skin_model(new_model).await
}

/// Get the current skin information from Mojang
#[tauri::command]
pub async fn get_current_skin_info() -> Result<CurrentSkin, String> {
    crate::skins::get_current_skin_info().await
}

/// Select a skin file using the system file dialog
#[tauri::command]
pub async fn select_skin_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    crate::skins::select_skin_file(app).await
}

/// Get all skins stored in the user's Microsoft/Mojang account
#[tauri::command]
pub async fn get_all_account_skins() -> Result<Vec<AccountSkin>, String> {
    crate::skins::get_all_account_skins().await
}

/// Apply an account skin (set it as the current skin)
#[tauri::command]
pub async fn apply_account_skin(skin_id: String) -> Result<SkinUploadResponse, String> {
    crate::skins::apply_account_skin(skin_id).await
}
