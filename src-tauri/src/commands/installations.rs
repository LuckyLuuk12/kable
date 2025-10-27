use crate::installations::*;
/// Gets all versions, either from cache or by building them, does not modify the cache
#[tauri::command]
pub async fn get_versions() -> Versions {
    crate::installations::get_versions().await
}

/// Gets all versions, either from cache or by building them
#[tauri::command]
pub async fn get_all_versions(force: bool) -> Versions {
    crate::installations::get_all_versions(force).await
}

#[tauri::command]
pub async fn get_version(version_id: String) -> Option<VersionData> {
    crate::installations::get_version(version_id).await
}

/// Returns all Kable installations, using cache. Ensures conversion if needed.
#[tauri::command]
pub async fn get_installations() -> Result<Vec<KableInstallation>, String> {
    crate::installations::get_installations().await
}

/// Returns a single installation by id, using cache.
#[tauri::command]
pub async fn get_installation(id: &str) -> Result<Option<KableInstallation>, String> {
    crate::installations::get_installation(id).await
}

/// Deletes a KableInstallation by ID from kable_profiles.json and invalidates cache
#[tauri::command]
pub async fn delete_installation(id: &str) -> Result<(), String> {
    crate::installations::delete_installation(id).await
}

/// Modifies an existing KableInstallation by ID in kable_profiles.json and invalidates cache
#[tauri::command]
pub async fn modify_installation(
    id: &str,
    new_installation: KableInstallation,
) -> Result<(), String> {
    crate::installations::modify_installation(id, new_installation).await
}

/// Creates a new KableInstallation with the given version_id, using default settings for other fields and invalidates cache
#[tauri::command]
pub async fn create_installation(version_id: &str) -> Result<KableInstallation, String> {
    crate::installations::create_installation(version_id).await
}

/// Creates a new KableInstallation by copying from an existing one
/// Optionally copies mods (with version updates), resource packs, and shaders
#[tauri::command]
pub async fn create_installation_from_existing(
    version_id: &str,
    source_installation_id: &str,
    copy_mods: bool,
    copy_resource_packs: bool,
    copy_shaders: bool,
) -> Result<KableInstallation, String> {
    crate::installations::create_installation_from_existing(
        version_id,
        source_installation_id,
        copy_mods,
        copy_resource_packs,
        copy_shaders,
    )
    .await
}

#[tauri::command]
pub async fn get_mod_info(installation: KableInstallation) -> Result<Vec<ModJarInfo>, String> {
    installation.get_mod_info()
}

/// Disable a mod by moving the jar into the installation's disabled/ subfolder
#[tauri::command]
pub async fn disable_mod(installation: KableInstallation, file_name: String) -> Result<(), String> {
    installation.disable_mod(&file_name)
}

/// Enable a mod by moving the jar out of the installation's disabled/ subfolder
#[tauri::command]
pub async fn enable_mod(installation: KableInstallation, file_name: String) -> Result<(), String> {
    installation.enable_mod(&file_name)
}

/// Toggle the disabled state for a mod; returns the new disabled state (true = disabled)
#[tauri::command]
pub async fn toggle_mod_disabled(
    installation: KableInstallation,
    file_name: String,
) -> Result<bool, String> {
    installation.toggle_mod_disabled(&file_name)
}

#[tauri::command]
pub async fn import(path: String) -> Result<KableInstallation, String> {
    KableInstallation::import(&path).await
}

#[tauri::command]
pub async fn export(installation: KableInstallation) -> Result<String, String> {
    installation.export().await
}

#[tauri::command]
pub async fn duplicate(installation: KableInstallation) -> Result<Vec<KableInstallation>, String> {
    installation.duplicate()
}
