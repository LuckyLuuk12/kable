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

/// Maximum total weight of serialized entries retained in memory.
const MEMORY_CACHE_MAX_BYTES: u64 = 256 * 1024 * 1024; // 256 MB

/// Maximum size of a single entry retained in memory.
const MEMORY_CACHE_MAX_ENTRY_BYTES: usize = 32 * 1024 * 1024; // 32 MB

/// Maximum size of a persistent cache entry accepted for reading.
const PERSISTENT_CACHE_MAX_ENTRY_BYTES: u64 = 64 * 1024 * 1024; // 64 MB

/// Maximum time an unused entry remains in the memory cache.
const MEMORY_CACHE_IDLE_SECS: u64 = 600; // 10 minutes

/// Process-wide L1 cache containing serialized CacheEntry values.
static MEMORY_CACHE: OnceLock<Cache<String, Vec<u8>>> = OnceLock::new();

/// Coordinates cache operations with invalidation and cache inspection.
static CACHE_OPERATION_LOCK: OnceLock<RwLock<()>> = OnceLock::new();

fn memory_cache() -> &'static Cache<String, Vec<u8>> {
    MEMORY_CACHE.get_or_init(|| {
        Cache::builder()
            .max_capacity(MEMORY_CACHE_MAX_BYTES)
            .weigher(|_key: &String, value: &Vec<u8>| value.len().min(u32::MAX as usize) as u32)
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

    if parent.trim().is_empty()
        || path.components().any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
    {
        return Err(format!("Invalid cache namespace: {parent}"));
    }

    Ok(())
}

/// Logs cache failures without making logging failures part of cache behavior.
fn log_cache_warning(message: &str) {
    Logger::warn_global(message, None);
}

/// Logs cache diagnostics.
fn log_cache_debug(message: &str) {
    Logger::debug_global(message, None);
}

/// Inserts a serialized entry into the in-memory cache when it fits the budget.
async fn insert_memory_cache(key: String, raw: Vec<u8>) {
    if raw.len() > MEMORY_CACHE_MAX_ENTRY_BYTES {
        log_cache_debug(&format!("Skipping memory cache insertion because the entry is too large ({} bytes)", raw.len()));
        return;
    }

    memory_cache().insert(key, raw).await;
}

/// Decodes and validates a serialized cache entry.
///
/// Invalid, expired, incompatible, or trailing data is treated as a cache miss.
fn decode_valid_entry<T: DeserializeOwned>(raw: &[u8]) -> Result<Option<T>, String> {
    if raw.len() as u64 > PERSISTENT_CACHE_MAX_ENTRY_BYTES {
        return Err(format!("Cache entry exceeds the maximum supported size of {} bytes", PERSISTENT_CACHE_MAX_ENTRY_BYTES));
    }

    let config = bincode::config::standard().with_limit::<33554432>();

    let (entry, consumed): (CacheEntry<T>, usize) =
        bincode::serde::decode_from_slice(raw, config).map_err(|e| format!("Failed to decode cache entry: {e}"))?;

    if consumed != raw.len() {
        return Err(format!("Cache entry contains {} trailing bytes", raw.len() - consumed));
    }

    if entry.is_expired(Utc::now().timestamp().max(0) as u64) || !entry.is_current_format() {
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

/// Reads a persistent cache file, rejecting oversized files before loading them.
async fn read_cache_file(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = tokio::fs::metadata(path).await.map_err(|e| format!("Failed to inspect cache file {}: {e}", path.display()))?;

    if !metadata.is_file() {
        return Err(format!("Cache entry is not a regular file: {}", path.display()));
    }

    if metadata.len() > PERSISTENT_CACHE_MAX_ENTRY_BYTES {
        return Err(format!("Cache file {} exceeds the maximum supported size ({} bytes)", path.display(), metadata.len()));
    }

    let raw = read(path).await.map_err(|e| format!("Failed to read cache file {}: {e}", path.display()))?;

    if raw.len() as u64 > PERSISTENT_CACHE_MAX_ENTRY_BYTES {
        return Err(format!("Cache file {} exceeds the maximum supported size after reading", path.display()));
    }

    Ok(raw)
}

/// Removes a persistent file, treating an already-missing file as success.
async fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("Failed to remove cache file {}: {error}", path.display())),
    }
}

/// Best-effort removal of an unusable entry.
async fn discard_cache_file(path: &Path) {
    if let Err(error) = remove_file_if_exists(path).await {
        log_cache_warning(&format!("Could not remove unusable cache entry {}: {error}", path.display()));
    }
}

