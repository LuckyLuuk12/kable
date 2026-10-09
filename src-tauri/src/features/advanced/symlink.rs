use crate::system::fs;
use crate::Logger;
use api_types::symlinks::{Symlink, SymlinkCreateRequest};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use tokio::sync::{Mutex, MutexGuard};

static SYMLINKS: Lazy<Mutex<SymlinkManager>> = Lazy::new(|| Mutex::new(SymlinkManager::new()));

const RUNTIME_LINKS_DIRECTORY: &str = "runtime-links";

pub struct SymlinkManager {
    enabled: bool,
    runtime_symlinks: HashMap<String, Vec<Symlink>>,
    symlinks: Vec<Symlink>,
}

impl SymlinkManager {
    fn new() -> Self {
        Self { enabled: false, runtime_symlinks: HashMap::new(), symlinks: Vec::new() }
    }

    // ---------------------------------------------------------------------
    // Persistent custom symlinks
    // ---------------------------------------------------------------------

    async fn load(&mut self) -> Result<(), String> {
        let config_path = fs::launcher_dir()?.join(crate::constants::CUSTOM_SYMLINKS_FILE);

        let contents = match fs::read_str(&config_path).await {
            Ok(contents) => contents,
            Err(error) => {
                if !matches!(
                    tokio::fs::metadata(&config_path).await,
                    Err(ref metadata_error)
                        if metadata_error.kind() == std::io::ErrorKind::NotFound
                ) {
                    return Err(format!("Failed to read custom symlink configuration {}: {}", config_path.display(), error));
                }

                self.symlinks.clear();
                return Ok(());
            }
        };

        if contents.trim().is_empty() {
            self.symlinks.clear();
            return Ok(());
        }

        self.symlinks = serde_json::from_str(&contents)
            .map_err(|error| format!("Failed to parse custom symlink configuration {}: {}", config_path.display(), error))?;

        Ok(())
    }

