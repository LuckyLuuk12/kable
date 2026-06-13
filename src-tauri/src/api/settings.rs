use api_types::settings::CategorizedLauncherSettings;
use crate::features::customization::settings;

#[tauri::command]
pub fn get_settings() -> Result<CategorizedLauncherSettings, String> {
    settings::load_settings()
}

#[tauri::command]
pub async fn set_settings(settings: CategorizedLauncherSettings) -> Result<(), String> {
    settings::save_settings(settings).await
}