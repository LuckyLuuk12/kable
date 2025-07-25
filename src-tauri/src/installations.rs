pub mod kable_profiles;
pub mod profiles;
pub mod versions;

use crate::installations::kable_profiles::*;
use crate::installations::profiles::*;
use crate::installations::versions::*;
use std::sync::OnceLock;
use std::path::PathBuf;
use once_cell::sync::OnceCell;

// Internal cache for installations
static INSTALLATIONS_CACHE: OnceCell<Vec<KableInstallation>> = OnceCell::new();
fn build_installations() -> Result<Vec<KableInstallation>, String> {
    // Here you could ensure conversion from launcher_profiles if needed
    // For now, just load kable_profiles
    let installations = read_kable_profiles()?;
    Ok(installations)
}

//!
//! Public API:
//!


/// Gets all versions, either from cache or by building them, does not modify the cache
#[tauri::command]
pub fn get_versions() -> Versions {
    VERSIONS_CACHE.get_or_init(build_versions).clone()
}

/// Gets all versions, either from cache or by building them
#[tauri::command]
pub fn get_all_versions(force: bool) -> Versions {
    if force {
        let versions = build_versions();
        VERSIONS_CACHE.set(versions.clone()).ok();
        versions
    } else {
        VERSIONS_CACHE.get_or_init(|| build_versions()).clone()
    }
}

#[tauri::command]
pub fn get_version(version_id: String) -> Option<VersionData> {
    let versions = get_versions();
    versions.get_version(&version_id).cloned()
}


/// Returns all Kable installations, using cache. Ensures conversion if needed.
#[tauri::command]
pub fn get_installations() -> Result<Vec<KableInstallation>, String> {
    if let Some(cached) = INSTALLATIONS_CACHE.get() {
        return Ok(cached.clone());
    }
    let installations = build_installations()?;
    INSTALLATIONS_CACHE.set(installations.clone()).ok();
    Ok(installations)
}

/// Returns a single installation by id, using cache.
#[tauri::command]
pub fn get_installation(id: &str) -> Result<Option<KableInstallation>, String> {
    let installations = get_installations()?;
    Ok(installations.into_iter().find(|i| i.id == id))
}
/// Deletes a KableInstallation by ID from kable_profiles.json and invalidates cache
#[tauri::command]
pub fn delete_installation(id: &str) -> Result<(), String> {
    let mut installations = read_kable_profiles()?;
    let orig_len = installations.len();
    installations.retain(|i| i.id != id);
    if installations.len() == orig_len {
        return Err(format!("No Kable installation found with id: {}", id));
    }
    INSTALLATIONS_CACHE.take();
    write_kable_profiles(&installations)
}

/// Modifies an existing KableInstallation by ID in kable_profiles.json and invalidates cache
#[tauri::command]
pub fn modify_installation(id: &str, new_installation: KableInstallation) -> Result<(), String> {
    let mut installations = read_kable_profiles()?;
    let index = installations.iter().position(|i| i.id == id);
    if let Some(index) = index {
        installations[index] = new_installation;
        INSTALLATIONS_CACHE.take();
        write_kable_profiles(&installations)
    } else {
        Err(format!("No Kable installation found with id: {}", id))
    }
}

/// Creates a new KableInstallation with the given version_id, using default settings for other fields and invalidates cache
#[tauri::command]
pub fn create_installation(version_id: &str) -> Result<KableInstallation, String> {
    let mut installations = read_kable_profiles()?;
    // Generate a default name (e.g., based on version_id and count)
    let base_name = version_id.to_string();
    let mut name = base_name.clone();
    let mut count = 1;
    while installations.iter().any(|i| i.name == name) {
        name = format!("{}-{}", base_name, count);
        count += 1;
    }
    let versions = get_versions();
    let version_data = versions.get_version(version_id).cloned().ok_or_else(|| format!("No version found for id: {}", version_id))?;
    let mut new_installation = KableInstallation::default();
    new_installation.name = name;
    new_installation.version_id = version_id.to_string();
    // Optionally set other fields if needed
    installations.push(new_installation.clone());
    INSTALLATIONS_CACHE.take();
    write_kable_profiles(&installations)?;
    Ok(new_installation)
}
