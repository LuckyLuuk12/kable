use crate::features::launcher;
use crate::system::java;
use api_types::launcher::LaunchResult;
use api_types::profiles::KableProfile;

// #[tauri::command]
// pub async fn launch_installation(
//     installation: KableProfile,
//     settings: CategorizedLauncherSettings,
//     account: LauncherAccount,
// ) -> Result<LaunchResult, String> {
//     launcher::launch_installation(installation, settings, account).await
// }

// #[tauri::command]
// pub async fn kill_minecraft_process(process_id: u32) -> Result<(), String> {
//     launcher::kill_minecraft_process(process_id).await
// }

// #[tauri::command]
// pub async fn get_running_minecraft_processes() -> Result<Vec<u32>, String> {
//     launcher::get_running_minecraft_processes().await
// }

// #[tauri::command]
// pub async fn is_minecraft_running() -> Result<bool, String> {
//     launcher::is_minecraft_running().await
// }

// #[tauri::command]
// pub async fn wait_for_minecraft_exit(process_id: u32) -> Result<(), String> {
//     launcher::wait_for_minecraft_exit(process_id).await.map(|_| ())
// }

#[tauri::command]
#[specta::specta]
pub async fn launch_game(profile: KableProfile) -> Result<LaunchResult, String> {
    launcher::launch::launch_game(profile).await
}

#[tauri::command]
#[specta::specta]
pub fn auto_detect_java() -> Result<String, String> {
    java::auto_detect_java()
}

#[tauri::command]
#[specta::specta]
pub fn get_java_path(java_path: Option<String>) -> Result<String, String> {
    java::find_java_executable(java_path.as_ref())
}
