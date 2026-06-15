use crate::constants::{KABLE_DIR_NAME, LAUNCHER_DIR};
use crate::logging::Logger;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;

/// Get the default Minecraft directory
pub fn get_default_minecraft_dir() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Could not find home directory".to_string())?;

    #[cfg(target_os = "windows")]
    let minecraft_dir = home_dir.join("AppData").join("Roaming").join(".minecraft");

    #[cfg(target_os = "macos")]
    let minecraft_dir = home_dir.join("Library").join("Application Support").join("minecraft");

    #[cfg(target_os = "linux")]
    let minecraft_dir = home_dir.join(".minecraft");

    Ok(minecraft_dir)
}

/// Gets the kable dir inside the .minecraft folder
/// Automatically migrates from old 'kable' to new '.kable' folder for existing users
pub fn get_minecraft_kable_dir() -> Result<PathBuf, String> {
    let default_dir = get_default_minecraft_dir()?;
    let new_kable_dir = default_dir.join(KABLE_DIR_NAME);
    let old_kable_dir = default_dir.join("kable");

    // Migration logic: if old folder exists and new doesn't, rename it
    if old_kable_dir.exists() && !new_kable_dir.exists() {
        fs::rename(&old_kable_dir, &new_kable_dir).map_err(|e| format!("Failed to migrate kable folder to {}: {}", KABLE_DIR_NAME, e))?;
        Logger::info_global(&format!("Migrated kable folder to {} at {}", KABLE_DIR_NAME, new_kable_dir.display()), None);
    }

    // Ensure .kable directory exists
    if !new_kable_dir.exists() {
        fs::create_dir_all(&new_kable_dir).map_err(|e| e.to_string())?;
    }

    Ok(new_kable_dir)
}

/// Gets the kable-launcher folder, on windows this is inside Roaming/kable-launcher
pub fn get_kable_launcher_dir() -> Result<PathBuf, String> {
    let kable_dir = get_minecraft_kable_dir()?;
    let launcher_dir = kable_dir.join(LAUNCHER_DIR);
    if !launcher_dir.exists() {
        fs::create_dir_all(&launcher_dir).map_err(|e| e.to_string())?;
    }
    Ok(launcher_dir)
}

/// Ensure the parent directory for `path` exists (async). No-op if the
/// path has no parent. Returns Err when directory creation fails.
pub async fn ensure_parent_dir_exists_async(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        async_fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("failed to create parent directories {}: {}", parent.display(), e))?;
    }
    Ok(())
}

/// Ensure a file exists at `path`. Creates parent directories and the file
/// if necessary. Returns the provided PathBuf on success.
pub async fn ensure_file(path: PathBuf) -> Result<PathBuf, String> {
    ensure_parent_dir_exists_async(&path).await?;
    // OpenOptions::new().create(true) will create the file if it does not exist
    async_fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .map_err(|e| format!("failed to create or open file {}: {}", path.display(), e))?;
    Ok(path)
}

/// Ensure a file exists at `path` and, if the file does not already exist,
/// write `contents` to it. Returns the provided PathBuf on success.
pub async fn ensure_file_with(path: PathBuf, contents: &str) -> Result<PathBuf, String> {
    match async_fs::metadata(&path).await {
        Ok(md) => {
            if md.is_file() {
                return Ok(path);
            }
            // If it exists but is not a file, return an error.
            Err(format!("path exists but is not a file: {}", path.display()))
        }
        Err(e) => {
            // If error is NotFound, create parent dirs and write the file.
            if e.kind() == std::io::ErrorKind::NotFound {
                ensure_parent_dir_exists_async(&path).await?;
                async_fs::write(&path, contents.as_bytes())
                    .await
                    .map_err(|e| format!("failed to write file {}: {}", path.display(), e))?;
                return Ok(path);
            }
            Err(format!("failed to stat path {}: {}", path.display(), e))
        }
    }
}

/// Ensure a directory exists at `path`. If it exists and is a file, return an error.
/// Creates parent directories as necessary. Returns Ok(()) on success.
pub async fn ensure_folder(path: &Path) -> Result<PathBuf, String> {
    match async_fs::metadata(path).await {
        Ok(md) => {
            if md.is_dir() {
                Ok(path.to_path_buf())
            } else {
                Err(format!("path exists but is not a directory: {}", path.display()))
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                async_fs::create_dir_all(path).await.map_err(|e| format!("failed to create directory {}: {}", path.display(), e))?;
                Ok(path.to_path_buf())
            } else {
                Err(format!("failed to stat path {}: {}", path.display(), e))
            }
        }
    }
}

/// Returns the standardized temp directory for modpack operations.
/// Example: .kable/tmp/<instance_id>/<modpack_id>/
pub fn get_temp_dir(instance_id: &str, modpack_id: &str) -> Result<PathBuf, String> {
    let kable_dir = get_minecraft_kable_dir()?;
    Ok(kable_dir.join("tmp").join(instance_id).join(modpack_id))
}

