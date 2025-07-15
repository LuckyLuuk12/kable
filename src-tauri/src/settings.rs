use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
// use crate::auth::MicrosoftAccount; // Skipped for now
use crate::AppError;

// Default functions for new settings fields
fn default_max_world_backups() -> u32 {
    5
}

fn default_shader_quality_preset() -> String {
    "medium".to_string()
}

fn default_enable_shader_caching() -> bool {
    true
}

fn default_selected_icon_template() -> String {
    "emoji".to_string()
}

fn default_sidebar_width() -> u32 {
    250
}

fn default_card_spacing() -> u32 {
    16
}

fn default_animation_speed() -> String {
    "normal".to_string()
}

fn default_parallel_downloads() -> u32 {
    3
}

fn default_connection_timeout() -> u32 {
    30
}

fn default_custom_settings() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

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
    // UI/UX settings - use default values if missing from file
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u32,
    #[serde(default = "default_card_spacing")]
    pub card_spacing: u32,
    #[serde(default = "default_animation_speed")]
    pub animation_speed: String,
    // Advanced settings - use default values if missing from file
    #[serde(default = "default_parallel_downloads")]
    pub parallel_downloads: u32,
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u32,
    #[serde(default)]
    pub enable_experimental_features: bool,
    // Accounts temporarily disabled - focus on content management
    // pub accounts: Vec<MicrosoftAccount>,
    // pub active_account_id: Option<String>,
    // New settings for content management - use default values if missing from file
    #[serde(default)]
    pub auto_backup_worlds: bool,
    #[serde(default = "default_max_world_backups")]
    pub max_world_backups: u32,
    #[serde(default = "default_shader_quality_preset")]
    pub shader_quality_preset: String,
    #[serde(default = "default_enable_shader_caching")]
    pub enable_shader_caching: bool,
    // Icon template settings
    #[serde(default = "default_selected_icon_template")]
    pub selected_icon_template: String,
    // Custom settings for extensibility - use default values if missing from file
    #[serde(default = "default_custom_settings")]
    pub custom: serde_json::Value,
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
            window_width: 1080,
            window_height: 720,
            sidebar_width: 250,
            card_spacing: 16,
            animation_speed: "normal".to_string(),
            parallel_downloads: 3,
            connection_timeout: 30,
            enable_experimental_features: false,
            // accounts: Vec::new(), // Disabled for now
            // active_account_id: None, // Disabled for now
            auto_backup_worlds: false,
            max_world_backups: 5,
            shader_quality_preset: "medium".to_string(),
            enable_shader_caching: true,
            selected_icon_template: "emoji".to_string(),
            custom: serde_json::Value::Object(serde_json::Map::new()),
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
