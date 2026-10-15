use crate::Logger;
use std::collections::{HashMap, HashSet};

use crate::integrations::modrinth::ferinth_client::{download_project, get_project, get_version_data};
use crate::system::fs;

use api_types::profiles::{KableProfile, KableProfileSettings};
use api_types::projects::{DependencyType, KableProject, Project, ProjectType, ResolvedDependency, UpdateMap};

/// List all projects from the global projects directory.
pub async fn list_all_projects() -> Result<HashMap<ProjectType, Vec<KableProject>>, String> {
    let mut all_projects = HashMap::new();

    for project_type in [ProjectType::Mod, ProjectType::Resourcepack, ProjectType::Shader] {
        all_projects.insert(project_type, list_projects_for_type(project_type).await?);
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

    Ok(projects_dir.join(type_folder_name))
}

fn normalize_loader(loader: &str) -> String {
    let normalized = loader.trim().to_lowercase().replace(['_', '-', ' '], "");

    match normalized.as_str() {
        "irisfabric" | "irisfabricloader" | "fabric" | "quiltedfabric" => "fabric".to_string(),
        "quilt" => "quilt".to_string(),
        "neoforge" => "neoforge".to_string(),
        "forge" => "forge".to_string(),
        "vanilla" => "vanilla".to_string(),
        other => other.to_string(),
    }
}

fn is_version_compatible(profile: &KableProfile, version: &api_types::projects::ProjectVersion, project_type: ProjectType) -> bool {
    let minecraft_compatible = profile
        .version
        .minecraft_version
        .as_ref()
        .map(|minecraft_version| version.game_versions.contains(minecraft_version))
        .unwrap_or(true);

    let loader_relevant = project_type == ProjectType::Mod || project_type == ProjectType::Modpack;
    let profile_loader = normalize_loader(&profile.version.loader.to_string());

    let loader_compatible = !loader_relevant || version.loaders.iter().any(|loader| normalize_loader(loader) == profile_loader);

    minecraft_compatible && loader_compatible
}

fn version_timestamp(date: &str) -> i64 {
    if let Ok(date) = chrono::DateTime::parse_from_rfc3339(date) {
        return date.timestamp_millis();
    }

    let utc_date = date.strip_suffix(" UTC").unwrap_or(date);

    if let Ok(date) = chrono::NaiveDateTime::parse_from_str(utc_date, "%Y-%m-%d %H:%M:%S%.f") {
        return date.and_utc().timestamp_millis();
    }

    i64::MIN
}

fn latest_compatible_version<'a>(profile: &KableProfile, project: &'a Project) -> Option<&'a api_types::projects::ProjectVersion> {
    project
        .versions
        .iter()
        .filter(|version| is_version_compatible(profile, version, project.project_type))
        .max_by_key(|version| version_timestamp(&version.date_published))
}

fn resolve_version_id<'a>(profile: &KableProfile, project: &'a Project, version_id: Option<&str>) -> Result<&'a str, String> {
    if let Some(version_id) = version_id {
        let version = project
            .versions
            .iter()
            .find(|version| version.id == version_id)
            .ok_or_else(|| format!("Version {} was not found for project {}", version_id, project.project_id))?;

        if !is_version_compatible(profile, version, project.project_type) {
            return Err(format!(
                "Version {} of project {} is not compatible with profile {}",
                version_id, project.project_id, profile.metadata.name
            ));
        }

        return Ok(version.id.as_str());
    }

    latest_compatible_version(profile, project)
        .map(|version| version.id.as_str())
        .ok_or_else(|| format!("No compatible version found for project {} and profile {}", project.project_id, profile.metadata.name))
}

fn remove_project_from_settings(settings: &mut KableProfileSettings, project_type: ProjectType, filename: &str) {
    let mut projects = settings.from(project_type);
    projects.enabled.remove(filename);
    projects.disabled.remove(filename);
    settings.set_projects(project_type, projects);
}

fn enable_project_in_settings(settings: &mut KableProfileSettings, project_type: ProjectType, filename: &str) {
    let mut projects = settings.from(project_type);
    projects.enable(filename.to_string());
    settings.set_projects(project_type, projects);
}

