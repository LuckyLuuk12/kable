use api_types::{
    mods::{KableMod, ModrinthResults, Project, ProjectSearch},
    profiles::KableProfile,
};

use crate::features::mods::{browser, management, mrpack};

#[tauri::command]
#[specta::specta]
pub async fn browse(profile: KableProfile, search: ProjectSearch, smart_filter: bool) -> Result<ModrinthResults, String> {
    browser::browse(profile, search, smart_filter).await
}

#[tauri::command]
#[specta::specta]
pub async fn list_mods(profile: KableProfile) -> Result<Vec<KableMod>, String> {
    management::list_mods(profile).await
}

#[tauri::command]
#[specta::specta]
pub async fn remove_mod(profile: KableProfile, kable_mod: KableMod) -> Result<KableMod, String> {
    management::remove_mod(profile, kable_mod).await
}

#[tauri::command]
#[specta::specta]
pub async fn download_mod(profile: KableProfile, project: Project, version_id: Option<&str>) -> Result<KableMod, String> {
    management::download_mod(profile, project, version_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn enable_mod(profile: KableProfile, kable_mod: KableMod) -> Result<KableMod, String> {
    management::enable_mod(profile, kable_mod).await
}

#[tauri::command]
#[specta::specta]
pub async fn disable_mod(profile: KableProfile, kable_mod: KableMod) -> Result<KableMod, String> {
    management::disable_mod(profile, kable_mod).await
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_mod(profile: KableProfile, kable_mod: KableMod) -> Result<KableMod, String> {
    management::toggle_mod(profile, kable_mod).await
}

#[tauri::command]
#[specta::specta]
pub async fn check_for_mod_update(kable_mod: KableMod) -> Result<Option<Project>, String> {
    management::check_for_update(kable_mod).await
}

#[tauri::command]
#[specta::specta]
pub async fn check_for_mod_updates(profile: KableProfile) -> Result<Vec<(KableMod, Project)>, String> {
    management::check_for_updates(profile).await
}

#[tauri::command]
#[specta::specta]
pub async fn update_mod(profile: KableProfile, kable_mod: KableMod) -> Result<KableMod, String> {
    management::update_mod(profile, kable_mod).await
}

#[tauri::command]
#[specta::specta]
pub async fn update_all_mods(profile: KableProfile) -> Result<Vec<KableMod>, String> {
    management::update_all_mods(profile).await
}
