use crate::system::fs;
use api_types::profiles::KableProfile;
use api_types::symlinks::{Symlink, SymlinkCreateRequest};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::Mutex;

static SYMLINKS: Lazy<Mutex<SymlinkManager>> = Lazy::new(|| Mutex::new(SymlinkManager::new()));

/*
"on disk": The actual symlink that exists in the filesystem,
"in memory": The representation of the symlink in the SymlinkManager's data structures (temporary_symlinks and symlinks),
"in config": The representation of the symlink in the custom_symlinks.json file in the launcher directory.


pub struct Symlink {
    pub id: String, // unique identifier being hash of source+destination.
    pub source: PathBuf,
    pub destination: PathBuf,
    pub is_temporary: bool,  // if true, this symlink is temporary and should be removed on launcher exit/ once a profile is "exited"
    pub enabled: bool,       // if false, this symlink is disabled and should not be created on startup
    pub from_launcher: bool, // if true, this symlink was created by the launcher (either temporary or custom), if false, it was imported from existing symlinks
}
*/
pub struct SymlinkManager {
    // if inside .minecraft folder we have allowed_symlinks.txt with "[regex].*". AND the user has enabled this feature. basically when the manager has been initialized properly.
    enabled: bool,
    // The temporary symlinks of which the destination part should be removed on exit. key=profile.id
    temporary_symlinks: HashMap<String, Symlink>,
    // The custom symlinks that the user has created via the launcher, or were found within .minecraft folder by scanning for existing symlinks.
    // once found by scanning they are included in the launcher_dir().join(constants::CUSTOM_SYMLINKS_FILE) file, and can be enabled/disabled via the launcher.
    symlinks: Vec<Symlink>,
}

impl SymlinkManager {
    fn new() -> Self {
        Self { enabled: false, temporary_symlinks: HashMap::new(), symlinks: Vec::new() }
    }

    // Load and Save the self.symlinks from/to disk using launcher_dir().join(constants::CUSTOM_SYMLINKS_FILE)
    async fn load(&mut self) -> Result<Vec<Symlink>, String> {
        let custom_symlinks_path = fs::launcher_dir()?.join(crate::constants::CUSTOM_SYMLINKS_FILE);
        let contents = fs::read_str(custom_symlinks_path).await?;
        let symlinks: Vec<Symlink> = serde_json::from_str(&contents).map_err(|e| e.to_string())?;
        self.symlinks = symlinks.clone();
        Ok(symlinks)
    }
    /// Save the self.symlinks to config using launcher_dir().join(constants::CUSTOM_SYMLINKS_FILE)
    async fn save(&mut self) -> Result<(), String> {
        let custom_symlinks_path = fs::launcher_dir()?.join(crate::constants::CUSTOM_SYMLINKS_FILE);
        let contents = serde_json::to_string_pretty(&self.symlinks).map_err(|e| e.to_string())?;
        fs::write_str(custom_symlinks_path, &contents, false).await?;
        Ok(())
    }

    /// Make the .minecraft/allowed_symlinks.txt file with "[regex].*" if it doesn't exist, or append the line if it doesn't contain it yet. This is required for symlinks to work in Minecraft.
    /// Then load self.symlinks from disk
    /// Then scan/walk .minecraft and find all existing symlinks and add them to self.symlinks if they are not already present.
    /// Then make sure all loaded symlinks are enabled if they are supposed to be enabled, and disabled if they are supposed to be disabled.
    /// This is done by checking if the source file/dir exists and if the destination file/dir exists and is a symlink to the source.
    /// Then set self.enabled = true if the file is now correct, or false if it failed.
    async fn initialize(&mut self) -> Result<(), String> {
        if self.enabled {
            return Ok(());
        }
        let minecraft_path = fs::mc_dir()?;
        let allowed_symlinks_file = minecraft_path.join(crate::constants::ALLOWED_SYMLINKS_FILE);
        let required_line = crate::constants::ALLOW_SYMLINKS_REGEX;
        let mut contents = fs::read_str(&allowed_symlinks_file).await.unwrap_or_default();

        if !contents.lines().any(|line| line == required_line) {
            if !contents.is_empty() && !contents.ends_with('\n') {
                contents.push('\n');
            }

            contents.push_str(required_line);
            contents.push('\n');

            fs::write_str(&allowed_symlinks_file, &contents, false).await?;
        }
        // Now scan the .minecraft folder for existing symlinks and add them to self.symlinks if they are not already present.
        let existing_symlinks = self.scan().await?;
        // Merge existing symlinks with self.symlinks, ensuring no duplicates based on id
        for symlink in existing_symlinks {
            if !self.symlinks.iter().any(|x| x.id == symlink.id) {
                self.symlinks.push(symlink);
            }
        }
        // load self.symlinks from disk (custom_symlinks.json) and merge with existing symlinks, ensuring no duplicates based on id
        let loaded_symlinks = self.load().await?;
        // Now make sure all loaded symlinks are enabled if they are supposed to be enabled, and disabled if they are supposed to be disabled.
        for symlink in loaded_symlinks.iter() {
            if symlink.enabled {
                Self::create(symlink.clone()).await?;
            } else {
                Self::delete(symlink).await?;
            }
        }
        self.enabled = true;
        Ok(())
    }
    /// Remove all temporary symlinks from disk and memory (and "destroy" them), also disable all custom symlinks (but don't remove them from memory or disk, just disable them). This is called on exit of the launcher.
    async fn cleanup(&mut self) -> Result<(), String> {
        // Remove all temporary symlinks from disk and memory
        for (_, symlink) in self.temporary_symlinks.iter() {
            if symlink.enabled {
                Self::delete(symlink).await?;
            }
        }
        self.temporary_symlinks.clear();
        // Disable all custom symlinks (but don't remove them from memory or disk, just disable them)
        for symlink in self.symlinks.iter_mut() {
            if symlink.enabled {
                symlink.enabled = false;
                Self::delete(symlink).await?;
            }
        }
        Ok(())
    }

