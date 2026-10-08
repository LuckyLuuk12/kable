use crate::Logger;

// List, Remove, Add, Enable/Disable projects.
// We make this kinda "project type agnostic" so we can use it for mods,
// resourcepacks, shaderpacks, etc.
//
// For now I implement all project features here: download/install, remove,
// enable/disable, update, list, check for updates, etc.
// Later we can split them into separate files if needed.

use crate::integrations::modrinth::ferinth_client::{download_project, get_project};
use crate::system::fs;

use api_types::profiles::{KableProfile, KableProfileSettings};
use api_types::projects::{KableProject, Project, ProjectType, UpdateMap};

/// List all projects from .kable/projects/<type>/* and return
/// Map<ProjectType, Vec<KableProject>> for all supported project types.
pub async fn list_all_projects() -> Result<std::collections::HashMap<ProjectType, Vec<KableProject>>, String> {
    let mut all_projects = std::collections::HashMap::<ProjectType, Vec<KableProject>>::new();

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

    Ok(projects_dir.join(type_folder_name))
}

/// Normalize a loader name for compatibility comparisons.
///
/// Iris-backed Fabric profiles may be represented as `IrisFabric`,
/// `iris-fabric`, `iris_fabric`, etc., while Modrinth normally reports
/// `fabric`. Normalize both sides to the same canonical loader name.
fn normalize_loader(loader: &str) -> String {
    let normalized = loader.trim().to_lowercase().replace(['_', '-', ' '], "");

    match normalized.as_str() {
        "irisfabric" | "irisfabricloader" => "fabric".to_string(),
        "fabric" => "fabric".to_string(),
        "quiltedfabric" => "fabric".to_string(),
        "quilt" => "quilt".to_string(),
        "neoforge" => "neoforge".to_string(),
        "forge" => "forge".to_string(),
        "vanilla" => "vanilla".to_string(),
        other => other.to_string(),
    }
}

/// Check whether a specific project version is compatible with a profile.
///
/// Minecraft version is relevant for every project type.
///
/// Loader compatibility is only relevant for mods and modpacks.
/// Resourcepacks and shaders do not use the profile's mod loader.
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

/// Parse the date representation used by ProjectVersion.
///
/// Modrinth/Ferinth timestamps may currently be serialized as:
///
///     2026-09-18 05:45:50.817305 UTC
///
/// which is not RFC3339. The old implementation only attempted
/// parse_from_rfc3339(), causing all such timestamps to become i64::MIN.
/// That meant max_by_key()/sorting could select an arbitrary compatible
/// version rather than the newest one.
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

/// Find the newest compatible version of a project for a profile.
///
/// This deliberately does not use `project.latest_version` because that is
/// the project's globally latest Modrinth version and may belong to another
/// loader, for example NeoForge while the profile uses Fabric.
///
/// The Project stored in the local KableProject metadata JSON is used
/// directly. No Modrinth request is performed here.
fn latest_compatible_version<'a>(profile: &KableProfile, project: &'a Project) -> Option<&'a api_types::projects::ProjectVersion> {
    let compatible_versions = project
        .versions
        .iter()
        .filter(|version| is_version_compatible(profile, version, project.project_type))
        .collect::<Vec<_>>();

    Logger::debug_global(
        &format!(
            "Project {} has {} total versions, {} compatible versions for MC {:?} loader {}",
            project.project_id,
            project.versions.len(),
            compatible_versions.len(),
            profile.version.minecraft_version,
            profile.version.loader,
        ),
        Some(profile.id.as_str()),
    );

    let mut sorted = compatible_versions;

    sorted.sort_by(|a, b| version_timestamp(&b.date_published).cmp(&version_timestamp(&a.date_published)));

    for version in sorted.iter().take(10) {
        Logger::debug_global(
            &format!(
                "Compatible candidate: {} {} published {} MC={:?} loaders={:?}",
                version.id, version.version_number, version.date_published, version.game_versions, version.loaders,
            ),
            Some(profile.id.as_str()),
        );
    }

    sorted.into_iter().next()
}

