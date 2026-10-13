use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use chrono::Utc;
use moka::future::Cache;
use serde::de::DeserializeOwned;
use serde::Serialize;

use super::config::cache_root;
use super::entry::CacheEntry;
use super::hashing::hash_args;

use crate::{
    system::fs::{read, write},
    Logger,
};

/// Maximum number of serialized cache entries retained in memory.
const MEMORY_CACHE_MAX_ENTRIES: u64 = 10_000;

/// Maximum time an unused entry remains in the memory cache.
///
/// This is an eviction policy, not the authoritative cache TTL.
/// CacheEntry::is_expired remains responsible for validating TTLs.
const MEMORY_CACHE_IDLE_SECS: u64 = 600;

/// Process-wide L1 cache.
///
/// Values are serialized CacheEntry blobs, identical to the representation
/// stored in persistent `.bin` files. This allows the cache to hold values
/// of different types without requiring T: Clone or TypeId-based storage.
static MEMORY_CACHE: OnceLock<Cache<String, Vec<u8>>> = OnceLock::new();

fn memory_cache() -> &'static Cache<String, Vec<u8>> {
    MEMORY_CACHE.get_or_init(|| {
        Cache::builder()
            .max_capacity(MEMORY_CACHE_MAX_ENTRIES)
            .time_to_idle(Duration::from_secs(MEMORY_CACHE_IDLE_SECS))
            .build()
    })
}

