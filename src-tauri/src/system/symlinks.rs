use crate::system::fs;
use std::path::Path;
use tokio::fs as async_fs;

type FsResult<T> = Result<T, String>;

// ============================================================
// INTERNAL HELPERS
// ============================================================

async fn is_symlink(path: &Path) -> FsResult<bool> {
    match async_fs::symlink_metadata(path).await {
        Ok(md) => Ok(md.file_type().is_symlink()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("symlink_metadata failed {}: {}", path.display(), e)),
    }
}

async fn remove_path(path: &Path) -> FsResult<()> {
    if is_symlink(path).await? {
        if path.is_dir() {
            async_fs::remove_dir(path).await.map_err(|e| format!("remove_dir failed {}: {}", path.display(), e))?;
        } else {
            async_fs::remove_file(path).await.map_err(|e| format!("remove_file failed {}: {}", path.display(), e))?;
        }
    }
    Ok(())
}

// ============================================================
// ALLOWED SYMLINKS FILE
// ============================================================

pub async fn ensure_symlinks_enabled(minecraft_path: &Path) -> FsResult<()> {
    let file = minecraft_path.join("allowed_symlinks.txt");
    let required = "[regex].*";

    let content = match async_fs::read_to_string(&file).await {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("read failed {}: {}", file.display(), e)),
    };

    let mut lines: Vec<&str> = content.lines().collect();

    if lines.iter().any(|l| l.trim() == required) {
        return Ok(());
    }

    lines.push(required);

    let new_content = lines.join("\n") + "\n";

    fs::write(&file, new_content.as_bytes(), false).await?;

    Ok(())
}

// ============================================================
// DIRECTORY SYMLINK
// ============================================================

#[cfg(windows)]
async fn create_dir_link(source: &Path, target: &Path) -> FsResult<()> {
    use std::os::windows::fs::symlink_dir;

    remove_path(target).await?;

    let s = source.to_path_buf();
    let t = target.to_path_buf();

    tokio::task::spawn_blocking(move || symlink_dir(s, t).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("join error: {}", e))??;

    Ok(())
}

#[cfg(unix)]
async fn create_dir_link(source: &Path, target: &Path) -> FsResult<()> {
    use std::os::unix::fs::symlink;

    remove_path(target).await?;

    let s = source.to_path_buf();
    let t = target.to_path_buf();

    tokio::task::spawn_blocking(move || symlink(s, t).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("join error: {}", e))??;

    Ok(())
}

// ============================================================
// FILE SYMLINK
// ============================================================

#[cfg(windows)]
async fn create_file_link(source: &Path, target: &Path) -> FsResult<()> {
    use std::os::windows::fs::symlink_file;

    remove_path(target).await?;

    let s = source.to_path_buf();
    let t = target.to_path_buf();

    tokio::task::spawn_blocking(move || symlink_file(s, t).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("join error: {}", e))??;

    Ok(())
}

#[cfg(unix)]
async fn create_file_link(source: &Path, target: &Path) -> FsResult<()> {
    use std::os::unix::fs::symlink;

    remove_path(target).await?;

    let s = source.to_path_buf();
    let t = target.to_path_buf();

    tokio::task::spawn_blocking(move || symlink(s, t).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("join error: {}", e))??;

    Ok(())
}

// ============================================================
// PUBLIC API
// ============================================================

pub async fn link_dir(source: impl AsRef<Path>, target: impl AsRef<Path>) -> FsResult<()> {
    let s = source.as_ref();
    let t = target.as_ref();

    if is_symlink(t).await? {
        remove_path(t).await?;
    }

    if t.exists() && !is_symlink(t).await? {
        return Err(format!("target exists and is not a symlink: {}", t.display()));
    }

    create_dir_link(s, t).await
}

pub async fn link_file(source: impl AsRef<Path>, target: impl AsRef<Path>) -> FsResult<()> {
    let s = source.as_ref();
    let t = target.as_ref();

    if t.exists() && !is_symlink(t).await? {
        return Err(format!("target exists and is not a symlink: {}", t.display()));
    }

    create_file_link(s, t).await
}

// ============================================================
// REMOVE SYMLINK
// ============================================================

pub async fn unlink(path: impl AsRef<Path>) -> FsResult<()> {
    let p = path.as_ref();

    if is_symlink(p).await? {
        remove_path(p).await?;
    }

    Ok(())
}
