use chrono::DateTime;
use serde::Deserialize;
use tauri::command;
use tauri_plugin_updater::UpdaterExt;
use url::Url;

#[derive(Debug, Deserialize, Clone)]
struct GitHubRelease {
    tag_name: String,
    prerelease: bool,
    draft: bool,
    #[allow(dead_code)]
    #[serde(default)]
    body: String,
    #[allow(dead_code)]
    published_at: String,
}

// Version comparator logic (same behavior as your closure)

fn is_update(current: &str, update: &str) -> bool {
    let current_has_pre = current.contains('-');
    let update_has_pre = update.contains('-');

    let current_base = match semver::Version::parse(current.split('-').next().unwrap_or(current)) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let update_base = match semver::Version::parse(update.split('-').next().unwrap_or(update)) {
        Ok(v) => v,
        Err(_) => return false,
    };

    if current_base != update_base {
        return update_base > current_base;
    }

    match (current_has_pre, update_has_pre) {
        (true, false) => false,
        (false, true) => true,
        (true, true) => {
            let current_build: u32 = current
                .split('-')
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let update_build: u32 = update
                .split('-')
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            update_build > current_build
        }
        (false, false) => false,
    }
}

// Sort releases using the same logic
//   - primarily by version recency (custom comparator)
//   - tie breaker by published_at timestamp

fn sort_releases(mut list: Vec<GitHubRelease>) -> Vec<GitHubRelease> {
    list.sort_by(|a, b| {
        let a_newer = is_update(&b.tag_name, &a.tag_name);
        let b_newer = is_update(&a.tag_name, &b.tag_name);

        if a_newer && !b_newer {
            return std::cmp::Ordering::Greater;
        }
        if b_newer && !a_newer {
            return std::cmp::Ordering::Less;
        }

        let ad = DateTime::parse_from_rfc3339(&a.published_at).unwrap();
        let bd = DateTime::parse_from_rfc3339(&b.published_at).unwrap();
        bd.cmp(&ad)
    });

    list
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

    Ok(sort_releases(releases))
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
        .filter(|r| is_update(&current, &r.tag_name))
        .collect();

    if newer.is_empty() {
        return Ok(None);
    }

    let latest = &newer[0];
    let tag = &latest.tag_name;

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
        .filter(|r| is_update(&current, &r.tag_name))
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
