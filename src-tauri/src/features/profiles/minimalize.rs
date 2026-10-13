use std::{
    collections::{HashMap, HashSet},
    path::{Component, Path},
};

use api_types::{
    profiles::KableProfile,
    projects::{KableProject, ProjectType},
};

use crate::{
    constants::{MODS_DIR, RESOURCEPACKS_DIR, RUNTIME_DIR_NAME, SHADERPACKS_DIR},
    system::fs as kable_fs,
    Logger,
};

const PROJECT_TYPES: [(ProjectType, &str); 3] =
    [(ProjectType::Mod, MODS_DIR), (ProjectType::Resourcepack, RESOURCEPACKS_DIR), (ProjectType::Shader, SHADERPACKS_DIR)];

/// Summary of resources removed during cleanup.
#[derive(Debug, Default)]
pub struct CleanupReport {
    pub removed_runtime_directories: usize,
    pub removed_project_files: usize,
    pub removed_project_metadata: usize,
}

/// Remove runtime directories and shared project files that are no longer
/// referenced by any saved profile.
///
/// Disabled projects are retained because they are still referenced by their
/// profiles and may be enabled again later.
///
/// Run this during startup, before normal profile and project operations begin.
pub async fn cleanup_orphaned_resources() -> Result<CleanupReport, String> {
    Logger::debug_global("[minimalize] Starting orphaned resource cleanup", None);

    let profiles = crate::features::profiles::management::list_profiles(true).await?;

    let profile_ids = collect_profile_ids(&profiles);
    let referenced_projects = collect_project_references(&profiles);

    let mut report =
        CleanupReport { removed_runtime_directories: cleanup_orphaned_runtime_directories(&profile_ids).await?, ..Default::default() };

    let projects_root = kable_fs::projects_dir()?;

    if is_real_directory(&projects_root).await? {
        for (project_type, directory_name) in PROJECT_TYPES {
            let type_directory = projects_root.join(directory_name);

            if !is_real_directory(&type_directory).await? {
                continue;
            }

            let references = referenced_projects
                .get(&project_type)
                .ok_or_else(|| format!("[minimalize] Missing project reference set for {:?}", project_type))?;

            let (removed_files, removed_metadata) = cleanup_project_directory(&type_directory, references).await?;

            report.removed_project_files += removed_files;
            report.removed_project_metadata += removed_metadata;
        }
    }

    Logger::debug_global(
        &format!(
            "[minimalize] Cleanup completed: {} runtime directories, {} project files, {} project metadata files removed",
            report.removed_runtime_directories, report.removed_project_files, report.removed_project_metadata
        ),
        None,
    );

    Ok(report)
}

/// Collect saved profile IDs for constant-time average membership checks.
fn collect_profile_ids(profiles: &[KableProfile]) -> HashSet<String> {
    profiles.iter().map(|profile| profile.id.clone()).collect()
}

/// Collect project filenames referenced by all profiles, including disabled
/// projects.
fn collect_project_references(profiles: &[KableProfile]) -> HashMap<ProjectType, HashSet<String>> {
    let mut references = HashMap::from([
        (ProjectType::Mod, HashSet::new()),
        (ProjectType::Resourcepack, HashSet::new()),
        (ProjectType::Shader, HashSet::new()),
    ]);

    for profile in profiles {
        for (project_type, _) in PROJECT_TYPES {
            let Some(filenames) = references.get_mut(&project_type) else {
                continue;
            };

            for enabled in [true, false] {
                filenames.extend(profile.settings.by_project_type(project_type, enabled));
            }
        }
    }

    references
}

/// Remove runtime directories that do not belong to a saved profile.
async fn cleanup_orphaned_runtime_directories(profile_ids: &HashSet<String>) -> Result<usize, String> {
    let runtime_root = kable_fs::kable_dir()?.join(RUNTIME_DIR_NAME);

    if !is_real_directory(&runtime_root).await? {
        return Ok(0);
    }

    let mut removed = 0;

    for entry in kable_fs::read_dir(&runtime_root).await? {
        let metadata = match tokio::fs::symlink_metadata(&entry).await {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!("[minimalize] Failed to inspect runtime entry {}: {}", entry.display(), error));
            }
        };

        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            continue;
        }

        let Some(profile_id) = entry.file_name().and_then(|name| name.to_str()) else {
            Logger::debug_global(&format!("[minimalize] Skipping runtime directory with a non-UTF-8 name: {}", entry.display()), None);
            continue;
        };

        if profile_ids.contains(profile_id) {
            continue;
        }

        Logger::debug_global(&format!("[minimalize] Removing orphaned runtime directory: {}", entry.display()), None);

        kable_fs::remove_dir_all(&entry)
            .await
            .map_err(|error| format!("[minimalize] Failed to remove runtime directory {}: {}", entry.display(), error))?;

        removed += 1;
    }

    Ok(removed)
}

