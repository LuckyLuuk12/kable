use crate::installations::kable_profiles::ShaderPackInfo;
use crate::installations::*;
use tauri_plugin_dialog::DialogExt;

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

/// Force refresh version manifests from the network
#[tauri::command]
pub async fn refresh_version_manifests() -> Versions {
    crate::installations::get_all_versions(true).await
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

/// Force refresh installations, bypassing cache
#[tauri::command]
pub async fn get_installations_force() -> Result<Vec<KableInstallation>, String> {
    crate::installations::get_installations_force().await
}

/// Convenience command to refresh installations and update cache
#[tauri::command]
pub async fn refresh_installations() -> Result<Vec<KableInstallation>, String> {
    crate::installations::get_installations_force().await
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

/// Get resource pack info for an installation
#[tauri::command]
pub async fn get_resourcepack_info_for_installation(
    installation: KableInstallation,
) -> Result<Vec<ResourcePackInfo>, String> {
    installation.get_resourcepack_info()
}

/// Get global resource packs from .minecraft/resourcepacks
#[tauri::command]
pub async fn get_global_resourcepacks() -> Result<Vec<ResourcePackInfo>, String> {
    KableInstallation::get_global_resourcepacks()
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

/// Delete/remove a mod from the installation
#[tauri::command]
pub async fn delete_mod(installation: KableInstallation, file_name: String) -> Result<(), String> {
    installation.delete_mod(&file_name)
}

/// Disable a resource pack by moving it into the installation's disabled/ subfolder
#[tauri::command]
pub async fn disable_resourcepack_for_installation(
    installation: KableInstallation,
    file_name: String,
) -> Result<(), String> {
    installation.disable_resourcepack(&file_name)
}

/// Enable a resource pack by moving it out of the installation's disabled/ subfolder
#[tauri::command]
pub async fn enable_resourcepack_for_installation(
    installation: KableInstallation,
    file_name: String,
) -> Result<(), String> {
    installation.enable_resourcepack(&file_name)
}

/// Toggle the disabled state for a resource pack; returns the new disabled state (true = disabled)
#[tauri::command]
pub async fn toggle_resourcepack_disabled_for_installation(
    installation: KableInstallation,
    file_name: String,
) -> Result<bool, String> {
    installation.toggle_resourcepack_disabled(&file_name)
}

/// Delete/remove a resource pack from the installation
#[tauri::command]
pub async fn delete_resourcepack_for_installation(
    installation: KableInstallation,
    file_name: String,
) -> Result<(), String> {
    installation.delete_resourcepack(&file_name)
}

/// Update resource pack order and merging settings for an installation
#[tauri::command]
pub async fn update_resourcepack_settings(
    installation_id: String,
    enable_pack_merging: bool,
    pack_order: Vec<String>,
) -> Result<(), String> {
    // Read all installations
    let mut installations =
        crate::installations::kable_profiles::read_kable_profiles_async().await?;

    // Find and update the installation
    let installation = installations
        .iter_mut()
        .find(|i| i.id == installation_id)
        .ok_or_else(|| format!("Installation not found: {}", installation_id))?;

    // Update settings
    installation.enable_pack_merging = enable_pack_merging;
    installation.pack_order = pack_order;

    // Write back all installations
    crate::installations::kable_profiles::write_kable_profiles_async(&installations).await?;

    // Refresh symlinks to apply changes
    let minecraft_dir = crate::get_default_minecraft_dir()?;
    crate::symlink_manager::SymlinkManager::new(minecraft_dir)
        .setup_resourcepack_symlinks(&installation_id)
        .await?;

    Ok(())
}

/// Get shader pack info for an installation
#[tauri::command]
pub async fn get_shaderpack_info_for_installation(
    installation: KableInstallation,
) -> Result<Vec<ShaderPackInfo>, String> {
    installation.get_shaderpack_info()
}

/// Get global shader packs from .minecraft/shaderpacks
#[tauri::command]
pub async fn get_global_shaderpacks() -> Result<Vec<ShaderPackInfo>, String> {
    KableInstallation::get_global_shaderpacks()
}

/// Disable a shader pack by moving it into the installation's disabled/ subfolder
#[tauri::command]
pub async fn disable_shader_for_installation(
    installation: KableInstallation,
    file_name: String,
) -> Result<(), String> {
    installation.disable_shader(&file_name)
}

/// Enable a shader pack by moving it out of the installation's disabled/ subfolder
#[tauri::command]
pub async fn enable_shader_for_installation(
    installation: KableInstallation,
    file_name: String,
) -> Result<(), String> {
    installation.enable_shader(&file_name)
}

/// Toggle the disabled state for a shader pack; returns the new disabled state (true = disabled)
#[tauri::command]
pub async fn toggle_shader_disabled_for_installation(
    installation: KableInstallation,
    file_name: String,
) -> Result<bool, String> {
    installation.toggle_shader_disabled(&file_name)
}

/// Delete/remove a shader pack from the installation
#[tauri::command]
pub async fn delete_shader_for_installation(
    installation: KableInstallation,
    file_name: String,
) -> Result<(), String> {
    installation.delete_shader(&file_name)
}

#[tauri::command]
pub async fn import(path: String) -> Result<KableInstallation, String> {
    KableInstallation::import(&path).await
}

#[tauri::command]
pub async fn import_from_minecraft_folder(path: String) -> Result<Vec<KableInstallation>, String> {
    KableInstallation::import_from_minecraft_folder(&path).await
}

#[tauri::command]
pub async fn export(installation: KableInstallation) -> Result<String, String> {
    installation.export().await
}

#[tauri::command]
pub async fn duplicate(installation: KableInstallation) -> Result<Vec<KableInstallation>, String> {
    installation.duplicate()
}

#[tauri::command]
pub async fn create_shortcut(installation: KableInstallation) -> Result<String, String> {
    installation.create_desktop_shortcut()
}

/// Select a zip file for importing a Kable installation
#[tauri::command]
pub async fn select_installation_zip(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let file_path = app
        .dialog()
        .file()
        .add_filter("Kable Installation", &["zip"])
        .set_title("Import Kable Installation")
        .blocking_pick_file();

    match file_path {
        Some(path) => match path.as_path() {
            Some(path_buf) => Ok(Some(path_buf.to_string_lossy().to_string())),
            None => Err("Invalid file path".to_string()),
        },
        None => Ok(None),
    }
}

/// Select a .minecraft folder for importing installations
#[tauri::command]
pub async fn select_minecraft_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let folder_path = app
        .dialog()
        .file()
        .set_title("Import from .minecraft Folder")
        .blocking_pick_folder();

    match folder_path {
        Some(path) => match path.as_path() {
            Some(path_buf) => Ok(Some(path_buf.to_string_lossy().to_string())),
            None => Err("Invalid folder path".to_string()),
        },
        None => Ok(None),
    }
}
