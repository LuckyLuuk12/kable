use crate::constants::SETTINGS_FILE;
use crate::system::fs::{launcher_dir, read_str, write_str};
use api_types::settings::CategorizedLauncherSettings;
use kable_macros::persistent_cache;
use std::path::PathBuf;

/// Get the settings file path
pub async fn get_settings_path() -> Result<PathBuf, String> {
    // Ok(launcher_dir()?.join(SETTINGS_FILE))
    // Create with default settings if it doesn't exist
    let path = launcher_dir()?.join(SETTINGS_FILE);
    if !path.exists() {
        let default_settings = CategorizedLauncherSettings::default();
        write_str(
            &path,
            &serde_json::to_string_pretty(&default_settings).map_err(|e| format!("Failed to serialize default settings: {}", e))?,
            false,
        )
        .await
        .map_err(|e| format!("Failed to write default settings file: {}", e))?;
    }
    // Return the path
    Ok(path)
}

#[persistent_cache(parent = "settings")]
pub async fn load_settings() -> Result<CategorizedLauncherSettings, String> {
    let settings_path = get_settings_path().await?;

    let content = read_str(&settings_path).await.map_err(|e| format!("Failed to read settings file: {}", e))?;

    let settings = serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings file: {}", e))?;

    Ok(settings)
}

/// Save settings to the settings.json file
pub async fn save_settings(settings: CategorizedLauncherSettings) -> Result<(), String> {
    let settings_path = get_settings_path().await?;
    let content = serde_json::to_string_pretty(&settings).map_err(|e| format!("Failed to serialize settings: {}", e))?;

    write_str(&settings_path, &content, false).await?;
    crate::system::cache::invalidate_no_args("settings", "load_settings").await?;
    Ok(())
}
