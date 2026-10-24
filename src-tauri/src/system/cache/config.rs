use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static CACHE_ROOT: OnceLock<PathBuf> = OnceLock::new();

pub async fn initialize<P>(cache_root: P) -> Result<(), String>
where
    P: AsRef<Path>,
{
    let root = cache_root.as_ref().to_path_buf();

    crate::system::fs::create_dir(&root).await.map_err(|e| format!("Failed to create cache root directory: {}", e))?;

    CACHE_ROOT.set(root).map_err(|_| "Cache root already initialized".to_string())?;

    Ok(())
}

pub fn cache_root() -> Result<&'static Path, String> {
    CACHE_ROOT.get().map(|p| p.as_path()).ok_or("Cache root not initialized".to_string())
}
