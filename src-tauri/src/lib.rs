use std::path::PathBuf;
use std::process::Command;
use std::fs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Module declarations
// mod auth; // Skipped for now - focus on content management
mod settings;
mod profile;
mod maps;
mod mods;
mod shaders;
mod skins;
mod installations;

// Re-export public items from modules (excluding auth for now)
// pub use auth::{MicrosoftAccount, start_microsoft_auth, complete_microsoft_auth, refresh_minecraft_token, get_oauth_callback_result};
pub use settings::{LauncherSettings, load_settings, save_settings, get_launcher_dir, get_default_minecraft_directory, validate_minecraft_directory, MinecraftDirectoryInfo};
pub use maps::{LocalWorld, WorldDownload, get_local_worlds, delete_world, backup_world};
pub use shaders::{ShaderPack, get_installed_shaders, toggle_shader, delete_shader, install_shader_pack, get_shader_info};
pub use skins::{MinecraftSkin, get_local_skins, save_skin, delete_skin, install_skin, get_skin_data, get_current_minecraft_skin, upload_skin_to_minecraft};
pub use installations::{MinecraftInstallation, get_minecraft_installations, refresh_installation, update_installation_last_played};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("OAuth error: {0}")]
    OAuth(String),
    #[error("Minecraft error: {0}")]
    Minecraft(String),
}

impl From<AppError> for String {
    fn from(err: AppError) -> String {
        err.to_string()
    }
}



// Existing structures
#[derive(Debug, Serialize, Deserialize)]
pub struct LaunchOptions {
    pub version: String,
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub memory: u32, // in MB
    pub java_path: Option<String>,
    pub jvm_args: Option<Vec<String>>,
    pub game_args: Option<Vec<String>>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub fullscreen: Option<bool>,
}




// Read usernames from usercache.json for offline mode reference
#[tauri::command]
async fn get_cached_usernames(minecraft_path: String) -> Result<Vec<String>, String> {
    let usercache_path = PathBuf::from(&minecraft_path).join("usercache.json");
    
    if !usercache_path.exists() {
        return Ok(Vec::new());
    }
    
    let contents = fs::read_to_string(usercache_path).map_err(|e| e.to_string())?;
    let usercache: Vec<serde_json::Value> = serde_json::from_str(&contents).map_err(|e| e.to_string())?;
    
    let usernames: Vec<String> = usercache
        .iter()
        .filter_map(|entry| entry["name"].as_str().map(|s| s.to_string()))
        .collect();
    
    Ok(usernames)
}

// Launch Minecraft with specified options
#[tauri::command]
async fn launch_minecraft(options: LaunchOptions, minecraft_path: String) -> Result<String, String> {
    let minecraft_dir = PathBuf::from(&minecraft_path);
    let versions_dir = minecraft_dir.join("versions").join(&options.version);
    let jar_path = versions_dir.join(format!("{}.jar", options.version));
    
    if !jar_path.exists() {
        return Err(format!("Minecraft version {} not found", options.version));
    }
    
    // Basic launch command - this is simplified
    let java_args = vec![
        format!("-Xmx{}m", options.memory),
        "-cp".to_string(),
        jar_path.to_string_lossy().to_string(),
        "net.minecraft.client.main.Main".to_string(),
        "--username".to_string(),
        options.username.clone(),
        "--uuid".to_string(),
        options.uuid,
        "--accessToken".to_string(),
        options.access_token,
        "--version".to_string(),
        options.version,
        "--gameDir".to_string(),
        minecraft_dir.to_string_lossy().to_string(),
    ];
    
    match Command::new("java")
        .args(&java_args)
        .current_dir(&minecraft_dir)
        .spawn()
    {
        Ok(_) => Ok("Minecraft launched successfully".to_string()),
        Err(e) => Err(format!("Failed to launch Minecraft: {}", e)),
    }
}

// Check if Java is installed
#[tauri::command]
async fn check_java_installation() -> Result<String, String> {
    match Command::new("java").arg("-version").output() {
        Ok(output) => {
            let version_info = String::from_utf8_lossy(&output.stderr);
            Ok(version_info.to_string())
        }
        Err(_) => Err("Java not found. Please install Java to run Minecraft.".to_string()),
    }
}

// Get the default Minecraft directory
#[tauri::command]
async fn get_default_minecraft_dir() -> Result<String, String> {
    let possible_paths = vec![
        // Windows
        dirs::data_dir().map(|p| p.join(".minecraft")),
        dirs::home_dir().map(|p| p.join("AppData").join("Roaming").join(".minecraft")),
        // macOS
        dirs::home_dir().map(|p| p.join("Library").join("Application Support").join("minecraft")),
        // Linux
        dirs::home_dir().map(|p| p.join(".minecraft")),
    ];
    
    for path in possible_paths.into_iter().flatten() {
        if path.exists() {
            return Ok(path.to_string_lossy().to_string());
        }
    }
    
    // If no existing installation found, return the default path
    if let Some(appdata) = dirs::data_dir() {
        let minecraft_dir = appdata.join(".minecraft");
        Ok(minecraft_dir.to_string_lossy().to_string())
    } else {
        Err("Could not determine default Minecraft directory".to_string())
    }
}



#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_cached_usernames,
            launch_minecraft,
            check_java_installation,
            get_default_minecraft_dir,
            // Auth commands - skipped for now
            // auth::start_microsoft_auth,
            // auth::complete_microsoft_auth,
            // auth::refresh_minecraft_token,
            // auth::get_oauth_callback_result,
            // Settings commands
            settings::load_settings,
            settings::save_settings,
            settings::get_launcher_dir,
            settings::get_default_minecraft_directory,
            settings::validate_minecraft_directory,
            // Installation commands
            installations::get_minecraft_installations,
            installations::refresh_installation,
            installations::update_installation_last_played,
            // Maps/Worlds commands
            maps::get_local_worlds,
            maps::delete_world,
            maps::backup_world,
            // Shaders commands
            shaders::get_installed_shaders,
            shaders::toggle_shader,
            shaders::delete_shader,
            shaders::install_shader_pack,
            shaders::get_shader_info,
            // Skins commands
            skins::get_local_skins,
            skins::save_skin,
            skins::delete_skin,
            skins::install_skin,
            skins::get_skin_data,
            skins::get_current_minecraft_skin,
            skins::upload_skin_to_minecraft
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
