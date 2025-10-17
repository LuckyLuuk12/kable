pub mod fabric;
pub mod forge;
pub mod launchables;
pub mod utils;
pub mod vanilla;

pub use fabric::*;
pub use forge::*;
pub use launchables::*;
pub use vanilla::*;

use crate::logging::Logger;
use crate::{
    get_default_minecraft_dir, kable_profiles::KableInstallation, CategorizedLauncherSettings,
    LauncherAccount,
};
use tauri::{Emitter, Manager};

use once_cell::sync::OnceCell;
use std::collections::HashSet;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;
use tauri::async_runtime::spawn_blocking;

static MINECRAFT_PIDS: OnceCell<Mutex<HashSet<u32>>> = OnceCell::new();

fn get_pid_set() -> &'static Mutex<HashSet<u32>> {
    MINECRAFT_PIDS.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Handle on_game_launch settings behavior
async fn handle_launch_settings(
    settings: &CategorizedLauncherSettings,
    app_handle: Option<tauri::AppHandle>,
) {
    let behavior = &settings.general.on_game_launch;
    Logger::info_global(
        &format!("Handling on_game_launch setting: {}", behavior),
        None,
    );

    match behavior.as_str() {
        "exit" => {
            Logger::info_global(
                "Closing launcher as requested by on_game_launch setting",
                None,
            );
            if let Some(app) = app_handle {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.close();
                }
            }
        }
        "minimize" => {
            Logger::info_global(
                "Minimizing launcher as requested by on_game_launch setting",
                None,
            );
            if let Some(app) = app_handle {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.minimize();
                }
            }
        }
        "open_logs" => {
            Logger::info_global(
                "Opening logs page as requested by on_game_launch setting",
                None,
            );
            if let Some(app) = app_handle {
                let _ = app.emit(
                    "navigate-to-logs",
                    serde_json::json!({"reason": "launch_setting"}),
                );
            }
        }
        "ask" => {
            Logger::info_global("Asking user what to do on game launch", None);
            if let Some(app) = app_handle {
                let _ = app.emit(
                    "ask-launch-behavior",
                    serde_json::json!({"options": ["keep_open", "exit", "minimize", "open_logs"]}),
                );
            }
        }
        "keep_open" => {
            Logger::info_global(
                "Keeping launcher open as requested by on_game_launch setting",
                None,
            );
            // Do nothing - default behavior
        }
        _ => {
            Logger::warn_global(
                &format!("Unknown on_game_launch setting: {}", behavior),
                None,
            );
        }
    }
}

/// Handle on_game_close settings behavior
async fn handle_close_settings(
    settings: &CategorizedLauncherSettings,
    app_handle: Option<tauri::AppHandle>,
    exit_code: i32,
) {
    let behavior = &settings.general.on_game_close;
    Logger::info_global(
        &format!(
            "Handling on_game_close setting: {} (exit code: {})",
            behavior, exit_code
        ),
        None,
    );

    match behavior.as_str() {
        "open_logs" => {
            Logger::info_global(
                "Opening logs page as requested by on_game_close setting",
                None,
            );
            if let Some(app) = app_handle {
                let _ = app.emit(
                    "navigate-to-logs",
                    serde_json::json!({"reason": "close_setting"}),
                );
            }
        }
        "open_home" => {
            Logger::info_global(
                "Navigating to home page as requested by on_game_close setting",
                None,
            );
            if let Some(app) = app_handle {
                let _ = app.emit(
                    "navigate-to-home",
                    serde_json::json!({"reason": "close_setting"}),
                );
            }
        }
        "exit" => {
            Logger::info_global(
                "Closing launcher as requested by on_game_close setting",
                None,
            );
            if let Some(app) = app_handle {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.close();
                }
            }
        }
        "minimize" => {
            Logger::info_global(
                "Minimizing launcher as requested by on_game_close setting",
                None,
            );
            if let Some(app) = app_handle {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.minimize();
                }
            }
        }
        "ask" => {
            Logger::info_global("Asking user what to do on game close", None);
            if let Some(app) = app_handle {
                let _ = app.emit("ask-close-behavior", serde_json::json!({"options": ["open_logs", "open_home", "exit", "minimize"], "exit_code": exit_code}));
            }
        }
        _ => {
            Logger::warn_global(
                &format!("Unknown on_game_close setting: {}", behavior),
                None,
            );
        }
    }
}