/// Synchronous variant of ensure_folder for use in blocking contexts.
/// TODO: remove or make it call async version
pub fn ensure_folder_sync(path: &Path) -> Result<PathBuf, String> {
    match std::fs::metadata(path) {
        Ok(md) => {
            if md.is_dir() {
                Ok(path.to_path_buf())
            } else {
                Err(format!("path exists but is not a directory: {}", path.display()))
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                std::fs::create_dir_all(path).map_err(|e| format!("failed to create directory {}: {}", path.display(), e))?;
                Ok(path.to_path_buf())
            } else {
                Err(format!("failed to stat path {}: {}", path.display(), e))
            }
        }
    }
}

/// Recursively copy a directory and all its contents (async version)
pub async fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    async_fs::create_dir_all(dst).await.map_err(|e| format!("Failed to create directory {}: {}", dst.display(), e))?;

    let mut entries = async_fs::read_dir(src).await.map_err(|e| format!("Failed to read directory {}: {}", src.display(), e))?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| format!("Failed to read entry: {}", e))? {
        let src_path = entry.path();
        let file_name = src_path.file_name().ok_or_else(|| "Invalid file name".to_string())?;
        let dst_path = dst.join(file_name);

        let metadata = async_fs::metadata(&src_path).await.map_err(|e| format!("Failed to get metadata: {}", e))?;

        if metadata.is_dir() {
            Box::pin(copy_dir_recursive(&src_path, &dst_path)).await?;
        } else {
            async_fs::copy(&src_path, &dst_path)
                .await
                .map_err(|e| format!("Failed to copy file from {} to {}: {}", src_path.display(), dst_path.display(), e))?;
        }
    }

    Ok(())
}

/// Synchronous variant of copy_dir_recursive for use in blocking contexts
/// TODO: remove or make it call async version
pub fn copy_dir_recursive_sync(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("Failed to create directory {}: {}", dst.display(), e))?;

    let entries = std::fs::read_dir(src).map_err(|e| format!("Failed to read directory {}: {}", src.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let ty = entry.file_type().map_err(|e| format!("Failed to get file type: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive_sync(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy file from {} to {}: {}", src_path.display(), dst_path.display(), e))?;
        }
    }

    Ok(())
}

/// Atomically write bytes to `path` by creating a temporary file in the same
/// directory and renaming it into place. This avoids partial file writes.
pub async fn write_file_atomic_async(path: &Path, bytes: &[u8]) -> Result<(), String> {
    // Ensure parent exists
    ensure_parent_dir_exists_async(path).await?;

    // Work with owned PathBufs so we can move them into the blocking task
    let path_buf = path.to_path_buf();
    let parent = path_buf.parent().ok_or_else(|| format!("Path has no parent: {}", path_buf.display()))?.to_path_buf();

    // Create a temp filename in the same directory
    let mut tmp = parent.clone();
    let tmp_name = format!(".{}.tmp", uuid::Uuid::new_v4());
    tmp.push(tmp_name);

    // Write to temp file asynchronously
    async_fs::write(&tmp, bytes).await.map_err(|e| format!("failed to write temp file {}: {}", tmp.display(), e))?;

    // Move owned PathBufs into the blocking task and rename
    let tmp_move = tmp.clone();
    let final_move = path_buf.clone();
    tokio::task::spawn_blocking(move || std::fs::rename(&tmp_move, &final_move))
        .await
        .map_err(|e| format!("rename join error: {}", e))?
        .map_err(|e| format!("failed to atomically rename into place: {}", e))?;

    Ok(())
}

pub async fn write_file(path: &PathBuf, text: &String) -> Result<(), String> {
    write_file_atomic_async(path, text.as_bytes()).await
}

/// Synchronous variant of atomic write: writes bytes to a temp file in the same
/// directory and renames it into place.
/// TODO: remove or make it call async version
pub fn write_file_atomic_sync(path: &Path, bytes: &[u8]) -> Result<(), String> {
    // Ensure parent exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("failed to create parent dirs {}: {}", parent.display(), e))?;
    }

    let parent = path.parent().ok_or_else(|| format!("Path has no parent: {}", path.display()))?;
    let mut tmp = parent.to_path_buf();
    let tmp_name = format!(".{}.tmp", uuid::Uuid::new_v4());
    tmp.push(tmp_name);

    std::fs::write(&tmp, bytes).map_err(|e| format!("failed to write temp file {}: {}", tmp.display(), e))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("failed to rename temp file into place: {}", e))?;
    Ok(())
}

pub async fn read_to_string(path: &Path) -> Result<String, String> {
    async_fs::read_to_string(path).await.map_err(|e| format!("Failed to read file {}: {}", path.display(), e))
}
