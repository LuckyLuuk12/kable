// List, Remove, Add, Enable/Disable projects, we make this kinda "project type agnostic" so we can use it for mods, resourcepacks, shaderpacks, etc.
// for now I implement all project features here download/install, remove, enable/disable, update, list, check for updates, etc. Later we can split them into separate files if needed
use crate::api::browse;
use crate::integrations::modrinth::ferinth_client::download_project;
use crate::system::fs;
use api_types::profiles::KableProfile;
use api_types::projects::{KableProject, Project, ProjectSearch, ProjectType, UpdateMap};
use futures::{stream, StreamExt, TryStreamExt};

// TODO: we removed the `enabled` field from KableProject and let the KableProfile.settings.{mods, resourcepacks, shaders} Projects struct handle the enabled/disabled state of each project.
// TODO: So we need to update the above functions to use that instead of the `enabled` field in KableProject.
// We should ensure this module still exposes functions such that we can enabled/disable/remove projects,
// clearly when removing we would have to check for all KableProfiles if the project is disabled in all of them before removing
// and on the frontend it should likely pop up a warning about fully removing a project...

/// List all projects from .kable/projects/<type>/* and return Map<ProjectType, Vec<KableProject>> for all projects.
pub async fn list_all_projects() -> Result<std::collections::HashMap<ProjectType, Vec<KableProject>>, String> {
    let mut all_projects: std::collections::HashMap<ProjectType, Vec<KableProject>> = std::collections::HashMap::new();
    for project_type in &[ProjectType::Mod, ProjectType::Resourcepack, ProjectType::Shader] {
        let projects = list_projects_for_type(*project_type).await?;
        all_projects.insert(*project_type, projects);
    }
    Ok(all_projects)
}

async fn folder_for_project_type(project_type: ProjectType) -> Result<std::path::PathBuf, String> {
    let projects_dir = fs::projects_dir()?;
    let type_folder_name = match project_type {
        ProjectType::Mod => crate::constants::MODS_DIR,
        ProjectType::Resourcepack => crate::constants::RESOURCEPACKS_DIR,
        ProjectType::Shader => crate::constants::SHADERPACKS_DIR,
        _ => return Err(format!("Unsupported project type: {:?}", project_type)),
    };
    let type_dir = projects_dir.join(type_folder_name);
    Ok(type_dir)
}

/// Given a string, check if the project is already installed in the global projects folder, and if so return the KableProject metadata, else return None
pub async fn get_installed_project(project_type: ProjectType, filename: &str) -> Result<Option<KableProject>, String> {
    let type_dir = folder_for_project_type(project_type).await?;
    let metadata_path = std::path::Path::new(&type_dir).join(filename).with_extension("json");
    if metadata_path.exists() {
        let serialized_project = fs::read_str(&metadata_path).await?;
        let project: KableProject = serde_json::from_str(&serialized_project).map_err(|e| e.to_string())?;
        return Ok(Some(project));
    }
    Ok(None)
}

/// List all projects of a specific type from .kable/projects/<type>/* and return Vec<KableProject>
/// We assume that the KableProject metadata is stored in .json files in the projects/<type> folder, and the jar files are stored in the same folder as the metadata.
async fn list_projects_for_type(project_type: ProjectType) -> Result<Vec<KableProject>, String> {
    let type_dir = folder_for_project_type(project_type).await?;
    if !type_dir.exists() {
        return Ok(Vec::new());
    }
    let mut projects = Vec::new();
    let metadata_dir = fs::read_dir(&type_dir).await?;
    for pathbuf in metadata_dir {
        if let Some(filename) = pathbuf.file_name() {
            if let Some(filename_str) = filename.to_str() {
                if filename_str.ends_with(".json") {
                    let metadata_path = std::path::Path::new(&type_dir).join(filename_str);
                    let serialized_project = fs::read_str(&metadata_path).await?;
                    let project: KableProject = serde_json::from_str(&serialized_project).map_err(|e| e.to_string())?;
                    projects.push(project);
                }
            }
        }
    }
    Ok(projects)
}