/// Returns a stable cache identity including the root, namespace and hash.
///
/// Including the full path prevents collisions between different cache
/// namespaces that happen to use the same argument hash.
fn cache_key(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Decode and validate a serialized cache entry.
///
/// Expired or outdated entries are treated as cache misses.
fn decode_valid_entry<T: DeserializeOwned>(raw: &[u8]) -> Result<Option<T>, String> {
    let (entry, _): (CacheEntry<T>, usize) =
        bincode::serde::decode_from_slice(raw, bincode::config::standard()).map_err(|e| format!("Failed to decode cache entry: {}", e))?;

    if entry.is_expired(Utc::now().timestamp() as u64) || !entry.is_current_format() {
        return Ok(None);
    }

    Ok(Some(entry.value))
}

/**
 * Public API for getting or computing cached values.
 *
 * Lookup order:
 * 1. In-memory cache.
 * 2. Persistent `.bin` cache.
 * 3. Compute the value and write both cache levels.
 *
 * The persistent cache format and macro-facing API remain unchanged.
 */
pub async fn get_or_compute<T, F, Fut>(
    root: PathBuf,
    parent: &str,
    args: impl Serialize,
    ttl_secs: Option<u64>,
    compute: F,
) -> Result<T, String>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, String>>,
{
    let hash = hash_args(&args);
    let path = root.join(parent).join(format!("{hash}.bin"));
    let path_str = path.to_string_lossy().to_string();
    let memory_key = cache_key(&path);

    // Keep memory and disk operations for the same entry serialized.
    let lock = super::locks::get_lock(&memory_key);
    let _guard = lock.lock().await;

    // L1: in-memory cache.
    //
    // The serialized entry is validated on every hit so that per-entry TTL
    // and format-version rules remain authoritative.
    if let Some(raw) = memory_cache().get(&memory_key).await {
        match decode_valid_entry::<T>(&raw) {
            Ok(Some(value)) => {
                Logger::debug_global(&format!("Memory cache hit for key: {path_str}"), None);
                return Ok(value);
            }
            Ok(None) => {
                // Expired or outdated. Continue to the persistent cache.
                memory_cache().invalidate(&memory_key).await;
            }
            Err(error) => {
                // The memory entry is invalid. Remove it and let the
                // persistent cache determine whether the entry is usable.
                memory_cache().invalidate(&memory_key).await;

                Logger::warn_global(&format!("Invalid in-memory cache entry for {path_str}: {error}"), None);
            }
        }
    }

    // L2: persistent cache.
    if let Ok(raw) = read(&path).await {
        match decode_valid_entry::<T>(&raw) {
            Ok(Some(value)) => {
                Logger::debug_global(&format!("Persistent cache hit for key: {path_str}"), None);

                // Warm L1 for subsequent requests.
                memory_cache().insert(memory_key, raw).await;

                return Ok(value);
            }
            Ok(None) => {
                // Expired or outdated. Recompute below.
            }
            Err(error) => {
                return Err(error);
            }
        }
    }

    // Cache miss: compute the value.
    let result = compute().await?;

    let entry = CacheEntry::new(&result, ttl_secs, Utc::now().timestamp() as u64);

    let bytes =
        bincode::serde::encode_to_vec(&entry, bincode::config::standard()).map_err(|e| format!("Failed to encode cache entry: {}", e))?;

    // Persist first. If persistence fails, return the error and do not
    // populate L1 with an entry that was not successfully persisted.
    Logger::debug_global(&format!("Cache miss for key: {path_str}. Storing new entry."), None);

    write(&path, &bytes, false).await.map_err(|e| format!("Failed to write cache entry: {}", e))?;

    // L1: populate memory after the persistent write succeeds.
    memory_cache().insert(memory_key, bytes).await;

    Ok(result)
}

/// Macro-facing wrapper (stable API for generated code).
#[doc(hidden)]
pub async fn __macro_get_or_compute<T, F, Fut>(parent: &str, args: impl Serialize, ttl_secs: Option<u64>, compute: F) -> Result<T, String>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, String>>,
{
    let root = cache_root().map_err(|e| format!("Failed to get cache root: {}", e))?.to_path_buf();

    get_or_compute(root, parent, args, ttl_secs, compute).await
}

/// Invalidate a single cached entry.
pub async fn invalidate<T: Serialize>(parent: &str, fn_name: &str, args: &T) -> Result<(), String> {
    let root = super::config::cache_root().map_err(|e| format!("Failed to get cache root: {}", e))?;

    let key = hash_args(&(fn_name, args));
    let path = root.join(parent).join(format!("{key}.bin"));
    let memory_key = cache_key(&path);

    memory_cache().invalidate(&memory_key).await;

    let _ = crate::system::fs::remove_file(&path).await;

    Ok(())
}

/// Invalidate a cached entry for a function without arguments.
pub async fn invalidate_no_args(parent: &str, fn_name: &str) -> Result<(), String> {
    let root = super::config::cache_root().map_err(|e| format!("Failed to get cache root: {}", e))?;

    let key = hash_args(&(fn_name, ()));
    let path = root.join(parent).join(format!("{key}.bin"));
    let memory_key = cache_key(&path);

    memory_cache().invalidate(&memory_key).await;

    let _ = crate::system::fs::remove_file(&path).await;

    Ok(())
}

/// Invalidate every cached entry in a namespace.
///
/// Only memory entries whose full path belongs to this namespace are
/// removed; unrelated namespaces remain warm.
pub async fn invalidate_group(parent: &str) -> Result<(), String> {
    let root = super::config::cache_root().map_err(|e| format!("Failed to get cache root: {}", e))?;

    let dir = root.join(parent);

    let mut entries = crate::system::fs::read_dir(&dir).await.map_err(|e| format!("Failed to read cache group directory: {}", e))?;

    for entry in entries.iter_mut() {
        let path = entry.as_path();

        if path.is_file() {
            let _ = crate::system::fs::remove_file(path).await;
        }
    }

    // Use a directory prefix with a separator so that namespace "foo"
    // does not accidentally match namespace "foobar".
    let mut prefix = dir.to_string_lossy().into_owned();

    if !prefix.ends_with(std::path::MAIN_SEPARATOR) {
        prefix.push(std::path::MAIN_SEPARATOR);
    }

    let keys: Vec<String> = memory_cache().iter().filter_map(|(key, _value)| key.starts_with(&prefix).then(|| key.to_string())).collect();

    for key in keys {
        memory_cache().invalidate(&key).await;
    }

    Ok(())
}

/// Invalidate the entire persistent cache and clear the in-memory cache.
pub async fn invalidate_all() -> Result<(), String> {
    let root = super::config::cache_root().map_err(|e| format!("Failed to get cache root: {}", e))?;

    let mut entries = crate::system::fs::read_dir(&root).await.map_err(|e| format!("Failed to read cache root directory: {}", e))?;

    for entry in entries.iter_mut() {
        let path = entry.as_path();

        if path.is_file() {
            let _ = crate::system::fs::remove_file(path).await;
        } else if path.is_dir() {
            let _ = crate::system::fs::remove_dir_all(path).await;
        }
    }

    // Clear all L1 entries after the persistent cache is cleared.
    memory_cache().invalidate_all();

    Ok(())
}
