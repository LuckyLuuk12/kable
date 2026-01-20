use serde::Deserialize;
use tauri::command;
use tauri_plugin_updater::UpdaterExt;
use tokio::fs as async_fs;
use url::Url;

#[derive(Debug, Deserialize, Clone)]
struct GitHubRelease {
    tag_name: String,
    name: String,
    prerelease: bool,
    draft: bool,
    #[allow(dead_code)]
    #[serde(default)]
    body: String,
}

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
            let cur_build = current
                .split('-')
                .nth(1)
                .and_then(|s| s.parse::<u32>().ok());
            let upd_build = update.split('-').nth(1).and_then(|s| s.parse::<u32>().ok());
            match (cur_build, upd_build) {
                (Some(c), Some(u)) => u > c,
                _ => false, // Can't parse build numbers, assume not an update
            }
        }
        (false, false) => false, // Both stable and equal
    }
}

async fn fetch_releases(include_prerelease: bool) -> Result<Vec<GitHubRelease>, String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/LuckyLuuk12/kable/releases")
        .header("User-Agent", "kable-updater")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases: {}", e))?;

    let mut releases: Vec<GitHubRelease> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse releases: {}", e))?;

    releases.retain(|r| !r.draft && (include_prerelease || !r.prerelease));

    Ok(releases)
}

#[command]
pub async fn check_for_updates(
    app: tauri::AppHandle,
    include_prerelease: bool,
) -> Result<Option<serde_json::Value>, String> {
    let list = fetch_releases(include_prerelease).await?;
    let current = env!("CARGO_PKG_VERSION").to_string();

    let newer: Vec<_> = list
        .into_iter()
        .filter(|r| is_update(&current, &r.name))
        .collect();

    if newer.is_empty() {
        return Ok(None);
    }

    let latest = &newer[0];
    let tag = &latest.tag_name; // Still use tag_name for download URL

    let endpoint_str = format!(
        "https://github.com/LuckyLuuk12/kable/releases/download/{}/latest.json",
        tag
    );

    let endpoint =
        Url::parse(&endpoint_str).map_err(|e| format!("Failed to parse endpoint URL: {}", e))?;

    let mut builder = app
        .updater_builder()
        .endpoints(vec![endpoint])
        .map_err(|e| format!("Failed to set endpoints: {}", e))?;

    builder = builder
        .version_comparator(|cur, upd| is_update(&cur.to_string(), &upd.version.to_string()));

    let updater = builder
        .build()
        .map_err(|e| format!("Failed to build updater: {}", e))?;

    match updater.check().await {
        Ok(Some(update)) => {
            let info = serde_json::json!({
                "version": update.version,
                "date": update.date.map(|d| d.to_string()),
                "body": update.body,
                "current_version": current
            });
            Ok(Some(info))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Failed to check for updates: {}", e)),
    }
}

#[command]
pub async fn install_update(app: tauri::AppHandle, include_prerelease: bool) -> Result<(), String> {
    let list = fetch_releases(include_prerelease).await?;
    let current = env!("CARGO_PKG_VERSION").to_string();

    let newer: Vec<_> = list
        .into_iter()
        .filter(|r| is_update(&current, &r.name))
        .collect();

    if newer.is_empty() {
        return Err("No update available".to_string());
    }

    let latest = &newer[0]; // Still use tag_name for download URL
    let endpoint_str = format!(
        "https://github.com/LuckyLuuk12/kable/releases/download/{}/latest.json",
        latest.tag_name
    );

    let endpoint =
        Url::parse(&endpoint_str).map_err(|e| format!("Failed to parse endpoint URL: {}", e))?;

    let mut builder = app
        .updater_builder()
        .endpoints(vec![endpoint])
        .map_err(|e| format!("Failed to set endpoints: {}", e))?;

    builder = builder
        .version_comparator(|cur, upd| is_update(&cur.to_string(), &upd.version.to_string()));

    let updater = builder
        .build()
        .map_err(|e| format!("Failed to get updater: {}", e))?;

    match updater.check().await {
        Ok(Some(update)) => {
            update
                .download_and_install(|_, _| {}, || {})
                .await
                .map_err(|e| format!("Failed to install update: {}", e))?;
            Ok(())
        }
        Ok(None) => Err("No update available".to_string()),
        Err(e) => Err(format!("Failed to check for updates: {}", e)),
    }
}

#[command]
pub async fn download_update(
    app: tauri::AppHandle,
    include_prerelease: bool,
) -> Result<String, String> {
    let list = fetch_releases(include_prerelease).await?;
    let current = env!("CARGO_PKG_VERSION").to_string();

    let newer: Vec<_> = list
        .into_iter()
        .filter(|r| is_update(&current, &r.name))
        .collect();

    if newer.is_empty() {
        return Err("No update available".to_string());
    }

    let latest = &newer[0];
    let endpoint_str = format!(
        "https://github.com/LuckyLuuk12/kable/releases/download/{}/latest.json",
        latest.tag_name
    );

    let endpoint =
        Url::parse(&endpoint_str).map_err(|e| format!("Failed to parse endpoint URL: {}", e))?;

    let mut builder = app
        .updater_builder()
        .endpoints(vec![endpoint])
        .map_err(|e| format!("Failed to set endpoints: {}", e))?;

    builder = builder
        .version_comparator(|cur, upd| is_update(&cur.to_string(), &upd.version.to_string()));

    let updater = builder
        .build()
        .map_err(|e| format!("Failed to build updater: {}", e))?;

    match updater.check().await {
        Ok(Some(update)) => {
            // Try to download the installer bytes using the updater
            let downloaded_bytes = update
                .download(|_, _| {}, || {})
                .await
                .map_err(|e| format!("Failed to download update: {}", e))?;

            // Persist the downloaded installer to disk so it can be applied on restart
            let launcher_dir = crate::get_kable_launcher_dir()?;
            let filename = format!("kable-installer-{}.bin", update.version);
            let download_path = launcher_dir.join(&filename);
            async_fs::write(&download_path, &downloaded_bytes)
                .await
                .map_err(|e| format!("Failed to write downloaded installer: {}", e))?;

            let pending_path = launcher_dir.join("pending_update.json");
            let payload = serde_json::json!({
                "installer_path": download_path.display().to_string(),
                "version": update.version.to_string()
            });

            crate::write_file_atomic_async(
                &pending_path,
                serde_json::to_string_pretty(&payload).unwrap().as_bytes(),
            )
            .await?;

            Ok(download_path.display().to_string())
        }
        Ok(None) => Err("No update available".to_string()),
        Err(e) => Err(format!("Failed to check for updates: {}", e)),
    }
}

#[command]
pub async fn apply_downloaded_update() -> Result<(), String> {
    let launcher_dir = crate::get_kable_launcher_dir()?;
    let pending_path = launcher_dir.join("pending_update.json");

    if !pending_path.exists() {
        return Err("No pending update found".to_string());
    }

    let contents = async_fs::read_to_string(&pending_path)
        .await
        .map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;
    let installer = v
        .get("installer_path")
        .and_then(|s| s.as_str())
        .ok_or_else(|| "Invalid pending update data".to_string())?;

    // Spawn the installer and exit to allow it to run
    match std::process::Command::new(installer).spawn() {
        Ok(_) => {
            // remove pending file
            let _ = async_fs::remove_file(&pending_path).await;
            std::process::exit(0);
        }
        Err(e) => Err(format!("Failed to spawn installer: {}", e)),
    }
}

#[command]
pub async fn get_current_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}