/// Given a profile, list all `enabled` KableProjects for that profile, in a map of ProjectType to Vec<KableProject>
pub async fn list_profile_projects(
    profile: KableProfile,
    enabled: bool,
) -> Result<std::collections::HashMap<ProjectType, Vec<KableProject>>, String> {
    let mut enabled_projects_map: std::collections::HashMap<ProjectType, Vec<KableProject>> = std::collections::HashMap::new();
    for project_type in [ProjectType::Mod, ProjectType::Resourcepack, ProjectType::Shader] {
        let enabled_project_filenames = profile.settings.by_project_type(project_type, enabled);
        // Using this we just filter in all projects of this type that have a filename matching that in the hashset.
        let enabled_projects = list_projects_for_type(project_type)
            .await?
            .into_iter()
            .filter(|p| enabled_project_filenames.contains(&p.filename))
            .collect::<Vec<KableProject>>();
        enabled_projects_map.insert(project_type, enabled_projects);
    }
    Ok(enabled_projects_map)
}

/// Given a profile and a KableProject, add/download the project to the global projects folder, and add the filename to the profile's settings for that project type (enabled or disabled)
pub async fn add_project_to_profile(profile: KableProfile, project: Project, version_id: Option<&str>) -> Result<KableProject, String> {
    let kable_project = add_project(project, version_id).await?;
    let mut updated_profile = profile.clone();
    updated_profile.settings.toggle(kable_project.project.project_type, &kable_project.filename);
    crate::features::profiles::management::modify_profile(profile, updated_profile.clone()).await?;
    Ok(kable_project)
}

/// Wrapper for the modrinth download_project and metadata creation, which downloads the project to the global projects folder and creates the KableProject metadata file in the same folder.
pub async fn add_project(project: Project, version_id: Option<&str>) -> Result<KableProject, String> {
    // 1. get profile project dir
    let folder = folder_for_project_type(project.project_type).await?;
    // 2. download the mod files to the mods folder
    let filenames = download_project(&project.clone(), version_id, folder.clone()).await?;
    // 3. assume 1 single file is downladed: the mod, and get the filename
    let filename = filenames.first().ok_or("No files downloaded")?.to_string();
    // 4. construct KableProject { project, version_id || project.latest_version unwrapped, filename, enabled: true }
    let metadata = Ok(KableProject {
        project: project.clone(),
        version_id: version_id
            .or(project.latest_version.as_deref())
            .unwrap_or_else(|| panic!("{}", format!("No version id was associated with project {}", project.project_id).to_string()))
            .to_string(),
        filename,
    });
    // 5. create <filename>.json metadata file containing the KableProject in the same folder as the downloaded file
    crate::system::fs::write_str(
        &folder.join(&metadata.as_ref().unwrap().filename).with_extension("json"),
        &serde_json::to_string(&metadata.as_ref().unwrap()).map_err(|e| e.to_string())?,
        false,
    )
    .await?;
    // 6. return the KableProject
    metadata
}

/// Ensures, given a profile that the mods, resourcepacks, and shaderpacks that are enabled in the profile's settings are actually present in the global projects folder, and if not, downloads them again.
pub async fn ensure_profile_projects(profile: KableProfile) -> Result<(), String> {
    for project_type in [ProjectType::Mod, ProjectType::Resourcepack, ProjectType::Shader] {
        let enabled_project_filenames = profile.settings.by_project_type(project_type, true);
        for filename in enabled_project_filenames {
            if let Some(_kable_project) = get_installed_project(project_type, &filename).await? {
                // project is installed, do nothing
            } else {
                // project is not installed, download it again
                let project_search = ProjectSearch { query: Some(filename.clone()), ..Default::default() };
                let results = browse(profile.clone(), project_search, true, project_type).await?;
                if let Some(latest_project) = results.hits.into_iter().next() {
                    let latest_version_id = latest_project.latest_version.as_deref();
                    add_project_to_profile(profile.clone(), latest_project.clone(), latest_version_id).await?;
                } else {
                    return Err(format!("No project found for mod {}", filename));
                }
            }
        }
    }
    Ok(())
}