async fn find_profile_project_filenames(
    profile: &KableProfile,
    project_type: ProjectType,
    project_id: &str,
) -> Result<Vec<String>, String> {
    let mut referenced_filenames = profile.settings.by_project_type(project_type, true);
    referenced_filenames.extend(profile.settings.by_project_type(project_type, false));

    let installed_projects = list_projects_for_type(project_type).await?;

    Ok(installed_projects
        .into_iter()
        .filter(|project| referenced_filenames.contains(&project.filename) && project.project.project_id == project_id)
        .map(|project| project.filename)
        .collect())
}

/// Delete global project files that are no longer referenced by any profile.
///
/// Always bypass the profile-list cache when checking references. The profile
/// settings on disk, rather than a cached snapshot, determine whether a file
/// can be deleted.
async fn cleanup_unreferenced_project_files(project_type: ProjectType, filenames: &[String]) -> Result<(), String> {
    if filenames.is_empty() {
        return Ok(());
    }

    let all_profiles = crate::features::profiles::management::list_profiles(true).await?;

    let folder = folder_for_project_type(project_type).await?;

    for filename in filenames {
        let still_referenced = all_profiles.iter().any(|profile| {
            profile.settings.by_project_type(project_type, true).contains(filename)
                || profile.settings.by_project_type(project_type, false).contains(filename)
        });

        if still_referenced {
            continue;
        }

        let project_path = folder.join(filename);
        let metadata_path = project_path.with_extension("json");

        if metadata_path.exists() {
            fs::remove_file(&metadata_path).await?;
        }

        if project_path.exists() {
            fs::remove_file(&project_path).await?;
        }

        Logger::debug_global(format!("Deleted unreferenced project file: {}", filename).as_str(), None);
    }

    Ok(())
}

pub async fn get_installed_project(project_type: ProjectType, filename: &str) -> Result<Option<KableProject>, String> {
    let type_dir = folder_for_project_type(project_type).await?;
    let metadata_path = type_dir.join(filename).with_extension("json");

    if !metadata_path.exists() {
        return Ok(None);
    }

    let serialized_project = fs::read_str(&metadata_path).await?;

    let project: KableProject = serde_json::from_str(&serialized_project).map_err(|error| error.to_string())?;

    Ok(Some(project))
}

async fn list_projects_for_type(project_type: ProjectType) -> Result<Vec<KableProject>, String> {
    let type_dir = folder_for_project_type(project_type).await?;

    if !type_dir.exists() {
        return Ok(Vec::new());
    }

    let mut projects = Vec::new();

    for path in fs::read_dir(&type_dir).await? {
        let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if !filename.ends_with(".json") {
            continue;
        }

        let metadata_path = type_dir.join(filename);
        let serialized_project = fs::read_str(&metadata_path).await?;

        let project: KableProject = serde_json::from_str(&serialized_project)
            .map_err(|error| format!("Failed to parse installed project metadata {}: {}", metadata_path.display(), error))?;

        projects.push(project);
    }

    Ok(projects)
}

/// List installed projects using the current persisted profile, not the
/// potentially stale profile settings supplied by the frontend.
pub async fn list_profile_projects(profile: KableProfile, enabled: bool) -> Result<HashMap<ProjectType, Vec<KableProject>>, String> {
    let current_profile = crate::features::profiles::management::get_profile(&profile.id).await?;

    let mut projects_map = HashMap::new();

    for project_type in [ProjectType::Mod, ProjectType::Resourcepack, ProjectType::Shader] {
        let project_filenames = current_profile.settings.by_project_type(project_type, enabled);

        let projects = list_projects_for_type(project_type)
            .await?
            .into_iter()
            .filter(|project| project_filenames.contains(&project.filename))
            .collect();

        projects_map.insert(project_type, projects);
    }

    Ok(projects_map)
}

async fn fetch_project_from_modrinth(project_id: &str) -> Result<Project, String> {
    Logger::debug_global(format!("Fetching project {} from Modrinth because local metadata is unavailable", project_id).as_str(), None);

    get_project(project_id).await
}

async fn load_project_metadata(kable_project: &KableProject) -> Result<Project, String> {
    if let Some(installed_project) = get_installed_project(kable_project.project.project_type, &kable_project.filename).await? {
        return Ok(installed_project.project);
    }

    Logger::debug_global(format!("Local metadata missing for {}, falling back to Modrinth", kable_project.filename).as_str(), None);

    fetch_project_from_modrinth(&kable_project.project.project_id).await
}

