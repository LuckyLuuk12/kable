use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
// use crate::auth::MicrosoftAccount; // Skipped for now
use crate::AppError;
use crate::logging::{Logger, LogLevel};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategorizedLauncherSettings {
    pub general: GeneralSettings,
    pub appearance: AppearanceSettings,
    pub logging: LoggingSettings,
    pub network: NetworkSettings,
    pub content: ContentSettings,
    pub advanced: AdvancedSettings,
    pub misc: MiscSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralSettings {
    pub java_path: Option<String>,
    pub game_directory: Option<String>,
    pub on_game_close: String, // 'exit' | 'minimize' | 'ask'
    pub on_game_crash: String, // 'restart' | 'close' | 'ask'
    pub on_game_launch: String, // 'keep_open' | 'close_launcher' | 'open_logs' | 'ask'
    pub auto_update_launcher: bool,
    pub show_ads: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppearanceSettings {
    pub theme: String, // 'light' | 'dark' | 'system'
    pub language: String,
    pub extra_spacing: u32,
    pub sidebar_width: u32,
    pub selected_icon_template: String,
    // For simplicity, icon_settings as raw JSON
    pub icon_settings: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingSettings {
    pub show_logs_page_in_nav: bool,
    pub enable_persistent_logging: bool,
    pub enable_log_compression: bool,
    pub log_file_size_limit_mb: serde_json::Value, // number or "disabled"
    pub log_retention_days: serde_json::Value, // number or "disabled"
    pub merge_log_tabs: bool,
    pub default_log_levels: Vec<String>, // 'debug' | 'info' | 'warn' | 'error'
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkSettings {
    pub parallel_downloads: u32,
    pub connection_timeout: u32,
    pub download_speed_limit: serde_json::Value, // number or "unlimited"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentSettings {
    pub max_world_backups: serde_json::Value, // number or "disabled"
    pub auto_backup_worlds: bool,
    pub use_per_installation_mods_folder: bool,
    pub use_per_installation_resource_packs: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdvancedSettings {
    pub enable_experimental_features: bool,
    pub default_memory: u32,
    pub separate_logs_window: bool,
    pub auto_save_interval: u32, // in seconds, 0 means no auto save
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MiscSettings {
    pub use_titlebar: bool,
    pub auth_preference: String, // 'code' | 'device_code'
}

impl Default for CategorizedLauncherSettings {
    fn default() -> Self {
        Self {
            general: GeneralSettings {
                java_path: None,
                game_directory: None,
                on_game_close: "exit".to_string(),
                on_game_crash: "restart".to_string(),
                on_game_launch: "keep_open".to_string(),
                auto_update_launcher: true,
                show_ads: false,
            },
            appearance: AppearanceSettings {
                theme: "dark".to_string(),
                language: "en".to_string(),
                extra_spacing: 0,
                sidebar_width: 250,
                selected_icon_template: "emoji".to_string(),
                icon_settings: serde_json::Value::Object(serde_json::Map::new()),
            },
            logging: LoggingSettings {
                show_logs_page_in_nav: true,
                enable_persistent_logging: false,
                enable_log_compression: true,
                log_file_size_limit_mb: serde_json::json!(10),
                log_retention_days: serde_json::json!(30),
                merge_log_tabs: false,
                default_log_levels: vec!["info".to_string(), "warn".to_string(), "error".to_string()],
            },
            network: NetworkSettings {
                parallel_downloads: 3,
                connection_timeout: 30,
                download_speed_limit: serde_json::json!("unlimited"),
            },
            content: ContentSettings {
                max_world_backups: serde_json::json!(5),
                auto_backup_worlds: false,
                use_per_installation_mods_folder: false,
                use_per_installation_resource_packs: false,
            },
            advanced: AdvancedSettings {
                enable_experimental_features: false,
                default_memory: 2048,
                separate_logs_window: false,
                auto_save_interval: 30000, // in milliseconds
            },
            misc: MiscSettings {
                use_titlebar: true,
                auth_preference: "code".to_string(),
            },
        }
    }
}

// Helper functions
pub fn get_launcher_data_dir() -> Result<PathBuf, AppError> {
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
pub async fn load_settings() -> Result<CategorizedLauncherSettings, String> {
    let settings_path = get_settings_path().map_err(|e| e.to_string())?;
    
    if settings_path.exists() {
        let contents = fs::read_to_string(&settings_path).map_err(|e| e.to_string())?;
        let settings: CategorizedLauncherSettings = serde_json::from_str(&contents).map_err(|e| e.to_string())?;
        Ok(settings)
    } else {
        let default_settings = CategorizedLauncherSettings::default();
        save_settings(default_settings.clone()).await?;
        Ok(default_settings)
    }
}

#[tauri::command]
pub async fn save_settings(settings: CategorizedLauncherSettings) -> Result<(), String> {
    let settings_path = get_settings_path().map_err(|e| e.to_string())?;
    let contents = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&settings_path, contents).map_err(|e| e.to_string())?;
    Logger::console_log(LogLevel::Info, &format!("Settings saved to: {}", settings_path.display()), None);
    Ok(())
}

// Get launcher data directory
#[tauri::command]
pub async fn get_launcher_dir() -> Result<String, String> {
    let launcher_dir = get_launcher_data_dir().map_err(|e| e.to_string())?;
    Ok(launcher_dir.to_string_lossy().to_string())
}

// Get default Minecraft directory
#[tauri::command]
pub async fn get_default_minecraft_directory() -> Result<String, String> {
    let minecraft_dir = if cfg!(target_os = "windows") {
        if let Some(appdata) = dirs::data_dir() {
            appdata.join(".minecraft")
        } else {
            return Err("Could not find AppData directory".to_string());
        }
    } else if cfg!(target_os = "macos") {
        if let Some(home) = dirs::home_dir() {
            home.join("Library").join("Application Support").join("minecraft")
        } else {
            return Err("Could not find home directory".to_string());
        }
    } else {
        // Linux
        if let Some(home) = dirs::home_dir() {
            home.join(".minecraft")
        } else {
            return Err("Could not find home directory".to_string());
        }
    };
    
    Ok(minecraft_dir.to_string_lossy().to_string())
}

// Validate Minecraft directory
#[tauri::command]
pub async fn validate_minecraft_directory(path: String) -> Result<MinecraftDirectoryInfo, String> {
    let minecraft_path = PathBuf::from(path);
    
    if !minecraft_path.exists() {
        return Err("Directory does not exist".to_string());
    }
    
    let saves_dir = minecraft_path.join("saves");
    let shaderpacks_dir = minecraft_path.join("shaderpacks");
    let resourcepacks_dir = minecraft_path.join("resourcepacks");
    
    let has_saves = saves_dir.exists();
    let has_shaderpacks = shaderpacks_dir.exists();
    let has_resourcepacks = resourcepacks_dir.exists();
    
    // Count directories/files
    let saves_count = if has_saves {
        count_directories(&saves_dir).unwrap_or(0)
    } else {
        0
    };
    
    let shaderpacks_count = if has_shaderpacks {
        count_files_with_extensions(&shaderpacks_dir, &[".zip", ".jar"]).unwrap_or(0)
    } else {
        0
    };
    
    let resourcepacks_count = if has_resourcepacks {
        count_files_with_extensions(&resourcepacks_dir, &[".zip", ".jar"]).unwrap_or(0)
    } else {
        0
    };
    
    // Calculate directory size (approximation)
    let size_mb = calculate_directory_size(&minecraft_path).unwrap_or(0) / (1024 * 1024);
    
    let is_valid = minecraft_path.join("launcher_profiles.json").exists() || 
                   minecraft_path.join("versions").exists() ||
                   has_saves || has_shaderpacks || has_resourcepacks;
    
    Ok(MinecraftDirectoryInfo {
        path: minecraft_path.to_string_lossy().to_string(),
        is_valid,
        has_saves,
        has_shaderpacks,
        has_resourcepacks,
        saves_count,
        shaderpacks_count,
        resourcepacks_count,
        size_mb,
    })
}

// Helper function to count directories
fn count_directories(path: &PathBuf) -> Result<u32, std::io::Error> {
    let mut count = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            count += 1;
        }
    }
    Ok(count)
}

// Helper function to count files with specific extensions
fn count_files_with_extensions(path: &PathBuf, extensions: &[&str]) -> Result<u32, std::io::Error> {
    let mut count = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            if let Some(file_name) = entry.file_name().to_str() {
                for ext in extensions {
                    if file_name.to_lowercase().ends_with(ext) {
                        count += 1;
                        break;
                    }
                }
            }
        }
    }
    Ok(count)
}

// Helper function to calculate directory size
fn calculate_directory_size(path: &PathBuf) -> Result<u64, std::io::Error> {
    let mut size = 0;
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                size += entry.metadata()?.len();
            } else if path.is_dir() {
                size += calculate_directory_size(&path)?;
            }
        }
    }
    Ok(size)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftDirectoryInfo {
    pub path: String,
    pub is_valid: bool,
    pub has_saves: bool,
    pub has_shaderpacks: bool,
    pub has_resourcepacks: bool,
    pub saves_count: u32,
    pub shaderpacks_count: u32,
    pub resourcepacks_count: u32,
    pub size_mb: u64,
}