/// Given a project and optional version ID, resolve the version that should
/// actually be installed.
///
/// If `version_id` is Some, that exact version must exist and be compatible
/// with the profile.
///
/// If `version_id` is None, the newest compatible version is selected.
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

/// Completely remove a project filename from a profile's project settings.
fn remove_project_from_settings(settings: &mut KableProfileSettings, project_type: ProjectType, filename: &str) {
    let mut projects = settings.from(project_type);

    projects.enabled.remove(filename);
    projects.disabled.remove(filename);

    settings.set_projects(project_type, projects);
}

/// Explicitly enable a project in a profile.
fn enable_project_in_settings(settings: &mut KableProfileSettings, project_type: ProjectType, filename: &str) {
    let mut projects = settings.from(project_type);

    projects.enable(filename.to_string());

    settings.set_projects(project_type, projects);
}

/// Find every filename in this profile that belongs to the given Modrinth
/// project ID.
async fn find_profile_project_filenames(
    profile: &KableProfile,
    project_type: ProjectType,
    project_id: &str,
) -> Result<Vec<String>, String> {
    let mut referenced_filenames = profile.settings.by_project_type(project_type, true);

    referenced_filenames.extend(profile.settings.by_project_type(project_type, false));

    let installed_projects = list_projects_for_type(project_type).await?;

    let mut filenames = Vec::new();

    for installed_project in installed_projects {
        if referenced_filenames.contains(&installed_project.filename) && installed_project.project.project_id == project_id {
            filenames.push(installed_project.filename);
        }
    }

    Ok(filenames)
}

/// Delete global project files that are no longer referenced by any profile.
async fn cleanup_unreferenced_project_files(project_type: ProjectType, filenames: &[String]) -> Result<(), String> {
    if filenames.is_empty() {
        return Ok(());
    }

    let all_profiles = crate::features::profiles::management::list_profiles().await?;

    let folder = folder_for_project_type(project_type).await?;

    for filename in filenames {
        let still_referenced = all_profiles.iter().any(|profile| {
            profile.settings.by_project_type(project_type, true).contains(filename)
                || profile.settings.by_project_type(project_type, false).contains(filename)
        });

        if still_referenced {
            continue;
        }

        let metadata_path = std::path::Path::new(&folder).join(filename).with_extension("json");

        if metadata_path.exists() {
            fs::remove_file(&metadata_path).await?;
        }

        let project_path = std::path::Path::new(&folder).join(filename);

        if project_path.exists() {
            fs::remove_file(&project_path).await?;
        }

        Logger::debug_global(format!("Deleted unreferenced project file: {}", filename).as_str(), None);
    }

    Ok(())
}

/// Given a string, check if the project is already installed in the global
/// projects folder, and if so return the KableProject metadata.
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

/// List all projects of a specific type from
/// .kable/projects/<type>/* and return Vec<KableProject>.
///
/// KableProject metadata is stored in .json files in the projects/<type>
/// folder, while the actual project files are stored beside the metadata.
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

/// Given a profile, list all enabled or disabled KableProjects for that
/// profile, grouped by project type.
pub async fn list_profile_projects(
    profile: KableProfile,
    enabled: bool,
) -> Result<std::collections::HashMap<ProjectType, Vec<KableProject>>, String> {
    let mut projects_map = std::collections::HashMap::<ProjectType, Vec<KableProject>>::new();

    for project_type in [ProjectType::Mod, ProjectType::Resourcepack, ProjectType::Shader] {
        let project_filenames = profile.settings.by_project_type(project_type, enabled);

        let projects = list_projects_for_type(project_type)
            .await?
            .into_iter()
            .filter(|project| project_filenames.contains(&project.filename))
            .collect::<Vec<KableProject>>();

        projects_map.insert(project_type, projects);
    }

    Ok(projects_map)
}

