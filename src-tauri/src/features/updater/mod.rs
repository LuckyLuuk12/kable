use api_types::updater::{GitHubRelease, UpdateData};
use tauri_plugin_updater::UpdaterExt;
use url::Url;

// Helper function to extract version from name field
// Strips everything until the first digit is found
// Examples: "app-v0.1.9" -> "0.1.9", "nightly 0.1.9-12345" -> "0.1.9-12345"
fn extract_version(name: &str) -> &str {
    let start_idx = name.find(|c: char| c.is_ascii_digit()).unwrap_or(0);
    &name[start_idx..]
}

// Version comparator logic
fn is_update(current: &str, update: &str) -> bool {
    // Extract versions from name field (strips prefixes like "app-v" or "nightly ")
    let current = extract_version(current);
    let update = extract_version(update);

    let current_has_pre = current.contains('-');
    let update_has_pre = update.contains('-');

    // Parse base version (before dash)
    let current_base = match semver::Version::parse(current.split('-').next().unwrap_or(current)) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let update_base = match semver::Version::parse(update.split('-').next().unwrap_or(update)) {
        Ok(v) => v,
        Err(_) => return false,
    };

    // If base versions differ, use semver comparison
    if current_base != update_base {
        return update_base > current_base;
    }

    // Base versions are equal, check pre-release/nightly status
    match (current_has_pre, update_has_pre) {
        (true, false) => false, // Stable is older than nightly with same base
        (false, true) => true,  // Nightly is newer than stable
        (true, true) => {
            // Both are nightlies with same base version
            if current == update {
                return false; // Same exact version, not an update
            }
            // Compare build numbers numerically (monotonically increasing per base version)
            let cur_build = current.split('-').nth(1).and_then(|s| s.parse::<u32>().ok());
            let upd_build = update.split('-').nth(1).and_then(|s| s.parse::<u32>().ok());
            match (cur_build, upd_build) {
                (Some(c), Some(u)) => u > c,
                _ => false, // Can't parse build numbers, assume not an update
            }
        }
        (false, false) => false, // Both stable and equal
    }
}

/// Checks if there is a pending update and if so execute it in sync and exit the app so the installer can run.
pub fn launch_pending_update() -> Result<(), String> {
    let launcher_dir = crate::system::fs::launcher_dir()?;
    let pending_path = launcher_dir.join(crate::constants::PENDING_UPDATE_FILE);
    // If the pending update file doesn't exist, there's nothing to do
    if !pending_path.exists() {
        return Ok(());
    }

    let contents = std::fs::read_to_string(&pending_path).map_err(|e| format!("Failed to read pending update file: {}", e))?;

    let v: serde_json::Value = serde_json::from_str(contents.as_str()).map_err(|e| e.to_string())?;

    let installer = v.get("installer_path").and_then(|s| s.as_str()).ok_or_else(|| "Invalid pending update data".to_string())?;

    std::process::Command::new(installer).spawn().map_err(|e| format!("Failed to spawn installer: {}", e))?;

    let _ = std::fs::remove_file(pending_path);

    std::process::exit(0);
}

async fn fetch_releases(include_prerelease: bool) -> Result<Vec<GitHubRelease>, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(crate::constants::KABLE_GITHUB_RELEASES)
        .header("User-Agent", "kable-updater")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases: {}", e))?;

    let mut releases: Vec<GitHubRelease> = response.json().await.map_err(|e| format!("Failed to parse releases: {}", e))?;

    releases.retain(|r| !r.draft && (include_prerelease || !r.prerelease));

    Ok(releases)
}

pub async fn check_for_updates(include_prerelease: bool) -> Result<UpdateData, String> {
    // Skip update checks in development builds
    if cfg!(debug_assertions) {
        return Err("Updates are disabled in development mode".into());
    }

    let list = fetch_releases(include_prerelease).await?;
    let current = env!("CARGO_PKG_VERSION").to_string();

    let newer: Vec<_> = list.into_iter().filter(|r| is_update(&current, &r.name)).collect();

    if newer.is_empty() {
        return Err("No updates available".into());
    }

    let latest = &newer[0];
    let tag = &latest.tag_name; // Still use tag_name for download URL

    let endpoint_str = format!("{}/download/{}/latest.json", crate::constants::KABLE_GITHUB_RELEASES, tag);

    let endpoint = Url::parse(&endpoint_str).map_err(|e| format!("Failed to parse endpoint URL: {}", e))?;

    let app = crate::app_handle()?;

    let mut builder = app.updater_builder().endpoints(vec![endpoint]).map_err(|e| format!("Failed to set endpoints: {}", e))?;

    builder = builder.version_comparator(|cur, upd| is_update(&cur.to_string(), &upd.version.to_string()));

    let updater = builder.build().map_err(|e| format!("Failed to build updater: {}", e))?;

    match updater.check().await {
        Ok(Some(update)) => {
            let info = UpdateData {
                version: update.version,
                date: update.date.map(|d| d.to_string()),
                body: update.body.unwrap_or_default(), // Use empty string if body is None
                current_version: current,
            };
            Ok(info)
        }
        Ok(None) => Err("No updates available".into()),
        Err(e) => Err(format!("Failed to check for updates: {}", e)),
    }
}