/// Download/install the selected version, then add its filename to the
/// current persisted profile. Profile settings are never replaced from the
/// caller's potentially stale snapshot.
pub async fn add_project_to_profile(profile: KableProfile, project: Project, version_id: Option<&str>) -> Result<KableProject, String> {
    let current_profile = crate::features::profiles::management::get_profile(&profile.id).await?;

    let resolved_version_id = resolve_version_id(&current_profile, &project, version_id)?.to_string();

    let kable_project = add_project(project, Some(&resolved_version_id)).await?;

    crate::features::profiles::management::modify_current_profile(&current_profile.id, |profile| {
        enable_project_in_settings(&mut profile.settings, kable_project.project.project_type, &kable_project.filename);
    })
    .await?;

    Ok(kable_project)
}

/// Download a project version and persist its metadata.
pub async fn add_project(project: Project, version_id: Option<&str>) -> Result<KableProject, String> {
    let folder = folder_for_project_type(project.project_type).await?;

    let filenames = download_project(&project, version_id, folder.clone()).await?;

    let filename = filenames.first().ok_or_else(|| "No files downloaded".to_string())?.to_string();

    let resolved_version_id = version_id
        .or(project.latest_version.as_deref())
        .ok_or_else(|| format!("No version id was associated with project {}", project.project_id))?;

    let metadata = KableProject { project: project.clone(), version_id: resolved_version_id.to_string(), filename };

    let metadata_path = folder.join(&metadata.filename).with_extension("json");
    let serialized_metadata = serde_json::to_string(&metadata).map_err(|error| error.to_string())?;

    fs::write_str(&metadata_path, &serialized_metadata, false).await?;

    Logger::debug_global(format!("Added project: {}", metadata.filename).as_str(), None);

    Ok(metadata)
}

/// Ensure every enabled project in the current persisted profile has local
/// metadata. Missing projects are reinstalled only when actually missing.
pub async fn ensure_profile_projects(profile: KableProfile) -> Result<(), String> {
    for project_type in [ProjectType::Mod, ProjectType::Resourcepack, ProjectType::Shader] {
        let current_profile = crate::features::profiles::management::get_profile(&profile.id).await?;

        let enabled_filenames = current_profile.settings.by_project_type(project_type, true);

        for filename in enabled_filenames {
            if get_installed_project(project_type, &filename).await?.is_some() {
                continue;
            }

            let project_search = api_types::projects::ProjectSearch { query: Some(filename.clone()), ..Default::default() };

            let results = crate::features::projects::browser::browse(current_profile.clone(), project_search, true, project_type).await?;

            let project =
                results.hits.into_iter().next().ok_or_else(|| format!("No project found for missing installed file {}", filename))?;

            add_project_to_profile(current_profile.clone(), project, None).await?;
        }
    }

    Ok(())
}

/// Remove every installed filename for this Modrinth project from the
/// current profile. Do not replace the profile with a stale caller snapshot.
pub async fn remove_project(profile: KableProfile, kable_project: KableProject) -> Result<KableProject, String> {
    let current_profile = crate::features::profiles::management::get_profile(&profile.id).await?;

    let project_type = kable_project.project.project_type;
    let project_id = kable_project.project.project_id.clone();

    let mut filenames = find_profile_project_filenames(&current_profile, project_type, &project_id).await?;

    if !filenames.contains(&kable_project.filename) {
        filenames.push(kable_project.filename.clone());
    }

    crate::features::profiles::management::modify_current_profile(&current_profile.id, |profile| {
        for filename in &filenames {
            remove_project_from_settings(&mut profile.settings, project_type, filename);
        }
    })
    .await?;

    cleanup_unreferenced_project_files(project_type, &filenames).await?;

    Logger::debug_global(format!("Removed project {} and {} profile reference(s)", project_id, filenames.len()).as_str(), None);

    Ok(kable_project)
}

pub async fn check_for_updates(profile: KableProfile, project_type: ProjectType) -> Result<Vec<UpdateMap>, String> {
    let current_profile = crate::features::profiles::management::get_profile(&profile.id).await?;

    let enabled_projects = list_profile_projects(current_profile.clone(), true).await?;

    let mut updates = Vec::new();

    if let Some(projects) = enabled_projects.get(&project_type) {
        for kable_project in projects {
            if let Ok(latest_project) = check_for_update(current_profile.clone(), kable_project.clone()).await {
                let latest_version = latest_compatible_version(&current_profile, &latest_project);

                Logger::debug_global(
                    format!(
                        "Update available for {}: {} -> {} ({})",
                        kable_project.project.project_id,
                        kable_project.version_id,
                        latest_version.map(|version| version.id.as_str()).unwrap_or("<unknown>"),
                        latest_version.map(|version| version.version_number.as_str()).unwrap_or("<unknown>"),
                    )
                    .as_str(),
                    Some(current_profile.id.as_str()),
                );

                updates.push(UpdateMap { kable_project: kable_project.clone(), update: Some(latest_project) });
            }
        }
    }

    Logger::debug_global(format!("Checked for updates: {} updates found", updates.len()).as_str(), Some(current_profile.id.as_str()));

    Ok(updates)
}

