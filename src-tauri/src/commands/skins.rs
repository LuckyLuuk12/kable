// Skin commands - Tauri command wrappers for skin functionality
// These commands call the functions in the skins module

use crate::skins::{
    skin_upload::{
        upload_skin_to_account as upload_skin_impl,
        change_skin_model as change_model_impl,
        get_current_skin_info as get_skin_info_impl,
        select_skin_file as select_file_impl,
        get_all_account_skins as get_account_skins_impl,
        apply_account_skin as apply_skin_impl,
    },
    types::{SkinUploadConfig, SkinUploadResponse, CurrentSkin, SkinModel, AccountSkin},
};

/// Upload a skin file to the authenticated Microsoft/Mojang account
#[tauri::command]
pub async fn upload_skin_to_account(config: SkinUploadConfig) -> Result<SkinUploadResponse, String> {
    upload_skin_impl(config).await
}

/// Change the skin model (slim/classic) for the current skin
#[tauri::command]
pub async fn change_skin_model(new_model: SkinModel) -> Result<SkinUploadResponse, String> {
    change_model_impl(new_model).await
}

/// Get the current skin information from Mojang
#[tauri::command]
pub async fn get_current_skin_info() -> Result<CurrentSkin, String> {
    get_skin_info_impl().await
}

/// Select a skin file using the system file dialog
#[tauri::command]
pub async fn select_skin_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    select_file_impl(app).await
}

/// Get all skins stored in the user's Microsoft/Mojang account
#[tauri::command]
pub async fn get_all_account_skins() -> Result<Vec<AccountSkin>, String> {
    get_account_skins_impl().await
}

/// Apply an account skin (set it as the current skin)
#[tauri::command]
pub async fn apply_account_skin(skin_id: String) -> Result<SkinUploadResponse, String> {
    apply_skin_impl(skin_id).await
}