    async fn save(&self) -> Result<(), String> {
        let config_path = fs::launcher_dir()?.join(crate::constants::CUSTOM_SYMLINKS_FILE);

        let contents =
            serde_json::to_string_pretty(&self.symlinks).map_err(|error| format!("Failed to serialize custom symlinks: {}", error))?;

        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|error| format!("Failed to create custom symlink configuration directory {}: {}", parent.display(), error))?;
        }

        fs::write_str(config_path, &contents, false).await?;
        Ok(())
    }

    async fn initialize(&mut self) -> Result<(), String> {
        if self.enabled {
            return Ok(());
        }

        let minecraft_dir = fs::mc_dir()?;

        self.ensure_allowed_symlinks_file(&minecraft_dir).await?;
        self.load().await?;

        let scanned = self.scan().await?;

        for link in scanned {
            if !self.symlinks.iter().any(|existing| existing.id == link.id) {
                self.symlinks.push(link);
            }
        }

        let configured_links = self.symlinks.clone();

        for link in &configured_links {
            if link.enabled {
                match Self::is_correct_symlink(link).await {
                    Ok(true) => {}
                    Ok(false) => match Self::path_exists_without_following_symlinks(&link.destination).await {
                        Ok(false) => {
                            if link.source.exists() {
                                if let Err(error) = Self::create(link).await {
                                    Logger::warn_global(
                                        &format!("Could not restore custom symlink {}: {}", link.destination.display(), error),
                                        None,
                                    );
                                }
                            }
                        }
                        Ok(true) => {
                            Logger::warn_global(
                                &format!("Custom symlink destination is occupied by another entry: {}", link.destination.display()),
                                None,
                            );
                        }
                        Err(error) => {
                            Logger::warn_global(&error, None);
                        }
                    },
                    Err(error) => {
                        Logger::warn_global(&error, None);
                    }
                }
            } else if let Err(error) = Self::delete_if_exists(link).await {
                Logger::warn_global(&format!("Could not remove disabled custom symlink {}: {}", link.destination.display(), error), None);
            }
        }

        self.save().await?;
        self.enabled = true;

        Ok(())
    }

    async fn ensure_allowed_symlinks_file(&self, minecraft_dir: &Path) -> Result<(), String> {
        let path = minecraft_dir.join(crate::constants::ALLOWED_SYMLINKS_FILE);

        let required_line = crate::constants::ALLOW_SYMLINKS_REGEX;
        let mut contents = fs::read_str(&path).await.unwrap_or_default();

        if contents.lines().any(|line| line == required_line) {
            return Ok(());
        }

        if !contents.is_empty() && !contents.ends_with('\n') {
            contents.push('\n');
        }

        contents.push_str(required_line);
        contents.push('\n');

        fs::write_str(path, &contents, false).await?;
        Ok(())
    }

    async fn add(&mut self, mut link: Symlink) -> Result<Symlink, String> {
        link.is_temporary = false;
        link.from_launcher = true;

        if self.symlinks.iter().any(|existing| existing.id == link.id) {
            return Err("Symlink already exists".to_string());
        }

        if self.symlinks.iter().any(|existing| same_path(&existing.destination, &link.destination)) {
            return Err(format!("A custom symlink already uses destination {}", link.destination.display()));
        }

        if link.enabled {
            Self::create(&link).await?;
        }

        self.symlinks.push(link.clone());

        if let Err(error) = self.save().await {
            self.symlinks.retain(|existing| existing.id != link.id);

            if link.enabled {
                let _ = Self::delete_if_exists(&link).await;
            }

            return Err(error);
        }

        Ok(link)
    }

    async fn remove(&mut self, link: &Symlink) -> Result<(), String> {
        let Some(index) = self.symlinks.iter().position(|existing| existing.id == link.id) else {
            return Err("Symlink not found".to_string());
        };

        let existing = self.symlinks[index].clone();

        Self::delete_if_exists(&existing).await?;

        self.symlinks.remove(index);

        if let Err(error) = self.save().await {
            self.symlinks.insert(index, existing);
            return Err(error);
        }

        Ok(())
    }

    async fn update(&mut self, old: Symlink, mut new: Symlink) -> Result<Symlink, String> {
        let Some(index) = self.symlinks.iter().position(|existing| existing.id == old.id) else {
            return Err("Symlink not found".to_string());
        };

        new.is_temporary = false;
        new.from_launcher = true;

        if self.symlinks.iter().enumerate().any(|(i, existing)| i != index && same_path(&existing.destination, &new.destination)) {
            return Err(format!("A custom symlink already uses destination {}", new.destination.display()));
        }

        let old_link = self.symlinks[index].clone();

        if new.enabled {
            if !new.source.exists() {
                return Err(format!("Source path does not exist: {}", new.source.display()));
            }

            if same_path(&new.source, &new.destination) {
                return Err("Source and destination must be different paths".to_string());
            }

            if !same_path(&old_link.destination, &new.destination) && Self::path_exists_without_following_symlinks(&new.destination).await?
            {
                return Err(format!("Destination path already exists: {}", new.destination.display()));
            }
        }

        Self::delete_if_exists(&old_link).await?;

        if new.enabled {
            if let Err(error) = Self::create(&new).await {
                if old_link.enabled {
                    let _ = Self::create(&old_link).await;
                }

                return Err(error);
            }
        }

        self.symlinks[index] = new.clone();

        if let Err(error) = self.save().await {
            if new.enabled {
                let _ = Self::delete_if_exists(&new).await;
            }

            if old_link.enabled {
                let _ = Self::create(&old_link).await;
            }

            self.symlinks[index] = old_link;
            return Err(error);
        }

        Ok(new)
    }

    async fn toggle(&mut self, link: &Symlink) -> Result<Symlink, String> {
        let Some(existing) = self.symlinks.iter().find(|existing| existing.id == link.id).cloned() else {
            return Err("Symlink not found".to_string());
        };

        let mut updated = existing.clone();
        updated.enabled = !existing.enabled;

        self.update(existing, updated).await
    }

    async fn cleanup(&mut self) -> Result<(), String> {
        let mut failures = Vec::new();

        // Runtime links persist across launches and are not cleaned up here.
        for link in &mut self.symlinks {
            if link.enabled {
                match Self::delete_if_exists(link).await {
                    Ok(()) => link.enabled = false,
                    Err(error) => failures.push(error),
                }
            }
        }

        if let Err(error) = self.save().await {
            failures.push(error);
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures.join("; "))
        }
    }

    async fn scan(&self) -> Result<Vec<Symlink>, String> {
        let minecraft_dir = fs::mc_dir()?;
        let mut discovered = Vec::new();

        for entry in walkdir::WalkDir::new(&minecraft_dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| {
                if entry.depth() == 0 {
                    return true;
                }

                let relative = match entry.path().strip_prefix(&minecraft_dir) {
                    Ok(relative) => relative,
                    Err(_) => return false,
                };

                let first_component = relative.components().next().and_then(|component| match component {
                    Component::Normal(name) => name.to_str(),
                    _ => None,
                });

                !matches!(first_component, Some(".kable") | Some(".kable-dev"))
            })
            .filter_map(Result::ok)
        {
            let destination = entry.path();

            let metadata = match std::fs::symlink_metadata(destination) {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };

            if !metadata.file_type().is_symlink() {
                continue;
            }

            let raw_target = match std::fs::read_link(destination) {
                Ok(target) => target,
                Err(error) => {
                    Logger::warn_global(&format!("Could not read symlink {}: {}", destination.display(), error), None);
                    continue;
                }
            };

            let source = Self::normalize_symlink_target(destination, &raw_target);
            let destination = destination.to_path_buf();
            let id = Self::symlink_id(&source, &destination);

            if self.symlinks.iter().any(|existing| existing.id == id) || discovered.iter().any(|existing: &Symlink| existing.id == id) {
                continue;
            }

            let mut link = Symlink::new(source, destination, true, false, false);
            link.id = id;
            discovered.push(link);
        }

        Ok(discovered)
    }

    fn symlink_id(source: &Path, destination: &Path) -> String {
        format!("{}-{}", source.display(), destination.display())
    }

    // ---------------------------------------------------------------------
    // Runtime symlinks
    // ---------------------------------------------------------------------

    fn runtime_config_path(profile_id: &str) -> Result<PathBuf, String> {
        if profile_id.is_empty() || profile_id == "." || profile_id == ".." || profile_id.contains('/') || profile_id.contains('\\') {
            return Err("Invalid profile ID for runtime symlink registry".to_string());
        }

        Ok(fs::kable_dir()?.join(RUNTIME_LINKS_DIRECTORY).join(format!("{profile_id}.json")))
    }

    async fn load_runtime_symlinks(&mut self, profile_id: &str) -> Result<Vec<Symlink>, String> {
        let path = Self::runtime_config_path(profile_id)?;

        let contents = match tokio::fs::read_to_string(&path).await {
            Ok(contents) => contents,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.runtime_symlinks.insert(profile_id.to_string(), Vec::new());
                return Ok(Vec::new());
            }
            Err(error) => {
                return Err(format!("Failed to read runtime symlink registry {}: {}", path.display(), error));
            }
        };

        if contents.trim().is_empty() {
            self.runtime_symlinks.insert(profile_id.to_string(), Vec::new());
            return Ok(Vec::new());
        }

        let links: Vec<Symlink> = serde_json::from_str(&contents)
            .map_err(|error| format!("Failed to parse runtime symlink registry {}: {}", path.display(), error))?;

        self.runtime_symlinks.insert(profile_id.to_string(), links.clone());

        Ok(links)
    }

    async fn save_runtime_symlinks(&mut self, profile_id: &str, links: &[Symlink]) -> Result<(), String> {
        let path = Self::runtime_config_path(profile_id)?;

        let parent = path.parent().ok_or_else(|| format!("Runtime symlink registry has no parent directory: {}", path.display()))?;

        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| format!("Failed to create runtime symlink registry directory {}: {}", parent.display(), error))?;

        let contents =
            serde_json::to_string_pretty(links).map_err(|error| format!("Failed to serialize runtime symlink registry: {}", error))?;

        let temporary_path = path.with_extension("json.tmp");

        tokio::fs::write(&temporary_path, contents.as_bytes())
            .await
            .map_err(|error| format!("Failed to write runtime symlink registry {}: {}", temporary_path.display(), error))?;

        // Replace the old registry only after the temporary file is ready.
        // Windows rename generally cannot replace an existing file.
        if tokio::fs::metadata(&path).await.is_ok() {
            tokio::fs::remove_file(&path)
                .await
                .map_err(|error| format!("Failed to replace runtime symlink registry {}: {}", path.display(), error))?;
        }

        if let Err(error) = tokio::fs::rename(&temporary_path, &path).await {
            return Err(format!("Failed to finalize runtime symlink registry {}: {}", path.display(), error));
        }

        self.runtime_symlinks.insert(profile_id.to_string(), links.to_vec());

        Ok(())
    }

    async fn create_runtime_link(&mut self, profile_id: &str, source: PathBuf, destination: PathBuf) -> Result<Symlink, String> {
        if same_path(&source, &destination) {
            return Err("Source and destination must be different paths".to_string());
        }

        if !source.exists() {
            return Err(format!("Source path does not exist: {}", source.display()));
        }

        let mut links = self.load_runtime_symlinks(profile_id).await?;
        let link = Symlink::new(source, destination, true, true, true);

        if let Some(existing) = links.iter().find(|existing| same_path(&existing.destination, &link.destination)).cloned() {
            if same_path(&existing.source, &link.source) && Self::is_correct_symlink(&link).await? {
                return Ok(existing);
            }

            Self::delete_if_exists(&existing).await?;
            links.retain(|existing| !same_path(&existing.destination, &link.destination));
        } else if Self::path_exists_without_following_symlinks(&link.destination).await? {
            if Self::is_correct_symlink(&link).await? {
                links.push(link.clone());
                self.save_runtime_symlinks(profile_id, &links).await?;
                return Ok(link);
            }

            return Err(format!("Destination path already exists and is not an owned runtime symlink: {}", link.destination.display()));
        }

        Self::create(&link).await?;
        links.push(link.clone());

        if let Err(error) = self.save_runtime_symlinks(profile_id, &links).await {
            let _ = Self::delete_if_exists(&link).await;
            return Err(error);
        }

        Ok(link)
    }

    /// Reconcile registered runtime symlinks with the complete desired set.
    ///
    /// All safe operations are attempted. Failures are accumulated and
    /// returned so callers can prevent launching an incomplete runtime.
    async fn reconcile_runtime(&mut self, profile_id: &str, desired: Vec<(PathBuf, PathBuf)>) -> Result<(), String> {
        let mut links = self.load_runtime_symlinks(profile_id).await?;
        let mut failures = Vec::new();
        let mut desired_links = Vec::new();
        let mut desired_destinations = HashSet::new();

        // Validate the desired set before changing the filesystem.
        for (source, destination) in desired {
            if same_path(&source, &destination) {
                failures.push(format!("Runtime symlink source and destination are identical: {}", source.display()));
                continue;
            }

            let destination_key = path_key(&destination);

            if !desired_destinations.insert(destination_key) {
                failures.push(format!("Duplicate runtime symlink destination: {}", destination.display()));
                continue;
            }

            desired_links.push(Symlink::new(source, destination, true, true, true));
        }

        // Remove stale links owned by this profile. Preserve failed deletions
        // in the registry so they can be retried on the next reconciliation.
        let mut retained_links = Vec::new();

        for existing in links.drain(..) {
            if desired_destinations.contains(&path_key(&existing.destination)) {
                retained_links.push(existing);
                continue;
            }

            match Self::delete_if_exists(&existing).await {
                Ok(()) => {}
                Err(error) => {
                    failures.push(format!("Failed to remove stale runtime symlink {}: {}", existing.destination.display(), error));
                    retained_links.push(existing);
                }
            }
        }

        links = retained_links;

        for desired_link in desired_links {
            if !desired_link.source.exists() {
                failures.push(format!("Runtime symlink source does not exist: {}", desired_link.source.display()));
                continue;
            }

            let existing_index = links.iter().position(|existing| same_path(&existing.destination, &desired_link.destination));

            if let Some(index) = existing_index {
                let existing = links[index].clone();

                if same_path(&existing.source, &desired_link.source) && Self::is_correct_symlink(&desired_link).await? {
                    continue;
                }

                // Delete only a symlink whose current target matches the
                // source recorded in the registry.
                match Self::delete_if_exists(&existing).await {
                    Ok(()) => {
                        links.remove(index);
                    }
                    Err(error) => {
                        failures.push(format!("Failed to replace runtime symlink at {}: {}", desired_link.destination.display(), error));
                        continue;
                    }
                }
            } else if Self::path_exists_without_following_symlinks(&desired_link.destination).await? {
                // Adopt an already-correct symlink, but never overwrite an
                // unrelated file, directory, or symlink.
                if Self::is_correct_symlink(&desired_link).await? {
                    links.push(desired_link);
                } else {
                    failures.push(format!(
                        "Runtime symlink destination is occupied by an unrelated entry: {}",
                        desired_link.destination.display()
                    ));
                }

                continue;
            }

            match Self::create(&desired_link).await {
                Ok(()) => links.push(desired_link),
                Err(error) => {
                    failures.push(format!("Failed to create runtime symlink at {}: {}", desired_link.destination.display(), error));
                }
            }
        }

        // Persist the actual managed-link state even when some filesystem
        // operations failed. This preserves ownership information for retries.
        if let Err(error) = self.save_runtime_symlinks(profile_id, &links).await {
            failures.push(error);
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(format!("Runtime symlink reconciliation failed for profile '{}':\n- {}", profile_id, failures.join("\n- ")))
        }
    }

    async fn cleanup_profile(&mut self, profile_id: &str) -> Result<(), String> {
        let mut links = self.load_runtime_symlinks(profile_id).await?;
        let mut retained_links = Vec::new();
        let mut failures = Vec::new();

        for link in links.drain(..) {
            match Self::delete_if_exists(&link).await {
                Ok(()) => {}
                Err(error) => {
                    failures.push(error);
                    retained_links.push(link);
                }
            }
        }

        if let Err(error) = self.save_runtime_symlinks(profile_id, &retained_links).await {
            failures.push(error);
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures.join("; "))
        }
    }

    // ---------------------------------------------------------------------
    // Low-level filesystem operations
    // ---------------------------------------------------------------------

    async fn create(link: &Symlink) -> Result<(), String> {
        if !link.enabled {
            return Err("Cannot create a disabled symlink".to_string());
        }

        if same_path(&link.source, &link.destination) {
            return Err("Source and destination must be different paths".to_string());
        }

        if !link.source.exists() {
            return Err(format!("Source path does not exist: {}", link.source.display()));
        }

        if Self::path_exists_without_following_symlinks(&link.destination).await? {
            if Self::is_correct_symlink(link).await? {
                return Ok(());
            }

            return Err(format!("Destination path already exists: {}", link.destination.display()));
        }

        if let Some(parent) = link.destination.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|error| format!("Failed to create symlink parent directory {}: {}", parent.display(), error))?;
        }

        #[cfg(unix)]
        {
            tokio::fs::symlink(&link.source, &link.destination).await.map_err(|error| {
                format!("Failed to create symlink {} -> {}: {}", link.destination.display(), link.source.display(), error)
            })?;
        }

        #[cfg(windows)]
        {
            if link.source.is_dir() {
                tokio::fs::symlink_dir(&link.source, &link.destination).await.map_err(|error| {
                    format!("Failed to create directory symlink {} -> {}: {}", link.destination.display(), link.source.display(), error)
                })?;
            } else {
                tokio::fs::symlink_file(&link.source, &link.destination).await.map_err(|error| {
                    format!("Failed to create file symlink {} -> {}: {}", link.destination.display(), link.source.display(), error)
                })?;
            }
        }

        Ok(())
    }

    async fn delete_if_exists(link: &Symlink) -> Result<(), String> {
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
            return Err(format!("Refusing to delete non-symlink destination: {}", link.destination.display()));
        }

        if !Self::is_correct_symlink(link).await? {
            return Err(format!(
                "Refusing to delete symlink because its target does not match the registered source: {}",
                link.destination.display()
            ));
        }

        #[cfg(windows)]
        {
            match tokio::fs::remove_file(&link.destination).await {
                Ok(()) => Ok(()),
                Err(file_error) => match tokio::fs::remove_dir(&link.destination).await {
                    Ok(()) => Ok(()),
                    Err(dir_error) => Err(format!(
                        "Failed to remove symlink {}: file removal failed ({}); directory removal failed ({})",
                        link.destination.display(),
                        file_error,
                        dir_error
                    )),
                },
            }
        }

        #[cfg(not(windows))]
        {
            tokio::fs::remove_file(&link.destination)
                .await
                .map_err(|error| format!("Failed to remove symlink {}: {}", link.destination.display(), error))
        }
    }

    async fn path_exists_without_following_symlinks(path: &Path) -> Result<bool, String> {
        match tokio::fs::symlink_metadata(path).await {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(format!("Failed to inspect path {}: {}", path.display(), error)),
        }
    }

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
            .map_err(|error| format!("Failed to read symlink {}: {}", link.destination.display(), error))?;

        let resolved_target = Self::normalize_symlink_target(&link.destination, &target);

        Ok(same_path(&resolved_target, &link.source))
    }

    fn normalize_symlink_target(destination: &Path, target: &Path) -> PathBuf {
        if target.is_absolute() {
            Self::normalize_path(target)
        } else {
            let parent = destination.parent().unwrap_or_else(|| Path::new(""));
            Self::normalize_path(&parent.join(target))
        }
    }

    fn normalize_path(path: &Path) -> PathBuf {
        let mut normalized = PathBuf::new();

        for component in path.components() {
            match component {
                Component::CurDir => {}
                Component::ParentDir => {
                    normalized.pop();
                }
                other => normalized.push(other.as_os_str()),
            }
        }

        normalized
    }
}

