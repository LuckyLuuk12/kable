use crate::features::updater;

#[tauri::command]
#[specta::specta]
pub async fn check_launcher_updates(include_prerelease: bool) -> Result<api_types::updater::UpdateData, String> {
    updater::check_for_updates(include_prerelease).await
}

#[tauri::command]
#[specta::specta]
pub async fn install_launcher_update(include_prerelease: bool) -> Result<(), String> {
    updater::install_update(include_prerelease).await
}

#[tauri::command]
#[specta::specta]
pub async fn download_launcher_update(include_prerelease: bool) -> Result<String, String> {
    updater::download_update(include_prerelease).await
}

#[tauri::command]
#[specta::specta]
pub async fn apply_downloaded_update() -> Result<(), String> {
    updater::apply_downloaded_update().await
}

#[tauri::command]
#[specta::specta]
pub async fn get_current_version() -> Result<String, String> {
    updater::get_current_version().await
}
