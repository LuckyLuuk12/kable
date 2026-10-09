use crate::system::fs;
use crate::Logger;
use api_types::profiles::KableProfile;
use api_types::symlinks::{Symlink, SymlinkCreateRequest};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::{Mutex, MutexGuard};

static SYMLINKS: Lazy<Mutex<SymlinkManager>> = Lazy::new(|| Mutex::new(SymlinkManager::new()));

/*
"on disk": The actual symlink that exists in the filesystem.
"in memory": The representation of the symlink in the SymlinkManager's data
             structures (temporary_symlinks and symlinks).
"in config": The representation of a persistent custom symlink in
             custom_symlinks.json.

There are two kinds of symlinks managed by Kable:

1. Persistent custom symlinks

   These are created by the user through the launcher frontend.

   They:
   - are stored in `symlinks`
   - are persisted to `custom_symlinks.json`
   - can be enabled/disabled/updated/removed by the user
   - survive launcher restarts

2. Temporary runtime symlinks

   These are created internally by the runtime system.

   They:
   - are stored in `temporary_symlinks`, grouped by profile ID
   - are never persisted to `custom_symlinks.json`
   - exist only while their corresponding runtime is active
   - are removed when the profile runtime is cleaned up or when the launcher exits

The symlink manager owns the low-level filesystem operations. Higher-level
systems such as the Minecraft runtime decide which paths should be linked.
*/

pub struct SymlinkManager {
    /// True once the manager has initialized the symlink support files and
    /// loaded the persistent custom symlink configuration.
    enabled: bool,

    /// Temporary runtime symlinks grouped by profile ID.
    ///
    /// A single profile can have many temporary symlinks, for example:
    ///
    ///     profile-id -> [
    ///         mods/foo.jar,
    ///         mods/bar.jar,
    ///         assets -> ...
    ///     ]
    temporary_symlinks: HashMap<String, Vec<Symlink>>,

    /// Persistent custom symlinks created by the user or imported from the
    /// existing `.minecraft` directory.
    ///
    /// These are persisted to `custom_symlinks.json`.
    symlinks: Vec<Symlink>,
}

impl SymlinkManager {
    fn new() -> Self {
        Self { enabled: false, temporary_symlinks: HashMap::new(), symlinks: Vec::new() }
    }

    /// Load persistent custom symlinks from disk.
    async fn load(&mut self) -> Result<(), String> {
        let custom_symlinks_path = fs::launcher_dir()?.join(crate::constants::CUSTOM_SYMLINKS_FILE);

        let contents = match fs::read_str(&custom_symlinks_path).await {
            Ok(contents) => contents,
            Err(_) => {
                self.symlinks.clear();
                return Ok(());
            }
        };

        if contents.trim().is_empty() {
            self.symlinks.clear();
            return Ok(());
        }

        let symlinks: Vec<Symlink> = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse custom symlinks {}: {}", custom_symlinks_path.display(), e))?;

        self.symlinks = symlinks;

        Ok(())
    }

    /// Save persistent custom symlinks to disk.
    async fn save(&self) -> Result<(), String> {
        let custom_symlinks_path = fs::launcher_dir()?.join(crate::constants::CUSTOM_SYMLINKS_FILE);

        let contents = serde_json::to_string_pretty(&self.symlinks).map_err(|e| e.to_string())?;

        fs::write_str(custom_symlinks_path, &contents, false).await?;

        Ok(())
    }

    /// Initialize the symlink manager.
    ///
    /// This:
    ///
    /// 1. Ensures Minecraft allows the symlinks Kable creates.
    /// 2. Loads persistent custom symlinks.
    /// 3. Scans `.minecraft` for existing symlinks and imports unknown ones.
    /// 4. Recreates enabled persistent custom symlinks.
    /// 5. Removes disabled persistent custom symlinks from disk.
    async fn initialize(&mut self) -> Result<(), String> {
        if self.enabled {
            return Ok(());
        }

        let minecraft_path = fs::mc_dir()?;

        self.ensure_allowed_symlinks_file(&minecraft_path).await?;

        // Load the persistent configuration first.
        self.load().await?;

        // Import existing symlinks from the Minecraft directory.
        let existing_symlinks = self.scan().await?;

        for symlink in existing_symlinks {
            if !self.symlinks.iter().any(|existing| existing.id == symlink.id) {
                self.symlinks.push(symlink);
            }
        }

        // Reconcile all persistent symlinks with their configured state.
        //
        // At this point `self.symlinks` contains both persisted and imported
        // symlinks.
        let symlinks = self.symlinks.clone();

        for symlink in symlinks {
            if symlink.enabled {
                if !Self::is_correct_symlink(&symlink).await? {
                    Self::delete_if_exists(&symlink).await?;
                    Self::create(&symlink).await?;
                }
            } else {
                Self::delete_if_exists(&symlink).await?;
            }
        }

        // Persist imported symlinks as well.
        self.save().await?;

        self.enabled = true;

        Ok(())
    }