// -------------------------------------------------------------------------
// Manager access
// -------------------------------------------------------------------------

async fn manager() -> Result<MutexGuard<'static, SymlinkManager>, String> {
    let mut manager = SYMLINKS.lock().await;

    if !manager.enabled {
        manager.initialize().await?;
    }

    Ok(manager)
}

// -------------------------------------------------------------------------
// Persistent custom symlink API
// -------------------------------------------------------------------------

/// Return all persistent custom symlinks.
pub async fn symlinks() -> Result<Vec<Symlink>, String> {
    Ok(manager().await?.symlinks.clone())
}

/// Create a persistent custom symlink from a frontend request.
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

// -------------------------------------------------------------------------
// Runtime symlink API
// -------------------------------------------------------------------------

/// Return runtime symlinks known to the manager in this process.
pub async fn runtime_symlinks() -> Result<HashMap<String, Vec<Symlink>>, String> {
    Ok(manager().await?.runtime_symlinks.clone())
}

/// Load and return persisted runtime symlinks for one profile.
pub async fn runtime_symlinks_for_profile(profile_id: &str) -> Result<Vec<Symlink>, String> {
    manager().await?.load_runtime_symlinks(profile_id).await
}

/// Create a runtime symlink for a profile.
///
/// The name is retained for compatibility with the previous runtime API.
pub async fn create_temporary(profile_id: &str, source: PathBuf, destination: PathBuf) -> Result<Symlink, String> {
    manager().await?.create_runtime_link(profile_id, source, destination).await
}

