use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::Manager;
use thiserror::Error;

// Module declarations
pub mod auth;
pub mod commands;
pub mod icons;
pub mod installations;
pub mod launcher;
pub mod maps;
pub mod mods;
pub mod profile;
pub mod settings;
pub mod shaders;
pub mod skins;
pub mod window_state;

#[macro_use]
pub mod logging;

// Re-export public items from modules
pub use auth::*;
pub use icons::*;
pub use installations::*;
pub use launcher::*;
pub use logging::*;
pub use maps::*;
pub use mods::*;
pub use settings::*;
pub use shaders::*;
pub use skins::*;
pub use window_state::*;
pub use commands::*;

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
    let usercache: Vec<serde_json::Value> =
        serde_json::from_str(&contents).map_err(|e| e.to_string())?;

    let usernames: Vec<String> = usercache
        .iter()
        .filter_map(|entry| entry["name"].as_str().map(|s| s.to_string()))
        .collect();

    Ok(usernames)
}

// Launch Minecraft with specified options
#[tauri::command]
async fn launch_minecraft(
    options: LaunchOptions,
    minecraft_path: String,
) -> Result<String, String> {
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

/// This starts the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            get_default_minecraft_dir,
            validate_minecraft_directory,
            // Main authentication commands
            auth::get_minecraft_account,
            auth::get_launch_auth_account,
            auth::refresh_minecraft_account,
            // Auth utilities (starting fresh) - using direct module paths
            auth::auth_util::read_launcher_accounts,
            auth::auth_util::write_launcher_accounts,
            auth::auth_util::write_launcher_account,
            auth::auth_util::remove_launcher_account,
            auth::auth_util::set_active_launcher_account,
            auth::auth_util::get_active_launcher_account,
            auth::auth_util::get_all_launcher_accounts,
            auth::auth_util::get_launcher_accounts_path_string,
            auth::auth_util::validate_and_cleanup_accounts,
            // Microsoft authentication commands - Device Code Flow
            auth::device_code_flow::start_microsoft_device_auth,
            auth::device_code_flow::poll_microsoft_device_auth,
            auth::device_code_flow::complete_minecraft_auth,
            // Microsoft authentication commands - Authorization Code Flow
            auth::code_flow::start_microsoft_auth_code,
            auth::code_flow::complete_minecraft_auth_code,
            auth::code_flow::poll_microsoft_auth_code,
            // Settings commands
            settings::load_settings,
            settings::save_settings,
            settings::validate_minecraft_directory,
            // Installation commands
            commands::installations::get_versions,
            commands::installations::get_all_versions,
            commands::installations::get_installations,
            commands::installations::get_installation,
            commands::installations::modify_installation,
            commands::installations::delete_installation,
            commands::installations::create_installation,
            // Launcher commands
            launcher::launch_installation,
            launcher::kill_minecraft_process,
            launcher::get_running_minecraft_processes,
            launcher::is_minecraft_running,
            launcher::wait_for_minecraft_exit,
            // Maps/Worlds commands
            maps::get_local_worlds,
            maps::delete_world,
            maps::backup_world,
            // Mods commands
            // mods::get_modded_installations,
            // mods::setup_installation_mods,
            // mods::get_installed_mods,
            // mods::toggle_mod_enabled,
            // mods::update_installation_mod_config,
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
            skins::upload_skin_to_minecraft,
            skins::select_skin_file,
            // Icons commands
            icons::get_custom_icon_templates,
            icons::save_custom_icon_template,
            icons::delete_custom_icon_template,
            icons::validate_icon_template,
            icons::get_icons_directory_path,
            icons::open_icons_directory,
            // Window state commands
            window_state::load_window_state,
            window_state::save_window_state,
            window_state::get_current_window_state,
            window_state::apply_window_state,
            window_state::get_monitor_info,
            window_state::show_main_window,
            // Logging commands
            logging::export_logs,
            logging::update_logging_config,
            logging::cleanup_old_logs,
            logging::get_log_stats,
            // System commands
            commands::system::open_url
        ])
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize global logger first
            logging::init_global_logger(app.handle());

            // Set up window state handlers
            if let Err(e) = setup_window_state_handlers(app) {
                Logger::console_log(
                    LogLevel::Error,
                    &format!("Failed to setup window state handlers: {}", e),
                    None,
                );
            }

            // Apply window state but don't show the window yet - let frontend trigger it
            if let Some(window) = app.get_webview_window("main") {
                tauri::async_runtime::spawn(async move {
                    if let Ok(state) = load_window_state().await {
                        if let Err(e) = apply_window_state(window.clone(), state).await {
                            Logger::console_log(
                                LogLevel::Warning,
                                &format!("Failed to apply window state: {}", e),
                                None,
                            );
                        }
                    } else {
                        Logger::console_log(
                            LogLevel::Warning,
                            "No saved window state found, using default settings",
                            None,
                        );
                    }
                    // Window will be shown by frontend after initialization
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Get the default Minecraft directory
#[tauri::command]
fn get_default_minecraft_dir() -> Result<PathBuf, String> {
    let possible_paths = vec![
        // Windows
        dirs::data_dir().map(|p| p.join(".minecraft")),
        dirs::home_dir().map(|p| p.join("AppData").join("Roaming").join(".minecraft")),
        // macOS
        dirs::home_dir().map(|p| {
            p.join("Library")
                .join("Application Support")
                .join("minecraft")
        }),
        // Linux
        dirs::home_dir().map(|p| p.join(".minecraft")),
    ];

    for path in possible_paths.into_iter().flatten() {
        if path.exists() {
            return Ok(path);
        }
    }

    // If no existing installation found, return the default path
    if let Some(appdata) = dirs::data_dir() {
        let minecraft_dir = appdata.join(".minecraft");
        Ok(minecraft_dir)
    } else {
        Err("Could not determine default Minecraft directory".to_string())
    }
}

/// Gets the kable dir inside the .minecraft folder
#[tauri::command]
fn get_minecraft_kable_dir() -> Result<PathBuf, String> {
    let default_dir = get_default_minecraft_dir()?;
    let kable_dir = default_dir.join("kable");
    if !kable_dir.exists() {
        fs::create_dir_all(&kable_dir).map_err(|e| e.to_string())?;
    }
    Ok(kable_dir)
}

/// Gets the kable-launcher folder, on windows this is inside Roaming/kable-launcher
#[tauri::command]
fn get_kable_launcher_dir() -> Result<PathBuf, String> {
    let kable_dir = get_minecraft_kable_dir()?;
    let launcher_dir = kable_dir.join("launcher");
    if !launcher_dir.exists() {
        fs::create_dir_all(&launcher_dir).map_err(|e| e.to_string())?;
    }
    Ok(launcher_dir)
}