    /// Ensure `.minecraft/allowed_symlinks.txt` contains the required
    /// Minecraft symlink permission rule.
    async fn ensure_allowed_symlinks_file(&self, minecraft_path: &Path) -> Result<(), String> {
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

        Ok(())
    }

    /// Remove all temporary runtime symlinks and disable all persistent
    /// custom symlinks.
    ///
    /// This is intended to be called when the launcher exits.
    async fn cleanup(&mut self) -> Result<(), String> {
        let temporary = std::mem::take(&mut self.temporary_symlinks);

        for (_, symlinks) in temporary {
            for symlink in symlinks {
                Self::delete_if_exists(&symlink).await?;
            }
        }

        for symlink in &mut self.symlinks {
            if symlink.enabled {
                Self::delete_if_exists(symlink).await?;
                symlink.enabled = false;
            }
        }

        self.save().await?;

        Ok(())
    }

    /// Remove all temporary symlinks belonging to one profile.
    async fn cleanup_profile(&mut self, profile_id: &str) -> Result<(), String> {
        let Some(symlinks) = self.temporary_symlinks.remove(profile_id) else {
            return Ok(());
        };

        for symlink in symlinks {
            Self::delete_if_exists(&symlink).await?;
        }

        Ok(())
    }

    /// Create a persistent custom symlink in memory and on disk.
    async fn add(&mut self, mut link: Symlink) -> Result<Symlink, String> {
        link.is_temporary = false;
        link.from_launcher = true;

        if self.symlinks.iter().any(|existing| existing.id == link.id) {
            return Err("Symlink already exists".to_string());
        }

        if link.enabled {
            Self::create(&link).await?;
        }

        self.symlinks.push(link.clone());
        self.save().await?;

        Ok(link)
    }

    /// Remove a persistent custom symlink.
    async fn remove(&mut self, link: &Symlink) -> Result<(), String> {
        let Some(pos) = self.symlinks.iter().position(|x| x.id == link.id) else {
            return Err("Symlink not found".to_string());
        };

        let existing = self.symlinks.remove(pos);

        Self::delete_if_exists(&existing).await?;

        self.save().await?;

        Ok(())
    }

    /// Update a persistent custom symlink.
    ///
    /// The old filesystem symlink is removed before the new one is created.
    async fn update(&mut self, old: Symlink, mut new: Symlink) -> Result<Symlink, String> {
        let Some(pos) = self.symlinks.iter().position(|x| x.id == old.id) else {
            return Err("Symlink not found".to_string());
        };

        new.is_temporary = false;
        new.from_launcher = true;

        let old_link = self.symlinks[pos].clone();

        // Remove the old filesystem entry first.
        Self::delete_if_exists(&old_link).await?;

        // If the new link is enabled, create it before modifying the in-memory
        // configuration. This prevents the manager from claiming a link exists
        // when creation failed.
        if new.enabled {
            if let Err(error) = Self::create(&new).await {
                // Attempt to restore the previous link if it was enabled.
                if old_link.enabled {
                    let _ = Self::create(&old_link).await;
                }

                return Err(error);
            }
        }

        self.symlinks[pos] = new.clone();

        if let Err(error) = self.save().await {
            // Best-effort filesystem rollback.
            let _ = Self::delete_if_exists(&new).await;

            if old_link.enabled {
                let _ = Self::create(&old_link).await;
            }

            self.symlinks[pos] = old_link;

            return Err(error);
        }

        Ok(new)
    }

    /// Toggle a persistent custom symlink.
    async fn toggle(&mut self, link: &Symlink) -> Result<Symlink, String> {
        let mut new_link = link.clone();
        new_link.enabled = !link.enabled;

        self.update(link.clone(), new_link).await
    }