/// Given a project, find all profiles that have it enabled as safeguard but if there are none, remove it.
pub async fn remove_project(profile: KableProfile, kable_project: KableProject) -> Result<KableProject, String> {
    // 1. remove the filename from the profile's settings for that project type (enabled or disabled)
    let mut updated_profile = profile.clone();
    updated_profile.settings.toggle(kable_project.project.project_type, &kable_project.filename);
    crate::features::profiles::management::modify_profile(profile, updated_profile.clone()).await?;

    // 2. check if any other profiles have this project enabled, if not, remove the project files and metadata
    let all_profiles = crate::features::profiles::management::list_profiles().await?;
    let mut is_enabled_in_any_profile = false;
    for p in all_profiles {
        if p.settings.by_project_type(kable_project.project.project_type, true).contains(&kable_project.filename) {
            is_enabled_in_any_profile = true;
            break;
        }
    }

    if !is_enabled_in_any_profile {
        let folder = folder_for_project_type(kable_project.project.project_type).await?;
        let metadata_path = std::path::Path::new(&folder).join(&kable_project.filename).with_extension("json");
        crate::system::fs::remove_file(&metadata_path).await?;
        let jar_path = std::path::Path::new(&folder).join(&kable_project.filename);
        if jar_path.exists() {
            crate::system::fs::remove_file(&jar_path).await?;
        }
    }

    Ok(kable_project)
}

/// Given a profile check for all enabled projects if there are updates available, and return a Vec<UpdateMap> of all projects that have updates available.
pub async fn check_for_updates(profile: KableProfile, project_type: ProjectType) -> Result<Vec<UpdateMap>, String> {
    let mut updates = Vec::new();
    let enabled_projects = list_profile_projects(profile.clone(), true).await?;
    if let Some(projects) = enabled_projects.get(&project_type) {
        for kable_project in projects {
            if let Ok(latest_project) = check_for_update(profile.clone(), kable_project.clone()).await {
                updates.push(UpdateMap { kable_project: kable_project.clone(), update: Some(latest_project) });
            }
        }
    }
    Ok(updates)
}

/// Given a profile and a KableProject, check for updates and if there is an update available.
pub async fn check_for_update(kable_profile: KableProfile, kable_project: KableProject) -> Result<Project, String> {
    let project_search = ProjectSearch { query: Some(kable_project.project.project_id.clone()), ..Default::default() };
    let results = browse(kable_profile, project_search, true, kable_project.project.project_type).await?;
    if let Some(latest_project) = results.hits.into_iter().next() {
        if latest_project.latest_version != Some(kable_project.version_id.clone()) {
            Ok(latest_project)
        } else {
            Err(format!("No update available for project {}", kable_project.filename))
        }
    } else {
        Err(format!("No project found for mod {}", kable_project.filename))
    }
}

/// Given a profile and a KableProject, check for updates and if there is an update available, download the latest version and update the KableProject metadata.
pub async fn update_project(profile: KableProfile, kable_project: KableProject) -> Result<KableProject, String> {
    let latest_project = check_for_update(profile.clone(), kable_project.clone()).await?;
    let latest_version_id = latest_project.latest_version.as_deref();
    add_project_to_profile(profile.clone(), latest_project.clone(), latest_version_id).await
}

pub async fn update_all_projects(profile: KableProfile, project_type: ProjectType) -> Result<Vec<KableProject>, String> {
    let updates = check_for_updates(profile.clone(), project_type).await?;

    stream::iter(updates)
        .map(|update_map| {
            let profile = profile.clone();
            async move { update_project(profile, update_map.kable_project).await }
        })
        .buffer_unordered(
            crate::features::customization::settings::load_settings().await?.network.max_download_threads.unwrap_or(8) as usize
        )
        .try_collect()
        .await
}

/// Given a profile, go over all projects and use their profile_versions to see if they even support the current profile version
/// If auto_disable is set, then disable, always return a list of incompatible projects.
pub async fn check_incompatible_projects(profile: KableProfile, auto_disable: bool) -> Result<Vec<KableProject>, String> {
    let mut incompatible_projects = Vec::new();
    let enabled_projects = list_profile_projects(profile.clone(), true).await?;
    if profile.version.minecraft_version.is_none() {
        return Err("Profile version is empty, cannot check for incompatible projects".to_string());
    }
    for (_project_type, projects) in enabled_projects {
        for kable_project in projects {
            for project_version in kable_project.clone().project.versions {
                if !project_version.game_versions.contains(&profile.version.minecraft_version.clone().unwrap()) {
                    incompatible_projects.push(kable_project.clone());
                    if auto_disable {
                        // we can safely use toggle as we requested only the enable projects.
                        crate::features::profiles::management::toggle_project(profile.clone(), kable_project.clone()).await?;
                    }
                }
            }
        }
    }
    Ok(incompatible_projects)
}
