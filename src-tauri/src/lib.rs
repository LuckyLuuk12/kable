use std::sync::OnceLock;

// Re-export procedural macros from the separate kable-macros crate
// The actual macro implementations are in `../kable-macros/src/lib.rs`
pub use kable_macros::*;
// Shared DTOs live in api-types.
mod api;
mod tests; // API should not be accessible outside this crate/file as this is the only place we should register Tauri commands.
use api::*;
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
    APP_HANDLE.get().expect("not initialized").clone()
}

/// This starts the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Handle CLI arguments before building Tauri app
    let args: Vec<String> = std::env::args().collect();
    crate::features::cli::args::handle_args(crate::features::cli::args::parse_args());
    // Check for --launch-installation argument
    // for arg in args.iter() {
    //     if arg.starts_with("--launch-installation=") {
    //         let installation_id = arg.trim_start_matches("--launch-installation=");
    //         hide_console_window();

    //         // Launch the installation directly without showing UI
    //         tauri::async_runtime::block_on(async {
    //             match crate::features::profiles::get_installation(installation_id).await {
    //                 Ok(Some(installation)) => {
    //                     eprintln!("Launching installation: {}", installation.name);

    //                     // Load settings and account
    //                     let settings = match crate::features::customization::settings::load_settings() {
    //                         Ok(s) => s,
    //                         Err(e) => {
    //                             eprintln!("Failed to load settings: {}", e);
    //                             std::process::exit(1);
    //                         }
    //                     };

    //                     let mut account = match crate::integrations::mojang_api::auth::auth_util::get_active_launcher_account().await {
    //                         Ok(Some(acc)) => acc,
    //                         Ok(None) => {
    //                             eprintln!("No active account found. Please log in through the launcher.");
    //                             std::process::exit(1);
    //                         }
    //                         Err(e) => {
    //                             eprintln!("Failed to get active account: {}", e);
    //                             std::process::exit(1);
    //                         }
    //                     };

    //                     // Refresh the account token to ensure it's still valid
    //                     eprintln!("Refreshing account token...");
    //                     account = match crate::integrations::mojang_api::auth::auth_util::refresh_microsoft_token(account.local_id.clone())
    //                         .await
    //                     {
    //                         Ok(refreshed) => {
    //                             eprintln!("Account token refreshed successfully");
    //                             refreshed
    //                         }
    //                         Err(e) => {
    //                             eprintln!("Failed to refresh account token: {}", e);
    //                             eprintln!("Please open the launcher to log in again.");
    //                             std::process::exit(1);
    //                         }
    //                     };

    //                     if let Err(e) = crate::features::launcher::launch_installation(installation, settings, account).await {
    //                         eprintln!("Failed to launch installation: {}", e);
    //                         std::process::exit(1);
    //                     }

    //                     eprintln!("Installation launched successfully");
    //                     std::process::exit(0);
    //                 }
    //                 Ok(None) => {
    //                     eprintln!("Installation not found: {}", installation_id);
    //                     std::process::exit(1);
    //                 }
    //                 Err(e) => {
    //                     eprintln!("Failed to get installation: {}", e);
    //                     std::process::exit(1);
    //                 }
    //             }
    //         });
    //     }
    // }

    tauri::Builder::default()
        .setup(|app| {
            // Initialize global logger with the app handle so modules that
            // use GLOBAL_APP_HANDLE (e.g. launcher utils) can emit events.
            set_app_handle(app.handle());
            Logger::init(app.handle()); // TODO: make sure the logger will use the global app handle later as well.

            // On startup: if a pending update was downloaded previously,
            // try to launch the installer and exit so the installer can run.
            crate::features::updater::launch_pending_update()?;

            // Initialize Discord Rich Presence
            crate::integrations::discord::initialize()?;

            // TODO: Refactor the symlink feature properly, the current refactor is AI crap again...
            // Clean up any leftover symlinks from previous crashes/exits
            // tauri::async_runtime::spawn(async {
            //     if let Ok(minecraft_dir) = crate::system::fs::mc_dir() {
            //         let symlink_manager = crate::system::symlinks::SymlinkManager::new(minecraft_dir);
            //         if let Err(e) = symlink_manager.cleanup_all_symlinks().await {
            //             Logger::warn_global(&format!("[STARTUP] Failed to cleanup leftover symlinks: {}", e), None);
            //         } else {
            //             Logger::info_global("[STARTUP] Cleaned up leftover symlinks from previous session", None);
            //         }
            //     }
            // });

            Ok(())
        })
        .on_window_event(|window, event| {
            // Clean up symlinks when the main window is closed
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if window.label() == "main" {
                    // Clear Discord presence immediately (blocking to ensure it completes)
                    crate::integrations::discord::clear().map_err(|e| {
                        Logger::warn_global(&format!("[SHUTDOWN] Failed to clear Discord RPC: {}", e), None);
                    });
                    crate::integrations::discord::disconnect().map_err(|e| {
                        Logger::warn_global(&format!("[SHUTDOWN] Failed to disconnect Discord RPC: {}", e), None);
                    });

                    // tauri::async_runtime::spawn(async {
                    //     if let Ok(minecraft_dir) = get_default_minecraft_dir() {
                    //         let symlink_manager = crate::features::advanced::symlink::SymlinkManager::new(minecraft_dir);
                    //         if let Err(e) = symlink_manager.cleanup_all_symlinks().await {
                    //             Logger::warn_global(&format!("[SHUTDOWN] Failed to cleanup symlinks: {}", e), None);
                    //         } else {
                    //             Logger::info_global("[SHUTDOWN] Cleaned up all symlinks on app close", None);
                    //         }
                    //     }
                    // });
                }
            }
        })
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            // TODO: once eveything is refactored, add all api commands here,..
        ])
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
