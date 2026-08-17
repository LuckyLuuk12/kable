use std::sync::OnceLock;

// Re-export procedural macros from the separate kable-macros crate
// The actual macro implementations are in `../kable-macros/src/lib.rs`
pub use kable_macros::*;
// Shared DTOs live in api-types.
mod api;
// mod tests; // Uncomment this if you want to run tests / get IDE support for the tests module.
pub mod typegen; // API should not be accessible outside this crate/file as this is the only place we should register Tauri commands.
pub use api_types::*;
pub use features::logging::Logger;
use tauri::AppHandle;

// Module declarations
pub mod constants;
pub mod features;
pub mod integrations;

#[macro_use]
pub mod system;

// ! A GLOBAL APP HANDLE - used for emitting events & avoiding passing it around everywhere.
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn set_app_handle(handle: &AppHandle) {
    APP_HANDLE.set(handle.clone()).ok();
}

pub fn app_handle() -> AppHandle {
    APP_HANDLE.get().expect("App Handle has not been initialized").clone()
}

/// ? This starts the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Handle CLI arguments before building Tauri app
    tauri::async_runtime::block_on(async {
        if let Err(e) = crate::features::cli::args::handle_args(crate::features::cli::args::parse_args()).await {
            eprintln!("Error handling CLI arguments: {}", e);
            std::process::exit(1);
        }
    });
    tauri::Builder::default()
        .setup(|app| {
            // Initialize global logger with the app handle so modules that
            // use GLOBAL_APP_HANDLE (e.g. launcher utils) can emit events.
            set_app_handle(app.handle());
            Logger::init();

            // On startup: if a pending update was downloaded previously,
            // try to launch the installer and exit so the installer can run.
            crate::features::updater::launch_pending_update()?;

            // Initialize Discord Rich Presence
            crate::features::discord::initialize()?;

            // Clean up any leftover symlinks from previous crashes/exits
            tauri::async_runtime::spawn(async { crate::features::advanced::symlink::cleanup().await });

            Ok(())
        })
        .on_window_event(|window, event| {
            // Clean up symlinks when the main window is closed
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if window.label() == "main" {
                    // Clear Discord presence immediately (blocking to ensure it completes)
                    let _ = crate::features::discord::clear().map_err(|e| {
                        Logger::warn_global(&format!("[SHUTDOWN] Failed to clear Discord RPC: {}", e), None);
                    });
                    let _ = crate::features::discord::disconnect().map_err(|e| {
                        Logger::warn_global(&format!("[SHUTDOWN] Failed to disconnect Discord RPC: {}", e), None);
                    });
                    // Spawn a background task to clean up symlinks asynchronously
                    tauri::async_runtime::spawn(async { crate::features::advanced::symlink::cleanup().await });
                }
            }
        })
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            // TODO: make sure all commands are registered in lib.rs AND in typegen module
            // #region Accounts
            api::start_authentication,
            api::poll_authentication,
            api::list_accounts,
            api::add_account,
            api::remove_account,
            api::set_active_account,
            api::get_active_account,
            // #endregion Accounts
            // #region Customization
            api::get_icon_templates,
            api::save_custom_icon_template,
            api::delete_custom_icon_template,
            api::open_icons_directory,
            api::resolve_image_path,
            api::get_settings,
            api::set_settings,
            api::list_soundpacks,
            api::get_soundpack_metadata,
            api::load_soundpack_file,
            api::import_soundpack_zip,
            api::get_sounds_directory_path,
            api::open_sounds_directory,
            api::list_css_themes,
            api::load_css,
            api::save_css_theme,
            api::delete_css_theme,
            api::open_css_themes_directory,
            // #endregion Customization
            // #region Discord
            api::initialize_discord_rpc,
            api::set_discord_enabled,
            api::set_discord_playing,
            api::set_discord_browsing,
            api::clear_discord_playing,
            api::clear_discord_presence,
            api::disconnect_discord,
            // #endregion Discord
            // #region Launcher
            api::launch_game,
            api::auto_detect_java,
            api::get_java_path,
            // #endregion Launcher
            // #region Projects
            api::browse,
            api::list_profile_projects,
            api::remove_project,
            api::add_project_to_profile,
            api::check_for_update,
            api::check_for_updates,
            api::update_project,
            api::update_all_projects,
            // #endregion Projects
            // #region Profiles
            api::get_profiles,
            api::get_profile,
            api::create_profile,
            api::modify_profile,
            api::delete_profile,
            api::get_versions,
            api::toggle_project,
            api::toggle_favorite,
            api::is_project_enabled,
            // #endregion Profiles
            // #region Symlinks
            api::get_symlinks,
            api::temporary_symlinks,
            api::create,
            api::remove,
            api::toggle,
            api::update,
            // #endregion Symlinks
            // #region System
            api::open_url,
            api::open_path,
            // #endregion System
            // #region Updater
            api::check_launcher_updates,
            api::install_launcher_update,
            api::download_launcher_update,
            api::apply_downloaded_update,
            api::get_current_version,
            // #endregion Updater
        ])
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