/// Fetch the complete Modrinth project directly by ID.
///
/// This should only be used when local metadata is unavailable. Normal
/// update checking must use the persisted KableProject metadata instead.
async fn fetch_project_from_modrinth(project_id: &str) -> Result<Project, String> {
    Logger::debug_global(format!("Fetching project {} from Modrinth because local metadata is unavailable", project_id).as_str(), None);

    get_project(project_id).await
}

/// Load a KableProject's persisted metadata.
///
/// The installed .json file is the normal source of truth. Modrinth is only
/// contacted when the local metadata file is genuinely unavailable.
///
/// This is intentionally separate from get_installed_project() because some
/// callers already have a KableProject and should not need to rediscover it.
async fn load_project_metadata(kable_project: &KableProject) -> Result<Project, String> {
    let installed_project = get_installed_project(kable_project.project.project_type, &kable_project.filename).await?;

    if let Some(installed_project) = installed_project {
        Logger::debug_global(
            format!("Using local Modrinth metadata for {} ({})", installed_project.filename, installed_project.project.project_id).as_str(),
            None,
        );

        return Ok(installed_project.project);
    }

    Logger::debug_global(format!("Local metadata missing for {}, falling back to Modrinth", kable_project.filename).as_str(), None);

    fetch_project_from_modrinth(&kable_project.project.project_id).await
}

/// Given a profile and a Project, download/install the selected compatible
/// version into the global projects folder and add the resulting filename to
/// the profile as enabled.
pub async fn add_project_to_profile(profile: KableProfile, project: Project, version_id: Option<&str>) -> Result<KableProject, String> {
    let resolved_version_id = resolve_version_id(&profile, &project, version_id)?;

    let kable_project = add_project(project.clone(), Some(resolved_version_id)).await?;

    let mut updated_profile = profile.clone();

    enable_project_in_settings(&mut updated_profile.settings, kable_project.project.project_type, &kable_project.filename);

    crate::features::profiles::management::modify_profile(profile, updated_profile).await?;

    Ok(kable_project)
}

/// Wrapper for Modrinth's download_project and metadata creation.
///
/// `version_id` is expected to already have been resolved and validated by
/// the caller.
pub async fn add_project(project: Project, version_id: Option<&str>) -> Result<KableProject, String> {
    let folder = folder_for_project_type(project.project_type).await?;

    let filenames = download_project(&project, version_id, folder.clone()).await?;

    let filename = filenames.first().ok_or("No files downloaded")?.to_string();

    let resolved_version_id = version_id
        .or(project.latest_version.as_deref())
        .ok_or_else(|| format!("No version id was associated with project {}", project.project_id))?;

    let metadata = KableProject { project: project.clone(), version_id: resolved_version_id.to_string(), filename };

    let metadata_path = folder.join(&metadata.filename).with_extension("json");

    fs::write_str(&metadata_path, &serde_json::to_string(&metadata).map_err(|e| e.to_string())?, false).await?;

    Logger::debug_global(format!("Added project: {}", metadata.filename).as_str(), None);

    Ok(metadata)
}

/// Ensures that all projects enabled in the profile are actually present in
/// the global projects folder.
///
/// If an enabled project is missing, search for it and reinstall the newest
/// compatible version.
pub async fn ensure_profile_projects(profile: KableProfile) -> Result<(), String> {
    for project_type in [ProjectType::Mod, ProjectType::Resourcepack, ProjectType::Shader] {
        let enabled_project_filenames = profile.settings.by_project_type(project_type, true);

        for filename in enabled_project_filenames {
            if get_installed_project(project_type, &filename).await?.is_some() {
                continue;
            }

            let project_search = api_types::projects::ProjectSearch { query: Some(filename.clone()), ..Default::default() };

            let results = crate::features::projects::browser::browse(profile.clone(), project_search, true, project_type).await?;

            if let Some(latest_project) = results.hits.into_iter().next() {
                add_project_to_profile(profile.clone(), latest_project, None).await?;
            } else {
                return Err(format!("No project found for {}", filename));
            }
        }
    }

    Ok(())
}

