use crate::constants::{CACHE_DIR, KABLE_DIR_NAME, LAUNCHER_DIR};

use dirs::home_dir;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;

type FsResult<T> = Result<T, String>;

/// Windows: C:\Users\<User>\AppData\Roaming\.minecraft
/// MacOS: /Users/<User>/Library/Application Support/minecraft
/// Linux: /home/<User>/.minecraft
pub fn mc_dir() -> FsResult<PathBuf> {
    let home = home_dir().ok_or_else(|| "missing home dir".to_string())?;

    #[cfg(target_os = "windows")]
    return Ok(home.join("AppData").join("Roaming").join(".minecraft"));

    #[cfg(target_os = "macos")]
    return Ok(home.join("Library").join("Application Support").join("minecraft"));

    #[cfg(target_os = "linux")]
    return Ok(home.join(".minecraft"));
}

/// Kable directory is a subdirectory of the .minecraft directory, used for storing Kable-specific data.
pub fn kable_dir() -> FsResult<PathBuf> {
    Ok(mc_dir()?.join(KABLE_DIR_NAME))
}

/// Launcher directory is a subdirectory of the Kable directory, used for storing launcher-specific data.
pub fn launcher_dir() -> FsResult<PathBuf> {
    Ok(kable_dir()?.join(LAUNCHER_DIR))
}

pub fn cache_dir() -> FsResult<PathBuf> {
    Ok(kable_dir()?.join(CACHE_DIR))
}

/// Resolve a path to an absolute path, relative to the kable_dir if not already absolute.
fn resolve(path: impl AsRef<Path>) -> FsResult<PathBuf> {
    let p = path.as_ref();
    // This should cover cases where mc_dir is used as mc_dir is returned as absolute path.
    if p.is_absolute() {
        Ok(p.to_path_buf())
    } else {
        // in all other cases we usually assume the path is relative to the kable_dir, so we resolve it against that.
        Ok(kable_dir()?.join(p))
    }
}

