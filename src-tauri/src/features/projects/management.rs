// List, Remove, Add, Enable/Disable projects, we make this kinda "project type agnostic" so we can use it for mods, resourcepacks, shaderpacks, etc.

use crate::api::browse;
// for now I implement all project features here download/install, remove, enable/disable, update, list, check for updates, etc. Later we can split them into separate files if needed
use crate::integrations::modrinth::client::download_project;
use crate::system::fs;
use api_types::profiles::KableProfile;
use api_types::projects::{KableProject, Project, ProjectSearch, ProjectType};
use futures::{stream, StreamExt, TryStreamExt};

fn get_folder_for(project_type: ProjectType, profile: &KableProfile, enabled: bool) -> Result<std::path::PathBuf, String> {
    // if enabled, append nothing to profile.dedicated_mods_folder, else append disabled
    let path_suffix = if enabled { "" } else { "disabled" };
    let path = std::path::PathBuf::from(match project_type {
        ProjectType::Mod => profile.dedicated_mods_folder.as_ref().ok_or("invalid mods folder path")?,
        ProjectType::Resourcepack => profile.dedicated_resource_pack_folder.as_ref().ok_or("invalid resourcepacks folder path")?,
        ProjectType::Shader => profile.dedicated_shaders_folder.as_ref().ok_or("invalid shaderpacks folder path")?,
        _ => return Err("Unsupported project type for listing".to_string()),
    })
    .join(path_suffix);
    Ok(path)
}

// ? LIST PROJECTS

async fn list_enabled_projects(project_type: ProjectType, profile: &KableProfile) -> Result<Vec<KableProject>, String> {
    let folder = get_folder_for(project_type, profile, true)?;
    let metadata_dir = fs::read_dir(&folder).await?;
    let mut enabled_projects = Vec::new();
    for pathbuf in metadata_dir {
        if let Some(filename) = pathbuf.file_name() {
            if let Some(filename_str) = filename.to_str() {
                if filename_str.ends_with(".json") {
                    let metadata_path = std::path::Path::new(&folder).join(filename_str);
                    let serialized_project = fs::read_str(&metadata_path).await?;
                    let project: KableProject = serde_json::from_str(&serialized_project).map_err(|e| e.to_string())?;
                    enabled_projects.push(project);
                }
            }
        }
    }
    Ok(enabled_projects)
}

async fn list_disabled_projects(project_type: ProjectType, profile: &KableProfile) -> Result<Vec<KableProject>, String> {
    let folder = get_folder_for(project_type, profile, false)?;
    let mut disabled_projects = Vec::new();
    if folder.exists() {
        let metadata_dir = fs::read_dir(&folder).await?;
        for pathbuf in metadata_dir {
            if let Some(filename) = pathbuf.file_name() {
                if let Some(filename_str) = filename.to_str() {
                    if filename_str.ends_with(".json") {
                        let metadata_path = std::path::Path::new(&folder).join(filename_str);
                        let serialized_project = fs::read_str(&metadata_path).await?;
                        let project: KableProject = serde_json::from_str(&serialized_project).map_err(|e| e.to_string())?;
                        disabled_projects.push(project);
                    }
                }
            }
        }
    }
    Ok(disabled_projects)
}

/// Given profile, list all KableProjects
pub async fn list_projects(profile: KableProfile, project_type: ProjectType) -> Result<Vec<KableProject>, String> {
    // do async-join on both list_enabled_projects and list_disabled_projects, then combine the results into a single Vec<KableProject>
    let (enabled_projects, disabled_projects) =
        tokio::join!(list_enabled_projects(project_type.clone(), &profile), list_disabled_projects(project_type, &profile));
    let mut projects = Vec::new();
    projects.extend(enabled_projects?);
    projects.extend(disabled_projects?);
    Ok(projects)
}

// ? REMOVE PROJECTS

/// Given profile and KableProject, remove the jar and json from wherever they are (enabled or disabled), and return the removed KableProject
/// NOTE: metadata filename is always same as jar filename, but with .json extension instead of .jar, which is KableProject.filename
pub async fn remove_project(profile: KableProfile, kable_project: KableProject) -> Result<KableProject, String> {
    let project_type = kable_project.project.project_type.clone();
    let folder = get_folder_for(project_type, &profile, kable_project.enabled)?;
    let metadata_path = std::path::Path::new(&folder).join(&kable_project.filename).with_extension("json");

    fs::remove_file(&metadata_path).await?;
    let jar_path = std::path::Path::new(&folder).join(&kable_project.filename);
    if jar_path.exists() {
        fs::remove_file(&jar_path).await?;
    }

    Ok(kable_project)
}

// ? DOWNLOAD + METADATA

/// Given a profile,and Project, add/download the mod
async fn add_project(profile: KableProfile, project: Project, version_id: Option<&str>) -> Result<KableProject, String> {
    let project_type = project.project_type.clone();
    // 1. get profile project dir
    let folder = get_folder_for(project_type, &profile, true)?;
    // 2. download the mod files to the mods folder
    let filenames = download_project(&project.clone(), version_id, std::path::PathBuf::from(folder)).await?;
    // 3. assume 1 single file is downladed: the mod, and get the filename
    let filename = filenames.get(0).ok_or("No files downloaded")?.to_string();
    // 4. construct KableProject { project, version_id || project.latest_version unwrapped, filename, enabled: true }
    Ok(KableProject {
        project: project.clone(),
        version_id: version_id
            .or_else(|| project.latest_version.as_deref())
            .expect(&format!("No version id was associated with project {}", project.project_id).to_string())
            .to_string(),
        filename,
        enabled: true,
    })
}

