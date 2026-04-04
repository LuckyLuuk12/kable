use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;

// ===== CUSTOM SYMLINK STRUCTURES =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSymlink {
    pub id: String,
    pub source: String,
    pub destination_parent: String,
    pub installation_id: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSymlinksConfig {
    pub symlinks: Vec<CustomSymlink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkInfo {
    pub id: Option<String>,
    pub source: String,
    pub destination: String,
    pub is_global: bool,
    pub installation_id: Option<String>,
    pub symlink_type: String,
    pub is_disabled: bool,
    pub exists: bool,
}

// ===== CONFIG FILE FUNCTIONS =====

fn get_custom_symlinks_config_path() -> Result<PathBuf, String> {
    let kable_dir = crate::get_minecraft_kable_dir()?;
    Ok(kable_dir.join("custom_symlinks.json"))
}

async fn read_custom_symlinks() -> Result<CustomSymlinksConfig, String> {
    let config_path = get_custom_symlinks_config_path()?;

    if !config_path.exists() {
        return Ok(CustomSymlinksConfig {
            symlinks: Vec::new(),
        });
    }

    let contents = async_fs::read_to_string(&config_path)
        .await
        .map_err(|e| format!("Failed to read custom symlinks config: {}", e))?;

    serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse custom symlinks config: {}", e))
}

async fn write_custom_symlinks(config: &CustomSymlinksConfig) -> Result<(), String> {
    let config_path = get_custom_symlinks_config_path()?;

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        async_fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let contents = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    async_fs::write(&config_path, contents)
        .await
        .map_err(|e| format!("Failed to write custom symlinks config: {}", e))
}

fn get_symlink_destination(symlink: &CustomSymlink) -> Result<PathBuf, String> {
    let source_path = PathBuf::from(&symlink.source);
    let dest_parent = PathBuf::from(&symlink.destination_parent);

    let filename = source_path
        .file_name()
        .ok_or_else(|| "Invalid source path: no filename".to_string())?;

    Ok(dest_parent.join(filename))
}

// ===== HELPER FUNCTIONS FOR SCANNING =====

fn should_skip_directory(dir_name: &str) -> bool {
    matches!(
        dir_name,
        "logs"
            | "crash-reports"
            | "versions"
            | "libraries"
            | "assets"
            | "natives"
            | ".fabric"
            | ".mixin.out"
    )
}

fn is_custom_symlink(
    dest_path: &Path,
    target: &Path,
    custom_config: &CustomSymlinksConfig,
) -> bool {
    let Some(dest_parent) = dest_path.parent() else {
        return false;
    };

    for custom in &custom_config.symlinks {
        let custom_source = PathBuf::from(&custom.source);
        let custom_dest_parent = PathBuf::from(&custom.destination_parent);

        if custom_source == target && custom_dest_parent == dest_parent {
            return true;
        }
    }

    false
}

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

fn extract_installation_from_path(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();

    if path_str.contains(".kable/resourcepacks/")
        || path_str.contains(".kable\\resourcepacks\\")
        || path_str.contains("kable/resourcepacks/")
        || path_str.contains("kable\\resourcepacks\\")
    {
        if let Some(pos) = path_str
            .find("resourcepacks/")
            .or_else(|| path_str.find("resourcepacks\\"))
        {
            let after = &path_str[pos + 14..];
            if let Some(slash_pos) = after.find('/').or_else(|| after.find('\\')) {
                return Some(after[..slash_pos].to_string());
            }
        }
    }

    if path_str.contains(".kable/shaderpacks/")
        || path_str.contains(".kable\\shaderpacks\\")
        || path_str.contains("kable/shaderpacks/")
        || path_str.contains("kable\\shaderpacks\\")
    {
        if let Some(pos) = path_str
            .find("shaderpacks/")
            .or_else(|| path_str.find("shaderpacks\\"))
        {
            let after = &path_str[pos + 12..];
            if let Some(slash_pos) = after.find('/').or_else(|| after.find('\\')) {
                return Some(after[..slash_pos].to_string());
            }
        }
    }

    None
}

fn scan_directory_for_managed_symlinks(
    dir: &Path,
    minecraft_root: &Path,
    custom_config: &CustomSymlinksConfig,
    symlinks: &mut Vec<SymlinkInfo>,
) -> Result<(), String> {
    if !dir.exists() || !dir.is_dir() {
        return Ok(());
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_symlink() {
            if let Ok(target) = std::fs::read_link(&path) {
                let is_custom = is_custom_symlink(&path, &target, custom_config);

                if !is_custom {
                    let symlink_type = determine_symlink_type(&path, minecraft_root);
                    let installation_id = extract_installation_from_path(&target);

                    let path_str = path.to_string_lossy().to_string();
                    let is_disabled = path_str.ends_with(".disabled");

                    symlinks.push(SymlinkInfo {
                        id: None,
                        source: target.to_string_lossy().to_string(),
                        destination: path_str,
                        is_global: installation_id.is_none(),
                        installation_id,
                        symlink_type,
                        is_disabled,
                        exists: target.exists(),
                    });
                }
            }
        } else if path.is_dir() {
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if !should_skip_directory(dir_name) {
                scan_directory_for_managed_symlinks(
                    &path,
                    minecraft_root,
                    custom_config,
                    symlinks,
                )?;
            }
        }
    }

    Ok(())
}

// ===== PUBLIC API FUNCTIONS =====

pub async fn list_all_symlinks() -> Result<Vec<SymlinkInfo>, String> {
    let minecraft_dir = crate::get_default_minecraft_dir()?;
    let mut symlinks = Vec::new();

    // Read custom symlinks from config (source of truth)
    let custom_config = read_custom_symlinks().await?;

    // Add all custom symlinks from config
    for custom in &custom_config.symlinks {
        let dest_path = get_symlink_destination(custom)?;
        let exists = dest_path.exists() && dest_path.is_symlink();

        symlinks.push(SymlinkInfo {
            id: Some(custom.id.clone()),
            source: custom.source.clone(),
            destination: dest_path.to_string_lossy().to_string(),
            is_global: custom.installation_id.is_none(),
            installation_id: custom.installation_id.clone(),
            symlink_type: "custom".to_string(),
            is_disabled: !custom.enabled,
            exists,
        });
    }

    // Scan for managed symlinks (from dedicated folders)
    scan_directory_for_managed_symlinks(
        &minecraft_dir,
        &minecraft_dir,
        &custom_config,
        &mut symlinks,
    )?;

    Ok(symlinks)
}

pub async fn create_custom_symlink(
    source: String,
    destination_parent: String,
    installation_id: Option<String>,
) -> Result<String, String> {
    let source_path = PathBuf::from(&source);
    let dest_parent = PathBuf::from(&destination_parent);

    if !source_path.exists() {
        return Err("Source path does not exist".to_string());
    }

    if !dest_parent.exists() {
        return Err("Destination parent directory does not exist".to_string());
    }

    let mut config = read_custom_symlinks().await?;

    let id = uuid::Uuid::new_v4().to_string();

    let custom_symlink = CustomSymlink {
        id: id.clone(),
        source: source.clone(),
        destination_parent: destination_parent.clone(),
        installation_id,
        enabled: true,
    };

    let dest_path = get_symlink_destination(&custom_symlink)?;

    if dest_path.exists() {
        return Err(format!(
            "Destination already exists: {}",
            dest_path.display()
        ));
    }

    config.symlinks.push(custom_symlink);
    write_custom_symlinks(&config).await?;

    if source_path.is_dir() {
        crate::create_directory_symlink(&source_path, &dest_path).await?;
    } else {
        crate::create_file_symlink(&source_path, &dest_path).await?;
    }

    Ok(id)
}

pub async fn remove_symlink(destination: String, id: Option<String>) -> Result<(), String> {
    let dest_path = PathBuf::from(&destination);

    if dest_path.exists() && dest_path.is_symlink() {
        crate::remove_symlink_if_exists(&dest_path).await?;
    }

    if let Some(symlink_id) = id {
        let mut config = read_custom_symlinks().await?;
        config.symlinks.retain(|s| s.id != symlink_id);
        write_custom_symlinks(&config).await?;
    }

    Ok(())
}

pub async fn toggle_symlink_disabled(
    destination: String,
    id: Option<String>,
) -> Result<bool, String> {
    let dest_path = PathBuf::from(&destination);

    if let Some(symlink_id) = id {
        let mut config = read_custom_symlinks().await?;

        let symlink = config
            .symlinks
            .iter_mut()
            .find(|s| s.id == symlink_id)
            .ok_or_else(|| "Custom symlink not found in config".to_string())?;

        symlink.enabled = !symlink.enabled;
        let new_state = !symlink.enabled;
        let is_enabled = symlink.enabled;
        let source = symlink.source.clone();
        let dest = get_symlink_destination(symlink)?;

        write_custom_symlinks(&config).await?;

        if is_enabled {
            let source_path = PathBuf::from(&source);

            if source_path.is_dir() {
                crate::create_directory_symlink(&source_path, &dest).await?;
            } else {
                crate::create_file_symlink(&source_path, &dest).await?;
            }
        } else if dest.exists() && dest.is_symlink() {
            crate::remove_symlink_if_exists(&dest).await?;
        }

        Ok(new_state)
    } else if dest_path.to_string_lossy().ends_with(".disabled") {
        let new_path = PathBuf::from(dest_path.to_string_lossy().trim_end_matches(".disabled"));
        std::fs::rename(&dest_path, &new_path)
            .map_err(|e| format!("Failed to enable symlink: {}", e))?;
        Ok(false)
    } else {
        let new_path = PathBuf::from(format!("{}.disabled", dest_path.display()));
        std::fs::rename(&dest_path, &new_path)
            .map_err(|e| format!("Failed to disable symlink: {}", e))?;
        Ok(true)
    }
}

pub async fn update_symlink(
    id: Option<String>,
    old_destination: String,
    new_source: String,
    new_destination_parent: String,
    new_installation_id: Option<Option<String>>,
) -> Result<(), String> {
    let new_source_path = PathBuf::from(&new_source);
    let new_dest_parent = PathBuf::from(&new_destination_parent);

    if !new_source_path.exists() {
        return Err("New source path does not exist".to_string());
    }

    if !new_dest_parent.exists() {
        return Err("New destination parent directory does not exist".to_string());
    }

    let old_dest_path = PathBuf::from(&old_destination);
    if old_dest_path.exists() && old_dest_path.is_symlink() {
        crate::remove_symlink_if_exists(&old_dest_path).await?;
    }

    if let Some(symlink_id) = id {
        let mut config = read_custom_symlinks().await?;

        let symlink = config
            .symlinks
            .iter_mut()
            .find(|s| s.id == symlink_id)
            .ok_or_else(|| "Custom symlink not found in config".to_string())?;

        symlink.source = new_source.clone();
        symlink.destination_parent = new_destination_parent.clone();

        if let Some(new_inst_id) = new_installation_id {
            symlink.installation_id = new_inst_id;
        }

        let is_enabled = symlink.enabled;
        let dest_path = get_symlink_destination(symlink)?;

        write_custom_symlinks(&config).await?;

        if is_enabled {
            if new_source_path.is_dir() {
                crate::create_directory_symlink(&new_source_path, &dest_path).await?;
            } else {
                crate::create_file_symlink(&new_source_path, &dest_path).await?;
            }
        }
    }

    Ok(())
}

pub async fn apply_custom_symlinks(installation_id: Option<&str>) -> Result<(), String> {
    let config = read_custom_symlinks().await?;

    for symlink in &config.symlinks {
        if !symlink.enabled {
            continue;
        }

        if let Some(req_inst) = installation_id {
            if let Some(ref symlink_inst) = symlink.installation_id {
                if symlink_inst != req_inst {
                    continue;
                }
            }
        } else if symlink.installation_id.is_some() {
            continue;
        }

        let source_path = PathBuf::from(&symlink.source);
        let dest_path = get_symlink_destination(symlink)?;

        if !source_path.exists() {
            continue;
        }

        if dest_path.exists() && dest_path.is_symlink() {
            if let Ok(existing_target) = std::fs::read_link(&dest_path) {
                if existing_target == source_path {
                    continue;
                }
            }
            crate::remove_symlink_if_exists(&dest_path).await?;
        }

        if source_path.is_dir() {
            crate::create_directory_symlink(&source_path, &dest_path).await?;
        } else {
            crate::create_file_symlink(&source_path, &dest_path).await?;
        }
    }

    Ok(())
}

pub async fn cleanup_installation_symlinks(installation_id: &str) -> Result<(), String> {
    let config = read_custom_symlinks().await?;

    for symlink in &config.symlinks {
        if let Some(ref symlink_inst) = symlink.installation_id {
            if symlink_inst == installation_id {
                let dest_path = get_symlink_destination(symlink)?;

                if dest_path.exists() && dest_path.is_symlink() {
                    crate::remove_symlink_if_exists(&dest_path).await?;
                }
            }
        }
    }

    Ok(())
}

// ===== SYMLINK MANAGER STRUCT =====

/// Manage dynamic symlinks for shaders and resource packs based on the installation being launched
pub struct SymlinkManager {
    minecraft_dir: PathBuf,
}

impl SymlinkManager {
    pub fn new(minecraft_dir: PathBuf) -> Self {
        Self { minecraft_dir }
    }

    /// Setup symlinks for an installation before launching
    /// This removes all existing symlinks and creates new ones for the current installation
    pub async fn setup_for_installation(&self, installation_id: &str) -> Result<(), String> {
        crate::logging::Logger::warn_global(
            &format!(
                "[SYMLINK] setup_for_installation start installation_id={} minecraft_dir={}",
                installation_id,
                self.minecraft_dir.display()
            ),
            None,
        );

        // Clean up any existing symlinks first (preserves global custom symlinks)
        self.cleanup_for_installation_switch(installation_id)
            .await?;

        // Setup shader symlinks for this installation
        self.setup_shader_symlinks(installation_id).await?;

        // Setup resource pack symlinks for this installation
        self.setup_resourcepack_symlinks(installation_id).await?;

        // Apply custom symlinks for this installation (includes global ones)
        apply_custom_symlinks(Some(installation_id)).await?;

        crate::logging::Logger::warn_global(
            &format!(
                "[SYMLINK] setup_for_installation complete installation_id={}",
                installation_id
            ),
            None,
        );

        Ok(())
    }

    /// Clean up installation-specific symlinks when switching installations
    /// Global custom symlinks are preserved
    async fn cleanup_for_installation_switch(&self, installation_id: &str) -> Result<(), String> {
        // Remove all symlinks from shaderpacks directory
        self.cleanup_directory_symlinks(&self.minecraft_dir.join("shaderpacks"))
            .await?;

        // Remove all symlinks from resourcepacks directory
        self.cleanup_directory_symlinks(&self.minecraft_dir.join("resourcepacks"))
            .await?;

        // Cleanup installation-specific custom symlinks only (preserve global)
        cleanup_installation_symlinks(installation_id).await?;

        Ok(())
    }

    /// Clean up ALL symlinks (including global custom symlinks)
    /// Used on app startup/shutdown
    pub async fn cleanup_all_symlinks(&self) -> Result<(), String> {
        // Remove all symlinks from shaderpacks directory
        self.cleanup_directory_symlinks(&self.minecraft_dir.join("shaderpacks"))
            .await?;

        // Remove all symlinks from resourcepacks directory
        self.cleanup_directory_symlinks(&self.minecraft_dir.join("resourcepacks"))
            .await?;

        // Note: We could optionally cleanup ALL custom symlinks here,
        // but for now we'll just let them stay since they're harmless
        // and users might want them to persist across app restarts

        Ok(())
    }

    /// Remove all symlinks from a specific directory
    async fn cleanup_directory_symlinks(&self, dir: &PathBuf) -> Result<(), String> {
        if !dir.exists() {
            return Ok(());
        }

        let entries = std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {:?}: {}", dir, e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            // Check if it's a symlink
            if path.is_symlink() {
                crate::remove_symlink_if_exists(&path).await?;
            }
        }

        Ok(())
    }

    /// Setup shader symlinks for a specific installation
    async fn setup_shader_symlinks(&self, installation_id: &str) -> Result<(), String> {
        let kable_dir = crate::get_minecraft_kable_dir()?;

        // Read the installation to get the dedicated shaders folder path
        let installations = crate::installations::kable_profiles::read_kable_profiles()?;
        let installation = installations.iter().find(|i| i.id == installation_id);

        let installation_shaders_dir = if let Some(inst) = installation {
            if let Some(ref dedicated_folder) = inst.dedicated_shaders_folder {
                let dedicated_path = std::path::PathBuf::from(dedicated_folder);
                if dedicated_path.is_absolute() {
                    dedicated_path
                } else {
                    kable_dir.join(dedicated_folder)
                }
            } else {
                return Ok(()); // No dedicated folder configured
            }
        } else {
            return Ok(()); // Installation not found
        };

        // Only setup if the installation has a dedicated shaders folder
        if !installation_shaders_dir.exists() {
            return Ok(());
        }

        let shaderpacks_dir = self.minecraft_dir.join("shaderpacks");
        crate::ensure_folder(&shaderpacks_dir).await?;

        // Read all shader packs in the installation's dedicated folder
        let entries = match std::fs::read_dir(&installation_shaders_dir) {
            Ok(e) => e,
            Err(_) => return Ok(()), // No shaders for this installation
        };

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read shader entry: {}", e))?;
            let path = entry.path();

            if path.is_file()
                && (path.extension().is_some_and(|ext| ext == "zip")
                    || path.extension().is_some_and(|ext| ext == "jar"))
            {
                // Create symlink in .minecraft/shaderpacks pointing to the dedicated folder file
                if let Some(filename) = path.file_name() {
                    let target_link = shaderpacks_dir.join(filename);

                    // Only create if doesn't exist
                    if !target_link.exists() {
                        crate::create_file_symlink(&path, &target_link).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Setup resource pack symlinks for a specific installation
    /// Handles both merged and individual packs based on folder structure:
    /// - Packs in merged/ subfolder are merged into kable-merged.zip
    /// - Packs in individual/ subfolder get individual symlinks
    ///
    /// Falls back to legacy behavior if subfolders don't exist
    pub async fn setup_resourcepack_symlinks(&self, installation_id: &str) -> Result<(), String> {
        use crate::logging::Logger;

        Logger::warn_global(
            &format!(
                "[SYMLINK] setup_resourcepack_symlinks start installation_id={}",
                installation_id
            ),
            None,
        );

        let kable_dir = crate::get_minecraft_kable_dir()?;

        // Read the installation to get the dedicated resource pack folder path
        let installations = crate::installations::kable_profiles::read_kable_profiles()?;
        let installation = installations.iter().find(|i| i.id == installation_id);

        let installation_data = if let Some(inst) = installation {
            inst
        } else {
            Logger::warn_global(
                &format!(
                    "[SYMLINK] setup_resourcepack_symlinks skipped: installation not found installation_id={}",
                    installation_id
                ),
                None,
            );
            return Ok(()); // Installation not found
        };

        let installation_packs_dir = if let Some(ref dedicated_folder) =
            installation_data.dedicated_resource_pack_folder
        {
            let dedicated_path = std::path::PathBuf::from(dedicated_folder);
            if dedicated_path.is_absolute() {
                dedicated_path
            } else {
                // Normalize relative configured path to the same shape used by
                // KableInstallation::find_resourcepacks_dir().
                let normalized = dedicated_folder.replace('\\', "/");
                let cleaned = normalized
                    .strip_prefix("resourcepacks/")
                    .unwrap_or(&normalized);
                kable_dir.join("resourcepacks").join(cleaned)
            }
        } else {
            Logger::warn_global(
                    &format!(
                        "[SYMLINK] setup_resourcepack_symlinks skipped: no dedicated_resource_pack_folder installation_id={}",
                        installation_id
                    ),
                    None,
                );
            return Ok(()); // No dedicated folder configured
        };

        Logger::warn_global(
            &format!(
                "[SYMLINK] resourcepack source resolved installation_id={} path={}",
                installation_id,
                installation_packs_dir.display()
            ),
            None,
        );

        // Only setup if the installation has a dedicated resource packs folder
        if !installation_packs_dir.exists() {
            Logger::warn_global(
                &format!(
                    "[SYMLINK] setup_resourcepack_symlinks skipped: source folder missing installation_id={} path={}",
                    installation_id,
                    installation_packs_dir.display()
                ),
                None,
            );
            return Ok(());
        }

        let resourcepacks_dir = self.minecraft_dir.join("resourcepacks");
        crate::ensure_folder(&resourcepacks_dir).await?;
        crate::ensure_symlinks_enabled(&self.minecraft_dir).await?;

        let merged_dir = installation_packs_dir.join("merged");
        let individual_dir = installation_packs_dir.join("individual");

        // Check if using new subfolder structure
        let using_subfolders = merged_dir.exists() || individual_dir.exists();

        Logger::warn_global(
            &format!(
                "[SYMLINK] resourcepack subfolder detection installation_id={} using_subfolders={} merged_exists={} individual_exists={}",
                installation_id,
                using_subfolders,
                merged_dir.exists(),
                individual_dir.exists()
            ),
            None,
        );

        if using_subfolders {
            // New behavior: handle merged/ and individual/ subfolders

            // Handle merged packs
            if merged_dir.exists() {
                let mut merged_packs: std::collections::HashSet<String> =
                    std::collections::HashSet::new();

                if let Ok(entries) = std::fs::read_dir(&merged_dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_file() && path.extension().is_some_and(|ext| ext == "zip") {
                            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                                // Never treat the generated output as an input pack.
                                if filename != "kable-merged.zip" {
                                    merged_packs.insert(filename.to_string());
                                }
                            }
                        }
                    }
                }

                // Merge packs if any exist
                if !merged_packs.is_empty() {
                    let mut pack_order: Vec<String> = if installation_data.pack_order.is_empty() {
                        merged_packs.iter().cloned().collect()
                    } else {
                        // Filter pack_order to only include packs that exist in merged/
                        installation_data
                            .pack_order
                            .iter()
                            .filter(|p| merged_packs.contains(*p))
                            .cloned()
                            .collect()
                    };

                    // If configured order has no overlap with merged/ contents,
                    // fall back to all discovered merged packs.
                    if pack_order.is_empty() {
                        pack_order = merged_packs.iter().cloned().collect();
                    }

                    // Merge packs into kable-merged.zip
                    let merged_path = crate::resourcepacks::merge_resourcepacks(
                        merged_dir.to_string_lossy().to_string(),
                        pack_order,
                        merged_packs,
                    )
                    .await?;

                    // Create symlink for the merged pack
                    let target_link = resourcepacks_dir.join("kable-merged.zip");
                    if target_link.exists() {
                        crate::remove_symlink_if_exists(&target_link).await?;
                    }
                    crate::create_file_symlink(&merged_path, &target_link).await?;
                } else {
                    // No merged source packs left; remove stale merged symlink if present.
                    let target_link = resourcepacks_dir.join("kable-merged.zip");
                    if target_link.exists() && target_link.is_symlink() {
                        crate::remove_symlink_if_exists(&target_link).await?;
                    }
                }
            }

            // Handle individual packs
            if individual_dir.exists() {
                let mut linked_count = 0usize;
                let mut skipped_conflicts = 0usize;

                if let Ok(entries) = std::fs::read_dir(&individual_dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();

                        if path.is_file() && path.extension().is_some_and(|ext| ext == "zip") {
                            if let Some(filename) = path.file_name() {
                                let target_link = resourcepacks_dir.join(filename);

                                if target_link.exists() {
                                    if target_link.is_symlink() {
                                        crate::remove_symlink_if_exists(&target_link).await?;
                                    } else {
                                        skipped_conflicts += 1;
                                        Logger::warn_global(
                                            &format!(
                                                "Skipping resource pack symlink; target exists and is not a symlink: {}",
                                                target_link.display()
                                            ),
                                            None,
                                        );
                                        continue;
                                    }
                                }

                                crate::create_file_symlink(&path, &target_link).await?;
                                linked_count += 1;
                            }
                        }
                    }
                }

                Logger::debug_global(
                    &format!(
                        "Resource pack symlink setup complete for installation {} (linked: {}, conflicts: {})",
                        installation_id, linked_count, skipped_conflicts
                    ),
                    None,
                );

                Logger::warn_global(
                    &format!(
                        "[SYMLINK] individual resourcepacks processed installation_id={} linked={} conflicts={}",
                        installation_id, linked_count, skipped_conflicts
                    ),
                    None,
                );
            }
        } else {
            // Legacy behavior: use enable_pack_merging flag for all packs
            if installation_data.enable_pack_merging {
                let entries = match std::fs::read_dir(&installation_packs_dir) {
                    Ok(e) => e,
                    Err(_) => return Ok(()), // No resource packs
                };

                let mut enabled_packs = std::collections::HashSet::new();
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_file() && path.extension().is_some_and(|ext| ext == "zip") {
                        if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                            // Skip the merged pack itself
                            if filename != "kable-merged.zip" {
                                enabled_packs.insert(filename.to_string());
                            }
                        }
                    }
                }

                // If we have enabled packs, merge them
                if !enabled_packs.is_empty() {
                    let pack_order = if installation_data.pack_order.is_empty() {
                        // No order specified, use all enabled packs in arbitrary order
                        enabled_packs.iter().cloned().collect()
                    } else {
                        // Use specified order
                        installation_data.pack_order.clone()
                    };

                    // Merge packs into kable-merged.zip
                    let merged_path = crate::resourcepacks::merge_resourcepacks(
                        installation_packs_dir.to_string_lossy().to_string(),
                        pack_order,
                        enabled_packs,
                    )
                    .await?;

                    // Create symlink for the merged pack
                    let target_link = resourcepacks_dir.join("kable-merged.zip");
                    if target_link.exists() {
                        crate::remove_symlink_if_exists(&target_link).await?;
                    }
                    crate::create_file_symlink(&merged_path, &target_link).await?;
                }
            } else {
                // Individual pack symlinks (original behavior)
                let entries = match std::fs::read_dir(&installation_packs_dir) {
                    Ok(e) => e,
                    Err(_) => return Ok(()), // No resource packs for this installation
                };

                for entry in entries.filter_map(|e| e.ok()) {
                    let entry_result = entry;
                    let path = entry_result.path();

                    if path.is_file() && path.extension().is_some_and(|ext| ext == "zip") {
                        // Create symlink in .minecraft/resourcepacks pointing to the dedicated folder file
                        if let Some(filename) = path.file_name() {
                            let target_link = resourcepacks_dir.join(filename);

                            // Only create if doesn't exist
                            if !target_link.exists() {
                                crate::create_file_symlink(&path, &target_link).await?;
                            }
                        }
                    }
                }
            }
        }

        Logger::warn_global(
            &format!(
                "[SYMLINK] setup_resourcepack_symlinks complete installation_id={}",
                installation_id
            ),
            None,
        );

        Ok(())
    }
}
