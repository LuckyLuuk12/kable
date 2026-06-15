use crate::features::profiles::management;
use crate::integrations::minecraft::versions;
use api_types::profiles::{KableProfile, VersionData};

#[tauri::command]
pub async fn get_profiles() -> Result<Vec<KableProfile>, String> {
    crate::features::profiles::read_kable_profiles().await
}

#[tauri::command]
pub async fn get_profile(id: String) -> Result<Option<KableProfile>, String> {
    crate::features::profiles::get_profile(&id).await
}

#[tauri::command]
pub async fn delete_profile(id: String) -> Result<Vec<KableProfile>, String> {
    management::delete_profile(&id).await
}

#[tauri::command]
pub async fn modify_profile(id: String, new_installation: KableProfile) -> Result<KableProfile, String> {
    management::modify_profile(&id, new_installation).await
}

// #[tauri::command]
// pub async fn create_installation(version_id: String) -> Result<KableInstallation, String> {
//     crate::features::profiles::create::create_installation(&version_id).await
// }
// We should make a create_profile() function that wraps all ways of creating a profile so:
// - from version_data (this data should include the version id and loader)
// - from exported profile data (should be a zip with all info required to recreate the profile exactly, including mods, resource packs, etc. This is for profile sharing)
// - from an (older) profile with newer/other version_data
// - from 1 (or more) mrpack's
// - from any combination of the above, where the user can choose which data to take from each source (e.g. version data from version_data, mods from mrpack, etc.)

#[tauri::command]
pub fn get_versions(force_refresh: bool) -> Result<Vec<VersionData>, String> {
    versions::get_vanilla_versions(force_refresh)
}
