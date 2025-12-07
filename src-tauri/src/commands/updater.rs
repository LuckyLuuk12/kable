use serde::Deserialize;
use tauri::command;
use tauri_plugin_updater::UpdaterExt;
use url::Url;

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    prerelease: bool,
    draft: bool,
    #[serde(default)]
    body: String,
    published_at: String,
}

async fn fetch_latest_release(include_prerelease: bool) -> Result<Option<GitHubRelease>, String> {
    println!("[Updater] Fetching all releases from GitHub API");

    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/LuckyLuuk12/kable/releases")
        .header("User-Agent", "kable-updater")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases: {}", e))?;

    let releases: Vec<GitHubRelease> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse releases: {}", e))?;

    println!("[Updater] Found {} total releases", releases.len());

    // Filter out drafts and apply prerelease filter
    let filtered: Vec<GitHubRelease> = releases
        .into_iter()
        .filter(|r| !r.draft)
        .filter(|r| include_prerelease || !r.prerelease)
        .collect();

    println!(
        "[Updater] After filtering (include_prerelease={}): {} releases",
        include_prerelease,
        filtered.len()
    );

    for release in &filtered {
        println!(
            "[Updater]   - {} (prerelease: {})",
            release.tag_name, release.prerelease
        );
    }

    // Return the first one (GitHub sorts by date descending)
    Ok(filtered.into_iter().next())
}

#[command]
pub async fn check_for_updates(
    app: tauri::AppHandle,
    include_prerelease: bool,
) -> Result<Option<serde_json::Value>, String> {
    println!(
        "[Updater] Checking for updates with include_prerelease={}",
        include_prerelease
    );
    println!("[Updater] Current version: {}", env!("CARGO_PKG_VERSION"));

    // Fetch the latest release from GitHub API
    let latest_release = match fetch_latest_release(include_prerelease).await? {
        Some(r) => r,
        None => {
            println!("[Updater] No releases found");
            return Ok(None);
        }
    };

    println!(
        "[Updater] Latest release from GitHub: {}",
        latest_release.tag_name
    );

    // Build updater with the specific release's latest.json
    let tag = &latest_release.tag_name;
    let endpoint_str = format!(
        "https://github.com/LuckyLuuk12/kable/releases/download/{}/latest.json",
        tag
    );
    println!("[Updater] Using endpoint: {}", endpoint_str);

    let endpoint = match Url::parse(&endpoint_str) {
        Ok(url) => url,
        Err(e) => return Err(format!("Failed to parse endpoint URL: {}", e)),
    };

    let mut builder = match app.updater_builder().endpoints(vec![endpoint]) {
        Ok(b) => b,
        Err(e) => return Err(format!("Failed to set endpoints: {}", e)),
    };

    // Configure version comparator
    builder = builder.version_comparator(|current, update| {
        // When checking nightly builds, compare chronologically using build numbers
        // Build numbers are minutes since epoch, so higher = newer
        // Nightly builds (0.1.7-XXXXX) are development versions AFTER the base release (0.1.7)
        // Examples:
        // - 0.1.7-12345 (current) -> 0.1.7-54321 (update): true (54321 > 12345, newer nightly)
        // - 0.1.7-54321 (current) -> 0.1.7-12345 (update): false (12345 < 54321, older nightly)
        // - 0.1.7-12345 (current) -> 0.1.7 (update): FALSE (nightly is NEWER than base, would be downgrade)
        // - 0.1.7-12345 (current) -> 0.1.8 (update): true (0.1.8 > 0.1.7)
        // - 0.1.7 (current) -> 0.1.7-12345 (update): true (nightly is newer development version)

        let current_str = current.to_string();
        let update_str = update.version.to_string();

        println!(
            "[Updater] Comparing versions: {} -> {}",
            current_str, update_str
        );

        let current_has_pre = current_str.contains('-');
        let update_has_pre = update_str.contains('-');

        // Get base versions (before the '-')
        let current_base = current_str.split('-').next().unwrap_or(&current_str);
        let update_base = update_str.split('-').next().unwrap_or(&update_str);

        // If base versions differ, use standard semver comparison
        if current_base != update_base {
            return update.version > current;
        }

        // Same base version - handle nightly logic
        match (current_has_pre, update_has_pre) {
            // Current is nightly, update is base -> reject (0.1.7-12345 -> 0.1.7 is downgrade)
            (true, false) => {
                println!(
                    "[Updater] Rejecting: current is nightly, update is base (would be downgrade)"
                );
                false
            }

            // Current is base, update is nightly -> allow (0.1.7 -> 0.1.7-12345 is upgrade)
            (false, true) => {
                println!("[Updater] Accepting: current is base, update is nightly (is upgrade)");
                true
            }

            // Both nightly -> compare build numbers (0.1.7-12345 -> 0.1.7-54321)
            (true, true) => {
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

                let is_newer = update_build > current_build;
                println!(
                    "[Updater] Both nightly: {} ({}) vs {} ({}) -> {}",
                    current_str, current_build, update_str, update_build, is_newer
                );
                is_newer
            }

            // Both base, same version -> no update
            (false, false) => {
                println!("[Updater] Both base versions, same version -> no update");
                false
            }
        }
    });

    let updater = match builder.build() {
        Ok(u) => u,
        Err(e) => return Err(format!("Failed to build updater: {}", e)),
    };

    match updater.check().await {
        Ok(Some(update)) => {
            println!("[Updater] Update found: version {}", update.version);
            let update_info = serde_json::json!({
                "version": update.version,
                "date": update.date.map(|d| d.to_string()),
                "body": update.body,
                "current_version": env!("CARGO_PKG_VERSION")
            });
            Ok(Some(update_info))
        }
        Ok(None) => {
            println!("[Updater] No update available");
            Ok(None)
        }
        Err(e) => {
            println!("[Updater] Error checking for updates: {}", e);
            Err(format!("Failed to check for updates: {}", e))
        }
    }
}

#[command]
pub async fn install_update(app: tauri::AppHandle, include_prerelease: bool) -> Result<(), String> {
    // Fetch the latest release from GitHub API
    let latest_release = match fetch_latest_release(include_prerelease).await? {
        Some(r) => r,
        None => return Err("No releases found".to_string()),
    };

    let tag = &latest_release.tag_name;
    let endpoint_str = format!(
        "https://github.com/LuckyLuuk12/kable/releases/download/{}/latest.json",
        tag
    );

    let endpoint = match Url::parse(&endpoint_str) {
        Ok(url) => url,
        Err(e) => return Err(format!("Failed to parse endpoint URL: {}", e)),
    };

    let builder = match app.updater_builder().endpoints(vec![endpoint]) {
        Ok(b) => b,
        Err(e) => return Err(format!("Failed to set endpoints: {}", e)),
    };

    let updater = match builder.build() {
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
