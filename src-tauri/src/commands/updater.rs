use tauri::command;
use tauri_plugin_updater::UpdaterExt;

#[command]
pub async fn check_for_updates(
    app: tauri::AppHandle,
    include_prerelease: bool,
) -> Result<Option<serde_json::Value>, String> {
    let mut builder = app.updater_builder();

    // Configure to include or exclude prereleases
    if include_prerelease {
        builder = builder.version_comparator(|current, update| {
            // When checking nightly builds, compare chronologically using build numbers
            // Build numbers are minutes since epoch, so higher = newer
            // Examples:
            // - 0.1.7-12345 (current) -> 0.1.7-54321 (update): true (54321 > 12345, newer nightly)
            // - 0.1.7-54321 (current) -> 0.1.7-12345 (update): false (12345 < 54321, older nightly)
            // - 0.1.7-12345 (current) -> 0.1.8 (update): true (0.1.8 > 0.1.7, newer stable)
            // - 0.1.7 (current) -> 0.1.7-12345 (update): false (0.1.7 is "newer" than its prerelease)

            // If update version is greater (ignoring prerelease), it's an update
            if update.version > current {
                return true;
            }

            // If both have same base version but different prerelease, compare build numbers
            if update.version != current {
                let current_str = current.to_string();
                let update_str = update.version.to_string();

                // If both contain '-' (both are prereleases), compare the numeric suffix
                if current_str.contains('-') && update_str.contains('-') {
                    let current_base = current_str.split('-').next().unwrap_or("");
                    let update_base = update_str.split('-').next().unwrap_or("");

                    // Same base version - compare build numbers chronologically
                    if current_base == update_base {
                        let current_build: u32 = current_str
                            .split('-')
                            .nth(1)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                        let update_build: u32 = update_str
                            .split('-')
                            .nth(1)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);

                        // Higher build number = newer
                        return update_build > current_build;
                    }
                }
            }

            false
        });
    }

    let updater = match builder.build() {
        Ok(u) => u,
        Err(e) => return Err(format!("Failed to build updater: {}", e)),
    };

    match updater.check().await {
        Ok(Some(update)) => {
            let update_info = serde_json::json!({
                "version": update.version,
                "date": update.date.map(|d| d.to_string()),
                "body": update.body,
                "current_version": env!("CARGO_PKG_VERSION")
            });
            Ok(Some(update_info))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Failed to check for updates: {}", e)),
    }
}

#[command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    let updater = match app.updater_builder().build() {
        Ok(u) => u,
        Err(e) => return Err(format!("Failed to get updater: {}", e)),
    };

    match updater.check().await {
        Ok(Some(update)) => {
            // Download and install with empty callbacks
            match update.download_and_install(|_, _| {}, || {}).await {
                Ok(_) => {
                    // App will restart automatically after update
                    Ok(())
                }
                Err(e) => Err(format!("Failed to install update: {}", e)),
            }
        }
        Ok(None) => Err("No update available".to_string()),
        Err(e) => Err(format!("Failed to check for updates: {}", e)),
    }
}

#[command]
pub async fn get_current_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}
