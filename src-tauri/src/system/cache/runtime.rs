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

    let lock = get_lock(&key);
    let _guard = lock.lock().await;

    if let Some(raw) = read_file(&path).await? {
        if let Ok((entry, _)) = bincode::serde::decode_from_slice(&raw, bincode::config::standard()) {
            let now = CacheEntry::<T>::new(entry.value.clone(), entry.ttl_secs, entry.created_at);

            if !now.is_expired(crate::entry::now()) {
                return Ok(entry.value);
            }
        }
    }

    let result = compute().await?;

    let entry = CacheEntry::new(result.clone(), ttl_secs, crate::entry::now());

    let bytes = bincode::serde::encode_to_vec(&entry, bincode::config::standard())?;

    write_atomic(&path, &bytes).await?;

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
    let root = crate::cache_root().to_path_buf();
    get_or_compute(root, parent, args, ttl_secs, compute).await
}