pub async fn exists(path: impl AsRef<Path>) -> FsResult<bool> {
    match async_fs::metadata(path.as_ref()).await {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn is_file(path: impl AsRef<Path>) -> FsResult<bool> {
    match async_fs::metadata(path.as_ref()).await {
        Ok(md) => Ok(md.is_file()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn is_dir(path: impl AsRef<Path>) -> FsResult<bool> {
    match async_fs::metadata(path.as_ref()).await {
        Ok(md) => Ok(md.is_dir()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn create_dir(path: impl AsRef<Path>) -> FsResult<PathBuf> {
    let p = resolve(path)?;

    async_fs::create_dir_all(&p).await.map_err(|e| format!("create_dir failed {}: {}", p.display(), e))?;

    Ok(p)
}

pub async fn read_dir(path: impl AsRef<Path>) -> FsResult<Vec<PathBuf>> {
    let p = resolve(path)?;

    let mut entries = async_fs::read_dir(&p).await.map_err(|e| format!("read_dir failed {}: {}", p.display(), e))?;

    let mut out = Vec::new();

    while let Some(entry) = entries.next_entry().await.map_err(|e| format!("read_dir entry error {}: {}", p.display(), e))? {
        out.push(entry.path());
    }

    Ok(out)
}

pub async fn read(path: impl AsRef<Path>) -> FsResult<Vec<u8>> {
    let p = resolve(path)?;

    async_fs::read(&p).await.map_err(|e| format!("read failed {}: {}", p.display(), e))
}

pub async fn read_str(path: impl AsRef<Path>) -> FsResult<String> {
    let p = resolve(path)?;

    async_fs::read_to_string(&p).await.map_err(|e| format!("read_str failed {}: {}", p.display(), e))
}

async fn ensure_parent(path: &Path) -> FsResult<PathBuf> {
    let parent = path.parent().ok_or_else(|| format!("no parent: {}", path.display()))?;

    async_fs::create_dir_all(parent).await.map_err(|e| format!("mkdir failed {}: {}", parent.display(), e))?;

    Ok(parent.to_path_buf())
}

async fn write_atomic(path: &Path, data: &[u8]) -> FsResult<PathBuf> {
    ensure_parent(path).await?;

    let mut tmp = path.to_path_buf();
    tmp.set_file_name(format!(".{}.tmp", uuid::Uuid::new_v4()));

    async_fs::write(&tmp, data).await.map_err(|e| format!("tmp write failed {}: {}", tmp.display(), e))?;

    async_fs::rename(&tmp, path).await.map_err(|e| format!("rename failed {}: {}", path.display(), e))?;

    Ok(path.to_path_buf())
}

async fn write_staged(path: &Path, data: &[u8]) -> FsResult<PathBuf> {
    let parent = ensure_parent(path).await?;

    let tmp_dir = parent.join(".kable_tmp");

    async_fs::create_dir_all(&tmp_dir).await.map_err(|e| format!("tmp dir failed {}: {}", tmp_dir.display(), e))?;

    let tmp = tmp_dir.join(format!("{}.tmp", uuid::Uuid::new_v4()));

    async_fs::write(&tmp, data).await.map_err(|e| format!("tmp write failed {}: {}", tmp.display(), e))?;

    async_fs::copy(&tmp, path).await.map_err(|e| format!("copy failed {}: {}", path.display(), e))?;

    let _ = async_fs::remove_file(tmp).await;

    Ok(path.to_path_buf())
}

pub async fn write(path: impl AsRef<Path>, data: &[u8], staged: bool) -> FsResult<PathBuf> {
    let p = resolve(path)?;

    if staged {
        write_staged(&p, data).await
    } else {
        write_atomic(&p, data).await
    }
}

pub async fn write_str(path: impl AsRef<Path>, data: &str, staged: bool) -> FsResult<PathBuf> {
    write(path, data.as_bytes(), staged).await
}

pub async fn create_file(path: impl AsRef<Path>) -> FsResult<PathBuf> {
    let p = resolve(path)?;
    ensure_parent(&p).await?;

    async_fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&p)
        .await
        .map_err(|e| format!("create file failed {}: {}", p.display(), e))?;

    Ok(p)
}

pub async fn remove_file(path: impl AsRef<Path>) -> FsResult<()> {
    let p = resolve(path)?;

    async_fs::remove_file(&p).await.map_err(|e| format!("remove_file failed {}: {}", p.display(), e))?;

    Ok(())
}

pub async fn remove_dir(path: impl AsRef<Path>) -> FsResult<()> {
    let p = resolve(path)?;

    async_fs::remove_dir(&p).await.map_err(|e| format!("remove_dir failed {}: {}", p.display(), e))?;

    Ok(())
}

pub async fn remove_dir_all(path: impl AsRef<Path>) -> FsResult<()> {
    let p = resolve(path)?;

    async_fs::remove_dir_all(&p).await.map_err(|e| format!("remove_dir_all failed {}: {}", p.display(), e))?;

    Ok(())
}

pub async fn copy(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> FsResult<u64> {
    let s = resolve(src)?;
    let d = resolve(dst)?;

    async_fs::copy(&s, &d).await.map_err(|e| format!("copy {} -> {} failed: {}", s.display(), d.display(), e))
}

pub async fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> FsResult<()> {
    let f = resolve(from)?;
    let t = resolve(to)?;

    async_fs::rename(&f, &t).await.map_err(|e| format!("rename failed {} -> {}: {}", f.display(), t.display(), e))?;

    Ok(())
}

pub fn tmp_dir(instance: &str, pack: &str) -> FsResult<PathBuf> {
    Ok(kable_dir()?.join("tmp").join(instance).join(pack))
}

pub async fn open_path(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Use start with empty title to allow paths with spaces
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| format!("Failed to open path on Windows: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&path).spawn().map_err(|e| format!("Failed to open path on macOS: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(&path).spawn().map_err(|e| format!("Failed to open path on Linux: {}", e))?;
    }

    Ok(())
}

pub async fn open_dir(dir: PathBuf) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer").arg(&dir).spawn().map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&dir).spawn().map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(&dir).spawn().map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    Ok(())
}
