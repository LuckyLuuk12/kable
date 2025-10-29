use std::fs;
use std::path::{Path, PathBuf};

// Tokio async fs helpers are used by async helpers below. These helpers
// allow other modules to ensure parent directories and files exist without
// blocking the executor. They return Result<T, String> for consistency with
// other commands in this crate (see get_default_minecraft_dir etc.).
use tokio::fs as async_fs;

// Re-export procedural macros from the separate kable-macros crate
// The actual macro implementations are in `../kable-macros/src/lib.rs`
pub use kable_macros::*;

// Module declarations
pub mod auth;
pub mod commands;
pub mod discord;
pub mod icons;
pub mod installations;
pub mod launcher;
pub mod maps;
pub mod mods;
pub mod profile;
pub mod resourcepacks;
pub mod settings;
pub mod shaders;
pub mod skins;
pub mod symlink_manager;

#[macro_use]
pub mod logging;

// Re-export public items from modules
pub use auth::*;
pub use commands::auth as commands_auth;
pub use commands::discord as commands_discord;
pub use commands::installations as commands_installations;
pub use commands::launcher as commands_launcher;
pub use commands::mods as commands_mods;
pub use commands::resourcepacks as commands_resourcepacks;
pub use commands::shaders as commands_shaders;
pub use commands::skins as commands_skins;
pub use commands::symlinks as commands_symlinks;
pub use commands::system as commands_system;
pub use commands::updater as commands_updater;
pub use icons::*;
pub use installations::*;
pub use launcher::*;
pub use logging::*;
pub use maps::*;
pub use mods::*;
pub use resourcepacks::*;
pub use settings::*;
pub use shaders::*;
pub use skins::*;

// !NOTE: READ THIS !!
// !TODO: Add funny sounds for certain buttons and actions
// !NOTE: READ THIS !!