/// Best-effort persistence. A successful computation remains successful if
/// encoding or disk persistence fails.
async fn persist_cache_entry<T: Serialize>(path: &Path, result: &T, ttl_secs: Option<u64>) -> Option<Vec<u8>> {
    let entry = CacheEntry::new(result, ttl_secs, Utc::now().timestamp().max(0) as u64);

    let config = bincode::config::standard().with_limit::<33554432>();

    let bytes = match bincode::serde::encode_to_vec(&entry, config) {
        Ok(bytes) => bytes,
        Err(error) => {
            log_cache_warning(&format!("Failed to encode cache entry {}: {error}", path.display()));
            return None;
        }
    };

    if bytes.len() as u64 > PERSISTENT_CACHE_MAX_ENTRY_BYTES {
        log_cache_warning(&format!("Skipping persistence for oversized cache entry {} ({} bytes)", path.display(), bytes.len()));
        return None;
    }

    if let Err(error) = write(path, &bytes, false).await {
        log_cache_warning(&format!("Failed to persist cache entry {}: {error}", path.display()));
        return None;
    }

    Some(bytes)
}

/// Public API for getting or computing cached values.
///
/// Lookup order:
/// 1. In-memory cache.
/// 2. Persistent cache file.
/// 3. Compute the value.
///
/// Cache storage is best-effort: cache failures do not turn a successful
/// computation into an error. Errors from the computation itself are returned.
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
    validate_namespace(parent)?;

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
                return Ok(value);
            }
            Ok(None) => {
                log_cache_debug(&format!("Invalidating expired or outdated memory cache entry: {path_str}"));
                memory_cache().invalidate(&memory_key).await;
            }
            Err(error) => {
                log_cache_warning(&format!("Invalid memory cache entry for {path_str}; discarding and rebuilding: {error}"));
                memory_cache().invalidate(&memory_key).await;
            }
        }
    }

    // L2: persistent cache. Any lookup or cleanup failure is treated as a miss.
    match read_cache_file(&path).await {
        Ok(raw) => match decode_valid_entry::<T>(&raw) {
            Ok(Some(value)) => {
                insert_memory_cache(memory_key, raw).await;
                return Ok(value);
            }
            Ok(None) => {
                log_cache_debug(&format!("Persistent cache entry is expired or outdated: {path_str}"));
                discard_cache_file(&path).await;
            }
            Err(error) => {
                log_cache_warning(&format!("Invalid persistent cache entry for {path_str}; rebuilding: {error}"));
                discard_cache_file(&path).await;
            }
        },
        Err(_error) => {
            // Missing files need no cleanup. For other errors, removal is
            // best-effort because the computation can proceed without the cache.
            if tokio::fs::metadata(&path).await.is_ok() {
                discard_cache_file(&path).await;
            }
        }
    }

    // Cache miss or unusable entry: compute the value.
    let result = compute().await?;

    // Persistence is optional. Return the computed value even if the cache
    // directory is unavailable, the entry is too large, or writing fails.
    if let Some(bytes) = persist_cache_entry(&path, &result, ttl_secs).await {
        insert_memory_cache(memory_key, bytes).await;
    }

    Ok(result)
}

/// Macro-facing wrapper. The generated-code API remains unchanged.
///
/// If the cache root cannot be resolved, execute the computation without caching.
#[doc(hidden)]
pub async fn __macro_get_or_compute<T, F, Fut>(parent: &str, args: impl Serialize, ttl_secs: Option<u64>, compute: F) -> Result<T, String>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, String>>,
{
    let root = match cache_root() {
        Ok(root) => root.to_path_buf(),
        Err(error) => {
            log_cache_warning(&format!("Could not resolve cache root; computing without cache: {error}"));
            return compute().await;
        }
    };

    get_or_compute(root, parent, args, ttl_secs, compute).await
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
pub async fn invalidate_group(parent: &str) -> Result<(), String> {
    validate_namespace(parent)?;

    let _operation_guard = operation_lock().write().await;

    let root = cache_root().map_err(|e| format!("Failed to get cache root: {e}"))?;
    let dir = root.join(parent);

    match tokio::fs::remove_dir_all(&dir).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!("Failed to remove cache namespace {}: {error}", dir.display()));
        }
    }

    // Use a trailing separator to avoid invalidating similarly named namespaces.
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
pub async fn list_entries() -> Result<Vec<CacheEntryInfo>, String> {
    let _operation_guard = operation_lock().write().await;

    let root = cache_root().map_err(|e| format!("Failed to get cache root: {e}"))?.to_path_buf();

    let mut pending = vec![root.clone()];
    let mut result = Vec::new();

    while let Some(dir) = pending.pop() {
        let mut entries = match tokio::fs::read_dir(&dir).await {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
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

/// Invalidates a cache entry by its relative path.
pub async fn invalidate_entry(path: &str) -> Result<(), String> {
    let relative_path = Path::new(path);

    // Reject traversal and absolute paths explicitly.
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
    {
        return Err(format!("Invalid cache entry path: {path}"));
    }

    let root = cache_root().map_err(|e| format!("Failed to get cache root: {e}"))?.to_path_buf();

    let full_path = root.join(relative_path);

    if !full_path.starts_with(&root) {
        return Err(format!("Invalid cache entry path: {path}"));
    }

    let memory_key = cache_key(&full_path);

    remove_file_if_exists(&full_path).await?;
    memory_cache().invalidate(&memory_key).await;

    Ok(())
}