pub async fn install_update(include_prerelease: bool) -> Result<(), String> {
    // Skip update installation in development builds
    if cfg!(debug_assertions) {
        return Err("Updates are disabled in development mode".to_string());
    }

    let list = fetch_releases(include_prerelease).await?;
    let current = env!("CARGO_PKG_VERSION").to_string();

    let newer: Vec<_> = list.into_iter().filter(|r| is_update(&current, &r.name)).collect();

    if newer.is_empty() {
        return Err("No update available".to_string());
    }

    let latest = &newer[0]; // Still use tag_name for download URL
    let endpoint_str = format!("{}/download/{}/latest.json", crate::constants::KABLE_GITHUB_RELEASES, latest.tag_name);

    let endpoint = Url::parse(&endpoint_str).map_err(|e| format!("Failed to parse endpoint URL: {}", e))?;

    let app = crate::app_handle()?;

    let mut builder = app.updater_builder().endpoints(vec![endpoint]).map_err(|e| format!("Failed to set endpoints: {}", e))?;

    builder = builder.version_comparator(|cur, upd| is_update(&cur.to_string(), &upd.version.to_string()));

    let updater = builder.build().map_err(|e| format!("Failed to get updater: {}", e))?;

    match updater.check().await {
        Ok(Some(update)) => {
            update.download_and_install(|_, _| {}, || {}).await.map_err(|e| format!("Failed to install update: {}", e))?;
            Ok(())
        }
        Ok(None) => Err("No update available".to_string()),
        Err(e) => Err(format!("Failed to check for updates: {}", e)),
    }
}

pub async fn download_update(include_prerelease: bool) -> Result<String, String> {
    // Skip update downloads in development builds
    if cfg!(debug_assertions) {
        return Err("Updates are disabled in development mode".to_string());
    }

    let list = fetch_releases(include_prerelease).await?;
    let current = env!("CARGO_PKG_VERSION").to_string();

    let newer: Vec<_> = list.into_iter().filter(|r| is_update(&current, &r.name)).collect();

    if newer.is_empty() {
        return Err("No update available".to_string());
    }

    let latest = &newer[0];
    let endpoint_str = format!("{}/download/{}/latest.json", crate::constants::KABLE_GITHUB_RELEASES, latest.tag_name);

    let endpoint = Url::parse(&endpoint_str).map_err(|e| format!("Failed to parse endpoint URL: {}", e))?;

    let app = crate::app_handle()?;

    let mut builder = app.updater_builder().endpoints(vec![endpoint]).map_err(|e| format!("Failed to set endpoints: {}", e))?;

    builder = builder.version_comparator(|cur, upd| is_update(&cur.to_string(), &upd.version.to_string()));

    let updater = builder.build().map_err(|e| format!("Failed to build updater: {}", e))?;

    match updater.check().await {
        Ok(Some(update)) => {
            // Try to download the installer bytes using the updater
            let downloaded_bytes = update.download(|_, _| {}, || {}).await.map_err(|e| format!("Failed to download update: {}", e))?;

            // Persist the downloaded installer to disk so it can be applied on restart
            let launcher_dir = crate::system::fs::launcher_dir()?;
            let filename = format!("kable-installer-{}.bin", update.version);
            let download_path = launcher_dir.join(&filename);
            crate::system::fs::write(&download_path, &downloaded_bytes, false)
                .await
                .map_err(|e| format!("Failed to write downloaded installer: {}", e))?;

            let pending_path = launcher_dir.join(crate::constants::PENDING_UPDATE_FILE);
            let payload = serde_json::json!({
                "installer_path": download_path.display().to_string(),
                "version": update.version.to_string()
            });

            crate::system::fs::write(&pending_path, serde_json::to_string_pretty(&payload).unwrap().as_bytes(), false).await?;

            Ok(download_path.display().to_string())
        }
        Ok(None) => Err("No update available".to_string()),
        Err(e) => Err(format!("Failed to check for updates: {}", e)),
    }
}

/// Applies a downloaded update by executing the installer and exiting the app, this is an async version of `launch_pending_update` that cannot be called on app startup
/// So this is only used to make a "install downloaded update" button in the app itself.
pub async fn apply_downloaded_update() -> Result<(), String> {
    let launcher_dir = crate::system::fs::launcher_dir()?;
    let pending_path = launcher_dir.join(crate::constants::PENDING_UPDATE_FILE);

    if !pending_path.exists() {
        return Err("No pending update found".to_string());
    }

    let contents = crate::system::fs::read_str(&pending_path).await?;
    let v: serde_json::Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;
    let installer = v.get("installer_path").and_then(|s| s.as_str()).ok_or_else(|| "Invalid pending update data".to_string())?;

    // Spawn the installer and exit to allow it to run
    match std::process::Command::new(installer).spawn() {
        Ok(_) => {
            // remove pending file
            let _ = crate::system::fs::remove_file(&pending_path).await;
            std::process::exit(0);
        }
        Err(e) => Err(format!("Failed to spawn installer: {}", e)),
    }
}

pub async fn get_current_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}