/// Handle on_game_crash settings behavior
async fn handle_crash_settings(
    settings: &CategorizedLauncherSettings,
    app_handle: Option<tauri::AppHandle>,
    exit_code: i32,
) {
    let behavior = &settings.general.on_game_crash;
    Logger::info_global(
        &format!(
            "Handling on_game_crash setting: {} (exit code: {})",
            behavior, exit_code
        ),
        None,
    );

    match behavior.as_str() {
        "restart" => {
            Logger::info_global(
                "Game restart requested by on_game_crash setting (not implemented yet)",
                None,
            );
            // TODO: Implement restart functionality - would need to store launch context
            if let Some(app) = app_handle {
                let _ = app.emit(
                    "game-restart-requested",
                    serde_json::json!({"exit_code": exit_code}),
                );
            }
        }
        "open_logs" => {
            Logger::info_global(
                "Opening logs page as requested by on_game_crash setting",
                None,
            );
            if let Some(app) = app_handle {
                let _ = app.emit(
                    "navigate-to-logs",
                    serde_json::json!({"reason": "crash_setting"}),
                );
            }
        }
        "open_home" => {
            Logger::info_global(
                "Navigating to home page as requested by on_game_crash setting",
                None,
            );
            if let Some(app) = app_handle {
                let _ = app.emit(
                    "navigate-to-home",
                    serde_json::json!({"reason": "crash_setting"}),
                );
            }
        }
        "exit" => {
            Logger::info_global(
                "Closing launcher as requested by on_game_crash setting",
                None,
            );
            if let Some(app) = app_handle {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.close();
                }
            }
        }
        "minimize" => {
            Logger::info_global(
                "Minimizing launcher as requested by on_game_crash setting",
                None,
            );
            if let Some(app) = app_handle {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.minimize();
                }
            }
        }
        "ask" => {
            Logger::info_global("Asking user what to do on game crash", None);
            if let Some(app) = app_handle {
                let _ = app.emit("ask-crash-behavior", serde_json::json!({"options": ["restart", "open_logs", "open_home", "exit", "minimize"], "exit_code": exit_code}));
            }
        }
        _ => {
            Logger::warn_global(
                &format!("Unknown on_game_crash setting: {}", behavior),
                None,
            );
        }
    }
}

async fn get_launchable_for_installation(
    context: &LaunchContext,
) -> Result<Box<dyn Launchable>, String> {
    // Detect loader type from context.installation or context.manifest
    match context.detect_loader_type().await? {
        LoaderType::Vanilla => Ok(Box::new(VanillaLaunchable)),
        LoaderType::Fabric => Ok(Box::new(FabricLaunchable)),
        LoaderType::IrisFabric => Ok(Box::new(FabricLaunchable)), // Iris is a Fabric mod but has its own loader which is identical to Fabric
        LoaderType::Quilt => Ok(Box::new(FabricLaunchable)),      // Quilt is a fork of Fabric
        LoaderType::Forge => Ok(Box::new(ForgeLaunchable)),       // Forge can be
        LoaderType::NeoForge => Ok(Box::new(ForgeLaunchable)),    // NeoForge is a fork of Forge
                                                                   // Add more as needed
    }
}

