use chrono::DateTime;
use serde::Deserialize;
use tauri::command;
use tauri_plugin_updater::UpdaterExt;
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
    #[allow(dead_code)]
    published_at: String,
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

    // Strip 'v' prefix if present (e.g., v0.1.9 -> 0.1.9)
    let current = current.strip_prefix('v').unwrap_or(current);
    let update = update.strip_prefix('v').unwrap_or(update);

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
        (true, false) => false, // Stable is newer than pre-release/nightly
        (false, true) => true,  // Pre-release/nightly is newer than stable
        (true, true) => {
            // Both are pre-release/nightly with same base version
            // Build numbers wrap around (modulo 65536), so can't compare them
            // Return true to indicate potential update - caller will use GitHub's
            // published_at timestamp to determine which is actually newer
            true
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

    // Sort by published_at timestamp (most recent first)
    // This ensures nightlies are ordered correctly even when build numbers wrap around
    releases.sort_by(|a, b| {
        let ad = DateTime::parse_from_rfc3339(&a.published_at).unwrap();
        let bd = DateTime::parse_from_rfc3339(&b.published_at).unwrap();
        bd.cmp(&ad) // Descending order (newest first)
    });

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

    let builder = app
        .updater_builder()
        .endpoints(vec![endpoint])
        .map_err(|e| format!("Failed to set endpoints: {}", e))?;

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
pub async fn get_current_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}
