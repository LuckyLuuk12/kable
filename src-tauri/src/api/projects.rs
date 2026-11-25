use std::collections::HashMap;

use api_types::{
    profiles::KableProfile,
    projects::{KableProject, ModrinthResults, ProfileProjects, Project, ProjectSearch, ProjectType, ResolvedDependency, UpdateMap},
};

use crate::features::projects::{browser, dependencies, liked, management};

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
pub async fn list_profile_projects(
    profile: KableProfile,
    enabled: bool,
    force_refresh: bool,
) -> Result<HashMap<ProjectType, Vec<KableProject>>, String> {
    management::list_profile_projects(profile, enabled, force_refresh).await
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
pub async fn check_for_update(kable_profile: KableProfile, kable_project: KableProject) -> Result<Vec<UpdateMap>, String> {
    management::check_for_update(kable_profile, kable_project).await
}

#[tauri::command]
#[specta::specta]
pub async fn check_for_updates(profile: KableProfile, project_type: ProjectType) -> Result<Vec<UpdateMap>, String> {
    management::check_for_updates(profile, project_type).await
}

#[tauri::command]
#[specta::specta]
pub async fn update_project(profile: KableProfile, kable_project: KableProject, version_id: Option<&str>) -> Result<KableProject, String> {
    management::update_project(profile, kable_project, version_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn update_all_projects(profile: KableProfile, project_type: ProjectType) -> Result<Vec<KableProject>, String> {
    management::update_all_projects(profile, project_type).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_modrinth_categories() -> Result<Vec<String>, String> {
    crate::integrations::modrinth::ferinth_client::get_categories().await
}

#[tauri::command]
#[specta::specta]
pub async fn get_modrinth_loaders() -> Result<Vec<String>, String> {
    crate::integrations::modrinth::ferinth_client::get_loaders().await
}

#[tauri::command]
#[specta::specta]
pub async fn get_modrinth_game_versions() -> Result<Vec<String>, String> {
    crate::integrations::modrinth::ferinth_client::get_game_versions().await
}

#[tauri::command]
#[specta::specta]
pub async fn get_modrinth_project_types() -> Result<Vec<String>, String> {
    crate::integrations::modrinth::ferinth_client::get_project_types().await
}

#[tauri::command]
#[specta::specta]
pub async fn get_modrinth_side_types() -> Result<Vec<String>, String> {
    crate::integrations::modrinth::ferinth_client::get_side_types().await
}

#[tauri::command]
#[specta::specta]
pub async fn resolve_project_dependencies(
    profile: KableProfile,
    project: Project,
    version_id: &str,
) -> Result<Vec<ResolvedDependency>, String> {
    dependencies::resolve_project_dependencies(profile, project, version_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn list_profile_projects_with_status(profile: KableProfile, force_refresh: bool) -> Result<ProfileProjects, String> {
    management::list_profile_projects_with_status(profile, force_refresh).await
}

#[tauri::command]
#[specta::specta]
pub async fn list_liked_projects() -> Result<Vec<Project>, String> {
    liked::list_liked_projects().await
}

#[tauri::command]
#[specta::specta]
pub fn like_project(project_id: String, project_type: String) -> Result<bool, String> {
    liked::like_project(project_id, project_type)
}

#[tauri::command]
#[specta::specta]
pub fn unlike_project(project_id: String, project_type: String) -> Result<bool, String> {
    liked::unlike_project(&project_id, &project_type)
}

#[tauri::command]
#[specta::specta]
pub fn toggle_like(project_id: String, project_type: String) -> Result<bool, String> {
    liked::toggle_like(project_id, project_type)
}
