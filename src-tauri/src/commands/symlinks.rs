use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkInfo {
    pub id: Option<String>, // Custom symlink ID if it's a stored custom symlink
    pub source: String,
    pub destination: String,
    pub is_global: bool,
    pub installation_id: Option<String>,
    pub symlink_type: String, // "resourcepack" | "shader" | "custom"
    pub is_disabled: bool,
    pub exists: bool,
}

/// List all symlinks found in the .minecraft directory recursively
#[tauri::command]
pub async fn list_symlinks() -> Result<Vec<SymlinkInfo>, String> {
    let minecraft_dir = crate::get_default_minecraft_dir()?;
    let mut symlinks = Vec::new();

    // Load custom symlinks configuration
    let custom_config = crate::custom_symlinks::read_custom_symlinks().await?;

    // Recursively scan the entire .minecraft directory for symlinks
    scan_directory_for_symlinks(&minecraft_dir, &minecraft_dir, &custom_config, &mut symlinks)?;

    Ok(symlinks)
}

/// Recursively scan a directory for symlinks
fn scan_directory_for_symlinks(
    dir: &Path,
    minecraft_root: &Path,
    custom_config: &crate::custom_symlinks::CustomSymlinksConfig,
    symlinks: &mut Vec<SymlinkInfo>,
) -> Result<(), String> {
    if !dir.exists() || !dir.is_dir() {
        return Ok(());
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()), // Skip directories we can't read
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // Check if it's a symlink
        if path.is_symlink() {
            // Determine if it's disabled based on file extension
            let path_str = path.to_string_lossy().to_string();
            let is_disabled = path_str.ends_with(".disabled");
            
            if let Ok(target) = std::fs::read_link(&path) {
                let symlink_type = determine_symlink_type(&path, minecraft_root);
                let installation_id = extract_installation_from_path(&target);
                
                // Try to find matching custom symlink config
                let custom_id = find_custom_symlink_id(&path, &target, custom_config);
                
                symlinks.push(SymlinkInfo {
                    id: custom_id,
                    source: target.to_string_lossy().to_string(),
                    destination: path_str,
                    is_global: installation_id.is_none(),
                    installation_id,
                    symlink_type,
                    is_disabled,
                    exists: target.exists(),
                });
            }
        } else if path.is_dir() {
            // Recursively scan subdirectories (skip common large/system directories)
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            
            // Skip common directories that shouldn't contain user symlinks
            if !should_skip_directory(dir_name) {
                scan_directory_for_symlinks(&path, minecraft_root, custom_config, symlinks)?;
            }
        }
    }

    Ok(())
}

/// Determine if a directory should be skipped during scanning
fn should_skip_directory(dir_name: &str) -> bool {
    matches!(
        dir_name,
        "logs" | "crash-reports" | "versions" | "libraries" | "assets" | "natives" | ".fabric" | ".mixin.out"
    )
}

/// Find the custom symlink ID that matches this symlink path and target
fn find_custom_symlink_id(
    dest_path: &Path,
    target: &Path,
    custom_config: &crate::custom_symlinks::CustomSymlinksConfig,
) -> Option<String> {
    // Get the parent folder of the destination
    let dest_parent = dest_path.parent()?;
    
    for custom in &custom_config.symlinks {
        let custom_source = PathBuf::from(&custom.source);
        let custom_dest_parent = PathBuf::from(&custom.destination_parent);
        
        // Check if source and destination parent match
        if custom_source == target && custom_dest_parent == dest_parent {
            return Some(custom.id.clone());
        }
    }
    
    None
}

/// Determine the type of symlink based on its location
fn determine_symlink_type(path: &Path, minecraft_root: &Path) -> String {
    let relative = path.strip_prefix(minecraft_root).unwrap_or(path);
    let path_str = relative.to_string_lossy();
    
    if path_str.starts_with("resourcepacks") || path_str.contains("resourcepacks") {
        "resourcepack".to_string()
    } else if path_str.starts_with("shaderpacks") || path_str.contains("shaderpacks") {
        "shader".to_string()
    } else if path_str.starts_with("saves") || path_str.contains("saves") {
        "world".to_string()
    } else if path_str.starts_with("mods") || path_str.contains("mods") {
        "mod".to_string()
    } else {
        "custom".to_string()
    }
}