/// Remove a project from a profile.
pub async fn remove_project(profile: KableProfile, kable_project: KableProject) -> Result<KableProject, String> {
    let project_type = kable_project.project.project_type;
    let project_id = kable_project.project.project_id.clone();

    let mut filenames = find_profile_project_filenames(&profile, project_type, &project_id).await?;

    if !filenames.contains(&kable_project.filename) {
        filenames.push(kable_project.filename.clone());
    }

    let mut updated_profile = profile.clone();

    for filename in &filenames {
        remove_project_from_settings(&mut updated_profile.settings, project_type, filename);
    }

    crate::features::profiles::management::modify_profile(profile, updated_profile).await?;

    cleanup_unreferenced_project_files(project_type, &filenames).await?;

    Logger::debug_global(format!("Removed project {} and {} profile reference(s)", project_id, filenames.len()).as_str(), None);

    Ok(kable_project)
}

/// Given a profile, check all enabled projects for updates.
///
/// This deliberately uses the persisted KableProject metadata. It does not
/// contact Modrinth for every installed project.
///
/// The metadata JSON contains the complete Project snapshot, including its
/// available versions, so compatibility and update selection can be done
/// locally.
pub async fn check_for_updates(profile: KableProfile, project_type: ProjectType) -> Result<Vec<UpdateMap>, String> {
    let mut updates = Vec::new();

    let enabled_projects = list_profile_projects(profile.clone(), true).await?;

    if let Some(projects) = enabled_projects.get(&project_type) {
        for kable_project in projects {
            match check_for_update(profile.clone(), kable_project.clone()).await {
                Ok(latest_project) => {
                    let latest_version = latest_compatible_version(&profile, &latest_project);

                    Logger::debug_global(
                        format!(
                            "Update available for {}: {} -> {} ({})",
                            kable_project.project.project_id,
                            kable_project.version_id,
                            latest_version.map(|version| version.id.as_str()).unwrap_or("<unknown>"),
                            latest_version.map(|version| { version.version_number.as_str() }).unwrap_or("<unknown>"),
                        )
                        .as_str(),
                        Some(profile.id.as_str()),
                    );

                    updates.push(UpdateMap { kable_project: kable_project.clone(), update: Some(latest_project) });
                }

                Err(error) => {
                    Logger::debug_global(
                        format!("No update for {} ({}): {}", kable_project.project.project_id, kable_project.filename, error).as_str(),
                        Some(profile.id.as_str()),
                    );
                }
            }
        }
    }

    Logger::debug_global(format!("Checked for updates: {} updates found", updates.len()).as_str(), Some(profile.id.as_str()));

    Ok(updates)
}

/// Find the newest compatible Project for an installed KableProject.
///
/// The persisted KableProject metadata is used directly. No Modrinth request
/// is made when the metadata JSON exists.
///
/// If the metadata file has disappeared, load_project_metadata() falls back
/// to a direct Modrinth lookup.
pub async fn check_for_update(profile: KableProfile, kable_project: KableProject) -> Result<Project, String> {
    Logger::debug_global(
        format!("Checking local project metadata for update: {} ({})", kable_project.filename, kable_project.project.project_id).as_str(),
        Some(profile.id.as_str()),
    );

    let project = load_project_metadata(&kable_project).await?;

    let latest_version = latest_compatible_version(&profile, &project)
        .ok_or_else(|| format!("No compatible version found for project {}", project.project_id))?;

    Logger::debug_global(
        format!(
            "Latest compatible version for {}: {} ({}) published {}",
            project.project_id, latest_version.id, latest_version.version_number, latest_version.date_published,
        )
        .as_str(),
        Some(profile.id.as_str()),
    );

    if latest_version.id == kable_project.version_id {
        return Err(format!("No update available for project {}", kable_project.filename));
    }

    Ok(project)
}

