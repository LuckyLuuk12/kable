use std::path::PathBuf;

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
        // Clean up any existing symlinks first
        self.cleanup_all_symlinks().await?;

        // Setup shader symlinks for this installation
        self.setup_shader_symlinks(installation_id).await?;

        // Setup resource pack symlinks for this installation
        self.setup_resourcepack_symlinks(installation_id).await?;

        // Apply custom symlinks for this installation (includes global ones)
        crate::custom_symlinks::apply_custom_symlinks(Some(installation_id)).await?;

        Ok(())
    }

    /// Clean up all symlinks after game closes or before launching a different installation
    pub async fn cleanup_all_symlinks(&self) -> Result<(), String> {
        // Remove all symlinks from shaderpacks directory
        self.cleanup_directory_symlinks(&self.minecraft_dir.join("shaderpacks")).await?;

        // Remove all symlinks from resourcepacks directory
        self.cleanup_directory_symlinks(&self.minecraft_dir.join("resourcepacks")).await?;

        // Cleanup custom symlinks
        crate::custom_symlinks::cleanup_custom_symlinks().await?;

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

            if path.is_file() && (path.extension().is_some_and(|ext| ext == "zip") || path.extension().is_some_and(|ext| ext == "jar")) {
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
    async fn setup_resourcepack_symlinks(&self, installation_id: &str) -> Result<(), String> {
        let kable_dir = crate::get_minecraft_kable_dir()?;
        
        // Read the installation to get the dedicated resource pack folder path
        let installations = crate::installations::kable_profiles::read_kable_profiles()?;
        let installation = installations.iter().find(|i| i.id == installation_id);
        
        let installation_packs_dir = if let Some(inst) = installation {
            if let Some(ref dedicated_folder) = inst.dedicated_resource_pack_folder {
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

        // Only setup if the installation has a dedicated resource packs folder
        if !installation_packs_dir.exists() {
            return Ok(());
        }

        let resourcepacks_dir = self.minecraft_dir.join("resourcepacks");
        crate::ensure_folder(&resourcepacks_dir).await?;

        // Read all resource packs in the installation's dedicated folder
        let entries = match std::fs::read_dir(&installation_packs_dir) {
            Ok(e) => e,
            Err(_) => return Ok(()), // No resource packs for this installation
        };

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read resourcepack entry: {}", e))?;
            let path = entry.path();

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

        Ok(())
    }
}
