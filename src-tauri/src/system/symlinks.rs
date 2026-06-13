use std::path::Path;
use tokio::fs as async_fs;
use crate::system::fs::{ensure_parent_dir_exists_async, write_file_atomic_async};

/// Ensure Minecraft allows symbolic links by writing to allowed_symlinks.txt
pub async fn ensure_symlinks_enabled(minecraft_path: &Path) -> Result<(), String> {
    let allowed_symlinks_file = minecraft_path.join("allowed_symlinks.txt");
    let required_line = "[regex].*";

    // Check if file exists and contains the required line
    if allowed_symlinks_file.exists() {
        let content = async_fs::read_to_string(&allowed_symlinks_file)
            .await
            .map_err(|e| format!("Failed to read allowed_symlinks.txt: {}", e))?;

        if content.lines().any(|line| line.trim() == required_line) {
            return Ok(());
        }

        // File exists but doesn't have the line, append it
        let mut new_content = content;
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str(required_line);
        new_content.push('\n');

        write_file_atomic_async(&allowed_symlinks_file, new_content.as_bytes()).await?;
    } else {
        // File doesn't exist, create it with the required line
        ensure_parent_dir_exists_async(&allowed_symlinks_file).await?;
        let content = format!("{}\n", required_line);
        write_file_atomic_async(&allowed_symlinks_file, content.as_bytes()).await?;
    }

    Ok(())
}

/// Create a symbolic link from source to target directory
pub async fn create_directory_symlink(source: &Path, target: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_dir;
        if target.exists() {
            if target.is_symlink() {
                async_fs::remove_dir(target).await.map_err(|e| format!("Failed to remove existing symlink: {}", e))?;
            } else {
                return Err(format!("Target path exists and is not a symlink: {}", target.display()));
            }
        }
        ensure_parent_dir_exists_async(target).await?;
        let s = source.to_path_buf();
        let t = target.to_path_buf();
        tokio::task::spawn_blocking(move || symlink_dir(s, t).map_err(|e| format!("Failed to create symlink: {}", e)))
            .await
            .map_err(|e| format!("Symlink task failed: {}", e))??;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        if target.exists() {
            if target.is_symlink() {
                async_fs::remove_file(target).await.map_err(|e| format!("Failed to remove existing symlink: {}", e))?;
            } else {
                return Err(format!("Target path exists and is not a symlink: {}", target.display()));
            }
        }
        ensure_parent_dir_exists_async(target).await?;
        let s = source.to_path_buf();
        let t = target.to_path_buf();
        tokio::task::spawn_blocking(move || symlink(s, t).map_err(|e| format!("Failed to create symlink: {}", e)))
            .await
            .map_err(|e| format!("Symlink task failed: {}", e))??;
    }

    Ok(())
}

pub async fn create_file_symlink(source: &Path, target: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_file;
        if target.exists() {
            if target.is_symlink() {
                async_fs::remove_file(target).await.map_err(|e| format!("Failed to remove existing symlink: {}", e))?;
            } else {
                return Err(format!("Target path exists and is not a symlink: {}", target.display()));
            }
        }
        ensure_parent_dir_exists_async(target).await?;
        let s = source.to_path_buf();
        let t = target.to_path_buf();
        tokio::task::spawn_blocking(move || symlink_file(s, t).map_err(|e| format!("Failed to create symlink: {}", e)))
            .await
            .map_err(|e| format!("Symlink task failed: {}", e))??;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        if target.exists() {
            if target.is_symlink() {
                async_fs::remove_file(target).await.map_err(|e| format!("Failed to remove existing symlink: {}", e))?;
            } else {
                return Err(format!("Target path exists and is not a symlink: {}", target.display()));
            }
        }
        ensure_parent_dir_exists_async(target).await?;
        let s = source.to_path_buf();
        let t = target.to_path_buf();
        tokio::task::spawn_blocking(move || symlink(s, t).map_err(|e| format!("Failed to create symlink: {}", e)))
            .await
            .map_err(|e| format!("Symlink task failed: {}", e))??;
    }

    Ok(())
}

pub async fn remove_symlink_if_exists(path: &Path) -> Result<(), String> {
    if path.exists() && path.is_symlink() {
        if path.is_dir() {
            async_fs::remove_dir(path).await.map_err(|e| format!("Failed to remove dir symlink: {}", e))?;
        } else {
            async_fs::remove_file(path).await.map_err(|e| format!("Failed to remove file symlink: {}", e))?;
        }
    }
    Ok(())
}