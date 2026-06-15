// Re-export procedural macros from the separate kable-macros crate
// The actual macro implementations are in `../kable-macros/src/lib.rs`
pub use kable_macros::*;

// Module declarations
pub mod api;
pub mod constants;
pub mod features;
pub mod integrations;

#[macro_use]
pub mod system;
pub mod logging;

// Shared DTOs live in api-types.
pub use api_types::*;

// Re-export common system utilities for backward compatibility or easier access
pub use crate::system::fs::*;
use crate::system::processes::hide_console_window;
pub use crate::system::symlinks::*;

/// This starts the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Handle CLI arguments before building Tauri app
    let args: Vec<String> = std::env::args().collect();
    crate::features::cli::args::handle_args(crate::features::cli::args::parse_args());
    // Check for --launch-installation argument
    for arg in args.iter() {
        if arg.starts_with("--launch-installation=") {
            let installation_id = arg.trim_start_matches("--launch-installation=");
            hide_console_window();

            // Launch the installation directly without showing UI
            tauri::async_runtime::block_on(async {
                match crate::features::profiles::get_installation(installation_id).await {
                    Ok(Some(installation)) => {
                        eprintln!("Launching installation: {}", installation.name);

                        // Load settings and account
                        let settings = match crate::features::customization::settings::load_settings() {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("Failed to load settings: {}", e);
                                std::process::exit(1);
                            }
                        };

                        let mut account = match crate::integrations::mojang_api::auth::auth_util::get_active_launcher_account().await {
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

                        // Refresh the account token to ensure it's still valid
                        eprintln!("Refreshing account token...");
                        account = match crate::integrations::mojang_api::auth::auth_util::refresh_microsoft_token(account.local_id.clone())
                            .await
                        {
                            Ok(refreshed) => {
                                eprintln!("Account token refreshed successfully");
                                refreshed
                            }
                            Err(e) => {
                                eprintln!("Failed to refresh account token: {}", e);
                                eprintln!("Please open the launcher to log in again.");
                                std::process::exit(1);
                            }
                        };

                        if let Err(e) = crate::features::launcher::launch_installation(installation, settings, account).await {
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

            // On startup: if a pending update was downloaded previously,
            // try to launch the installer and exit so the installer can run.
            if let Ok(launcher_dir) = crate::get_kable_launcher_dir() {
                let pending = launcher_dir.join("pending_update.json");
                if pending.exists() {
                    if let Ok(contents) = std::fs::read_to_string(&pending) {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&contents) {
                            if let Some(installer) = v.get("installer_path").and_then(|s| s.as_str()) {
                                if std::path::Path::new(installer).exists() {
                                    match std::process::Command::new(installer).spawn() {
                                        Ok(_) => {
                                            let _ = std::fs::remove_file(&pending);
                                            std::process::exit(0);
                                        }
                                        Err(e) => {
                                            Logger::warn_global(&format!("[STARTUP] Failed to launch pending installer: {}", e), None)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Initialize Discord Rich Presence
            tauri::async_runtime::spawn(async {
                if let Err(e) = crate::discord::initialize() {
                    Logger::warn_global(&format!("[STARTUP] Failed to initialize Discord RPC: {}", e), None);
                } else {
                    Logger::info_global("[STARTUP] Discord Rich Presence initialized", None);
                }
            });

            // Clean up any leftover symlinks from previous crashes/exits
            tauri::async_runtime::spawn(async {
                if let Ok(minecraft_dir) = get_default_minecraft_dir() {
                    let symlink_manager = crate::symlink_manager::SymlinkManager::new(minecraft_dir);
                    if let Err(e) = symlink_manager.cleanup_all_symlinks().await {
                        Logger::warn_global(&format!("[STARTUP] Failed to cleanup leftover symlinks: {}", e), None);
                    } else {
                        Logger::info_global("[STARTUP] Cleaned up leftover symlinks from previous session", None);
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            // Clean up symlinks when the main window is closed
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if window.label() == "main" {
                    // Clear Discord presence immediately (blocking to ensure it completes)
                    if let Err(e) = crate::discord::clear() {
                        Logger::warn_global(&format!("[SHUTDOWN] Failed to clear Discord presence: {}", e), None);
                    }
                    if let Err(e) = crate::discord::disconnect() {
                        Logger::warn_global(&format!("[SHUTDOWN] Failed to disconnect Discord RPC: {}", e), None);
                    }

                    tauri::async_runtime::spawn(async {
                        if let Ok(minecraft_dir) = get_default_minecraft_dir() {
                            let symlink_manager = crate::symlink_manager::SymlinkManager::new(minecraft_dir);
                            if let Err(e) = symlink_manager.cleanup_all_symlinks().await {
                                Logger::warn_global(&format!("[SHUTDOWN] Failed to cleanup symlinks: {}", e), None);
                            } else {
                                Logger::info_global("[SHUTDOWN] Cleaned up all symlinks on app close", None);
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
            // Settings commands
            api::settings::get_settings,
            api::settings::set_settings,
            // Installation commands
            api::profiles::get_versions,
            api::profiles::get_installations,
            api::profiles::get_installation,
            api::profiles::modify_installation,
            api::profiles::delete_installation,
            api::profiles::create_installation,
            commands_installations::create_installation_from_existing,
            commands_installations::get_mod_info,
            commands_installations::disable_mod,
            commands_installations::enable_mod,
            commands_installations::toggle_mod_disabled,
            commands_installations::delete_mod,
            commands_installations::disable_resourcepack_for_installation,
            commands_installations::enable_resourcepack_for_installation,
            commands_installations::toggle_resourcepack_disabled_for_installation,
            commands_installations::delete_resourcepack_for_installation,
            commands_installations::get_resourcepack_info_for_installation,
            commands_installations::get_global_resourcepacks,
            commands_installations::update_resourcepack_settings,
            commands_installations::disable_shader_for_installation,
            commands_installations::enable_shader_for_installation,
            commands_installations::toggle_shader_disabled_for_installation,
            commands_installations::delete_shader_for_installation,
            commands_installations::get_shaderpack_info_for_installation,
            commands_installations::get_global_shaderpacks,
            commands_installations::import,
            commands_installations::import_from_minecraft_folder,
            commands_installations::export,
            commands_installations::duplicate,
            commands_installations::create_shortcut,
            commands_installations::select_installation_zip,
            commands_installations::select_minecraft_folder,
            // Launcher commands
            api::launcher::launch_installation_command,
            api::launcher::kill_minecraft_process,
            api::launcher::get_running_minecraft_processes,
            api::launcher::is_minecraft_running,
            api::launcher::wait_for_minecraft_exit,
            api::launcher::auto_detect_java,
            api::launcher::get_java_path,
            // Maps/Worlds commands
            maps::get_local_worlds,
            maps::delete_world,
            maps::backup_world,
            // Mods commands
            commands_mods::get_mods,
            commands_mods::download_mod,
            commands_mods::download_or_prepare_mod,
            commands_mods::get_projects,
            commands_mods::get_project_versions,
            commands_mods::set_provider_filter,
            commands_mods::set_provider_limit,
            commands_mods::clear_provider_cache,
            commands_mods::purge_stale_provider_cache,
            commands_mods::get_extended_mod_info,
            commands_mods::get_mod_metadata,
            commands_mods::get_modpack_source_records,
            commands_mods::apply_modpack_selection,
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
            commands_resourcepacks::move_pack_to_merged,
            commands_resourcepacks::move_pack_to_individual,
            commands_resourcepacks::migrate_resourcepack_structure,
            // Discord commands
            commands_discord::discord_set_browsing,
            commands_discord::discord_set_enabled,
            commands_discord::discord_clear,
            // Skins commands
            commands_skins::upload_skin_to_account,
            commands_skins::change_skin_model,
            commands_skins::get_skin_url_by_uuid,
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
            api::icons::get_custom_icon_templates,
            api::icons::save_custom_icon_template,
            api::icons::delete_custom_icon_template,
            api::icons::open_icons_directory,
            // Image helper
            api::images::resolve_image_path,
            // Logging commands
            logging::export_logs,
            logging::update_logging_config,
            logging::cleanup_old_logs,
            logging::get_log_stats,
            // System commands
            commands_system::open_url,
            commands_system::open_path,
            // Sounds commands
            api::sounds::list_soundpacks,
            api::sounds::get_soundpack_metadata,
            api::sounds::load_soundpack_file,
            api::sounds::import_soundpack_zip,
            api::sounds::get_sounds_directory_path,
            api::sounds::open_sounds_directory,
            // Updater commands
            commands_updater::check_for_updates,
            commands_updater::install_update,
            commands_updater::download_update,
            commands_updater::apply_downloaded_update,
            commands_updater::get_current_version
        ])
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