/// This starts the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Handle CLI arguments before building Tauri app
    let args: Vec<String> = std::env::args().collect();
    
    // Check for --launch-installation argument
    for arg in args.iter() {
        if arg.starts_with("--launch-installation=") {
            let installation_id = arg.trim_start_matches("--launch-installation=");
            
            // Launch the installation directly without showing UI
            tauri::async_runtime::block_on(async {
                match crate::installations::get_installation(installation_id).await {
                    Ok(Some(installation)) => {
                        eprintln!("Launching installation: {}", installation.name);
                        
                        // Load settings and account
                        let settings = match crate::settings::load_settings().await {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("Failed to load settings: {}", e);
                                std::process::exit(1);
                            }
                        };
                        
                        let account = match crate::auth::auth_util::get_active_launcher_account().await {
                            Ok(Some(acc)) => acc,
                            Ok(None) => {
                                eprintln!("No active account found. Please log in through the launcher.");
                                std::process::exit(1);
                            }
                            Err(e) => {
                                eprintln!("Failed to get active account: {}", e);
                                std::process::exit(1);
                            }
                        };
                        
                        if let Err(e) = crate::launcher::launch_installation(installation, settings, account).await {
                            eprintln!("Failed to launch installation: {}", e);
                            std::process::exit(1);
                        }
                        
                        eprintln!("Installation launched successfully");
                        std::process::exit(0);
                    }
                    Ok(None) => {
                        eprintln!("Installation not found: {}", installation_id);
                        std::process::exit(1);
                    }
                    Err(e) => {
                        eprintln!("Failed to get installation: {}", e);
                        std::process::exit(1);
                    }
                }
            });
        }
    }
    
    tauri::Builder::default()
        .setup(|app| {
            // Initialize global logger with the app handle so modules that
            // use GLOBAL_APP_HANDLE (e.g. launcher utils) can emit events.
            crate::logging::init_global_logger(app.handle());

            // Initialize Discord Rich Presence
            tauri::async_runtime::spawn(async {
                if let Err(e) = crate::discord::initialize() {
                    Logger::warn_global(
                        &format!("[STARTUP] Failed to initialize Discord RPC: {}", e),
                        None,
                    );
                } else {
                    Logger::info_global("[STARTUP] Discord Rich Presence initialized", None);
                }
            });

            // Clean up any leftover symlinks from previous crashes/exits
            tauri::async_runtime::spawn(async {
                if let Ok(minecraft_dir) = get_default_minecraft_dir() {
                    let symlink_manager =
                        crate::symlink_manager::SymlinkManager::new(minecraft_dir);
                    if let Err(e) = symlink_manager.cleanup_all_symlinks().await {
                        Logger::warn_global(
                            &format!("[STARTUP] Failed to cleanup leftover symlinks: {}", e),
                            None,
                        );
                    } else {
                        Logger::info_global(
                            "[STARTUP] Cleaned up leftover symlinks from previous session",
                            None,
                        );
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            // Clean up symlinks when the main window is closed
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if window.label() == "main" {
                    tauri::async_runtime::spawn(async {
                        // Disconnect Discord RPC
                        if let Err(e) = crate::discord::disconnect() {
                            Logger::warn_global(
                                &format!("[SHUTDOWN] Failed to disconnect Discord RPC: {}", e),
                                None,
                            );
                        }

                        if let Ok(minecraft_dir) = get_default_minecraft_dir() {
                            let symlink_manager =
                                crate::symlink_manager::SymlinkManager::new(minecraft_dir);
                            if let Err(e) = symlink_manager.cleanup_all_symlinks().await {
                                Logger::warn_global(
                                    &format!("[SHUTDOWN] Failed to cleanup symlinks: {}", e),
                                    None,
                                );
                            } else {
                                Logger::info_global(
                                    "[SHUTDOWN] Cleaned up all symlinks on app close",
                                    None,
                                );
                            }
                        }
                    });
                }
            }
        })
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_default_minecraft_dir,
            validate_minecraft_directory,
            // Main authentication commands
            commands_auth::refresh_microsoft_token,
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
            settings::save_settings_command,
            settings::validate_minecraft_directory,
            settings::load_custom_css,
            settings::set_selected_css_theme,
            settings::get_selected_css_theme,
            settings::get_css_themes,
            settings::save_css_theme,
            settings::delete_css_theme,
            settings::load_css_theme,
            settings::open_css_themes_directory,
            // Installation commands
            commands_installations::get_versions,
            commands_installations::get_all_versions,
            commands_installations::get_installations,
            commands_installations::get_installation,
            commands_installations::modify_installation,
            commands_installations::delete_installation,
            commands_installations::create_installation,
            commands_installations::create_installation_from_existing,
            commands_installations::get_mod_info,
            commands_installations::disable_mod,
            commands_installations::enable_mod,
            commands_installations::toggle_mod_disabled,
            commands_installations::import,
            commands_installations::export,
            commands_installations::duplicate,
            commands_installations::create_shortcut,
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
            commands_mods::get_mods,
            commands_mods::download_mod,
            commands_mods::get_project_versions,
            commands_mods::set_provider_filter,
            commands_mods::set_provider_limit,
            commands_mods::clear_provider_cache,
            commands_mods::purge_stale_provider_cache,
            commands_mods::get_extended_mod_info,
            // Shaders commands
            commands_shaders::get_installed_shaders,
            commands_shaders::toggle_shader,
            commands_shaders::delete_shader,
            commands_shaders::install_shader_pack,
            commands_shaders::get_shader_info,
            commands_shaders::search_modrinth_shaders,
            commands_shaders::get_modrinth_shader_details,
            commands_shaders::download_and_install_shader,
            commands_shaders::download_and_install_shader_to_dedicated,
            commands_shaders::setup_shader_symlink,
            commands_shaders::remove_shader_symlink,
            commands_shaders::delete_shader_from_dedicated,
            commands_shaders::search_modrinth_shaders_with_facets,
            // Resource packs commands
            commands_resourcepacks::get_installed_resourcepacks,
            commands_resourcepacks::delete_resourcepack,
            commands_resourcepacks::install_resourcepack,
            commands_resourcepacks::get_resourcepack_info,
            commands_resourcepacks::search_modrinth_resourcepacks,
            commands_resourcepacks::search_modrinth_resourcepacks_with_facets,
            commands_resourcepacks::get_modrinth_resourcepack_details,
            commands_resourcepacks::download_and_install_resourcepack,
            commands_resourcepacks::download_and_install_resourcepack_to_dedicated,
            commands_resourcepacks::setup_resourcepack_symlink,
            commands_resourcepacks::remove_resourcepack_symlink,
            commands_resourcepacks::delete_resourcepack_from_dedicated,
            // Discord commands
            commands_discord::discord_set_browsing,
            commands_discord::discord_set_enabled,
            commands_discord::discord_clear,
            // Skins commands
            commands_skins::upload_skin_to_account,
            commands_skins::change_skin_model,
            commands_skins::get_current_skin_info,
            commands_skins::select_skin_file,
            commands_skins::get_all_account_skins,
            commands_skins::get_local_skins,
            commands_skins::apply_account_skin,
            commands_skins::remove_skin_by_id,
            commands_skins::modify_skin_by_id,
            commands_skins::get_player_profile,
            commands_skins::get_active_cape,
            commands_skins::apply_cape,
            // Symlinks commands
            commands_symlinks::list_symlinks,
            commands_symlinks::create_custom_symlink,
            commands_symlinks::remove_symlink,
            commands_symlinks::toggle_symlink_disabled,
            commands_symlinks::update_symlink,
            commands_symlinks::select_file_for_symlink,
            commands_symlinks::select_folder_for_symlink,
            // Icons commands
            icons::get_custom_icon_templates,
            icons::save_custom_icon_template,
            icons::delete_custom_icon_template,
            icons::validate_icon_template,
            icons::get_icons_directory_path,
            icons::open_icons_directory,
            // Image helper
            commands::images::resolve_image_path,
            // Logging commands
            logging::export_logs,
            logging::update_logging_config,
            logging::cleanup_old_logs,
            logging::get_log_stats,
            // System commands
            commands_system::open_url,
            commands_system::open_path,
            // Updater commands
            commands_updater::check_for_updates,
            commands_updater::install_update,
            commands_updater::get_current_version
        ])
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Get the default Minecraft directory
#[tauri::command]
fn get_default_minecraft_dir() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Could not find home directory".to_string())?;

    #[cfg(target_os = "windows")]
    let minecraft_dir = home_dir.join("AppData").join("Roaming").join(".minecraft");

    #[cfg(target_os = "macos")]
    let minecraft_dir = home_dir
        .join("Library")
        .join("Application Support")
        .join("minecraft");

    #[cfg(target_os = "linux")]
    let minecraft_dir = home_dir.join(".minecraft");

    Ok(minecraft_dir)
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

/// Ensure the parent directory for `path` exists (async). No-op if the
/// path has no parent. Returns Err when directory creation fails.
pub async fn ensure_parent_dir_exists_async(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        async_fs::create_dir_all(parent).await.map_err(|e| {
            format!(
                "failed to create parent directories {}: {}",
                parent.display(),
                e
            )
        })?;
    }
    Ok(())
}

