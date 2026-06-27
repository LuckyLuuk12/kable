use std::path::{Path, PathBuf};

use api_types::symlinks::Symlink;

/// Create symlink (file or directory)
pub async fn create(link: &Symlink) -> Result<(), String> {
    if crate::system::fs::exists(&link.destination).await? {
        return Err("Destination already exists".to_string());
    }

    // ensure parent exists via fs layer
    crate::system::fs::create_dir(link.destination.parent().ok_or("Invalid destination")?).await?;

    if link.source.is_dir() {
        crate::create_directory_symlink(&link.source, &link.destination).await?;
    } else {
        crate::create_file_symlink(&link.source, &link.destination).await?;
    }

    Ok(())
}

/// Remove symlink ONLY at destination
pub async fn remove(link: &Symlink) -> Result<(), String> {
    if !crate::system::fs::exists(&link.destination).await? {
        return Ok(());
    }

    if link.destination.is_symlink() {
        crate::remove_symlink_if_exists(&link.destination).await?;
    }

    Ok(())
}

/// Check if symlink exists at destination
pub async fn exists(link: &Symlink) -> Result<bool, String> {
    Ok(link.destination.exists() && link.destination.is_symlink())
}

/// Read symlink target (system-level introspection)
pub fn read_target(path: &Path) -> Result<PathBuf, String> {
    std::fs::read_link(path).map_err(|e| format!("read_link failed: {}", e))
}

/// Validate basic correctness
pub fn validate(link: &Symlink) -> Result<(), String> {
    if !link.source.exists() {
        return Err("Source does not exist".to_string());
    }

    if link.destination.parent().is_none() {
        return Err("Destination has no parent".to_string());
    }

    Ok(())
}
