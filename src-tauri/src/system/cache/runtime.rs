use std::future::Future;
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use chrono::Utc;
use moka::future::Cache;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

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
const MEMORY_CACHE_IDLE_SECS: u64 = 600;

/// Process-wide L1 cache containing serialized CacheEntry values.
static MEMORY_CACHE: OnceLock<Cache<String, Vec<u8>>> = OnceLock::new();

/// Coordinates cache operations with invalidation and cache inspection.
///
/// Normal operations hold a shared lock. Invalidation and listing hold an
/// exclusive lock to prevent concurrent writes while they operate.
static CACHE_OPERATION_LOCK: OnceLock<RwLock<()>> = OnceLock::new();

fn memory_cache() -> &'static Cache<String, Vec<u8>> {
    MEMORY_CACHE.get_or_init(|| {
        Cache::builder()
            .max_capacity(MEMORY_CACHE_MAX_ENTRIES)
            .time_to_idle(Duration::from_secs(MEMORY_CACHE_IDLE_SECS))
            .build()
    })
}

fn operation_lock() -> &'static RwLock<()> {
    CACHE_OPERATION_LOCK.get_or_init(|| RwLock::new(()))
}

/// Returns the stable identity used by the memory cache.
fn cache_key(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Rejects absolute paths and parent-directory traversal in namespaces.
fn validate_namespace(parent: &str) -> Result<(), String> {
    let path = Path::new(parent);

    if parent.is_empty()
        || path.components().any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
    {
        return Err(format!("Invalid cache namespace: {parent}"));
    }

    Ok(())
}

/// Decodes and validates a serialized cache entry.
///
/// Expired or outdated entries are treated as cache misses.
fn decode_valid_entry<T: DeserializeOwned>(raw: &[u8]) -> Result<Option<T>, String> {
    let (entry, _): (CacheEntry<T>, usize) =
        bincode::serde::decode_from_slice(raw, bincode::config::standard()).map_err(|e| format!("Failed to decode cache entry: {e}"))?;

    if entry.is_expired(Utc::now().timestamp() as u64) || !entry.is_current_format() {
        return Ok(None);
    }

    Ok(Some(entry.value))
}

/// Persistent cache entry metadata suitable for frontend consumption.
///
/// Paths are relative to the configured cache root.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CacheEntryInfo {
    /// Relative path, for example `projects/abc123.bin`.
    pub path: PathBuf,

    /// Namespace directory relative to the cache root.
    pub namespace: String,

    /// Persistent file size in bytes.
    pub size_bytes: f64,
}

/// Public API for getting or computing cached values.
///
/// Lookup order:
/// 1. In-memory cache.
/// 2. Persistent `.bin` cache.
/// 3. Compute the value and write both cache levels.
///
/// Shared operation locking permits concurrent requests for different
/// entries. The per-entry lock prevents duplicate computation for the
/// same cache path.
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
    let _operation_guard = operation_lock().read().await;

    let hash = hash_args(&args);
    let path = root.join(parent).join(format!("{hash}.bin"));
    let path_str = path.to_string_lossy().into_owned();
    let memory_key = cache_key(&path);

    let lock = super::locks::get_lock(&memory_key);
    let _entry_guard = lock.lock().await;

    // L1: in-memory cache.
    if let Some(raw) = memory_cache().get(&memory_key).await {
        match decode_valid_entry::<T>(&raw) {
            Ok(Some(value)) => {
                Logger::debug_global(&format!("Memory cache hit for key: {path_str}"), None);

                return Ok(value);
            }
            Ok(None) => {
                memory_cache().invalidate(&memory_key).await;
            }
            Err(error) => {
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

                memory_cache().insert(memory_key, raw).await;

                return Ok(value);
            }
            Ok(None) => {}
            Err(error) => return Err(error),
        }
    }

    // Cache miss: compute the value.
    let result = compute().await?;

    let entry = CacheEntry::new(&result, ttl_secs, Utc::now().timestamp() as u64);

    let bytes =
        bincode::serde::encode_to_vec(&entry, bincode::config::standard()).map_err(|e| format!("Failed to encode cache entry: {e}"))?;

    Logger::debug_global(&format!("Cache miss for key: {path_str}. Storing new entry."), None);

    write(&path, &bytes, false).await.map_err(|e| format!("Failed to write cache entry: {e}"))?;

    memory_cache().insert(memory_key, bytes).await;

    Ok(result)
}

/// Macro-facing wrapper. The generated-code API remains unchanged.
#[doc(hidden)]
pub async fn __macro_get_or_compute<T, F, Fut>(parent: &str, args: impl Serialize, ttl_secs: Option<u64>, compute: F) -> Result<T, String>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, String>>,
{
    let root = cache_root().map_err(|e| format!("Failed to get cache root: {e}"))?.to_path_buf();

    get_or_compute(root, parent, args, ttl_secs, compute).await
}

/// Removes a persistent file, treating an already-missing file as success.
async fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("Failed to remove cache file {}: {error}", path.display())),
    }
}

/// Invalidates a single cached entry.
pub async fn invalidate<T: Serialize>(parent: &str, fn_name: &str, args: &T) -> Result<(), String> {
    validate_namespace(parent)?;

    let _operation_guard = operation_lock().write().await;

    let root = cache_root().map_err(|e| format!("Failed to get cache root: {e}"))?;

    let key = hash_args(&(fn_name, args));
    let path = root.join(parent).join(format!("{key}.bin"));
    let memory_key = cache_key(&path);

    remove_file_if_exists(&path).await?;
    memory_cache().invalidate(&memory_key).await;

    Ok(())
}