/// Ensure a file exists at `path`. Creates parent directories and the file
/// if necessary. Returns the provided PathBuf on success.
pub async fn ensure_file(path: PathBuf) -> Result<PathBuf, String> {
    ensure_parent_dir_exists_async(&path).await?;
    // OpenOptions::new().create(true) will create the file if it does not exist
    async_fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .map_err(|e| format!("failed to create or open file {}: {}", path.display(), e))?;
    Ok(path)
}

/// Ensure a file exists at `path` and, if the file does not already exist,
/// write `contents` to it. Returns the provided PathBuf on success.
pub async fn ensure_file_with(path: PathBuf, contents: &str) -> Result<PathBuf, String> {
    match async_fs::metadata(&path).await {
        Ok(md) => {
            if md.is_file() {
                return Ok(path);
            }
            // If it exists but is not a file, return an error.
            Err(format!("path exists but is not a file: {}", path.display()))
        }
        Err(e) => {
            // If error is NotFound, create parent dirs and write the file.
            if e.kind() == std::io::ErrorKind::NotFound {
                ensure_parent_dir_exists_async(&path).await?;
                async_fs::write(&path, contents.as_bytes())
                    .await
                    .map_err(|e| format!("failed to write file {}: {}", path.display(), e))?;
                return Ok(path);
            }
            Err(format!("failed to stat path {}: {}", path.display(), e))
        }
    }
}

/// Ensure a directory exists at `path`. If it exists and is a file, return an error.
/// Creates parent directories as necessary. Returns Ok(()) on success.
pub async fn ensure_folder(path: &Path) -> Result<PathBuf, String> {
    match async_fs::metadata(path).await {
        Ok(md) => {
            if md.is_dir() {
                Ok(path.to_path_buf())
            } else {
                Err(format!(
                    "path exists but is not a directory: {}",
                    path.display()
                ))
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                async_fs::create_dir_all(path)
                    .await
                    .map_err(|e| format!("failed to create directory {}: {}", path.display(), e))?;
                Ok(path.to_path_buf())
            } else {
                Err(format!("failed to stat path {}: {}", path.display(), e))
            }
        }
    }
}