/// Extract installation ID from a path if it's in kable managed folders
fn extract_installation_from_path(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();
    
    // Check if path contains kable/resourcepacks/ or kable/shaderpacks/
    if path_str.contains("kable/resourcepacks/") || path_str.contains("kable\\resourcepacks\\") {
        // Extract the installation ID (folder name after resourcepacks/)
        if let Some(pos) = path_str.find("resourcepacks/").or_else(|| path_str.find("resourcepacks\\")) {
            let after = &path_str[pos + 14..]; // "resourcepacks/".len() = 14
            if let Some(slash_pos) = after.find('/').or_else(|| after.find('\\')) {
                return Some(after[..slash_pos].to_string());
            }
        }
    }
    
    if path_str.contains("kable/shaderpacks/") || path_str.contains("kable\\shaderpacks\\") {
        if let Some(pos) = path_str.find("shaderpacks/").or_else(|| path_str.find("shaderpacks\\")) {
            let after = &path_str[pos + 12..]; // "shaderpacks/".len() = 12
            if let Some(slash_pos) = after.find('/').or_else(|| after.find('\\')) {
                return Some(after[..slash_pos].to_string());
            }
        }
    }
    
    None
}

/// Create a custom symlink
/// destination should be the parent folder where the symlink will be created
/// The symlink will have the same name as the source file/folder
/// installation_id is optional - None means global (always active)
#[tauri::command]
pub async fn create_custom_symlink(
    source: String,
    destination_parent: String,
    installation_id: Option<String>,
) -> Result<String, String> {
    let source_path = PathBuf::from(&source);
    let destination_parent_path = PathBuf::from(&destination_parent);

    if !source_path.exists() {
        return Err(format!("Source path does not exist: {}", source));
    }

    if !destination_parent_path.exists() {
        return Err(format!("Destination parent folder does not exist: {}", destination_parent));
    }

    if !destination_parent_path.is_dir() {
        return Err(format!("Destination parent must be a folder: {}", destination_parent));
    }

    // Get the source file/folder name and create the symlink path
    let source_name = source_path
        .file_name()
        .ok_or_else(|| format!("Invalid source path: {}", source))?;
    
    let dest_path = destination_parent_path.join(source_name);

    if dest_path.exists() {
        return Err(format!("Symlink already exists: {}", dest_path.to_string_lossy()));
    }

    // Add to custom symlinks configuration
    let custom_symlink = crate::custom_symlinks::add_custom_symlink(
        source.clone(),
        destination_parent.clone(),
        installation_id.clone(),
    ).await?;

    // Create the actual symlink on disk
    if source_path.is_dir() {
        crate::create_directory_symlink(&source_path, &dest_path).await?;
    } else {
        crate::create_file_symlink(&source_path, &dest_path).await?;
    }

    Ok(custom_symlink.id)
}

/// Remove a symlink by its destination path or ID
#[tauri::command]
pub async fn remove_symlink(destination: String, id: Option<String>) -> Result<(), String> {
    let dest_path = PathBuf::from(&destination);

    if !dest_path.exists() {
        return Err(format!("Symlink does not exist: {}", destination));
    }

    if !dest_path.is_symlink() {
        return Err(format!("Path is not a symlink: {}", destination));
    }

    // Remove from disk
    crate::remove_symlink_if_exists(&dest_path).await?;

    // Remove from custom symlinks config if it has an ID
    if let Some(symlink_id) = id {
        crate::custom_symlinks::remove_custom_symlink(&symlink_id).await?;
    }

    Ok(())
}

