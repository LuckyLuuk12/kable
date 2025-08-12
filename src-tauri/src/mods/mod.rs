pub mod cache;
// pub mod curseforge;
pub mod manager;
pub mod modrinth;

pub use self::cache::*;
// pub use self::curseforge::*;
pub use self::manager::*;
pub use self::modrinth::*;

use crate::installations::kable_profiles::KableInstallation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderKind {
    Modrinth,
    // CurseForge, // TODO
}

use once_cell::sync::Lazy;
use std::sync::Mutex;

static MODRINTH: Lazy<Mutex<ModrinthProvider>> =
    Lazy::new(|| Mutex::new(ModrinthProvider::default()));

pub fn set_provider_filter(
    provider: ProviderKind,
    installation: Option<&KableInstallation>,
    filter: Option<ModFilter>,
) {
    match provider {
        ProviderKind::Modrinth => {
            MODRINTH.lock().unwrap().filter(installation, filter);
        } //ProviderKind::CurseForge => { /* TODO */ }
    }
}

pub fn set_provider_limit(provider: ProviderKind, limit: usize) {
    match provider {
        ProviderKind::Modrinth => {
            MODRINTH.lock().unwrap().set_limit(limit);
        } //ProviderKind::CurseForge => { /* TODO */ }
    }
}

pub async fn get_mods(provider: ProviderKind, offset: usize) -> Result<Vec<ModInfoKind>, String> {
    match provider {
        ProviderKind::Modrinth => {
            let mut prov = {
                let prov_guard = MODRINTH.lock().unwrap();
                prov_guard.clone()
            };
            prov.get(offset).await
        }
        //ProviderKind::CurseForge => todo!(),
    }
}

pub async fn download_mod(
    provider: ProviderKind,
    mod_id: &str,
    version_id: Option<&str>,
    installation: &KableInstallation,
) -> Result<(), String> {
    match provider {
        ProviderKind::Modrinth => {
            let prov = {
                let prov_guard = MODRINTH.lock().unwrap();
                prov_guard.clone()
            };
            prov.download(mod_id, version_id, installation).await
        }
        //ProviderKind::CurseForge => todo!(),
    }
}

pub fn clear_provider_cache(provider: ProviderKind) {
    match provider {
        ProviderKind::Modrinth => {
            let mut prov = MODRINTH.lock().unwrap();
            prov.cache.clear();
            let _ = prov.cache.save_to_disk(&prov.cache_path);
        } //ProviderKind::CurseForge => { /* TODO */ }
    }
}

pub fn purge_stale_provider_cache(provider: ProviderKind) {
    match provider {
        ProviderKind::Modrinth => {
            let mut prov = MODRINTH.lock().unwrap();
            prov.cache.purge_stale();
            let _ = prov.cache.save_to_disk(&prov.cache_path);
        } //ProviderKind::CurseForge => { /* TODO */ }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtendedModInfo {
    pub mod_jar_info: crate::ModJarInfo,
    pub page_uri: Option<String>, // URI to the mod's page, e.g., on Modrinth
    pub icon_uri: Option<String>,
    pub description: Option<String>,
    pub authors: Vec<String>,
}

pub async fn get_extended_mod_info(
    mod_jar_info: crate::ModJarInfo,
) -> Result<ExtendedModInfo, String> {
    // Only hold the lock for the cache lookup
    let (found_info, mod_name, _loader) = {
        let modrinth = MODRINTH.lock().unwrap();
        let mod_name = mod_jar_info.mod_name.as_deref().unwrap_or("");
        let loader = mod_jar_info.loader.as_deref();
        let mut found_info = None;
        for entry in modrinth.cache.entries.values() {
            if let Some(found) = entry.value.iter().find(|info| {
                let name_match = info.title.eq_ignore_ascii_case(mod_name)
                    || info.slug.eq_ignore_ascii_case(mod_name);
                let loader_match = if let Some(loader) = loader {
                    info.loaders.as_ref().is_some_and(|ls| ls.iter().any(|l| l.eq_ignore_ascii_case(loader)))
                } else {
                    true
                };
                name_match && loader_match
            }) {
                found_info = Some(found.clone());
                break;
            }
        }
        (found_info, mod_name.to_string(), loader.map(|s| s.to_string()))
    };

    if let Some(found) = found_info {
        return Ok(ExtendedModInfo {
            mod_jar_info,
            icon_uri: found.icon_url.clone(),
            description: Some(found.description.clone()),
            authors: vec![found.author.clone()],
            page_uri: found.source_url.clone(),
        });
    }

    // Not found in cache, try Modrinth API search (no lock held here)
    let query = if !mod_name.is_empty() {
        &mod_name
    } else if let Some(name) = mod_jar_info.mod_name.as_deref() {
        name
    } else {
        mod_jar_info.file_name.as_str()
    };
    let url = format!("https://api.modrinth.com/v2/search?query={}&limit=1", urlencoding::encode(query));
    let resp = reqwest::get(&url).await.map_err(|e| format!("Modrinth API error: {e}"))?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("Modrinth API read error: {e}"))?;
    if !status.is_success() {
        return Err(format!("Modrinth API HTTP error: {} - {}", status, text));
    }
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("Modrinth API parse error: {}\nResponse body: {}", e, text))?;
    if let Some(hits) = json.get("hits").and_then(|v| v.as_array()) {
        if let Some(hit) = hits.first() {
            if let Ok(info) = serde_json::from_value::<crate::modrinth::ModrinthInfo>(hit.clone()) {
                // Cache the result to avoid repeated API calls
                {
                    let mut modrinth = MODRINTH.lock().unwrap();
                    let cache_key = format!("search:{}", query);
                    // If an entry exists, update it; otherwise, insert a new Vec
                    let mut mods = if let Some(entry) = modrinth.cache.entries.get(&cache_key) {
                        entry.value.clone()
                    } else {
                        Vec::new()
                    };
                    if !mods.iter().any(|m| m.project_id == info.project_id) {
                        mods.push(info.clone());
                    }
                    modrinth.cache.insert(cache_key, mods);
                    let _ = modrinth.cache.save_to_disk(&modrinth.cache_path);
                }
                return Ok(ExtendedModInfo {
                    mod_jar_info,
                    icon_uri: info.icon_url.clone(),
                    description: Some(info.description.clone()),
                    authors: vec![info.author.clone()],
                    page_uri: info.source_url.clone(),
                });
            }
        }
    }
    Err(format!("Mod '{}' not found in Modrinth cache or API", mod_name))
}