pub async fn launch_installation(
    mut installation: KableInstallation,
    settings: CategorizedLauncherSettings,
    account: LauncherAccount,
) -> Result<LaunchResult, String> {
    // Use installation.id for log grouping and event correlation
    let instance_id = Some(installation.id.as_str());
    
    Logger::info_global(
        &format!("Launching installation: {}", installation.name),
        None,
    );
    
    // Update last_used and times_launched before launching
    installation.last_used = chrono::Utc::now().to_rfc3339();
    installation.times_launched = installation.times_launched.saturating_add(1);
    
    // Save the updated installation to disk
    let installation_id = installation.id.clone();
    if let Err(e) = crate::installations::modify_installation(&installation_id, installation.clone()).await {
        Logger::warn_global(
            &format!("Failed to update installation last_used: {}", e),
            instance_id,
        );
        // Continue launching even if update fails
    }
    
    // Validate installation
    let minecraft_dir = match get_default_minecraft_dir() {
        Ok(dir) => dir.to_string_lossy().to_string(),
        Err(e) => {
            Logger::error_global(
                &format!("Failed to get default Minecraft dir: {}", e),
                instance_id,
            );
            return Err(format!("Failed to get default Minecraft dir: {}", e));
        }
    };
    // Build context
    let context = match LaunchContext::new(
        installation.clone(),
        settings.clone(),
        account,
        minecraft_dir,
    ) {
        Ok(ctx) => ctx,
        Err(e) => {
            Logger::error_global(
                &format!("Failed to build launch context: {}", e),
                instance_id,
            );
            return Err(format!("Failed to build launch context: {}", e));
        }
    };
    // Detect loader and get Launchable
    let launchable = match get_launchable_for_installation(&context).await {
        Ok(l) => l,
        Err(e) => {
            Logger::error_global(&format!("Failed to detect loader: {}", e), instance_id);
            return Err(format!("Failed to detect loader: {}", e));
        }
    };
    // Prepare (download, patch, etc.)
    if let Err(e) = launchable.prepare(&context).await {
        Logger::error_global(&format!("Failed to prepare launch: {}", e), instance_id);
        return Err(format!("Failed to prepare launch: {}", e));
    }
    // Build and run the launch command
    let result = match launchable.launch(&context).await {
        Ok(res) => {
            Logger::info_global(
                &format!("Minecraft launched successfully (PID: {})", res.pid),
                None,
            );
            res
        }
        Err(e) => {
            Logger::error_global(&format!("Failed to launch Minecraft: {}", e), None);
            return Err(format!("Failed to launch Minecraft: {}", e));
        }
    };
    // Track the launched PID
    {
        let mut pids = get_pid_set().lock().unwrap();
        pids.insert(result.pid);
    }
    
    // Compute app handle so we can emit events (if available)
    let app_handle = if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
        handle_guard.as_ref().map(|global| (**global).clone())
    } else {
        None
    };

    // Handle on_game_launch settings behavior
    handle_launch_settings(&settings, app_handle.clone()).await;

    // Start monitoring the process for exit behavior
    let settings_clone = settings.clone();
    let app_handle_clone = app_handle.clone();
    let pid = result.pid;
    let installation_for_tracking = installation.clone();
    let launch_start_time = std::time::Instant::now();

    // Spawn a task to monitor the process exit and handle settings
    tauri::async_runtime::spawn(async move {
        Logger::info_global(
            &format!("[SETTINGS TASK] Starting exit monitoring for PID {}", pid),
            None,
        );
        // Wait for the process to exit and get exit code
        match wait_for_minecraft_exit(pid).await {
            Ok(exit_code) => {
                // Calculate playtime in milliseconds
                let playtime_ms = launch_start_time.elapsed().as_millis() as u64;
                Logger::info_global(
                    &format!("[SETTINGS TASK] Game session lasted {} ms ({:.1} minutes)", 
                        playtime_ms, playtime_ms as f64 / 60000.0),
                    None,
                );
                
                // Update total_time_played_ms
                let mut updated_installation = installation_for_tracking.clone();
                updated_installation.total_time_played_ms = 
                    updated_installation.total_time_played_ms.saturating_add(playtime_ms);
                
                let installation_id = updated_installation.id.clone();
                
                // Save the updated playtime to disk
                if let Err(e) = crate::installations::modify_installation(
                    &installation_id, 
                    updated_installation
                ).await {
                    Logger::warn_global(
                        &format!("Failed to update installation playtime: {}", e),
                        None,
                    );
                }
                

                Logger::info_global(
                    &format!("[SETTINGS TASK] Process {} exited with code {}", pid, exit_code),
                    None,
                );

                // Determine if it was a crash or normal exit
                let is_crash = exit_code != 0 && exit_code != 130 && exit_code != 143; // 130 = Ctrl+C, 143 = SIGTERM

                if is_crash {
                    Logger::info_global(
                        &format!("[SETTINGS TASK] Handling crash settings for exit code {}", exit_code),
                        None,
                    );
                } else {
                    Logger::info_global(
                        &format!("[SETTINGS TASK] Handling close settings for exit code {}", exit_code),
                        None,
                    );
                    handle_close_settings(&settings_clone, app_handle_clone, exit_code).await;
                }
            }
            Err(e) => {
                Logger::error_global(&format!("[SETTINGS TASK] Error waiting for process exit: {}", e), None);
                // Handle as normal close if we can't determine exit code
                handle_close_settings(&settings_clone, app_handle_clone, 0).await;
            }
        }
        Logger::info_global(
            &format!("[SETTINGS TASK] Completed for PID {}", pid),
            None,
        );
    });

    Ok(result)
}

