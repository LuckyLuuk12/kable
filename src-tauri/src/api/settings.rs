use crate::features::customization::settings;
use api_types::settings::CategorizedLauncherSettings;

#[tauri::command]
pub async fn get_settings() -> Result<CategorizedLauncherSettings, String> {
    settings::load_settings().await
}

#[tauri::command]
pub async fn set_settings(settings: CategorizedLauncherSettings) -> Result<(), String> {
    settings::save_settings(settings).await
}