/// Synchronous variant of ensure_folder for use in blocking contexts.
pub fn ensure_folder_sync(path: &Path) -> Result<PathBuf, String> {
    match std::fs::metadata(path) {
        Ok(md) => {
            if md.is_dir() {
                Ok(path.to_path_buf())
            } else {
                Err(format!(
                    "path exists but is not a directory: {}",
                    path.display()
                ))
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                std::fs::create_dir_all(path)
                    .map_err(|e| format!("failed to create directory {}: {}", path.display(), e))?;
                Ok(path.to_path_buf())
            } else {
                Err(format!("failed to stat path {}: {}", path.display(), e))
            }
        }
    }
}

/// Atomically write bytes to `path` by creating a temporary file in the same
/// directory and renaming it into place. This avoids partial file writes.
pub async fn write_file_atomic_async(path: &Path, bytes: &[u8]) -> Result<(), String> {
    // Ensure parent exists
    ensure_parent_dir_exists_async(path).await?;

    // Work with owned PathBufs so we can move them into the blocking task
    let path_buf = path.to_path_buf();
    let parent = path_buf
        .parent()
        .ok_or_else(|| format!("Path has no parent: {}", path_buf.display()))?
        .to_path_buf();

    // Create a temp filename in the same directory
    let mut tmp = parent.clone();
    let tmp_name = format!(".{}.tmp", uuid::Uuid::new_v4());
    tmp.push(tmp_name);

    // Write to temp file asynchronously
    async_fs::write(&tmp, bytes)
        .await
        .map_err(|e| format!("failed to write temp file {}: {}", tmp.display(), e))?;

    // Move owned PathBufs into the blocking task and rename
    let tmp_move = tmp.clone();
    let final_move = path_buf.clone();
    tokio::task::spawn_blocking(move || std::fs::rename(&tmp_move, &final_move))
        .await
        .map_err(|e| format!("rename join error: {}", e))?
        .map_err(|e| format!("failed to atomically rename into place: {}", e))?;

    Ok(())
}

/// Synchronous variant of atomic write: writes bytes to a temp file in the same
/// directory and renames it into place.
pub fn write_file_atomic_sync(path: &Path, bytes: &[u8]) -> Result<(), String> {
    // Ensure parent exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create parent dirs {}: {}", parent.display(), e))?;
    }

    let parent = path
        .parent()
        .ok_or_else(|| format!("Path has no parent: {}", path.display()))?;
    let mut tmp = parent.to_path_buf();
    let tmp_name = format!(".{}.tmp", uuid::Uuid::new_v4());
    tmp.push(tmp_name);

    std::fs::write(&tmp, bytes)
        .map_err(|e| format!("failed to write temp file {}: {}", tmp.display(), e))?;
    std::fs::rename(&tmp, path)
        .map_err(|e| format!("failed to rename temp file into place: {}", e))?;
    Ok(())
}

/// Ensure Minecraft allows symbolic links by writing to allowed_symlinks.txt
pub async fn ensure_symlinks_enabled(minecraft_path: &Path) -> Result<(), String> {
    let allowed_symlinks_file = minecraft_path.join("allowed_symlinks.txt");
    let required_line = "[regex].*";

    // Check if file exists and contains the required line
    if allowed_symlinks_file.exists() {
        let content = async_fs::read_to_string(&allowed_symlinks_file)
            .await
            .map_err(|e| format!("Failed to read allowed_symlinks.txt: {}", e))?;

        if content.lines().any(|line| line.trim() == required_line) {
            return Ok(());
        }

        // File exists but doesn't have the line, append it
        let mut new_content = content;
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str(required_line);
        new_content.push('\n');

        write_file_atomic_async(&allowed_symlinks_file, new_content.as_bytes()).await?;
    } else {
        // File doesn't exist, create it with the required line
        ensure_parent_dir_exists_async(&allowed_symlinks_file).await?;
        let content = format!("{}\n", required_line);
        write_file_atomic_async(&allowed_symlinks_file, content.as_bytes()).await?;
    }

    Ok(())
}

