use std::path::{Path, PathBuf};

use crate::features::profiles::{create, import, management};
use crate::integrations::loaders;
use api_types::profiles::CreateProfileRequest;
use api_types::{
    profiles::{KableProfile, Versions},
    projects::KableProject,
};

#[tauri::command]
#[specta::specta]
pub async fn get_profiles(force: bool) -> Result<Vec<KableProfile>, String> {
    management::list_profiles(force).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_profile(id: String) -> Result<KableProfile, String> {
    management::get_profile(&id).await
}

#[tauri::command]
#[specta::specta]
pub async fn delete_profile(id: String) -> Result<(), String> {
    management::delete_profile(&id).await
}

#[tauri::command]
#[specta::specta]
pub async fn modify_profile(old_profile: KableProfile, new_profile: KableProfile) -> Result<KableProfile, String> {
    management::modify_profile(old_profile, new_profile).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_versions() -> Result<Versions, String> {
    loaders::get_versions().await
}

#[tauri::command]
#[specta::specta]
pub async fn refresh_versions() -> Result<Versions, String> {
    loaders::refresh_versions().await
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_project(profile: KableProfile, kable_project: KableProject) -> Result<KableProfile, String> {
    management::toggle_project(profile, kable_project).await
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_favorite(profile: KableProfile) -> Result<KableProfile, String> {
    management::toggle_favorite(profile).await
}

#[tauri::command]
#[specta::specta]
pub async fn is_project_enabled(profile: KableProfile, kable_project: KableProject) -> Result<bool, String> {
    management::is_project_enabled(profile, kable_project).await
}

#[tauri::command]
#[specta::specta]
pub async fn export_profile(profile: KableProfile, destination: String) -> Result<String, String> {
    let path = import::export_profile(&profile, &destination).await?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
#[specta::specta]
pub async fn create_profile(request: CreateProfileRequest) -> Result<KableProfile, String> {
    create::create_profile(request).await
}
// We should make a create_profile() function that wraps all ways of creating a profile so:
// - from version_data (this data should include the version id and loader)
// - from exported profile data (should be a zip with all info required to recreate the profile exactly, including mods, resource packs, etc. This is for profile sharing)
// - from an (older) profile with newer/other version_data
// - from 1 (or more) mrpack's
// - from any combination of the above, where the user can choose which data to take from each source (e.g. version data from version_data, mods from mrpack, etc.)
