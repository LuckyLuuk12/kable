pub use crate::launcher::*;
use crate::{CategorizedLauncherSettings, LauncherAccount, KableInstallation};

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
    crate::launcher::wait_for_minecraft_exit(process_id).await
}