    // Add, remove, update, toggle symlinks in memory and on disk. These are for custom symlinks only, not temporary symlinks.

    /// Update a symlink in memory and on disk. This is for custom symlinks only, not temporary symlinks. If the old symlink is enabled, it will be deleted from disk. If the new symlink is enabled, it will be created on disk. If the new symlink is disabled, it will not be created on disk.
    async fn update(&mut self, old: Symlink, new: Symlink) -> Result<Symlink, String> {
        // Find old symlink in self.symlinks, replace it with new. Then save to disk. Return the new symlink.
        if let Some(pos) = self.symlinks.iter().position(|x| x.id == old.id) {
            self.symlinks[pos] = new.clone();
            // remove the old symlink on disk if it exists, and create the new symlink on disk if it is enabled. If the new symlink is disabled, don't create it on disk.
            if let Some(old_link) = self.symlinks.get(pos) {
                if old_link.enabled {
                    Self::delete(old_link).await?;
                }
            }
            if new.enabled {
                Self::create(new.clone()).await?;
            }
            self.save().await?;
            Ok(new)
        } else {
            Err("Symlink not found".to_string())
        }
    }
    async fn add(&mut self, link: Symlink) -> Result<Symlink, String> {
        self.symlinks.push(link.clone());
        self.save().await?;
        Ok(link)
    }
    /// Remove a symlink from memory and disk. This is for custom symlinks only, not temporary symlinks. If the symlink is enabled, it will be deleted from disk. If it is disabled, it will just be removed from memory and disk.
    async fn remove(&mut self, link: &Symlink) -> Result<(), String> {
        if let Some(pos) = self.symlinks.iter().position(|x| x.id == link.id) {
            self.symlinks.remove(pos);
            if link.enabled {
                Self::delete(link).await?;
            }
            self.save().await?;
            Ok(())
        } else {
            Err("Symlink not found".to_string())
        }
    }
    /// Toggle a symlink in memory and on disk. If it is enabled, disable it. If it is disabled, enable it. Return the new symlink.
    async fn toggle(&mut self, link: &Symlink) -> Result<Symlink, String> {
        let mut new_link = link.clone();
        new_link.enabled = !link.enabled;
        self.update(link.clone(), new_link.clone()).await?;
        Ok(new_link)
    }

    /// Make sure a KableProfile gets it dedicated_resource_packs_folder and/or dedicated_shaders_folder symlinked to .minecraft/resourcepacks and .minecraft/shaderpacks respectively.
    /// Checks if these are not already existing, source is not already .minecraft/resourcepacks or .minecraft/shaderpacks and if source exists.
    /// Symlinks created with this function are temporary and will be removed on exit of the launcher. If the profile has no dedicated folders, do nothing.
    async fn setup_profile_symlinks(&mut self, _profile: &KableProfile) -> Result<(), String> {
        // TODO: We will have "projects" in global folders which we should symlink to temp/session ones and also implement cleanup
        // if let Some(dedicated_resourcepacks_folder) = &profile.dedicated_resource_pack_folder {
        //     let source = std::path::PathBuf::from(dedicated_resourcepacks_folder);
        //     let destination = fs::mc_dir()?.join(crate::constants::RESOURCEPACKS_DIR);
        //     let symlink = Symlink::new(source, destination, true, true, true);
        //     Self::create(symlink).await?;
        // }
        // if let Some(dedicated_shaderpacks_folder) = &profile.dedicated_shaders_folder {
        //     let source = std::path::PathBuf::from(dedicated_shaderpacks_folder);
        //     let destination = fs::mc_dir()?.join(crate::constants::SHADERPACKS_DIR);
        //     let symlink = Symlink::new(source, destination, true, true, true);
        //     Self::create(symlink).await?;
        // }
        Ok(())
    }

