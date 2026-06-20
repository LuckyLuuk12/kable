use crate::constants::{ASSETS_DIR, LATEST_RELEASE, LATEST_SNAPSHOT, MINECRAFT_VERSION_MANIFEST_URL, VERSIONS_DIR};
use crate::logging::Logger;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Library {
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
    pub rules: Option<Vec<Rule>>,
    pub natives: Option<HashMap<String, String>>,
    pub extract: Option<Extract>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artifact {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
    pub action: String,
    pub os: Option<OsRule>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OsRule {
    pub name: Option<String>,
    pub arch: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum AssetMode {
    Minimal,
    MinimalWithSounds,
    Full,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Extract {
    pub exclude: Option<Vec<String>>,
}

/// Loads a Minecraft version manifest, recursively merging inherited manifests if needed.
/// Returns the fully merged manifest as serde_json::Value.
pub fn load_and_merge_manifest_sync(minecraft_dir: &str, version_id: &str, instance_id: Option<&str>) -> Result<Value, String> {
    Logger::debug_global(&format!("Loading manifest for version_id: {}", version_id), instance_id);

    // If version_id is a placeholder like latest-release/latest-snapshot/latest, resolve it first
    let effective_version = if version_id == LATEST_RELEASE || version_id == LATEST_SNAPSHOT || version_id == "latest" {
        let client = reqwest::blocking::Client::new();
        let resp = client.get(MINECRAFT_VERSION_MANIFEST_URL).send().map_err(|err| format!("Failed to fetch version list: {}", err))?;
        let manifest_list: Value = resp.json().map_err(|err| format!("Failed to parse version list: {}", err))?;
        let resolved_version = if let Some(latest) = manifest_list.get("latest") {
            if version_id == LATEST_SNAPSHOT {
                latest.get("snapshot").and_then(|v| v.as_str()).map(|s| s.to_string())
            } else {
                latest.get("release").and_then(|v| v.as_str()).map(|s| s.to_string())
            }
        } else {
            None
        };
        let resolved_version = resolved_version
            .ok_or_else(|| format!("Failed to resolve '{}' to a concrete version from version_manifest.json", version_id))?;
        Logger::debug_global(&format!("Resolved {} => {}", version_id, resolved_version), instance_id);
        resolved_version
    } else {
        version_id.to_string()
    };

    let manifest_path =
        PathBuf::from(minecraft_dir).join(VERSIONS_DIR).join(&effective_version).join(format!("{}.json", effective_version));

    // Read the manifest for the effective version
    let manifest_str = fs::read_to_string(&manifest_path).map_err(|e| {
        Logger::debug_global(&format!("Failed to read manifest: {}. Tried path: {}", e, manifest_path.display()), instance_id);
        format!("Failed to read manifest: {}", e)
    })?;

    // Parse the manifest we successfully read for the original (non-placeholder) id
    let mut manifest: Value = match serde_json::from_str(&manifest_str) {
        Ok(m) => m,
        Err(e) => {
            Logger::debug_global(&format!("Failed to parse manifest: {}", e), instance_id);
            return Err(format!("Failed to parse manifest: {}", e));
        }
    };

    // If inheritsFrom, recursively merge
    if let Some(parent_id) = manifest.get("inheritsFrom").and_then(|v| v.as_str()) {
        Logger::debug_global(&format!("Manifest {} inherits from {}. Recursively merging...", version_id, parent_id), instance_id);
        let parent = load_and_merge_manifest_sync(minecraft_dir, parent_id, instance_id)?;
        manifest = merge_manifests_with_instance(parent, manifest, instance_id);
    }

    Ok(manifest)
}

pub fn merge_manifests_with_instance(parent: Value, child: Value, _instance_id: Option<&str>) -> Value {
    let mut merged = parent;
    if let (Some(p_obj), Some(c_obj)) = (merged.as_object_mut(), child.as_object()) {
        for (k, v) in c_obj {
            if k == "libraries" {
                if let (Some(p_libs), Some(c_libs)) = (p_obj.get_mut("libraries"), v.as_array()) {
                    if let Some(p_libs_arr) = p_libs.as_array_mut() {
                        p_libs_arr.extend(c_libs.clone());
                    }
                } else {
                    p_obj.insert(k.clone(), v.clone());
                }
            } else {
                p_obj.insert(k.clone(), v.clone());
            }
        }
    }
    merged
}

pub fn merge_manifests(parent: Value, child: Value) -> Value {
    merge_manifests_with_instance(parent, child, None)
}

pub fn compare_versions(v1: &str, v2: &str) -> i32 {
    let parts1: Vec<&str> = v1.split('.').collect();
    let parts2: Vec<&str> = v2.split('.').collect();

    for i in 0..std::cmp::max(parts1.len(), parts2.len()) {
        let p1 = parts1.get(i).unwrap_or(&"0").parse::<i32>().unwrap_or(0);
        let p2 = parts2.get(i).unwrap_or(&"0").parse::<i32>().unwrap_or(0);

        if p1 != p2 {
            return p1 - p2;
        }
    }
    0
}

pub async fn ensure_assets_for_manifest(
    minecraft_dir: &str,
    manifest: &serde_json::Value,
    mode: AssetMode,
    instance_id: Option<&str>,
) -> Result<(), String> {
    use reqwest::Client;

    // Determine assets index name from manifest
    let assets_index_name = match manifest.get("assets").and_then(|v| v.as_str()) {
        Some(n) if !n.is_empty() => n.to_string(),
        _ => {
            Logger::debug_global("No assets index in manifest; skipping assets", instance_id);
            return Ok(());
        }
    };

    let indexes_dir = PathBuf::from(minecraft_dir).join(ASSETS_DIR).join("indexes");
    let objects_dir = PathBuf::from(minecraft_dir).join(ASSETS_DIR).join("objects");
    crate::system::fs::create_dir(&indexes_dir).await.map_err(|e| format!("Failed to create indexes dir: {}", e))?;
    crate::system::fs::create_dir(&objects_dir).await.map_err(|e| format!("Failed to create objects dir: {}", e))?;

    let index_path = indexes_dir.join(format!("{}.json", assets_index_name));
    let client = Client::new();

    // Fetch index JSON if missing
    if !index_path.exists() {
        if let Some(asset_index_obj) = manifest.get("assetIndex").and_then(|v| v.as_object()) {
            if let Some(url) = asset_index_obj.get("url").and_then(|v| v.as_str()) {
                let resp = client.get(url).send().await.map_err(|e| format!("Failed to fetch assets index: {e}"))?;
                let txt = resp.text().await.map_err(|e| format!("Failed to read assets index text: {e}"))?;
                crate::system::fs::create_dir(&index_path).await?;
                crate::system::fs::write_str(&index_path, txt.as_str(), false)
                    .await
                    .map_err(|e| format!("Failed to write assets index: {e}"))?;
            } else {
                Logger::debug_global("No assetIndex.url in manifest; skipping index download", instance_id);
                return Ok(());
            }
        } else {
            Logger::debug_global("No assetIndex object in manifest; skipping index download", instance_id);
            return Ok(());
        }
    }

    // Parse index JSON
    let index_str = tokio::fs::read_to_string(&index_path).await.map_err(|e| format!("Failed to read assets index: {}", e))?;
    let index_json: serde_json::Value = serde_json::from_str(&index_str).map_err(|e| format!("Failed to parse assets index: {}", e))?;
    let objects = index_json.get("objects").and_then(|v| v.as_object()).ok_or("No objects in assets index")?;

    // Build list of required object hashes depending on mode
    let mut required_hashes: Vec<String> = Vec::new();
    if let AssetMode::Minimal = mode {
        let minimal_paths = vec![
            "minecraft/textures/gui/title/background/panorama_0.png",
            "minecraft/textures/gui/title/background/panorama_1.png",
            "minecraft/textures/gui/title/background/panorama_2.png",
            "minecraft/textures/gui/title/background/panorama_3.png",
            "minecraft/textures/gui/title/background/panorama_4.png",
            "minecraft/textures/gui/title/background/panorama_5.png",
            "minecraft/textures/gui/title/background/panorama_blur.png",
            "icons/icon_16x16.png",
            "icons/icon_128x128.png",
        ];
        for p in minimal_paths {
            if let Some(obj) = objects.get(p) {
                if let Some(hash) = obj.get("hash").and_then(|h| h.as_str()) {
                    required_hashes.push(hash.to_string());
                }
            }
        }
    } else {
        for (_k, v) in objects.iter() {
            if let Some(hash) = v.get("hash").and_then(|h| h.as_str()) {
                if mode == AssetMode::Full {
                    required_hashes.push(hash.to_string());
                } else if mode == AssetMode::MinimalWithSounds {
                    // Collect sounds (simplified)
                    required_hashes.push(hash.to_string());
                }
            }
        }
    }

    // Download missing objects
    for hash in required_hashes {
        let prefix = &hash[..2];
        let obj_path = objects_dir.join(prefix).join(&hash);
        if !obj_path.exists() {
            let url = format!("https://resources.download.minecraft.net/{}/{}", prefix, hash);
            let resp = client.get(url).send().await;
            if let Ok(r) = resp {
                if let Ok(bytes) = r.bytes().await {
                    crate::system::fs::create_dir(&obj_path).await?;
                    let _ = crate::system::fs::write(&obj_path, &bytes, false).await;
                }
            }
        }
    }

    Ok(())
}
