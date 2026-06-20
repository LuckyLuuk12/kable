use std::future::Future;
use std::path::PathBuf;

use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::Serialize;

use super::config::cache_root;
use super::entry::CacheEntry;
use super::error::CacheError;
use super::hashing::hash_args;

use crate::system::fs::{read, write};

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
) -> Result<T, CacheError>
where
    T: Serialize + DeserializeOwned + Clone,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, CacheError>>,
{
    let key = hash_args(&args);
    let path = root.join(parent).join(format!("{key}.bin"));

    let lock = super::locks::get_lock(&key);
    let _guard = lock.lock().await;

    if let Some(raw) = read(&path).await.ok() {
        let (entry, _): (CacheEntry<T>, usize) = match bincode::serde::decode_from_slice(&raw, bincode::config::standard()) {
            Ok(v) => v,
            Err(_e) => {
                return Err(CacheError::CorruptedEntry { path });
            } // Err(_) => return Ok(TODO_FALLBACK_OR_SKIP), // see note below
        };

        let now = CacheEntry::<T>::new(entry.value.clone(), entry.ttl_secs, entry.created_at);

        if !now.is_expired(Utc::now().timestamp() as u64) && now.is_current_format() {
            return Ok(entry.value);
        }
    }

    let result = compute().await?;

    let entry = CacheEntry::new(result.clone(), ttl_secs, Utc::now().timestamp() as u64);

    let bytes = bincode::serde::encode_to_vec(&entry, bincode::config::standard())?;

    write(&path, &bytes, false).await.map_err(|e| CacheError::SystemError(e))?;

    Ok(result)
}

/// Macro-facing wrapper (stable ABI for generated code)
#[doc(hidden)]
pub async fn __macro_get_or_compute<T, F, Fut>(
    parent: &str,
    args: impl Serialize,
    ttl_secs: Option<u64>,
    compute: F,
) -> Result<T, CacheError>
where
    T: Serialize + DeserializeOwned + Clone,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, CacheError>>,
{
    let root = cache_root().to_path_buf();
    get_or_compute(root, parent, args, ttl_secs, compute).await
}
