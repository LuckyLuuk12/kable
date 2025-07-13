use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::auth::MicrosoftAccount;
use crate::AppError;

// Launcher settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LauncherSettings {
    pub theme: String,
    pub language: String,
    pub minecraft_path: Option<String>,
    pub default_memory: u32,
    pub max_memory: u32,
    pub java_path: Option<String>,
    pub keep_launcher_open: bool,
    pub show_logs_on_launch: bool,
    pub auto_update_launcher: bool,
    pub close_launcher_on_game_start: bool,
    pub window_width: u32,
    pub window_height: u32,
    pub accounts: Vec<MicrosoftAccount>,
    pub active_account_id: Option<String>,
}

impl Default for LauncherSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            language: "en".to_string(),
            minecraft_path: None,
            default_memory: 2048,
            max_memory: 8192,
            java_path: None,
            keep_launcher_open: true,
            show_logs_on_launch: false,
            auto_update_launcher: true,
            close_launcher_on_game_start: false,
            window_width: 1200,
            window_height: 800,
            accounts: Vec::new(),
            active_account_id: None,
        }
    }
}

// Helper functions
fn get_launcher_data_dir() -> Result<PathBuf, AppError> {
    let base_dir = if let Some(appdata) = dirs::data_dir() {
        appdata
    } else {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find data directory"
        )));
    };
    
    let launcher_dir = base_dir.join("kable-launcher");
    
    if !launcher_dir.exists() {
        fs::create_dir_all(&launcher_dir)?;
    }
    
    Ok(launcher_dir)
}

fn get_settings_path() -> Result<PathBuf, AppError> {
    Ok(get_launcher_data_dir()?.join("settings.json"))
}

// Settings management
#[tauri::command]
pub async fn load_settings() -> Result<LauncherSettings, String> {
    let settings_path = get_settings_path().map_err(|e| e.to_string())?;
    
    if settings_path.exists() {
        let contents = fs::read_to_string(&settings_path).map_err(|e| e.to_string())?;
        let settings: LauncherSettings = serde_json::from_str(&contents).map_err(|e| e.to_string())?;
        Ok(settings)
    } else {
        let default_settings = LauncherSettings::default();
        save_settings(default_settings.clone()).await?;
        Ok(default_settings)
    }
}

#[tauri::command]
pub async fn save_settings(settings: LauncherSettings) -> Result<(), String> {
    let settings_path = get_settings_path().map_err(|e| e.to_string())?;
    let contents = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&settings_path, contents).map_err(|e| e.to_string())?;
    Ok(())
}

// Get launcher data directory
#[tauri::command]
pub async fn get_launcher_dir() -> Result<String, String> {
    let launcher_dir = get_launcher_data_dir().map_err(|e| e.to_string())?;
    Ok(launcher_dir.to_string_lossy().to_string())
}