    /// Create a temporary symlink belonging to a profile runtime.
    ///
    /// Temporary symlinks are not persisted to `custom_symlinks.json`.
    async fn create_temporary(&mut self, profile_id: &str, source: PathBuf, destination: PathBuf) -> Result<Symlink, String> {
        let link = Symlink::new(source, destination, true, true, true);

        // Do not silently create duplicate runtime links.
        if let Some(existing) =
            self.temporary_symlinks.get(profile_id).and_then(|links| links.iter().find(|existing| existing.id == link.id))
        {
            return Ok(existing.clone());
        }

        Self::create(&link).await?;

        self.temporary_symlinks.entry(profile_id.to_string()).or_default().push(link.clone());

        Ok(link)
    }

    /// Create or delete the actual symlink on disk.
    async fn create(link: &Symlink) -> Result<(), String> {
        if !link.source.exists() {
            Logger::warn_global(format!("Skipping symlink because its source does not exist: {}", link.source.display()).as_str(), None);
            return Ok(());
        }
        if !link.enabled {
            return Err("Symlink is not enabled".to_string());
        }

        if link.source == link.destination {
            return Err("Source and destination are the same, cannot create symlink".to_string());
        }

        if !link.source.exists() {
            return Err(format!("Source path does not exist: {}", link.source.display()));
        }

        if Self::path_exists_without_following_symlinks(&link.destination).await? {
            return Err(format!("Destination path already exists: {}", link.destination.display()));
        }

        if let Some(parent) = link.destination.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create symlink parent directory {}: {}", parent.display(), e))?;
        }

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&link.source, &link.destination)
                .map_err(|e| format!("Failed to create symlink {} -> {}: {}", link.destination.display(), link.source.display(), e))?;
        }

        #[cfg(windows)]
        {
            if link.source.is_dir() {
                std::os::windows::fs::symlink_dir(&link.source, &link.destination).map_err(|e| {
                    format!("Failed to create directory symlink {} -> {}: {}", link.destination.display(), link.source.display(), e)
                })?;
            } else {
                std::os::windows::fs::symlink_file(&link.source, &link.destination).map_err(|e| {
                    format!("Failed to create file symlink {} -> {}: {}", link.destination.display(), link.source.display(), e)
                })?;
            }
        }

        Ok(())
    }

    /// Delete a symlink from disk.
    ///
    /// Unlike `Path::exists()`, symlink metadata is used so dangling symlinks
    /// are also detected and removed.
    async fn delete(link: &Symlink) -> Result<(), String> {
        let metadata = match tokio::fs::symlink_metadata(&link.destination).await {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(());
            }
            Err(error) => {
                return Err(format!("Failed to inspect symlink destination {}: {}", link.destination.display(), error));
            }
        };

        if !metadata.file_type().is_symlink() {
            return Err(format!("Destination path is not a symlink: {}", link.destination.display()));
        }

        tokio::fs::remove_file(&link.destination)
            .await
            .map_err(|e| format!("Failed to remove symlink {}: {}", link.destination.display(), e))
    }

    /// Delete a symlink if it exists.
    async fn delete_if_exists(link: &Symlink) -> Result<(), String> {
        match tokio::fs::symlink_metadata(&link.destination).await {
            Ok(metadata) => {
                if !metadata.file_type().is_symlink() {
                    return Err(format!("Destination path is not a symlink: {}", link.destination.display()));
                }

                tokio::fs::remove_file(&link.destination)
                    .await
                    .map_err(|e| format!("Failed to remove symlink {}: {}", link.destination.display(), e))?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                // Already removed.
            }
            Err(error) => {
                return Err(format!("Failed to inspect symlink destination {}: {}", link.destination.display(), error));
            }
        }

        Ok(())
    }

    /// Check whether a filesystem path exists without following symlinks.
    async fn path_exists_without_following_symlinks(path: &Path) -> Result<bool, String> {
        match tokio::fs::symlink_metadata(path).await {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(format!("Failed to inspect path {}: {}", path.display(), error)),
        }
    }

    /// Check whether the destination is a symlink pointing to the expected
    /// source.
    async fn is_correct_symlink(link: &Symlink) -> Result<bool, String> {
        let metadata = match tokio::fs::symlink_metadata(&link.destination).await {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(false);
            }
            Err(error) => {
                return Err(format!("Failed to inspect symlink destination {}: {}", link.destination.display(), error));
            }
        };

        if !metadata.file_type().is_symlink() {
            return Ok(false);
        }

        let target = tokio::fs::read_link(&link.destination)
            .await
            .map_err(|e| format!("Failed to read symlink {}: {}", link.destination.display(), e))?;

        Ok(Self::normalize_symlink_target(&link.destination, &target) == Self::normalize_path(&link.source))
    }

    /// Normalize a symlink target relative to its destination.
    ///
    /// `read_link()` returns a relative target as-is, so it needs to be
    /// resolved relative to the directory containing the symlink before it
    /// can be compared to the configured source.
    fn normalize_symlink_target(destination: &Path, target: &Path) -> PathBuf {
        if target.is_absolute() {
            Self::normalize_path(target)
        } else {
            let base = destination.parent().unwrap_or_else(|| Path::new(""));
            Self::normalize_path(&base.join(target))
        }
    }

    /// Normalize a path lexically without requiring it to exist.
    fn normalize_path(path: &Path) -> PathBuf {
        let mut result = PathBuf::new();

        for component in path.components() {
            match component {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    result.pop();
                }
                component => result.push(component),
            }
        }

        result
    }

    /// Scan `.minecraft` recursively for existing symlinks.
    ///
    /// Existing symlinks are imported as persistent custom symlinks with
    /// `from_launcher = false`.
    async fn scan(&self) -> Result<Vec<Symlink>, String> {
        let minecraft_path = fs::mc_dir()?;
        let mut symlinks = Vec::new();

        for entry in walkdir::WalkDir::new(&minecraft_path).into_iter().filter_map(Result::ok) {
            let metadata = match entry.path().symlink_metadata() {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };

            if !metadata.file_type().is_symlink() {
                continue;
            }

            let destination = entry.path().to_path_buf();

            let source =
                std::fs::read_link(&destination).map_err(|e| format!("Failed to read symlink {}: {}", destination.display(), e))?;

            let id = Self::symlink_id(&source, &destination);

            if self.symlinks.iter().any(|link| link.id == id) {
                continue;
            }

            symlinks.push(Symlink::new(source, destination, false, true, false));
        }

        Ok(symlinks)
    }

    fn symlink_id(source: &Path, destination: &Path) -> String {
        format!("{}-{}", source.display(), destination.display())
    }
}

