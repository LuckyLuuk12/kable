pub mod kable_profiles;
pub mod profiles;
pub mod versions;

pub use self::kable_profiles::*;
pub use self::profiles::*;
pub use self::versions::*;
use tokio::sync::RwLock;
use std::sync::Arc;
use once_cell::sync::Lazy;

/// Ensures that a modded installation has a dedicated mods folder set and created.
/// Returns true if the folder was set/created, false otherwise.
async fn ensure_dedicated_mods_folder(
    installation: &mut KableInstallation,
) -> Result<bool, String> {
    use crate::installations::get_version;
    let version_id = &installation.version_id;
    let version_data = get_version(version_id.clone()).await;
    let is_modded = match &version_data {
        Some(v) => v.loader != LoaderKind::Vanilla,
        None => false,
    };
    if is_modded && installation.dedicated_mods_folder.is_none() {
        // Set mods_folder to installation id
        installation.dedicated_mods_folder = Some(installation.id.clone());
        // Create .minecraft/kable/mods/<id> if not exists
        let minecraft_dir = crate::get_default_minecraft_dir()
            .map_err(|e| format!("Failed to get default Minecraft dir: {e}"))?;
        let mods_dir = minecraft_dir
            .join("kable")
            .join("mods")
            .join(&installation.id);
        if !mods_dir.exists() {
            crate::ensure_folder(&mods_dir)
                .await
                .map_err(|e| format!("Failed to create mods dir: {}", e))?;
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

// Internal cache for installations using RwLock for read/write access
static INSTALLATIONS_CACHE: Lazy<Arc<RwLock<Option<Vec<KableInstallation>>>>> = 
    Lazy::new(|| Arc::new(RwLock::new(None)));

/// Builds the list of installations by merging kable_profiles and converted launcher_profiles.
async fn build_installations_async() -> Result<Vec<KableInstallation>, String> {
    let mut installations = kable_profiles::read_kable_profiles_async().await?;
    crate::logging::debug(&format!("Read {} installations from kable_profiles.json", installations.len()));
    match profiles::read_launcher_profiles_async().await {
        Ok(launcher_profiles) => {
            // Use a tuple of (name, last_version_id, created) for deduplication, all as String
            let kable_keys: std::collections::HashSet<(String, String, String)> = installations
                .iter()
                .map(|i| (i.name.clone(), i.version_id.clone(), i.created.clone()))
                .collect();
            let mut new_converted: Vec<KableInstallation> = launcher_profiles
                .into_iter()
                .map(|lp| lp.into())
                .filter(|ki: &KableInstallation| {
                    let key = (ki.name.clone(), ki.version_id.clone(), ki.created.clone());
                    !kable_keys.contains(&key)
                })
                .collect();
            installations.append(&mut new_converted);
            kable_profiles::write_kable_profiles_async(&installations).await?;
        }
        Err(e) => {
            crate::logging::Logger::warn_global(
                &format!(
                    "Failed to read launcher profiles, only kable profiles will be used. Error: {}",
                    e
                ),
                None,
            );
        }
    }
    crate::logging::debug(&format!("Total installations after merging: {}", installations.len()));
    Ok(installations)
}

// !NOTE: Public API:

static VERSIONS_CACHE: Lazy<Arc<RwLock<Option<Versions>>>> = 
    Lazy::new(|| Arc::new(RwLock::new(None)));

/// Gets all versions, either from cache or by building them, does not modify the cache
pub async fn get_versions() -> Versions {
    {
        let cache_read = VERSIONS_CACHE.read().await;
        if let Some(cached) = cache_read.as_ref() {
            return cached.clone();
        }
    }
    
    // Cache miss - build and cache versions
    let versions = build_versions().await;
    {
        let mut cache_write = VERSIONS_CACHE.write().await;
        *cache_write = Some(versions.clone());
    }
    crate::logging::debug(&format!("Built versions: {} items", versions.len()));
    versions
}

/// Gets all versions, either from cache or by building them
pub async fn get_all_versions(force: bool) -> Versions {
    if force {
        let versions = build_versions().await;
        crate::logging::debug(&format!("Fetched versions: {} items", versions.len()));
        {
            let mut cache_write = VERSIONS_CACHE.write().await;
            *cache_write = Some(versions.clone());
        }
        versions
    } else {
        get_versions().await
    }
}

pub async fn get_version(version_id: String) -> Option<VersionData> {
    let versions = get_versions().await;
    versions.get_version(&version_id).cloned()
}

/// Returns all Kable installations, using cache. Ensures conversion if needed.
pub async fn get_installations() -> Result<Vec<KableInstallation>, String> {
    // Try to get from cache first
    {
        let cache_read = INSTALLATIONS_CACHE.read().await;
        if let Some(cached) = cache_read.as_ref() {
            crate::logging::debug(&format!("Using cached installations: {} items", cached.len()));
            return Ok(cached.clone());
        }
    }
    
    // Cache miss - build and cache installations
    let installations = build_installations_async().await?;
    {
        let mut cache_write = INSTALLATIONS_CACHE.write().await;
        *cache_write = Some(installations.clone());
    }
    crate::logging::debug(&format!("Built installations: {} items", installations.len()));
    Ok(installations)
}

/// Returns a single installation by id, using cache.
pub async fn get_installation(id: &str) -> Result<Option<KableInstallation>, String> {
    let installations = get_installations().await?;
    Ok(installations.into_iter().find(|i| i.id == id))
}
/// Deletes a KableInstallation by ID from kable_profiles.json and invalidates cache
pub async fn delete_installation(id: &str) -> Result<(), String> {
    let mut installations = kable_profiles::read_kable_profiles_async().await?;
    let orig_len = installations.len();
    installations.retain(|i| i.id != id);
    if installations.len() == orig_len {
    crate::logging::Logger::warn_global(&format!("No Kable installation found with id: {}", id), None);
        return Err(format!("No Kable installation found with id: {}", id));
    }
    let result = kable_profiles::write_kable_profiles_async(&installations).await;
    {
        let mut cache_write = INSTALLATIONS_CACHE.write().await;
        *cache_write = Some(installations.clone());
    }
    match &result {
        Ok(_) => crate::logging::info(&format!("Installation '{}' deleted successfully.", id)),
        Err(e) => crate::logging::error(&format!("Failed to delete installation '{}': {}", id, e)),
    }
    result
}

/// Modifies an existing KableInstallation by ID in kable_profiles.json and invalidates cache
pub async fn modify_installation(
    id: &str,
    mut new_installation: KableInstallation,
) -> Result<(), String> {
    let mut installations = kable_profiles::read_kable_profiles_async().await?;
    let index = installations.iter().position(|i| i.id == id);
    if let Some(index) = index {
        // Ensure dedicated mods folder if needed
        let _ = ensure_dedicated_mods_folder(&mut new_installation).await?;
        installations[index] = new_installation;
        let result = kable_profiles::write_kable_profiles_async(&installations).await;
        {
            let mut cache_write = INSTALLATIONS_CACHE.write().await;
            *cache_write = Some(installations.clone());
        }
        match &result {
            Ok(_) => crate::logging::info(&format!("Installation '{}' modified successfully.", id)),
            Err(e) => crate::logging::error(&format!("Failed to modify installation '{}': {}", id, e)),
        }
        result
    } else {
    crate::logging::Logger::warn_global(&format!("No Kable installation found with id: {}", id), None);
        Err(format!("No Kable installation found with id: {}", id))
    }
}

/// Creates a new KableInstallation with the given version_id, using default settings for other fields and invalidates cache
pub async fn create_installation(version_id: &str) -> Result<KableInstallation, String> {
    let mut installations = kable_profiles::read_kable_profiles_async().await?;
    // Generate a default name (e.g., based on version_id and count)
    let base_name = version_id.to_string();
    let mut name = base_name.clone();
    let mut count = 1;
    while installations.iter().any(|i| i.name == name) {
        name = format!("{}-{}", base_name, count);
        count += 1;
    }
    let versions = get_versions().await;
    let version_data = versions
        .get_version(version_id)
        .cloned()
        .ok_or_else(|| format!("No version found for id: {}", version_id))?;
    crate::logging::info(&format!(
        "Creating new installation: name='{}', version_id='{}'",
        name, version_data.version_id
    ));
    let mut new_installation = KableInstallation {
        name,
        version_id: version_data.version_id.clone(),
        ..Default::default()
    };
    // Ensure dedicated mods folder if needed
    let _ = ensure_dedicated_mods_folder(&mut new_installation).await?;
    installations.push(new_installation.clone());
    let result = kable_profiles::write_kable_profiles_async(&installations).await;
    {
        let mut cache_write = INSTALLATIONS_CACHE.write().await;
        *cache_write = Some(installations.clone());
    }
    match &result {
        Ok(_) => crate::logging::info(&format!("Installation '{}' created successfully.", new_installation.name)),
        Err(e) => crate::logging::error(&format!("Failed to create installation '{}': {}", new_installation.name, e)),
    }
    result?;
    Ok(new_installation)
}
