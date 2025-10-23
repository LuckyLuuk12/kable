use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs as async_fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSymlink {
    pub id: String,
    pub source: String,
    pub destination_parent: String,
    pub installation_id: Option<String>, // None = global (always active)
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSymlinksConfig {
    pub symlinks: Vec<CustomSymlink>,
}

impl CustomSymlinksConfig {
    pub fn new() -> Self {
        Self {
            symlinks: Vec::new(),
        }
    }
}

impl Default for CustomSymlinksConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the path to the custom symlinks configuration file
pub fn get_custom_symlinks_config_path() -> Result<PathBuf, String> {
    let kable_dir = crate::get_minecraft_kable_dir()?;
    Ok(kable_dir.join("custom_symlinks.json"))
}

/// Read the custom symlinks configuration from disk
pub async fn read_custom_symlinks() -> Result<CustomSymlinksConfig, String> {
    let config_path = get_custom_symlinks_config_path()?;
    
    if !config_path.exists() {
        return Ok(CustomSymlinksConfig::new());
    }
    
    let content = async_fs::read_to_string(&config_path)
        .await
        .map_err(|e| format!("Failed to read custom symlinks config: {}", e))?;
    
    let config: CustomSymlinksConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse custom symlinks config: {}", e))?;
    
    Ok(config)
}

/// Write the custom symlinks configuration to disk
pub async fn write_custom_symlinks(config: &CustomSymlinksConfig) -> Result<(), String> {
    let config_path = get_custom_symlinks_config_path()?;
    
    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        crate::ensure_folder(parent).await?;
    }
    
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize custom symlinks config: {}", e))?;
    
    async_fs::write(&config_path, content)
        .await
        .map_err(|e| format!("Failed to write custom symlinks config: {}", e))?;
    
    Ok(())
}

/// Add a new custom symlink to the configuration
pub async fn add_custom_symlink(
    source: String,
    destination_parent: String,
    installation_id: Option<String>,
) -> Result<CustomSymlink, String> {
    let mut config = read_custom_symlinks().await?;
    
    // Generate a unique ID
    let id = uuid::Uuid::new_v4().to_string();
    
    let symlink = CustomSymlink {
        id: id.clone(),
        source: source.clone(),
        destination_parent: destination_parent.clone(),
        installation_id: installation_id.clone(),
        enabled: true,
    };
    
    config.symlinks.push(symlink.clone());
    write_custom_symlinks(&config).await?;
    
    Ok(symlink)
}

/// Remove a custom symlink from the configuration by ID
pub async fn remove_custom_symlink(id: &str) -> Result<(), String> {
    let mut config = read_custom_symlinks().await?;
    
    config.symlinks.retain(|s| s.id != id);
    
    write_custom_symlinks(&config).await?;
    
    Ok(())
}

/// Update a custom symlink in the configuration
pub async fn update_custom_symlink(
    id: &str,
    source: Option<String>,
    destination_parent: Option<String>,
    installation_id: Option<Option<String>>,
    enabled: Option<bool>,
) -> Result<(), String> {
    let mut config = read_custom_symlinks().await?;
    
    let symlink = config
        .symlinks
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| format!("Custom symlink not found: {}", id))?;
    
    if let Some(src) = source {
        symlink.source = src;
    }
    
    if let Some(dest) = destination_parent {
        symlink.destination_parent = dest;
    }
    
    if let Some(inst_id) = installation_id {
        symlink.installation_id = inst_id;
    }
    
    if let Some(en) = enabled {
        symlink.enabled = en;
    }
    
    write_custom_symlinks(&config).await?;
    
    Ok(())
}

/// Get the actual symlink destination path for a custom symlink
pub fn get_symlink_destination(symlink: &CustomSymlink) -> Result<PathBuf, String> {
    let dest_parent = PathBuf::from(&symlink.destination_parent);
    let source = PathBuf::from(&symlink.source);
    
    let source_name = source
        .file_name()
        .ok_or_else(|| format!("Invalid source path: {}", symlink.source))?;
    
    Ok(dest_parent.join(source_name))
}

/// Apply all enabled custom symlinks (create the actual symlinks on disk)
/// If installation_id is provided, only apply symlinks for that installation + global symlinks
pub async fn apply_custom_symlinks(installation_id: Option<&str>) -> Result<(), String> {
    let config = read_custom_symlinks().await?;
    
    for symlink in &config.symlinks {
        // Skip disabled symlinks
        if !symlink.enabled {
            continue;
        }
        
        // Skip installation-specific symlinks if they don't match
        if let Some(req_inst) = installation_id {
            if let Some(ref symlink_inst) = symlink.installation_id {
                if symlink_inst != req_inst {
                    continue;
                }
            }
        } else {
            // If no installation is specified, only apply global symlinks
            if symlink.installation_id.is_some() {
                continue;
            }
        }
        
        let source_path = PathBuf::from(&symlink.source);
        let dest_path = get_symlink_destination(symlink)?;
        
        // Skip if source doesn't exist
        if !source_path.exists() {
            continue;
        }
        
        // Skip if symlink already exists and points to the correct source
        if dest_path.exists() && dest_path.is_symlink() {
            if let Ok(existing_target) = std::fs::read_link(&dest_path) {
                if existing_target == source_path {
                    continue; // Already correct
                }
            }
            // Remove incorrect symlink
            crate::remove_symlink_if_exists(&dest_path).await?;
        }
        
        // Create the symlink
        if source_path.is_dir() {
            crate::create_directory_symlink(&source_path, &dest_path).await?;
        } else {
            crate::create_file_symlink(&source_path, &dest_path).await?;
        }
    }
    
    Ok(())
}

/// Remove all custom symlinks from disk (cleanup)
pub async fn cleanup_custom_symlinks() -> Result<(), String> {
    let config = read_custom_symlinks().await?;
    
    for symlink in &config.symlinks {
        let dest_path = get_symlink_destination(symlink)?;
        
        if dest_path.exists() && dest_path.is_symlink() {
            crate::remove_symlink_if_exists(&dest_path).await?;
        }
    }
    
    Ok(())
}