/// Reconcile registered runtime links with the complete desired link set.
pub async fn reconcile_runtime(profile_id: &str, desired: Vec<(PathBuf, PathBuf)>) -> Result<(), String> {
    manager().await?.reconcile_runtime(profile_id, desired).await
}

/// Remove registered runtime links for one profile.
///
/// Intended for explicit profile reset or deletion, not normal launch.
pub async fn cleanup_profile(profile_id: &str) -> Result<(), String> {
    manager().await?.cleanup_profile(profile_id).await
}

// -------------------------------------------------------------------------
// Launcher lifecycle
// -------------------------------------------------------------------------

/// Disable persistent custom symlinks and persist their disabled state.
///
/// Runtime symlinks are intentionally left in place because their registry is
/// reconciled on the next launch.
pub async fn cleanup() -> Result<(), String> {
    manager().await?.cleanup().await
}

// -------------------------------------------------------------------------
// Path helpers
// -------------------------------------------------------------------------

fn same_path(left: &Path, right: &Path) -> bool {
    path_key(left) == path_key(right)
}

fn path_key(path: &Path) -> String {
    let normalized = SymlinkManager::normalize_path(path);
    let value = normalized.to_string_lossy().to_string();

    #[cfg(windows)]
    {
        value.to_lowercase()
    }

    #[cfg(not(windows))]
    {
        value
    }
}
