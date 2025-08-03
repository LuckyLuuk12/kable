use std::fs;
use std::path::PathBuf;
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
pub use commands::auth as commands_auth;
pub use commands::installations as commands_installations;
pub use commands::launcher as commands_launcher;
pub use commands::mods as commands_mods;
pub use commands::system as commands_system;


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

/// This starts the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            get_default_minecraft_dir,
            validate_minecraft_directory,
            // Main authentication commands
            commands_auth::get_minecraft_account,
            commands_auth::get_launch_auth_account,
            commands_auth::refresh_minecraft_account,
            // Auth utilities (starting fresh) - using direct module paths
            commands_auth::read_launcher_accounts,
            commands_auth::write_launcher_accounts,
            commands_auth::write_launcher_account,
            commands_auth::remove_launcher_account,
            commands_auth::set_active_launcher_account,
            commands_auth::get_active_launcher_account,
            commands_auth::get_all_launcher_accounts,
            commands_auth::get_launcher_accounts_path_string,
            commands_auth::validate_and_cleanup_accounts,
            // Microsoft authentication commands - Device Code Flow
            commands_auth::start_microsoft_device_auth,
            commands_auth::poll_microsoft_device_auth,
            commands_auth::complete_minecraft_auth,
            // Microsoft authentication commands - Authorization Code Flow
            commands_auth::start_microsoft_auth_code,
            commands_auth::complete_minecraft_auth_code,
            commands_auth::poll_microsoft_auth_code,
            // Settings commands
            settings::load_settings,
            settings::save_settings,
            settings::validate_minecraft_directory,
            // Installation commands
            commands_installations::get_versions,
            commands_installations::get_all_versions,
            commands_installations::get_installations,
            commands_installations::get_installation,
            commands_installations::modify_installation,
            commands_installations::delete_installation,
            commands_installations::create_installation,
            // Launcher commands
            commands_launcher::launch_installation,
            commands_launcher::kill_minecraft_process,
            commands_launcher::get_running_minecraft_processes,
            commands_launcher::is_minecraft_running,
            commands_launcher::wait_for_minecraft_exit,
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
            commands_system::open_url
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