/// Create a symbolic link from source to target directory
#[cfg(windows)]
pub async fn create_directory_symlink(source: &Path, target: &Path) -> Result<(), String> {
    use std::os::windows::fs::symlink_dir;

    if target.exists() {
        if target.is_symlink() {
            // Remove existing symlink
            async_fs::remove_dir(target)
                .await
                .map_err(|e| format!("Failed to remove existing symlink: {}", e))?;
        } else {
            return Err(format!(
                "Target path exists and is not a symlink: {}",
                target.display()
            ));
        }
    }

    ensure_parent_dir_exists_async(target).await?;

    tokio::task::spawn_blocking({
        let source = source.to_path_buf();
        let target = target.to_path_buf();
        move || {
            symlink_dir(&source, &target).map_err(|e| format!("Failed to create symlink: {}", e))
        }
    })
    .await
    .map_err(|e| format!("Symlink creation task failed: {}", e))??;

    Ok(())
}

#[cfg(windows)]
pub async fn create_file_symlink(source: &Path, target: &Path) -> Result<(), String> {
    use std::os::windows::fs::symlink_file;

    if target.exists() {
        if target.is_symlink() {
            // Remove existing symlink
            async_fs::remove_file(target)
                .await
                .map_err(|e| format!("Failed to remove existing symlink: {}", e))?;
        } else {
            return Err(format!(
                "Target path exists and is not a symlink: {}",
                target.display()
            ));
        }
    }

    ensure_parent_dir_exists_async(target).await?;

    tokio::task::spawn_blocking({
        let source = source.to_path_buf();
        let target = target.to_path_buf();
        move || {
            symlink_file(&source, &target)
                .map_err(|e| format!("Failed to create file symlink: {}", e))
        }
    })
    .await
    .map_err(|e| format!("File symlink creation task failed: {}", e))??;

    Ok(())
}

#[cfg(unix)]
pub async fn create_directory_symlink(source: &Path, target: &Path) -> Result<(), String> {
    use std::os::unix::fs::symlink;

    if target.exists() {
        if target.is_symlink() {
            // Remove existing symlink
            async_fs::remove_file(target)
                .await
                .map_err(|e| format!("Failed to remove existing symlink: {}", e))?;
        } else {
            return Err(format!(
                "Target path exists and is not a symlink: {}",
                target.display()
            ));
        }
    }

    ensure_parent_dir_exists_async(target).await?;

    tokio::task::spawn_blocking({
        let source = source.to_path_buf();
        let target = target.to_path_buf();
        move || symlink(&source, &target).map_err(|e| format!("Failed to create symlink: {}", e))
    })
    .await
    .map_err(|e| format!("Symlink creation task failed: {}", e))??;

    Ok(())
}

#[cfg(unix)]
pub async fn create_file_symlink(source: &Path, target: &Path) -> Result<(), String> {
    // On Unix, symlink works for both files and directories
    create_directory_symlink(source, target).await
}

/// Remove a symbolic link if it exists
pub async fn remove_symlink_if_exists(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    if !path.is_symlink() {
        return Err(format!("Path is not a symlink: {}", path.display()));
    }

    #[cfg(windows)]
    {
        // On Windows, directory and file symlinks require different removal methods
        // Try to read the symlink to determine if the target is a directory or file
        let is_dir_symlink = match std::fs::read_link(path) {
            Ok(target) => {
                // Check if the target (when it exists) is a directory
                target.is_dir()
            }
            Err(_) => {
                // If we can't read the link, try metadata as fallback
                match tokio::fs::symlink_metadata(path).await {
                    Ok(metadata) => metadata.is_dir(),
                    Err(_) => false,
                }
            }
        };

        // Try the appropriate removal method first, with fallback to the other method
        if is_dir_symlink {
            if let Err(e) = async_fs::remove_dir(path).await {
                // Directory removal failed, try file removal
                async_fs::remove_file(path)
                    .await
                    .map_err(|_| format!("Failed to remove directory symlink: {}", e))?;
            }
        } else if let Err(e) = async_fs::remove_file(path).await {
            // File removal failed, try directory removal
            async_fs::remove_dir(path)
                .await
                .map_err(|_| format!("Failed to remove file symlink: {}", e))?;
        }
    }

    #[cfg(unix)]
    {
        async_fs::remove_file(path)
            .await
            .map_err(|e| format!("Failed to remove symlink: {}", e))?;
    }

    Ok(())
}
