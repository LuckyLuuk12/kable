use crate::constants::SETTINGS_FILE;
use crate::system::fs::{get_kable_launcher_dir, read_to_string, write_file_atomic_async};
use api_types::settings::CategorizedLauncherSettings;
use std::path::PathBuf;

/// Get the settings file path
pub fn get_settings_path() -> Result<PathBuf, String> {
    Ok(get_kable_launcher_dir()?.join(SETTINGS_FILE))
}

// Load settings from the settings.json file
// pub fn load_settings() -> Result<CategorizedLauncherSettings, String> {
//     let settings_path = get_settings_path()?;

//     if !settings_path.exists() {
//         return Ok(CategorizedLauncherSettings::default());
//     }

//     let content = fs::read_to_string(&settings_path)
//         .map_err(|e| format!("Failed to read settings file: {}", e))?;

//     let settings: CategorizedLauncherSettings = serde_json::from_str(&content)
//         .map_err(|e| format!("Failed to parse settings file: {}", e))?;

//     Ok(settings)
// }

pub async fn load_settings() -> Result<CategorizedLauncherSettings, String> {
    let settings_path = get_settings_path()?;

    if !settings_path.exists() {
        return Ok(CategorizedLauncherSettings::default());
    }

    let content = read_to_string(&settings_path)
        .await
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    let settings = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings file: {}", e))?;

    Ok(settings)
}

/// Save settings to the settings.json file
pub async fn save_settings(settings: CategorizedLauncherSettings) -> Result<(), String> {
    let settings_path = get_settings_path()?;
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    write_file_atomic_async(&settings_path, content.as_bytes()).await?;

    Ok(())
}
