use crate::constants::{CACHE_DIR, FABRIC_META_URL, MANIFESTS_DIR, MINECRAFT_VERSION_MANIFEST_URL, QUILT_META_URL};
use crate::system::fs::{ensure_folder_sync, get_kable_launcher_dir, write_file_atomic_sync};
use api_types::profiles::{LoaderKind, VersionData};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

pub struct Versions(pub Vec<VersionData>);

pub fn get_manifests_cache_dir() -> Result<PathBuf, String> {
    let launcher_dir = get_kable_launcher_dir()?;
    let cache_dir = launcher_dir.join(CACHE_DIR).join(MANIFESTS_DIR);
    ensure_folder_sync(&cache_dir)?;
    Ok(cache_dir)
}

pub fn fetch_with_cache(url: &str, cache_filename: &str, force_refresh: bool) -> Result<Value, String> {
    let cache_dir = get_manifests_cache_dir()?;
    let cache_path = cache_dir.join(cache_filename);

    if !force_refresh && cache_path.exists() {
        if let Ok(content) = fs::read_to_string(&cache_path) {
            if let Ok(val) = serde_json::from_str(&content) {
                return Ok(val);
            }
        }
    }

    let client = reqwest::blocking::Client::new();
    let resp = client.get(url).send().map_err(|e| e.to_string())?;
    let val: Value = resp.json().map_err(|e| e.to_string())?;

    let content = serde_json::to_string(&val).map_err(|e| e.to_string())?;
    let _ = write_file_atomic_sync(&cache_path, content.as_bytes());

    Ok(val)
}

pub fn get_vanilla_versions(force_refresh: bool) -> Result<Vec<VersionData>, String> {
    let manifest = fetch_with_cache(MINECRAFT_VERSION_MANIFEST_URL, "vanilla.json", force_refresh)?;

    let mut versions = Vec::new();
    if let Some(versions_arr) = manifest.get("versions").and_then(|v| v.as_array()) {
        for v in versions_arr {
            let id = v.get("id").and_then(|i| i.as_str()).unwrap_or_default().to_string();
            let release_type = v.get("type").and_then(|t| t.as_str()).unwrap_or_default().to_string();

            versions.push(VersionData {
                version_id: id.clone(),
                display_name: id,
                loader: LoaderKind::Vanilla,
                is_stable: release_type == "release",
                extra: Value::Null, // No extra data for vanilla versions
            });
        }
    }

    Ok(versions)
}

pub fn get_fabric_loaders(force_refresh: bool) -> Result<Value, String> {
    let url = format!("{}/v2/versions/loader", FABRIC_META_URL);
    fetch_with_cache(&url, "fabric_loaders.json", force_refresh)
}

pub fn get_quilt_loaders(force_refresh: bool) -> Result<Value, String> {
    let url = format!("{}/v3/versions/loader", QUILT_META_URL);
    fetch_with_cache(&url, "quilt_loaders.json", force_refresh)
}
