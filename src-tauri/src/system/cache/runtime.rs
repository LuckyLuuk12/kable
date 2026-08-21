use std::future::Future;
use std::path::PathBuf;

use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::Serialize;

use super::config::cache_root;
use super::entry::CacheEntry;
use super::hashing::hash_args;

use crate::{
    system::fs::{read, write},
    Logger,
};

/**
 * This is the only public API of the cache module. It provides a simple interface for getting or computing cached values.
 * The macro `persistent_cache` defined in the `kable-macros` crate will generate calls to `__macro_get_or_compute`, which in turn calls this function with the appropriate root path.
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
    let key = hash_args(&args);
    let path = root.join(parent).join(format!("{key}.bin"));
    let path_str = path.to_string_lossy().to_string();

    let lock = super::locks::get_lock(&key);
    let _guard = lock.lock().await;

    if let Ok(raw) = read(&path).await {
        let (entry, _): (CacheEntry<T>, usize) = match bincode::serde::decode_from_slice(&raw, bincode::config::standard()) {
            Ok(v) => v,
            Err(_e) => {
                return Err(format!("Failed to decode cache entry: {}", _e));
            }
        };

        if !entry.is_expired(Utc::now().timestamp() as u64) && entry.is_current_format() {
            Logger::debug_global(&format!("Cache hit for key: {path_str}"), None);
            return Ok(entry.value);
        }
    }

    let result = compute().await?;

    let entry = CacheEntry::new(&result, ttl_secs, Utc::now().timestamp() as u64);

    let bytes =
        bincode::serde::encode_to_vec(&entry, bincode::config::standard()).map_err(|e| format!("Failed to encode cache entry: {}", e))?;

    Logger::debug_global(&format!("Cache miss for key: {path_str}. Storing new entry."), None);
    write(&path, &bytes, false).await.map_err(|e| format!("Failed to write cache entry: {}", e))?;

    Ok(result)
}

/// Macro-facing wrapper (stable ABI for generated code)
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

/// Invalidate cache:
pub async fn invalidate<T: Serialize>(parent: &str, fn_name: &str, args: &T) -> Result<(), String> {
    let root = super::config::cache_root().map_err(|e| format!("Failed to get cache root: {}", e))?;

    let key = super::hashing::hash_args(&(fn_name, args));
    let path = root.join(parent).join(format!("{key}.bin"));

    let _ = crate::system::fs::remove_file(&path).await;

    Ok(())
}

pub async fn invalidate_no_args(parent: &str, fn_name: &str) -> Result<(), String> {
    let root = super::config::cache_root().map_err(|e| format!("Failed to get cache root: {}", e))?;

    let key = super::hashing::hash_args(&(fn_name, ()));
    let path = root.join(parent).join(format!("{key}.bin"));

    let _ = crate::system::fs::remove_file(&path).await;

    Ok(())
}

pub async fn invalidate_group(parent: &str) -> Result<(), String> {
    let root = super::config::cache_root().map_err(|e| format!("Failed to get cache root: {}", e))?;
    let dir = root.join(parent);

    let mut entries = crate::system::fs::read_dir(&dir).await.map_err(|e| format!("Failed to read cache group directory: {}", e))?;
    // Iterate over the entries (PathBufs) and remove them
    for entry in entries.iter_mut() {
        let path = entry.as_path();
        if path.is_file() {
            let _ = crate::system::fs::remove_file(&path).await;
        }
    }

    Ok(())
}

pub async fn invalidate_all() -> Result<(), String> {
    let root = super::config::cache_root().map_err(|e| format!("Failed to get cache root: {}", e))?;
    let mut entries = crate::system::fs::read_dir(&root).await.map_err(|e| format!("Failed to read cache root directory: {}", e))?;
    // Iterate over the entries (PathBufs) and remove them
    for entry in entries.iter_mut() {
        let path = entry.as_path();
        if path.is_file() {
            let _ = crate::system::fs::remove_file(&path).await;
        } else if path.is_dir() {
            let _ = crate::system::fs::remove_dir_all(&path).await;
        }
    }

    Ok(())
}