/// Update a project.
///
/// The persisted project metadata is used instead of fetching the project
/// from Modrinth again. If the metadata file is missing, a direct lookup is
/// used as a fallback.
///
/// If version_id is Some, that exact version is installed after verifying
/// that it is compatible with the profile.
///
/// If version_id is None, the newest compatible version is selected
/// automatically.
pub async fn update_project(profile: KableProfile, kable_project: KableProject, version_id: Option<&str>) -> Result<KableProject, String> {
    Logger::debug_global(
        format!("Updating project: {} to version {:?}", kable_project.filename, version_id).as_str(),
        Some(profile.id.as_str()),
    );

    let project = load_project_metadata(&kable_project).await?;

    let selected_version_id = resolve_version_id(&profile, &project, version_id)?;

    if selected_version_id == kable_project.version_id {
        return Err(format!("Project {} is already using version {}", kable_project.filename, selected_version_id));
    }

    let project_type = project.project_type;
    let project_id = project.project_id.clone();

    let mut old_filenames = find_profile_project_filenames(&profile, project_type, &project_id).await?;

    if !old_filenames.contains(&kable_project.filename) {
        old_filenames.push(kable_project.filename.clone());
    }

    let updated_project = add_project(project.clone(), Some(selected_version_id)).await?;

    let new_filename = updated_project.filename.clone();

    let mut updated_profile = profile.clone();

    for old_filename in &old_filenames {
        remove_project_from_settings(&mut updated_profile.settings, project_type, old_filename);
    }

    enable_project_in_settings(&mut updated_profile.settings, project_type, &new_filename);

    crate::features::profiles::management::modify_profile(profile.clone(), updated_profile).await?;

    let files_to_cleanup = old_filenames.into_iter().filter(|filename| filename != &new_filename).collect::<Vec<_>>();

    cleanup_unreferenced_project_files(project_type, &files_to_cleanup).await?;

    Logger::debug_global(format!("Updated project {} to {}", project_id, new_filename).as_str(), Some(profile.id.as_str()));

    Ok(updated_project)
}

/// Update all projects of a type using automatic version selection.
///
/// Updates are intentionally performed sequentially.
///
/// `check_for_updates()` now uses only local metadata, so this operation no
/// longer performs a Modrinth request for every project before updating.
pub async fn update_all_projects(profile: KableProfile, project_type: ProjectType) -> Result<Vec<KableProject>, String> {
    let updates = check_for_updates(profile.clone(), project_type).await?;

    let mut updated_projects = Vec::with_capacity(updates.len());

    let mut current_profile = profile;

    for update_map in updates {
        let updated_project = update_project(current_profile.clone(), update_map.kable_project, None).await?;

        updated_projects.push(updated_project);

        current_profile = crate::features::profiles::management::list_profiles()
            .await?
            .into_iter()
            .find(|profile| profile.id == current_profile.id)
            .ok_or_else(|| format!("Profile {} could not be found after updating a project", current_profile.id))?;
    }

    Ok(updated_projects)
}

/// Given a profile, check all enabled projects for compatibility with the
/// current profile.
///
/// A project is incompatible only if NONE of its available versions supports
/// the profile's Minecraft version and, where relevant, loader.
///
/// If auto_disable is true, incompatible projects are disabled.
pub async fn check_incompatible_projects(profile: KableProfile, auto_disable: bool) -> Result<Vec<KableProject>, String> {
    let minecraft_version = profile
        .version
        .minecraft_version
        .clone()
        .ok_or_else(|| "Profile version is empty, cannot check for incompatible projects".to_string())?;

    let enabled_projects = list_profile_projects(profile.clone(), true).await?;

    let mut incompatible_projects = Vec::new();

    for (project_type, projects) in enabled_projects {
        for kable_project in projects {
            let compatible = kable_project.project.versions.iter().any(|version| is_version_compatible(&profile, version, project_type));

            if !compatible {
                incompatible_projects.push(kable_project.clone());

                if auto_disable {
                    crate::features::profiles::management::toggle_project(profile.clone(), kable_project.clone()).await?;
                }
            }
        }
    }

    Logger::debug_global(
        format!("Checked incompatible projects for Minecraft {}: {} incompatible", minecraft_version, incompatible_projects.len()).as_str(),
        Some(profile.id.as_str()),
    );

    Ok(incompatible_projects)
}
