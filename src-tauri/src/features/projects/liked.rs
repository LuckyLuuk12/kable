use std::{
    fs,
    io::ErrorKind,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use api_types::projects::{LikedProject, Project};

use crate::{constants::LIKED_PROJECTS_FILE, system::fs::launcher_dir};

static LIKED_PROJECTS_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn lock() -> Result<std::sync::MutexGuard<'static, ()>, String> {
    LIKED_PROJECTS_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|err| format!("Failed to acquire liked projects lock: {err}"))
}

fn file_path() -> Result<PathBuf, String> {
    Ok(launcher_dir()?.join(LIKED_PROJECTS_FILE))
}

fn read_liked_projects() -> Result<Vec<LikedProject>, String> {
    let path = file_path()?;

    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        Err(err) => {
            return Err(format!("Failed to read liked projects from {}: {err}", path.display()));
        }
    };

    serde_json::from_str(&contents).map_err(|err| format!("Failed to deserialize liked projects from {}: {err}", path.display()))
}

fn write_liked_projects(projects: &[LikedProject]) -> Result<(), String> {
    let path = file_path()?;

    let parent = path.parent().ok_or_else(|| "Liked projects file has no parent directory".to_string())?;

    fs::create_dir_all(parent).map_err(|err| format!("Failed to create liked projects directory {}: {err}", parent.display()))?;

    let contents = serde_json::to_string_pretty(projects).map_err(|err| format!("Failed to serialize liked projects: {err}"))?;

    let temporary_path = path.with_extension("json.tmp");

    fs::write(&temporary_path, contents)
        .map_err(|err| format!("Failed to write temporary liked projects file {}: {err}", temporary_path.display()))?;

    if path.exists() {
        fs::remove_file(&path).map_err(|err| format!("Failed to replace liked projects file {}: {err}", path.display()))?;
    }

    fs::rename(&temporary_path, &path).map_err(|err| format!("Failed to move liked projects file into place {}: {err}", path.display()))
}

/// Returns all liked projects.
pub async fn list_liked_projects() -> Result<Vec<Project>, String> {
    let liked_projects = {
        let _guard = lock()?;
        read_liked_projects()?
    };

    let mut projects = Vec::with_capacity(liked_projects.len());

    for liked_project in liked_projects {
        let project = super::management::fetch_project_from_modrinth(liked_project.project_id).await?;

        projects.push(project);
    }

    Ok(projects)
}

/// Checks whether a project is liked.
pub fn is_project_liked(project_id: &str, project_type: &str) -> Result<bool, String> {
    let _guard = lock()?;

    let projects = read_liked_projects()?;

    Ok(projects.iter().any(|project| project.project_id == project_id && project.project_type == project_type))
}

/// Adds a project to the liked projects collection.
///
/// Returns `true` if the project was added, `false` if it was already liked.
pub fn like_project(project_id: String, project_type: String) -> Result<bool, String> {
    let _guard = lock()?;

    let mut projects = read_liked_projects()?;

    if projects.iter().any(|project| project.project_id == project_id && project.project_type == project_type) {
        return Ok(false);
    }

    projects.push(LikedProject { project_id, project_type });

    write_liked_projects(&projects)?;

    Ok(true)
}

/// Removes a project from the liked projects collection.
///
/// Returns `true` if the project was removed, `false` if it was not liked.
pub fn unlike_project(project_id: &str, project_type: &str) -> Result<bool, String> {
    let _guard = lock()?;

    let mut projects = read_liked_projects()?;
    let original_len = projects.len();

    projects.retain(|project| project.project_id != project_id || project.project_type != project_type);

    if projects.len() == original_len {
        return Ok(false);
    }

    write_liked_projects(&projects)?;

    Ok(true)
}

/// Toggles a project's liked state.
///
/// Returns the new liked state.
pub fn toggle_like(project_id: String, project_type: String) -> Result<bool, String> {
    let _guard = lock()?;

    let mut projects = read_liked_projects()?;

    let index = projects.iter().position(|project| project.project_id == project_id && project.project_type == project_type);

    match index {
        Some(index) => {
            projects.remove(index);
            write_liked_projects(&projects)?;
            Ok(false)
        }
        None => {
            projects.push(LikedProject { project_id, project_type });

            write_liked_projects(&projects)?;
            Ok(true)
        }
    }
}
