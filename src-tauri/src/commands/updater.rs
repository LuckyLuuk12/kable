use tauri::command;
use tauri_plugin_updater::UpdaterExt;

#[command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<Option<serde_json::Value>, String> {
    match app.updater() {
        Ok(updater) => {
            match updater.check().await {
                Ok(Some(update)) => {
                    let update_info = serde_json::json!({
                        "version": update.version,
                        "date": update.date.map(|d| d.to_string()),
                        "body": update.body
                    });
                    Ok(Some(update_info))
                }
                Ok(None) => Ok(None),
                Err(e) => Err(format!("Failed to check for updates: {}", e))
            }
        }
        Err(e) => Err(format!("Failed to get updater: {}", e))
    }
}

#[command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    match app.updater() {
        Ok(updater) => {
            match updater.check().await {
                Ok(Some(update)) => {
                    // Download and install with empty callbacks
                    match update.download_and_install(|_, _| {}, || {}).await {
                        Ok(_) => {
                            // App will restart automatically after update
                            Ok(())
                        }
                        Err(e) => Err(format!("Failed to install update: {}", e))
                    }
                }
                Ok(None) => Err("No update available".to_string()),
                Err(e) => Err(format!("Failed to check for updates: {}", e))
            }
        }
        Err(e) => Err(format!("Failed to get updater: {}", e))
    }
}

#[command]
pub async fn get_current_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}
