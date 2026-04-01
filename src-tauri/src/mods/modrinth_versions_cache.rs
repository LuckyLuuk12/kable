use crate::mods::cache::ModCache;
use crate::mods::modrinth::ModrinthVersion;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Mutex;

// 6 hour TTL for version cache
const VERSION_CACHE_TTL_SECS: u64 = 6 * 60 * 60;

static MODRINTH_VERSIONS_CACHE: Lazy<Mutex<ModCache<Vec<ModrinthVersion>>>> = Lazy::new(|| {
    let cache_path = match crate::get_minecraft_kable_dir() {
        Ok(dir) => dir.join("modrinth_versions_cache.json"),
        Err(_) => PathBuf::from("modrinth_versions_cache.json"),
    };
    let cache = ModCache::load_from_disk(&cache_path)
        .unwrap_or_else(|_| ModCache::new(VERSION_CACHE_TTL_SECS));
    Mutex::new(cache)
});

pub fn get_cached_versions(key: &str) -> Option<Vec<ModrinthVersion>> {
    let cache = MODRINTH_VERSIONS_CACHE.lock().unwrap();
    if let Some(entry) = cache.get(key) {
        if !cache.is_stale(key) {
            return Some(entry.value.clone());
        }
    }
    None
}

pub fn set_cached_versions(key: &str, versions: Vec<ModrinthVersion>) {
    let mut cache = MODRINTH_VERSIONS_CACHE.lock().unwrap();
    cache.insert(key.to_string(), versions);
    // Save to disk in background
    let cache_path = match crate::get_minecraft_kable_dir() {
        Ok(dir) => dir.join("modrinth_versions_cache.json"),
        Err(_) => PathBuf::from("modrinth_versions_cache.json"),
    };
    let _ = cache.save_to_disk(&cache_path);
}

pub fn clear_versions_cache() {
    let mut cache = MODRINTH_VERSIONS_CACHE.lock().unwrap();
    cache.clear();
    let cache_path = match crate::get_minecraft_kable_dir() {
        Ok(dir) => dir.join("modrinth_versions_cache.json"),
        Err(_) => PathBuf::from("modrinth_versions_cache.json"),
    };
    let _ = cache.save_to_disk(&cache_path);
}