pub async fn check_for_update(profile: KableProfile, kable_project: KableProject) -> Result<Project, String> {
    let current_profile = crate::features::profiles::management::get_profile(&profile.id).await?;

    let project = load_project_metadata(&kable_project).await?;

    let latest_version = latest_compatible_version(&current_profile, &project)
        .ok_or_else(|| format!("No compatible version found for project {}", project.project_id))?;

    if latest_version.id == kable_project.version_id {
        return Err(format!("No update available for project {}", kable_project.filename));
    }

    Ok(project)
}

/// Update a project without overwriting unrelated or newer profile settings.
pub async fn update_project(profile: KableProfile, kable_project: KableProject, version_id: Option<&str>) -> Result<KableProject, String> {
    let current_profile = crate::features::profiles::management::get_profile(&profile.id).await?;

    Logger::debug_global(
        format!("Updating project: {} to version {:?}", kable_project.filename, version_id).as_str(),
        Some(current_profile.id.as_str()),
    );

    let project = load_project_metadata(&kable_project).await?;
    let selected_version_id = resolve_version_id(&current_profile, &project, version_id)?.to_string();

    if selected_version_id == kable_project.version_id {
        return Err(format!("Project {} is already using version {}", kable_project.filename, selected_version_id));
    }

    let project_type = project.project_type;
    let project_id = project.project_id.clone();

    let mut old_filenames = find_profile_project_filenames(&current_profile, project_type, &project_id).await?;

    if !old_filenames.contains(&kable_project.filename) {
        old_filenames.push(kable_project.filename.clone());
    }

    let updated_project = add_project(project, Some(&selected_version_id)).await?;

    let new_filename = updated_project.filename.clone();

    crate::features::profiles::management::modify_current_profile(&current_profile.id, |profile| {
        for old_filename in &old_filenames {
            remove_project_from_settings(&mut profile.settings, project_type, old_filename);
        }

        enable_project_in_settings(&mut profile.settings, project_type, &new_filename);
    })
    .await?;

    let files_to_cleanup = old_filenames.into_iter().filter(|filename| filename != &new_filename).collect::<Vec<_>>();

    cleanup_unreferenced_project_files(project_type, &files_to_cleanup).await?;

    Logger::debug_global(format!("Updated project {} to {}", project_id, new_filename).as_str(), Some(current_profile.id.as_str()));

    Ok(updated_project)
}

pub async fn update_all_projects(profile: KableProfile, project_type: ProjectType) -> Result<Vec<KableProject>, String> {
    let current_profile = crate::features::profiles::management::get_profile(&profile.id).await?;

    let updates = check_for_updates(current_profile.clone(), project_type).await?;

    let mut updated_projects = Vec::with_capacity(updates.len());

    for update_map in updates {
        let latest_profile = crate::features::profiles::management::get_profile(&current_profile.id).await?;

        let updated_project = update_project(latest_profile, update_map.kable_project, None).await?;

        updated_projects.push(updated_project);
    }

    Ok(updated_projects)
}

pub async fn check_incompatible_projects(profile: KableProfile, auto_disable: bool) -> Result<Vec<KableProject>, String> {
    let current_profile = crate::features::profiles::management::get_profile(&profile.id).await?;

    let minecraft_version = current_profile
        .version
        .minecraft_version
        .clone()
        .ok_or_else(|| "Profile version is empty, cannot check for incompatible projects".to_string())?;

    let enabled_projects = list_profile_projects(current_profile.clone(), true).await?;

    let mut incompatible_projects = Vec::new();

    for (project_type, projects) in enabled_projects {
        for kable_project in projects {
            let compatible =
                kable_project.project.versions.iter().any(|version| is_version_compatible(&current_profile, version, project_type));

            if compatible {
                continue;
            }

            incompatible_projects.push(kable_project.clone());

            if auto_disable {
                crate::features::profiles::management::set_project_enabled(&current_profile.id, kable_project, false).await?;
            }
        }
    }

    Logger::debug_global(
        format!("Checked incompatible projects for Minecraft {}: {} incompatible", minecraft_version, incompatible_projects.len()).as_str(),
        Some(current_profile.id.as_str()),
    );

    Ok(incompatible_projects)
}

