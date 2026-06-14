use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::CacheError;

static CACHE_ROOT: OnceLock<PathBuf> = OnceLock::new();

pub fn initialize<P>(cache_root: P) -> Result<(), CacheError>
where
    P: AsRef<Path>,
{
    let path = cache_root.as_ref().to_path_buf();

    std::fs::create_dir_all(&path)?;

    CACHE_ROOT
        .set(path)
        .map_err(|_| CacheError::AlreadyInitialized)?;

    Ok(())
}

pub fn cache_root() -> &'static Path {
    CACHE_ROOT.get().expect("cache not initialized").as_path()
}
