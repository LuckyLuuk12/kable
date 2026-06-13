use crate::features::profiles::management;
use crate::integrations::minecraft::versions;
use api_types::installations::{KableInstallation, VersionData};

#[tauri::command]
pub async fn get_installations() -> Result<Vec<KableInstallation>, String> {
    crate::features::profiles::read_kable_profiles().await
}

#[tauri::command]
pub async fn get_installation(id: String) -> Result<Option<KableInstallation>, String> {
    crate::features::profiles::get_installation(&id).await
}

#[tauri::command]
pub async fn delete_installation(id: String) -> Result<(), String> {
    management::delete_installation(&id).await
}

#[tauri::command]
pub async fn modify_installation(
    id: String,
    new_installation: KableInstallation,
) -> Result<(), String> {
    management::modify_installation(&id, new_installation).await
}

#[tauri::command]
pub async fn create_installation(version_id: String) -> Result<KableInstallation, String> {
    crate::features::profiles::create::create_installation(&version_id).await
}

#[tauri::command]
pub fn get_versions(force_refresh: bool) -> Result<Vec<VersionData>, String> {
    versions::get_vanilla_versions(force_refresh)
}