/// Resolve a project's dependencies recursively without installing anything.
///
/// Dependencies that already exist in the profile are excluded regardless
/// of whether they are enabled or disabled. Project identity is determined
/// by `(ProjectType, project_id)`, never by filename or version.
pub async fn resolve_project_dependencies(
    profile: KableProfile,
    project: Project,
    version_id: &str,
) -> Result<Vec<ResolvedDependency>, String> {
    let current_profile = crate::features::profiles::management::get_profile(&profile.id).await?;

    let root_version = project
        .versions
        .iter()
        .find(|version| version.id == version_id)
        .ok_or_else(|| format!("Version {} was not found for project {}", version_id, project.project_id))?;

    if !is_version_compatible(&current_profile, root_version, project.project_type) {
        return Err(format!(
            "Version {} of project {} is not compatible with profile {}",
            version_id, project.project_id, current_profile.metadata.name
        ));
    }

    type VersionKey = (String, String);
    type ProjectKey = (ProjectType, String);

    struct DependencyNode {
        project: KableProject,
        dependency_type: DependencyType,
        children: Vec<usize>,
    }

    struct WorkItem {
        project: Project,
        version_id: String,
        parent_node: Option<usize>,
        ancestry: HashSet<VersionKey>,
    }

    // Build a set of all projects already referenced by this profile.
    // Include both enabled and disabled projects so that a disabled project
    // is not accidentally offered for installation as a dependency.
    let mut existing_projects: HashSet<ProjectKey> = HashSet::new();

    for project_type in [ProjectType::Mod, ProjectType::Resourcepack, ProjectType::Shader] {
        let mut filenames = current_profile.settings.by_project_type(project_type, true);

        filenames.extend(current_profile.settings.by_project_type(project_type, false));

        for filename in filenames {
            if let Some(installed_project) = get_installed_project(project_type, &filename).await? {
                existing_projects.insert((installed_project.project.project_type, installed_project.project.project_id));
            }
        }
    }

    let root_key = (project.project_id.clone(), version_id.to_string());
    let mut root_ancestry = HashSet::new();
    root_ancestry.insert(root_key);

    let mut pending =
        vec![WorkItem { project: project.clone(), version_id: version_id.to_string(), parent_node: None, ancestry: root_ancestry }];

    let mut project_cache: HashMap<String, Project> = HashMap::new();
    project_cache.insert(project.project_id.clone(), project.clone());

    let mut selected_versions: HashMap<String, String> = HashMap::new();
    selected_versions.insert(project.project_id.clone(), version_id.to_string());

    let mut nodes: Vec<DependencyNode> = Vec::new();
    let mut root_nodes: Vec<usize> = Vec::new();

    while let Some(work) = pending.pop() {
        let parent_version = work.project.versions.iter().find(|version| version.id == work.version_id).ok_or_else(|| {
            format!("Version {} was not found while resolving dependencies for {}", work.version_id, work.project.project_id)
        })?;

        for dependency in parent_version.dependencies.clone().unwrap_or_default() {
            let dependency_type = dependency.dependency_type;

            match dependency_type {
                DependencyType::Incompatible | DependencyType::Embedded => continue,
                DependencyType::Required | DependencyType::Optional | DependencyType::Other => {}
            }

            let dependency_version_id =
                dependency.version_id.as_ref().and_then(|value| value.as_deref()).filter(|id| !id.trim().is_empty()).map(str::to_owned);

            let mut dependency_project_id =
                dependency.project_id.as_ref().and_then(|value| value.as_deref()).filter(|id| !id.trim().is_empty()).map(str::to_owned);

            // Some dependencies specify only a version ID. Resolve its
            // project ID before checking whether the profile already has it.
            if dependency_project_id.is_none() {
                let Some(exact_version_id) = dependency_version_id.as_ref() else {
                    continue;
                };

                let versions = get_version_data(vec![exact_version_id.clone()]).await.map_err(|error| {
                    format!("Failed to resolve dependency version {} required by {}: {}", exact_version_id, work.project.title, error)
                })?;

                let version = versions.into_iter().find(|version| version.id == *exact_version_id).ok_or_else(|| {
                    format!("Dependency version {} required by {} was not found on Modrinth", exact_version_id, work.project.title)
                })?;

                dependency_project_id = Some(version.project_id);
            }

            let Some(dependency_project_id) = dependency_project_id else {
                continue;
            };

            let dependency_project = if let Some(cached) = project_cache.get(&dependency_project_id) {
                cached.clone()
            } else {
                let fetched = get_project(&dependency_project_id).await.map_err(|error| {
                    format!("Failed to fetch dependency project {} required by {}: {}", dependency_project_id, work.project.title, error)
                })?;

                project_cache.insert(dependency_project_id.clone(), fetched.clone());
                fetched
            };

            // A project already present in the profile satisfies the
            // dependency, regardless of its enabled state or installed
            // version. Compare only project type and project ID.
            let project_key = (dependency_project.project_type, dependency_project.project_id.clone());

            if existing_projects.contains(&project_key) {
                Logger::debug_global(
                    &format!(
                        "Skipping dependency {} ({:?}): already present in profile {}",
                        dependency_project.project_id, dependency_project.project_type, current_profile.metadata.name
                    ),
                    Some(current_profile.id.as_str()),
                );

                continue;
            }

            let selected_version_id =
                resolve_version_id(&current_profile, &dependency_project, dependency_version_id.as_deref())?.to_string();

            // A project may be encountered through multiple dependency
            // branches. Reject incompatible version requirements instead
            // of returning duplicate entries for different versions.
            if let Some(existing_version_id) = selected_versions.get(&dependency_project_id) {
                if existing_version_id != &selected_version_id {
                    return Err(format!(
                        "Conflicting dependency versions for project {}: {} and {}",
                        dependency_project.title, existing_version_id, selected_version_id
                    ));
                }
            } else {
                selected_versions.insert(dependency_project_id.clone(), selected_version_id.clone());
            }

            let selected_version =
                dependency_project.versions.iter().find(|version| version.id == selected_version_id).ok_or_else(|| {
                    format!("Selected version {} was not found for dependency {}", selected_version_id, dependency_project.title)
                })?;

            let primary_files: Vec<_> = selected_version.files.iter().filter(|file| file.primary).collect();

            if primary_files.len() != 1 {
                return Err(format!(
                    "Version {} of dependency {} must have exactly one primary file; found {}",
                    selected_version.version_number,
                    dependency_project.title,
                    primary_files.len()
                ));
            }

            let candidate = KableProject {
                project: dependency_project.clone(),
                version_id: selected_version_id.clone(),
                filename: primary_files[0].filename.clone(),
            };

            let node_index = nodes.len();

            nodes.push(DependencyNode { project: candidate, dependency_type, children: Vec::new() });

            if let Some(parent_index) = work.parent_node {
                nodes[parent_index].children.push(node_index);
            } else {
                root_nodes.push(node_index);
            }

            let child_key = (dependency_project_id, selected_version_id.clone());

            if work.ancestry.contains(&child_key) {
                Logger::debug_global(
                    &format!(
                        "Detected dependency cycle at {} version {}; stopping expansion of this branch",
                        dependency_project.title, selected_version_id
                    ),
                    Some(current_profile.id.as_str()),
                );

                continue;
            }

            let mut child_ancestry = work.ancestry.clone();
            child_ancestry.insert(child_key);

            pending.push(WorkItem {
                project: dependency_project,
                version_id: selected_version_id,
                parent_node: Some(node_index),
                ancestry: child_ancestry,
            });
        }
    }

    fn build_tree(index: usize, nodes: &[DependencyNode]) -> ResolvedDependency {
        let node = &nodes[index];

        ResolvedDependency {
            project: node.project.clone(),
            dependency_type: node.dependency_type,
            dependencies: node.children.iter().map(|&child_index| build_tree(child_index, nodes)).collect(),
        }
    }

    let resolved_projects = root_nodes.iter().map(|&index| build_tree(index, &nodes)).collect::<Vec<_>>();

    Logger::debug_global(
        &format!(
            "Resolved {} top-level dependencies ({} dependency occurrences) for project {} version {}",
            resolved_projects.len(),
            nodes.len(),
            project.project_id,
            version_id
        ),
        Some(current_profile.id.as_str()),
    );

    Ok(resolved_projects)
}
