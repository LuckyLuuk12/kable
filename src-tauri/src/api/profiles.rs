use std::path::Path;

use crate::features::profiles::management;
use crate::integrations::loaders;
use api_types::{
    profiles::{KableProfile, Versions},
    projects::KableProject,
};

#[tauri::command]
#[specta::specta]
pub async fn get_profiles() -> Result<Vec<KableProfile>, String> {
    management::list_profiles().await
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
pub async fn create_profile(
    version_id: Option<String>,
    base_profile: Option<KableProfile>,
    exported_profile: Option<std::path::PathBuf>,
    mrpack: Option<String>,
) -> Result<KableProfile, String> {
    crate::features::profiles::create::create_profile(version_id, base_profile, exported_profile, mrpack).await
}
// We should make a create_profile() function that wraps all ways of creating a profile so:
// - from version_data (this data should include the version id and loader)
// - from exported profile data (should be a zip with all info required to recreate the profile exactly, including mods, resource packs, etc. This is for profile sharing)
// - from an (older) profile with newer/other version_data
// - from 1 (or more) mrpack's
// - from any combination of the above, where the user can choose which data to take from each source (e.g. version data from version_data, mods from mrpack, etc.)

#[tauri::command]
#[specta::specta]
pub async fn get_versions() -> Result<Versions, String> {
    loaders::get_versions().await
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_project(profile: KableProfile, kable_project: KableProject) -> Result<KableProfile, String> {
    management::toggle_project(profile, kable_project).await
}