/// Remove unreferenced project artifacts and their matching metadata from one
/// project-type directory.
///
/// Malformed metadata and inconsistent filenames are preserved.
async fn cleanup_project_directory(directory: &Path, referenced_filenames: &HashSet<String>) -> Result<(usize, usize), String> {
    let mut removed_files = 0;
    let mut removed_metadata = 0;

    for metadata_path in kable_fs::read_dir(directory).await? {
        let metadata = match tokio::fs::symlink_metadata(&metadata_path).await {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!("[minimalize] Failed to inspect project entry {}: {}", metadata_path.display(), error));
            }
        };

        if metadata.file_type().is_symlink() || !metadata.is_file() {
            continue;
        }

        let Some(metadata_filename) = metadata_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if !metadata_filename.ends_with(".json") {
            continue;
        }

        let serialized = match kable_fs::read_str(&metadata_path).await {
            Ok(serialized) => serialized,
            Err(error) => {
                Logger::debug_global(&format!("[minimalize] Skipping unreadable metadata {}: {}", metadata_path.display(), error), None);
                continue;
            }
        };

        let project: KableProject = match serde_json::from_str(&serialized) {
            Ok(project) => project,
            Err(error) => {
                Logger::debug_global(&format!("[minimalize] Skipping invalid metadata {}: {}", metadata_path.display(), error), None);
                continue;
            }
        };

        if !is_safe_project_filename(&project.filename) {
            Logger::debug_global(
                &format!("[minimalize] Skipping unsafe project filename {:?} in {}", project.filename, metadata_path.display()),
                None,
            );
            continue;
        }

        let artifact_path = directory.join(&project.filename);

        if artifact_path.with_extension("json") != metadata_path {
            Logger::debug_global(&format!("[minimalize] Skipping inconsistent metadata {}", metadata_path.display()), None);
            continue;
        }

        if referenced_filenames.contains(&project.filename) {
            continue;
        }

        let artifact_metadata = match tokio::fs::symlink_metadata(&artifact_path).await {
            Ok(metadata) => Some(metadata),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => {
                return Err(format!("[minimalize] Failed to inspect project artifact {}: {}", artifact_path.display(), error));
            }
        };

        if let Some(artifact_metadata) = artifact_metadata {
            if artifact_metadata.file_type().is_symlink() || !artifact_metadata.is_file() {
                Logger::debug_global(
                    &format!("[minimalize] Skipping project with non-regular artifact: {}", artifact_path.display()),
                    None,
                );
                continue;
            }

            Logger::debug_global(&format!("[minimalize] Removing orphaned project file: {}", artifact_path.display()), None);

            kable_fs::remove_file(&artifact_path)
                .await
                .map_err(|error| format!("[minimalize] Failed to remove project file {}: {}", artifact_path.display(), error))?;

            removed_files += 1;
        }

        Logger::debug_global(&format!("[minimalize] Removing orphaned project metadata: {}", metadata_path.display()), None);

        kable_fs::remove_file(&metadata_path)
            .await
            .map_err(|error| format!("[minimalize] Failed to remove project metadata {}: {}", metadata_path.display(), error))?;

        removed_metadata += 1;
    }

    Ok((removed_files, removed_metadata))
}

/// Ensure the filename is a single path component, not a path traversal.
fn is_safe_project_filename(filename: &str) -> bool {
    if filename.is_empty() || filename == "." || filename == ".." || filename.contains('/') || filename.contains('\\') {
        return false;
    }

    let mut components = Path::new(filename).components();

    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

/// Check for a real directory without following a symlink.
async fn is_real_directory(path: &Path) -> Result<bool, String> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) => Ok(metadata.is_dir() && !metadata.file_type().is_symlink()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("[minimalize] Failed to inspect directory {}: {}", path.display(), error)),
    }
}
