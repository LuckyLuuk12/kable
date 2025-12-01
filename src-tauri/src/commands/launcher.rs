pub use crate::launcher::*;
use crate::{CategorizedLauncherSettings, KableInstallation, LauncherAccount};

#[tauri::command]
pub async fn launch_installation(
    installation: KableInstallation,
    settings: CategorizedLauncherSettings,
    account: LauncherAccount,
) -> Result<LaunchResult, String> {
    crate::launcher::launch_installation(installation, settings, account).await
}

/// Kill a Minecraft process by PID (only if tracked)
#[tauri::command]
pub async fn kill_minecraft_process(process_id: u32) -> Result<(), String> {
    crate::launcher::kill_minecraft_process(process_id).await
}

/// Get all running Minecraft process IDs (tracked by launcher)
#[tauri::command]
pub async fn get_running_minecraft_processes() -> Result<Vec<u32>, String> {
    crate::launcher::get_running_minecraft_processes().await
}

/// Check if any Minecraft process is running (tracked by launcher)
#[tauri::command]
pub async fn is_minecraft_running() -> Result<bool, String> {
    crate::launcher::is_minecraft_running().await
}

/// Wait for a Minecraft process to exit (tracked by launcher)
#[tauri::command]
pub async fn wait_for_minecraft_exit(process_id: u32) -> Result<(), String> {
    // Call the internal function but ignore the exit code for the command interface
    crate::launcher::wait_for_minecraft_exit(process_id)
        .await
        .map(|_exit_code| ())
}

/// Auto-detect Java executable path
#[tauri::command]
pub fn auto_detect_java() -> Result<String, String> {
    java::auto_detect_java()
}

/// Tauri command: Returns the path to a working Java executable, using the provided path or searching common locations.
///
/// Used by the frontend to validate or auto-detect Java installations.
///
/// # Arguments
/// * `java_path` - Optional user-specified Java path.
///
/// # Returns
/// Ok(path to Java executable) or Err if not found.
#[tauri::command]
pub fn get_java_path(java_path: Option<String>) -> Result<String, String> {
    java::find_java_executable(java_path.as_ref())
}