    /// Create or delete the actual symlink on disk using the source and destination paths.
    /// This is done by checking if the source file/dir exists and then create the symlink at the destination path
    async fn create(link: Symlink) -> Result<(), String> {
        if !link.enabled {
            return Err("Symlink is not enabled".to_string());
        }
        if link.source == link.destination {
            return Err("Source and destination are the same, cannot create symlink".to_string());
        }
        if !link.source.exists() {
            return Err(format!("Source path does not exist: {}", link.source.display()));
        }
        if link.destination.exists() {
            return Err(format!("Destination path already exists: {}", link.destination.display()));
        }
        // Create the symlink using std::os::unix::fs::symlink or std::os::windows::fs::symlink_file/symlink_dir depending on the platform
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&link.source, &link.destination).map_err(|e| e.to_string())?;
        }
        #[cfg(windows)]
        {
            if link.source.is_dir() {
                std::os::windows::fs::symlink_dir(&link.source, &link.destination).map_err(|e| e.to_string())?;
            } else {
                std::os::windows::fs::symlink_file(&link.source, &link.destination).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
    /// Delete the symlink on disk using the destination path. This is done by checking if the destination path exists and is a symlink, and then remove it.
    async fn delete(link: &Symlink) -> Result<(), String> {
        if !link.destination.exists() {
            return Err(format!("Destination path does not exist: {}", link.destination.display()));
        }
        if !link.destination.is_symlink() {
            return Err(format!("Destination path is not a symlink: {}", link.destination.display()));
        }
        std::fs::remove_file(&link.destination).map_err(|e| e.to_string())
    }

    // scans/walksdir .minecraft recursively to find existing symlinks (likely not managed by the launcher) and basically "imports" them
    async fn scan(&mut self) -> Result<Vec<Symlink>, String> {
        // Use walkdir to recursively scan .minecraft for symlinks, and return a Vec<Symlink> of them. If they are not already in self.symlinks, add them to self.symlinks and save to disk.
        let minecraft_path = fs::mc_dir()?;
        let mut symlinks = Vec::new();
        for entry in walkdir::WalkDir::new(&minecraft_path).into_iter().filter_map(|e| e.ok()) {
            if entry.metadata().is_ok_and(|m| m.is_symlink()) {
                // Check if the symlink id ("source.display()-destination.display()") is already in self.symlinks, if not, add it to symlinks and self.symlinks and save to disk.
                let source = std::fs::read_link(entry.path()).map_err(|e| format!("read_link failed: {}", e))?;
                let id = format!("{}-{}", source.display(), entry.path().display());
                if !self.symlinks.iter().any(|l| l.id == id) {
                    symlinks.push(Symlink::new(source, entry.path().to_path_buf(), false, true, false));
                }
            }
        }
        Ok(symlinks)
    }
}

// ? Public functions which should be used in lib.rs and in tauri command wrappers to get the manager and access to the above functions.
async fn manager() -> Result<tokio::sync::MutexGuard<'static, SymlinkManager>, String> {
    let mut manager = SYMLINKS.lock().await;

    if !manager.enabled {
        manager.initialize().await?;
    }

    Ok(manager)
}

pub async fn symlinks() -> Result<Vec<Symlink>, String> {
    Ok(manager().await?.symlinks.clone())
}

pub async fn temporary_symlinks() -> Result<HashMap<String, Symlink>, String> {
    Ok(manager().await?.temporary_symlinks.clone())
}

pub async fn create(link: SymlinkCreateRequest) -> Result<Symlink, String> {
    manager().await?.add(link.into()).await
}

pub async fn remove(link: &Symlink) -> Result<(), String> {
    manager().await?.remove(link).await
}

pub async fn toggle(link: &Symlink) -> Result<Symlink, String> {
    manager().await?.toggle(link).await
}

pub async fn update(old: Symlink, new: Symlink) -> Result<Symlink, String> {
    manager().await?.update(old, new).await
}

pub async fn setup_profile(profile: &KableProfile) -> Result<(), String> {
    manager().await?.setup_profile_symlinks(profile).await
}

pub async fn cleanup() -> Result<(), String> {
    manager().await?.cleanup().await
}