/// Kill a Minecraft process by PID (only if tracked)
pub async fn kill_minecraft_process(process_id: u32) -> Result<(), String> {
    let mut pids = get_pid_set().lock().unwrap();
    if !pids.contains(&process_id) {
        return Err(format!(
            "Process {} is not tracked by the launcher",
            process_id
        ));
    }
    // Try to kill the process
    match Command::new("taskkill")
        .args(["/PID", &process_id.to_string(), "/F"])
        .status()
    {
        Ok(status) if status.success() => {
            pids.remove(&process_id);
            Ok(())
        }
        _ => Err(format!("Failed to kill process {}", process_id)),
    }
}

/// Get all running Minecraft process IDs (tracked by launcher)
pub async fn get_running_minecraft_processes() -> Result<Vec<u32>, String> {
    let pids = get_pid_set().lock().unwrap();
    // Optionally, check if the process is still alive
    let mut running = Vec::new();
    for &pid in pids.iter() {
        if is_process_alive(pid) {
            running.push(pid);
        }
    }
    Ok(running)
}

fn is_process_alive(pid: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
        };
        const STILL_ACTIVE: u32 = 259;
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle.is_null() {
                return false;
            }
            let mut code = 0u32;
            let ok = GetExitCodeProcess(handle, &mut code as *mut u32);
            CloseHandle(handle);
            ok != 0 && code == STILL_ACTIVE
        }
    }
    #[cfg(unix)]
    {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
}

/// Check if any Minecraft process is running (tracked by launcher)
// #[tauri::command]
pub async fn is_minecraft_running() -> Result<bool, String> {
    let running = get_running_minecraft_processes().await?;
    Ok(!running.is_empty())
}

/// Wait for a Minecraft process to exit (tracked by launcher) and return exit code
pub async fn wait_for_minecraft_exit(process_id: u32) -> Result<i32, String> {
    let mut found = false;
    {
        let pids = get_pid_set().lock().unwrap();
        if pids.contains(&process_id) {
            found = true;
        }
    }
    if !found {
        return Err(format!(
            "Process {} is not tracked by the launcher",
            process_id
        ));
    }
    spawn_blocking(move || {
        while is_process_alive(process_id) {
            std::thread::sleep(Duration::from_millis(500));
        }
        // Process has exited
        let mut pids = get_pid_set().lock().unwrap();
        pids.remove(&process_id);

        // Try to get exit code (platform-specific)
        let exit_code = get_process_exit_code(process_id).unwrap_or(-1);
        Ok(exit_code)
    })
    .await
    .unwrap()
}

/// Get the exit code of a process (platform-specific implementation)
fn get_process_exit_code(pid: u32) -> Option<i32> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
        };
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle.is_null() {
                return None;
            }
            let mut code = 0u32;
            let ok = GetExitCodeProcess(handle, &mut code as *mut u32);
            CloseHandle(handle);
            if ok != 0 {
                Some(code as i32)
            } else {
                None
            }
        }
    }
    #[cfg(unix)]
    {
        // On Unix systems, we can't easily get the exit code after the process has exited
        // without being its parent. For now, return None to indicate unknown.
        // A more sophisticated approach would require process monitoring from launch.
        None
    }
}
