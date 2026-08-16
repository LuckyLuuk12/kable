use std::collections::HashMap;

use api_types::{
    profiles::KableProfile,
    projects::{KableProject, ModrinthResults, Project, ProjectSearch, ProjectType, UpdateMap},
};

use crate::features::projects::{browser, management};

#[tauri::command]
#[specta::specta]
pub async fn browse(
    profile: KableProfile,
    search: ProjectSearch,
    smart_filter: bool,
    project_type: ProjectType,
) -> Result<ModrinthResults, String> {
    browser::browse(profile, search, smart_filter, project_type).await
}

#[tauri::command]
#[specta::specta]
pub async fn list_profile_projects(profile: KableProfile, enabled: bool) -> Result<HashMap<ProjectType, Vec<KableProject>>, String> {
    management::list_profile_projects(profile, enabled).await
}

#[tauri::command]
#[specta::specta]
pub async fn remove_project(profile: KableProfile, kable_project: KableProject) -> Result<KableProject, String> {
    management::remove_project(profile, kable_project).await
}

#[tauri::command]
#[specta::specta]
pub async fn add_project_to_profile(profile: KableProfile, project: Project, version_id: Option<&str>) -> Result<KableProject, String> {
    management::add_project_to_profile(profile, project, version_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn check_for_update(kable_profile: KableProfile, kable_project: KableProject) -> Result<Project, String> {
    management::check_for_update(kable_profile, kable_project).await
}

#[tauri::command]
#[specta::specta]
pub async fn check_for_updates(profile: KableProfile, project_type: ProjectType) -> Result<Vec<UpdateMap>, String> {
    management::check_for_updates(profile, project_type).await
}

#[tauri::command]
#[specta::specta]
pub async fn update_project(profile: KableProfile, kable_project: KableProject) -> Result<KableProject, String> {
    management::update_project(profile, kable_project).await
}

#[tauri::command]
#[specta::specta]
pub async fn update_all_projects(profile: KableProfile, project_type: ProjectType) -> Result<Vec<KableProject>, String> {
    management::update_all_projects(profile, project_type).await
}