/// Toggle symlink disabled state (rename to .disabled)
#[tauri::command]
pub async fn toggle_symlink_disabled(destination: String, id: Option<String>) -> Result<bool, String> {
    let dest_path = PathBuf::from(&destination);

    if !dest_path.exists() && !dest_path.with_extension("disabled").exists() {
        return Err(format!("Symlink does not exist: {}", destination));
    }

    let is_currently_disabled = destination.ends_with(".disabled");
    
    if is_currently_disabled {
        // Enable: rename from .disabled to original
        let enabled_path = PathBuf::from(destination.trim_end_matches(".disabled"));
        tokio::fs::rename(&dest_path, &enabled_path)
            .await
            .map_err(|e| format!("Failed to enable symlink: {}", e))?;
        
        // Update config if custom symlink
        if let Some(symlink_id) = id {
            crate::custom_symlinks::update_custom_symlink(&symlink_id, None, None, None, Some(true)).await?;
        }
        
        Ok(false) // Now enabled
    } else {
        // Disable: rename to .disabled
        let disabled_path = dest_path.with_extension("disabled");
        tokio::fs::rename(&dest_path, &disabled_path)
            .await
            .map_err(|e| format!("Failed to disable symlink: {}", e))?;
        
        // Update config if custom symlink
        if let Some(symlink_id) = id {
            crate::custom_symlinks::update_custom_symlink(&symlink_id, None, None, None, Some(false)).await?;
        }
        
        Ok(true) // Now disabled
    }
}

/// Update an existing symlink with new source/destination paths
/// old_destination is the full symlink path
/// new_destination_parent is the parent folder where the symlink will be created
/// The symlink will have the same name as the new source
/// installation_id can be updated for custom symlinks
#[tauri::command]
pub async fn update_symlink(
    id: Option<String>,
    old_destination: String,
    new_source: String,
    new_destination_parent: String,
    new_installation_id: Option<Option<String>>,
) -> Result<(), String> {
    let old_dest_path = PathBuf::from(&old_destination);
    let new_source_path = PathBuf::from(&new_source);
    let new_dest_parent_path = PathBuf::from(&new_destination_parent);

    // Validate new source exists
    if !new_source_path.exists() {
        return Err(format!("New source path does not exist: {}", new_source));
    }

    // Validate destination parent exists and is a directory
    if !new_dest_parent_path.exists() {
        return Err(format!("Destination parent folder does not exist: {}", new_destination_parent));
    }

    if !new_dest_parent_path.is_dir() {
        return Err(format!("Destination parent must be a folder: {}", new_destination_parent));
    }

    // Get the source name and construct the full destination path
    let source_name = new_source_path
        .file_name()
        .ok_or_else(|| format!("Invalid source path: {}", new_source))?;
    
    let new_dest_path = new_dest_parent_path.join(source_name);

    // Remove old symlink if it exists
    if old_dest_path.exists() && old_dest_path.is_symlink() {
        crate::remove_symlink_if_exists(&old_dest_path).await?;
    }

    // Check if new destination already exists (unless it's the same as old)
    if new_dest_path.exists() && old_dest_path != new_dest_path {
        return Err(format!("Symlink already exists: {}", new_dest_path.to_string_lossy()));
    }

    // Update custom symlinks config if it has an ID
    if let Some(symlink_id) = &id {
        crate::custom_symlinks::update_custom_symlink(
            symlink_id,
            Some(new_source.clone()),
            Some(new_destination_parent.clone()),
            new_installation_id,
            None,
        ).await?;
    }

    // Create new symlink
    if new_source_path.is_dir() {
        crate::create_directory_symlink(&new_source_path, &new_dest_path).await?;
    } else {
        crate::create_file_symlink(&new_source_path, &new_dest_path).await?;
    }

    Ok(())
}

/// Select a file using the system file dialog
#[tauri::command]
pub async fn select_file_for_symlink(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let file_path = app
        .dialog()
        .file()
        .set_title("Select File")
        .blocking_pick_file();

    match file_path {
        Some(path) => match path.as_path() {
            Some(path_buf) => Ok(Some(path_buf.to_string_lossy().to_string())),
            None => Err("Invalid file path".to_string()),
        },
        None => Ok(None), // User cancelled
    }
}

/// Select a folder using the system file dialog
#[tauri::command]
pub async fn select_folder_for_symlink(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let folder_path = app
        .dialog()
        .file()
        .set_title("Select Folder")
        .blocking_pick_folder();

    match folder_path {
        Some(path) => match path.as_path() {
            Some(path_buf) => Ok(Some(path_buf.to_string_lossy().to_string())),
            None => Err("Invalid folder path".to_string()),
        },
        None => Ok(None), // User cancelled
    }
}