// -----------------------------------------------------------------------------
// Manager access
// -----------------------------------------------------------------------------

async fn manager() -> Result<MutexGuard<'static, SymlinkManager>, String> {
    let mut manager = SYMLINKS.lock().await;

    if !manager.enabled {
        manager.initialize().await?;
    }

    Ok(manager)
}

// -----------------------------------------------------------------------------
// Persistent custom symlinks
// -----------------------------------------------------------------------------

/// Return all persistent custom symlinks.
pub async fn symlinks() -> Result<Vec<Symlink>, String> {
    Ok(manager().await?.symlinks.clone())
}

/// Create a persistent custom symlink.
pub async fn create(link: SymlinkCreateRequest) -> Result<Symlink, String> {
    manager().await?.add(link.into()).await
}

/// Remove a persistent custom symlink.
pub async fn remove(link: &Symlink) -> Result<(), String> {
    manager().await?.remove(link).await
}

/// Toggle a persistent custom symlink.
pub async fn toggle(link: &Symlink) -> Result<Symlink, String> {
    manager().await?.toggle(link).await
}

/// Update a persistent custom symlink.
pub async fn update(old: Symlink, new: Symlink) -> Result<Symlink, String> {
    manager().await?.update(old, new).await
}

// -----------------------------------------------------------------------------
// Temporary runtime symlinks
// -----------------------------------------------------------------------------

/// Return all temporary runtime symlinks grouped by profile ID.
pub async fn temporary_symlinks() -> Result<HashMap<String, Vec<Symlink>>, String> {
    Ok(manager().await?.temporary_symlinks.clone())
}

/// Create a temporary symlink belonging to a profile runtime.
///
/// Temporary symlinks are not persisted and cannot be manipulated through
/// the custom symlink API.
pub async fn create_temporary(profile_id: &str, source: PathBuf, destination: PathBuf) -> Result<Symlink, String> {
    manager().await?.create_temporary(profile_id, source, destination).await
}

/// Remove all temporary symlinks belonging to one profile runtime.
pub async fn cleanup_profile(profile_id: &str) -> Result<(), String> {
    manager().await?.cleanup_profile(profile_id).await
}

// -----------------------------------------------------------------------------
// Launcher lifecycle
// -----------------------------------------------------------------------------

/// Remove all temporary runtime symlinks and disable all persistent custom
/// symlinks.
pub async fn cleanup() -> Result<(), String> {
    manager().await?.cleanup().await
}
