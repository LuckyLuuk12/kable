#[tauri::command]
pub async fn download_or_prepare_mod(
    provider: crate::mods::ProviderKind,
    mod_id: String,
    version_id: Option<String>,
    installation: crate::installations::kable_profiles::KableInstallation,
) -> Result<serde_json::Value, String> {
    // Call backend logic to handle mod or modpack
    match crate::mods::download_or_prepare_mod(
        provider,
        &mod_id,
        version_id.as_deref(),
        &installation,
    )
    .await
    {
        Ok(crate::mods::DownloadOrPrepareResult::ModInstalled) => {
            Ok(serde_json::json!({"success": true}))
        }
        Ok(crate::mods::DownloadOrPrepareResult::Modpack { modpack, context }) => {
            Ok(serde_json::json!({"modpack": modpack, "context": context}))
        }
        Err(e) => Err(e),
    }
}
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[tauri::command]
pub async fn complete_modpack_download(
    instance_id: String,
    modpack_id: String,
    mods: crate::installations::mrpack::PackFileDetailedGroup,
    resourcepacks: crate::installations::mrpack::PackFileDetailedGroup,
    shaderpacks: crate::installations::mrpack::PackFileDetailedGroup,
    install_dir: String,
) -> Result<(), String> {
    crate::mods::complete_modpack_download(
        instance_id,
        modpack_id,
        mods,
        resourcepacks,
        shaderpacks,
        install_dir,
    )
    .await
}

#[tauri::command]
pub async fn prepare_modpack_diff(
    mrpack_path: String,
    installation_dir: String,
    subfolder: Option<String>,
) -> Result<crate::mods::ModpackDiffResult, String> {
    crate::mods::prepare_modpack_diff(mrpack_path, installation_dir, subfolder).await
}

#[tauri::command]
pub async fn apply_modpack_selection(
    installation: crate::installations::kable_profiles::KableInstallation,
    selection: ModpackSelection,
    context: ModpackContext,
) -> Result<(), String> {
    crate::mods::apply_modpack_selection(installation, selection, context).await
}

#[tauri::command]
pub async fn extract_mrpack_and_load_index(
    mrpack_path: String,
    temp_dir: String,
) -> Result<MrpackIndex, String> {
    let mrpack_path = PathBuf::from(mrpack_path);
    let temp_dir = PathBuf::from(temp_dir);
    crate::mrpack::extract_mrpack(&mrpack_path, &temp_dir)
        .map_err(|e| format!("Failed to extract mrpack: {e}"))?;
    let index = crate::mrpack::load_index(&temp_dir)
        .map_err(|e| format!("Failed to load modrinth.index.json: {e}"))?;
    Ok(index)
}

#[tauri::command]
pub async fn list_pack_files_from_index(index: MrpackIndex) -> Result<PackFileGroups, String> {
    Ok(crate::mrpack::list_pack_files(&index))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub conflicts: Vec<PackFileInfo>,
    pub new_files: Vec<PackFileInfo>,
}

#[tauri::command]
pub async fn diff_pack_files_against_profile(
    pack_files: Vec<PackFileInfo>,
    profile_dir: String,
    subfolder: String,
) -> Result<DiffResult, String> {
    let profile_dir = PathBuf::from(profile_dir);
    let (conflicts, new_files) =
        crate::mrpack::diff_pack_files(&pack_files, &profile_dir, &subfolder)
            .map_err(|e| format!("Failed to diff pack files: {e}"))?;
    Ok(DiffResult {
        conflicts,
        new_files,
    })
}

#[tauri::command]
pub async fn copy_selected_pack_files_to_profile(
    extracted_dir: String,
    profile_dir: String,
    files: Vec<PackFileInfo>,
    subfolder: String,
    overwrite: bool,
) -> Result<(), String> {
    let extracted_dir = PathBuf::from(extracted_dir);
    let profile_dir = PathBuf::from(profile_dir);
    crate::mrpack::copy_selected_pack_files(
        &extracted_dir,
        &profile_dir,
        &files,
        &subfolder,
        overwrite,
    )
    .map_err(|e| format!("Failed to copy selected pack files: {e}"))
}

#[tauri::command]
pub async fn extract_overrides_to_profile(
    extracted_dir: String,
    profile_dir: String,
) -> Result<(), String> {
    let extracted_dir = PathBuf::from(extracted_dir);
    let profile_dir = PathBuf::from(profile_dir);
    crate::mrpack::extract_overrides_to_profile(&extracted_dir, &profile_dir)
        .map_err(|e| format!("Failed to extract overrides: {e}"))
}
use crate::installations::kable_profiles::KableInstallation;
use crate::mrpack::MrpackIndex;
use crate::{mods::*, ModJarInfo, PackFileGroups, PackFileInfo};

#[tauri::command]
pub async fn get_mods(provider: ProviderKind, offset: usize) -> Result<Vec<ModInfoKind>, String> {
    crate::mods::get_mods(provider, offset).await
}

#[tauri::command]
pub async fn get_mod_metadata(
    installation: KableInstallation,
    jar_filename: String,
) -> Result<ModMetadata, String> {
    crate::mods::get_mod_metadata(&installation, &jar_filename).await
}

#[tauri::command]
pub async fn download_mod(
    provider: ProviderKind,
    mod_id: String,
    version_id: Option<String>,
    installation: KableInstallation,
) -> Result<(), String> {
    crate::mods::download_mod(provider, &mod_id, version_id.as_deref(), &installation).await
}

#[tauri::command]
pub async fn get_projects(
    provider: ProviderKind,
    project_ids: Vec<String>,
) -> Result<Vec<ModInfoKind>, String> {
    match provider {
        ProviderKind::Modrinth => {
            let projects = modrinth::get_projects(project_ids).await?;
            Ok(projects.into_iter().map(ModInfoKind::Modrinth).collect())
        }
        ProviderKind::CurseForge => {
            Err("CurseForge bulk project fetching not yet implemented".to_string())
        }
    }
}

#[tauri::command]
pub async fn get_project_versions(
    provider: ProviderKind,
    project_id: String,
    loaders: Option<Vec<String>>,
    game_versions: Option<Vec<String>>,
) -> Result<Vec<modrinth::ModrinthVersion>, String> {
    match provider {
        ProviderKind::Modrinth => {
            modrinth::get_project_versions_filtered(&project_id, loaders, game_versions).await
        }
        ProviderKind::CurseForge => {
            Err("CurseForge version fetching not yet implemented".to_string())
        }
    }
}

#[tauri::command]
pub async fn set_provider_filter(
    provider: ProviderKind,
    installation: Option<KableInstallation>,
    filter: Option<ModFilter>,
) {
    crate::mods::set_provider_filter(provider, installation.as_ref(), filter);
}

#[tauri::command]
pub async fn set_provider_limit(provider: ProviderKind, limit: usize) {
    crate::mods::set_provider_limit(provider, limit);
}

#[tauri::command]
pub async fn clear_provider_cache(provider: ProviderKind) {
    crate::mods::clear_provider_cache(provider);
}

#[tauri::command]
pub async fn purge_stale_provider_cache(provider: ProviderKind) {
    crate::mods::purge_stale_provider_cache(provider);
}

#[tauri::command]
pub async fn get_extended_mod_info(mod_jar_info: ModJarInfo) -> Result<ExtendedModInfo, String> {
    crate::mods::get_extended_mod_info(mod_jar_info).await
}