/// Invalidates an entry for a function without arguments.
pub async fn invalidate_no_args(parent: &str, fn_name: &str) -> Result<(), String> {
    validate_namespace(parent)?;

    let _operation_guard = operation_lock().write().await;

    let root = cache_root().map_err(|e| format!("Failed to get cache root: {e}"))?;

    let key = hash_args(&(fn_name, ()));
    let path = root.join(parent).join(format!("{key}.bin"));
    let memory_key = cache_key(&path);

    remove_file_if_exists(&path).await?;
    memory_cache().invalidate(&memory_key).await;

    Ok(())
}

/// Invalidates every cached entry in a namespace.
///
/// The namespace directory is removed recursively. Its in-memory entries
/// are invalidated using a path prefix with a trailing separator.
pub async fn invalidate_group(parent: &str) -> Result<(), String> {
    validate_namespace(parent)?;

    let _operation_guard = operation_lock().write().await;

    let root = cache_root().map_err(|e| format!("Failed to get cache root: {e}"))?;

    let dir = root.join(parent);

    // Remove persistent entries, including nested directories.
    match tokio::fs::remove_dir_all(&dir).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!("Failed to remove cache namespace {}: {error}", dir.display()));
        }
    }

    // Keep unrelated namespaces, such as "foobar", untouched.
    let mut prefix = cache_key(&dir);

    if !prefix.ends_with(std::path::MAIN_SEPARATOR) {
        prefix.push(std::path::MAIN_SEPARATOR);
    }

    let keys: Vec<String> = memory_cache().iter().filter_map(|(key, _value)| key.starts_with(&prefix).then(|| key.to_string())).collect();

    for key in keys {
        memory_cache().invalidate(&key).await;
    }

    Ok(())
}

/// Invalidates every persistent cache entry and clears the memory cache.
pub async fn invalidate_all() -> Result<(), String> {
    let _operation_guard = operation_lock().write().await;

    let root = cache_root().map_err(|e| format!("Failed to get cache root: {e}"))?;

    let mut entries = match tokio::fs::read_dir(&root).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            memory_cache().invalidate_all();
            return Ok(());
        }
        Err(error) => {
            return Err(format!("Failed to read cache root directory {}: {error}", root.display()));
        }
    };

    while let Some(entry) = entries.next_entry().await.map_err(|e| format!("Failed to enumerate cache root: {e}"))? {
        let path = entry.path();
        let file_type = entry.file_type().await.map_err(|e| format!("Failed to inspect {}: {e}", path.display()))?;

        if file_type.is_dir() {
            tokio::fs::remove_dir_all(&path).await.map_err(|e| format!("Failed to remove {}: {e}", path.display()))?;
        } else {
            remove_file_if_exists(&path).await?;
        }
    }

    memory_cache().invalidate_all();

    Ok(())
}

/// Clears only the in-memory cache.
///
/// Persistent `.bin` files are left untouched.
pub async fn invalidate_memory() {
    let _operation_guard = operation_lock().write().await;

    memory_cache().invalidate_all();
}

/// Lists persistent `.bin` cache entries.
///
/// The returned paths are relative to the configured cache root.
/// Entries are sorted by relative path.
///
/// An exclusive operation lock ensures that application cache operations
/// cannot modify the persistent cache while this listing is in progress.
pub async fn list_entries() -> Result<Vec<CacheEntryInfo>, String> {
    let _operation_guard = operation_lock().write().await;

    let root = cache_root().map_err(|e| format!("Failed to get cache root: {e}"))?.to_path_buf();

    let mut pending = vec![root.clone()];
    let mut result = Vec::new();

    while let Some(dir) = pending.pop() {
        let mut entries = match tokio::fs::read_dir(&dir).await {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                continue;
            }
            Err(error) => {
                return Err(format!("Failed to read cache directory {}: {error}", dir.display()));
            }
        };

        while let Some(entry) =
            entries.next_entry().await.map_err(|e| format!("Failed to enumerate cache directory {}: {e}", dir.display()))?
        {
            let path = entry.path();

            let file_type = entry.file_type().await.map_err(|e| format!("Failed to inspect {}: {e}", path.display()))?;

            if file_type.is_dir() {
                pending.push(path);
                continue;
            }

            if !file_type.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("bin") {
                continue;
            }

            let metadata = entry.metadata().await.map_err(|e| format!("Failed to read metadata for {}: {e}", path.display()))?;

            let relative_path = path.strip_prefix(&root).map_err(|e| format!("Cache entry is outside the cache root: {e}"))?.to_path_buf();

            let namespace = relative_path.parent().unwrap_or_else(|| Path::new("")).to_string_lossy().into_owned();

            result.push(CacheEntryInfo { path: relative_path, namespace, size_bytes: metadata.len() as f64 });
        }
    }

    result.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(result)
}

pub async fn invalidate_entry(path: &str) -> Result<(), String> {
    let root = cache_root().map_err(|e| format!("Failed to get cache root: {e}"))?.to_path_buf();

    let full_path = root.join(path);

    if !full_path.starts_with(&root) {
        return Err(format!("Invalid cache entry path: {path}"));
    }

    let memory_key = cache_key(&full_path);

    remove_file_if_exists(&full_path).await?;
    memory_cache().invalidate(&memory_key).await;

    Ok(())
}