async fn add_metadata(profile: KableProfile, kable_project: KableProject) -> Result<KableProject, String> {
    let project_type = kable_project.project.project_type.clone();
    // 1. get profile project dir
    let folder = get_folder_for(project_type, &profile, true)?;
    // 2. construct metadata path
    let metadata_path = std::path::Path::new(&folder).join(&kable_project.filename).with_extension("json");
    // 3. serialize the KableProject and write it to the metadata file
    let serialized_project = serde_json::to_string(&kable_project).map_err(|e| e.to_string())?;
    fs::write_str(&metadata_path, serialized_project.as_str(), false).await?;
    Ok(kable_project)
}

pub async fn download_kable_project(profile: KableProfile, project: Project, version_id: Option<&str>) -> Result<KableProject, String> {
    let kable_project = add_project(profile.clone(), project, version_id).await?;
    add_metadata(profile, kable_project.clone()).await
}

// ? ENABLE/DISABLE PROJECTS

pub async fn enable_project(profile: KableProfile, kable_project: KableProject) -> Result<KableProject, String> {
    if kable_project.enabled {
        return Err(format!("Project {} is already enabled", kable_project.filename));
    }
    let folder = get_folder_for(kable_project.project.project_type.clone(), &profile, true)?;
    let disabled_metadata_path = std::path::Path::new(&folder).join("disabled").join(&kable_project.filename).with_extension("json");
    let disabled_jar_path = std::path::Path::new(&folder).join("disabled").join(&kable_project.filename);

    if disabled_metadata_path.exists() {
        let enabled_metadata_path = std::path::Path::new(&folder).join(&kable_project.filename).with_extension("json");
        fs::rename(&disabled_metadata_path, &enabled_metadata_path).await?;
    }

    if disabled_jar_path.exists() {
        let enabled_jar_path = std::path::Path::new(&folder).join(&kable_project.filename);
        fs::rename(&disabled_jar_path, &enabled_jar_path).await?;
    }

    Ok(KableProject { enabled: true, ..kable_project })
}

pub async fn disable_project(profile: KableProfile, kable_project: KableProject) -> Result<KableProject, String> {
    if !kable_project.enabled {
        return Err(format!("Project {} is already disabled", kable_project.filename));
    }
    let folder = get_folder_for(kable_project.project.project_type.clone(), &profile, true)?;
    let enabled_metadata_path = std::path::Path::new(&folder).join(&kable_project.filename).with_extension("json");
    let enabled_jar_path = std::path::Path::new(&folder).join(&kable_project.filename);

    if enabled_metadata_path.exists() {
        let disabled_metadata_path = std::path::Path::new(&folder).join("disabled").join(&kable_project.filename).with_extension("json");
        fs::rename(&enabled_metadata_path, &disabled_metadata_path).await?;
    }

    if enabled_jar_path.exists() {
        let disabled_jar_path = std::path::Path::new(&folder).join("disabled").join(&kable_project.filename);
        fs::rename(&enabled_jar_path, &disabled_jar_path).await?;
    }

    Ok(KableProject { enabled: false, ..kable_project })
}

pub async fn toggle_project(profile: KableProfile, kable_project: KableProject) -> Result<KableProject, String> {
    if kable_project.enabled {
        disable_project(profile, kable_project).await
    } else {
        enable_project(profile, kable_project).await
    }
}

// ? UPDATE + CHECK FOR UPDATES

/// Search again for this profile and specific KableProject project id, check if the latest version is different from the current version, if so return Some(Project) with the latest version, else return None
pub async fn check_for_update(kable_profile: KableProfile, kable_project: KableProject) -> Result<Option<Project>, String> {
    let project_search = ProjectSearch { query: Some(kable_project.project.project_id.clone()), ..Default::default() };
    let results = browse(kable_profile, project_search, true, kable_project.project.project_type.clone()).await?;
    if let Some(latest_project) = results.hits.into_iter().next() {
        if latest_project.latest_version != Some(kable_project.version_id.clone()) {
            Ok(Some(latest_project))
        } else {
            Ok(None)
        }
    } else {
        Err(format!("No project found for mod {}", kable_project.filename))
    }
}

pub async fn check_for_updates(profile: KableProfile, project_type: ProjectType) -> Result<Vec<(KableProject, Project)>, String> {
    let projects = list_projects(profile.clone(), project_type).await?;
    let mut updates = Vec::new();
    for kable_project in projects {
        if let Some(latest_project) = check_for_update(profile.clone(), kable_project.clone()).await? {
            updates.push((kable_project, latest_project));
        }
    }
    Ok(updates)
}

pub async fn update_project(profile: KableProfile, kable_project: KableProject) -> Result<KableProject, String> {
    if let Some(latest_project) = check_for_update(profile.clone(), kable_project.clone()).await? {
        let latest_version_id = latest_project.latest_version.as_deref();
        let updated_project = download_kable_project(profile.clone(), latest_project.clone(), latest_version_id).await?;
        Ok(updated_project)
    } else {
        Err(format!("No update available for project {}", kable_project.filename))
    }
}

pub async fn update_all_projects(profile: KableProfile, project_type: ProjectType) -> Result<Vec<KableProject>, String> {
    let updates = check_for_updates(profile.clone(), project_type).await?;

    stream::iter(updates)
        .map(|(kable_project, _)| {
            let profile = profile.clone();
            async move { update_project(profile, kable_project).await }
        })
        .buffer_unordered(8) // TODO: read from NetworkSettings the max_threads.
        .try_collect()
        .await
}
