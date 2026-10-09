/// # Minecraft Runtime Storage Model
///
/// Each Kable profile receives an isolated runtime directory used as
/// Minecraft's `gameDir`. Kable-managed projects are symlinked into this
/// directory, while shared Minecraft installation data is symlinked by default.
///
/// Runtime-owned settings and configuration are seeded from `.minecraft`
/// only when missing. Existing runtime data is preserved between launches.
///
/// Project directories (`mods/`, `resourcepacks/`, and `shaderpacks/`) are
/// composed per profile. Enabled Kable-managed projects take precedence over
/// global projects with the same filename.
///
/// The symlink manager owns creation, persistence, and reconciliation of
/// runtime symlinks.
use crate::constants::{MODS_DIR, RESOURCEPACKS_DIR, SHADERPACKS_DIR};
use crate::features::advanced::symlink;
use crate::system::fs;
use crate::Logger;
use api_types::profiles::KableProfile;
use api_types::projects::{KableProject, ProjectType};
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

const RUNTIME_SEED_FILES: &[&str] = &["options.txt", "optionsof.txt", "optionsshaders.txt", "servers.dat"];

const RUNTIME_SEED_DIRECTORIES: &[&str] = &["config"];

const RUNTIME_OWNED_DIRECTORIES: &[&str] = &["logs", "crash-reports", "screenshots"];

const KABLE_COMPOSED_DIRECTORIES: &[&str] = &[MODS_DIR, RESOURCEPACKS_DIR, SHADERPACKS_DIR];

const KABLE_DIRECTORIES: &[&str] = &[".kable", ".kable-dev"];

pub struct MinecraftRuntime {
    pub profile_id: String,
    pub game_dir: PathBuf,
}

impl MinecraftRuntime {
    pub fn new(profile_id: impl Into<String>, game_dir: PathBuf) -> Self {
        Self { profile_id: profile_id.into(), game_dir }
    }
}

/// Ensure the profile's runtime directory exists.
pub async fn ensure_runtime_dir(profile_id: &str) -> Result<PathBuf, String> {
    fs::create_dir(fs::runtime_dir(profile_id)?).await
}

/// Prepare a profile's runtime without overwriting existing runtime-owned data.
pub async fn prepare(profile: &KableProfile) -> Result<MinecraftRuntime, String> {
    Logger::debug_global(&format!("Preparing runtime for profile {}", profile.id), None);

    let game_dir = ensure_runtime_dir(&profile.id).await?;

    prepare_runtime_owned_data(&game_dir).await?;

    let desired_symlinks = collect_desired_symlinks(profile, &game_dir).await?;

    Logger::debug_global(&format!("Reconciling {} desired runtime symlinks for profile {}", desired_symlinks.len(), profile.id), None);

    // Do not launch if reconciliation reports an error.
    //
    // NOTE: reconcile_runtime() must propagate individual creation,
    // replacement, and deletion failures. The current symlink.rs implementation
    // logs some of these errors but still returns Ok(()); that must also be
    // corrected for this check to be fully effective.
    symlink::reconcile_runtime(&profile.id, desired_symlinks).await?;

    Logger::debug_global(&format!("Runtime preparation complete for profile {}", profile.id), None);

    Ok(MinecraftRuntime::new(profile.id.clone(), game_dir))
}

/// Build and validate the complete desired symlink set without creating links.
async fn collect_desired_symlinks(profile: &KableProfile, game_dir: &Path) -> Result<Vec<(PathBuf, PathBuf)>, String> {
    let mut desired = Vec::new();

    collect_shared_data(game_dir, &mut desired).await?;

    // The returned map is the authoritative project set for this profile.
    // Mods are never inherited from the global .minecraft/mods directory.
    let projects = crate::features::projects::management::list_profile_projects(profile.clone(), true).await?;

    Logger::debug_global(
        &format!("Profile {} enabled mods: {:?}", profile.id, profile.settings.by_project_type(ProjectType::Mod, true)),
        None,
    );

    collect_project_type(ProjectType::Mod, MODS_DIR, game_dir, &projects, &mut desired, false).await?;

    collect_project_type(ProjectType::Resourcepack, RESOURCEPACKS_DIR, game_dir, &projects, &mut desired, true).await?;

    collect_project_type(ProjectType::Shader, SHADERPACKS_DIR, game_dir, &projects, &mut desired, true).await?;

    validate_desired_destinations(&desired)?;

    Logger::debug_global(&format!("Collected {} desired runtime symlinks for profile {}", desired.len(), profile.id), None);

    Ok(desired)
}

/// Reject duplicate destinations before the symlink manager starts modifying
/// the filesystem. Windows paths are compared case-insensitively.
fn validate_desired_destinations(desired: &[(PathBuf, PathBuf)]) -> Result<(), String> {
    let mut destinations: HashMap<String, &Path> = HashMap::new();

    for (source, destination) in desired {
        if same_path(source, destination) {
            continue;
        }

        let key = path_key(destination);

        if let Some(previous_source) = destinations.insert(key, source) {
            return Err(format!(
                "Multiple runtime sources target the same destination {}: {} and {}",
                destination.display(),
                previous_source.display(),
                source.display(),
            ));
        }
    }

    Ok(())
}

/// Share top-level Minecraft entries except paths managed separately.
///
/// Existing symlinks are retained in the desired set so the symlink manager
/// can validate and reconcile them. Existing real files and directories are
/// preserved instead of being overwritten.
async fn collect_shared_data(game_dir: &Path, desired: &mut Vec<(PathBuf, PathBuf)>) -> Result<(), String> {
    let minecraft_dir = fs::mc_dir()?;

    for entry in fs::read_dir(&minecraft_dir).await? {
        let Some(filename) = entry.file_name() else {
            continue;
        };

        let filename = filename.to_string_lossy().into_owned();

        if is_shared_data_excluded(&filename) {
            continue;
        }

        let source = entry;
        let destination = game_dir.join(&filename);

        if same_path(&source, &destination) {
            continue;
        }

        if destination_exists(&destination).await? {
            let metadata = tokio::fs::symlink_metadata(&destination)
                .await
                .map_err(|error| format!("Failed to inspect runtime destination {}: {}", destination.display(), error))?;

            if !metadata.file_type().is_symlink() {
                // Preserve existing runtime-owned files and directories.
                continue;
            }
        }

        desired.push((source, destination));
    }

    Ok(())
}

fn is_shared_data_excluded(filename: &str) -> bool {
    KABLE_DIRECTORIES.contains(&filename)
        || KABLE_COMPOSED_DIRECTORIES.contains(&filename)
        || RUNTIME_SEED_FILES.contains(&filename)
        || RUNTIME_SEED_DIRECTORIES.contains(&filename)
        || RUNTIME_OWNED_DIRECTORIES.contains(&filename)
}

/// Seed known settings and configuration only when their runtime destinations
/// are absent, then ensure runtime-owned directories exist.
async fn prepare_runtime_owned_data(game_dir: &Path) -> Result<(), String> {
    let minecraft_dir = fs::mc_dir()?;

    for filename in RUNTIME_SEED_FILES {
        let source = minecraft_dir.join(filename);
        let destination = game_dir.join(filename);

        if destination_exists(&destination).await? {
            continue;
        }

        if fs::is_file(&source).await? {
            copy_file(&source, &destination).await?;
        }
    }

    for directory in RUNTIME_SEED_DIRECTORIES {
        let source = minecraft_dir.join(directory);
        let destination = game_dir.join(directory);

        if destination_exists(&destination).await? {
            continue;
        }

        if fs::is_dir(&source).await? {
            copy_directory_recursive(&source, &destination).await?;
        }
    }

    for directory in RUNTIME_OWNED_DIRECTORIES {
        fs::create_dir(game_dir.join(directory)).await?;
    }

    Ok(())
}

async fn copy_file(source: &Path, destination: &Path) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir(parent).await?;
    }

    tokio::fs::copy(source, destination)
        .await
        .map(|_| ())
        .map_err(|error| format!("Failed to copy {} to {}: {}", source.display(), destination.display(), error))
}

/// Recursively copy configuration while deliberately skipping symbolic links.
fn copy_directory_recursive<'a>(
    source: &'a Path,
    destination: &'a Path,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + 'a>> {
    Box::pin(async move {
        fs::create_dir(destination).await?;

        for entry in fs::read_dir(source).await? {
            let Some(filename) = entry.file_name() else {
                continue;
            };

            let source_path = entry.clone();
            let destination_path = destination.join(filename);

            let metadata = tokio::fs::symlink_metadata(&source_path)
                .await
                .map_err(|error| format!("Failed to inspect {}: {}", source_path.display(), error))?;

            let file_type = metadata.file_type();

            if file_type.is_symlink() {
                continue;
            }

            if file_type.is_dir() {
                if !destination_exists(&destination_path).await? {
                    copy_directory_recursive(&source_path, &destination_path).await?;
                }
            } else if file_type.is_file() && !destination_exists(&destination_path).await? {
                copy_file(&source_path, &destination_path).await?;
            }
        }

        Ok(())
    })
}

/// Collect enabled Kable projects and optionally global projects for one type.
///
/// Kable-managed projects take precedence over global projects with the same
/// filename. Duplicate Kable filenames are treated as configuration errors
/// rather than silently selecting whichever project happens to appear first.
async fn collect_project_type(
    project_type: ProjectType,
    directory: &str,
    game_dir: &Path,
    projects: &HashMap<ProjectType, Vec<KableProject>>,
    desired: &mut Vec<(PathBuf, PathBuf)>,
    include_global_projects: bool,
) -> Result<(), String> {
    let runtime_directory = game_dir.join(directory);

    fs::create_dir(runtime_directory.clone())
        .await
        .map_err(|error| format!("Failed to prepare {} directory for runtime {}: {}", directory, game_dir.display(), error))?;

    let mut kable_filenames = HashSet::new();

    if let Some(projects_for_type) = projects.get(&project_type) {
        Logger::debug_global(
            &format!(
                "[runtime] {:?}: {:?} enabled project records",
                project_type,
                projects_for_type.iter().map(|f| f.filename.clone()).collect::<Vec<_>>()
            ),
            None,
        );
        for project in projects_for_type {
            validate_project_filename(&project.filename, &project_type)?;

            let filename_key = filename_key(&project.filename);

            if !kable_filenames.insert(filename_key) {
                return Err(format!("Duplicate enabled {:?} project filename in profile: {}", project_type, project.filename));
            }
            Logger::debug_global(&format!("[runtime] Checking {:?} project filename={:?}", project_type, project.filename), None);
            let source = project_path(project, project_type).await?;
            Logger::debug_global(
                &format!("[runtime] Project source: {} (exists={}, is_file={})", source.display(), source.exists(), source.is_file()),
                None,
            );
            if !fs::is_file(&source).await? {
                return Err(format!(
                    "Enabled {:?} project '{}' is missing its project file: {}",
                    project_type,
                    project.filename,
                    source.display()
                ));
            }

            let destination = runtime_directory.join(&project.filename);

            desired.push((source, destination));
        }
    }

    if include_global_projects {
        collect_global_projects(directory, &runtime_directory, &kable_filenames, desired).await?;
    }

    Ok(())
}

/// Ensure a project filename is exactly one normal path component.
///
/// This prevents absolute paths, parent-directory traversal, and nested paths
/// from escaping the intended project directory.
fn validate_project_filename(filename: &str, project_type: &ProjectType) -> Result<(), String> {
    if filename.is_empty() {
        return Err(format!("Enabled {:?} project has an empty filename", project_type));
    }

    let mut components = Path::new(filename).components();

    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Ok(()),
        _ => Err(format!("Invalid filename for enabled {:?} project: {:?}", project_type, filename)),
    }
}

/// Collect global project files unless an enabled Kable project has the same
/// filename. Only regular files are considered.
async fn collect_global_projects(
    directory: &str,
    runtime_directory: &Path,
    kable_filenames: &HashSet<String>,
    desired: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<(), String> {
    let minecraft_directory = fs::mc_dir()?.join(directory);

    if !fs::is_dir(&minecraft_directory).await? {
        return Ok(());
    }

    for entry in fs::read_dir(&minecraft_directory).await? {
        if !entry.is_file() {
            continue;
        }

        let Some(filename) = entry.file_name() else {
            continue;
        };

        let filename = filename.to_string_lossy().into_owned();
        validate_project_filename(&filename, &ProjectType::Resourcepack)?;

        if kable_filenames.contains(&filename_key(&filename)) {
            continue;
        }

        let destination = runtime_directory.join(&filename);
        desired.push((entry, destination));
    }

    Ok(())
}

async fn project_path(project: &KableProject, project_type: ProjectType) -> Result<PathBuf, String> {
    let projects_directory = fs::projects_dir()?;

    let directory = match project_type {
        ProjectType::Mod => MODS_DIR,
        ProjectType::Resourcepack => RESOURCEPACKS_DIR,
        ProjectType::Shader => SHADERPACKS_DIR,
        _ => {
            return Err(format!("Unsupported project type: {:?}", project_type));
        }
    };

    Ok(projects_directory.join(directory).join(&project.filename))
}

/// Check existence without following symlinks, including dangling symlinks.
async fn destination_exists(path: &Path) -> Result<bool, String> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("Failed to inspect {}: {}", path.display(), error)),
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    path_key(left) == path_key(right)
}

fn path_key(path: &Path) -> String {
    let normalized = normalize_path(path);
    let value = normalized.to_string_lossy().to_string();

    #[cfg(windows)]
    {
        value.to_lowercase()
    }

    #[cfg(not(windows))]
    {
        value
    }
}

fn filename_key(filename: &str) -> String {
    #[cfg(windows)]
    {
        filename.to_lowercase()
    }

    #[cfg(not(windows))]
    {
        filename.to_string()
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }

    normalized
}

/// Explicitly reset a profile runtime. Normal preparation never calls this.
pub async fn reset_runtime_dir(profile_id: &str) -> Result<PathBuf, String> {
    symlink::cleanup_profile(profile_id).await?;

    let path = fs::runtime_dir(profile_id)?;

    if destination_exists(&path).await? {
        fs::remove_dir_all(&path)
            .await
            .map_err(|error| format!("Failed to remove runtime directory {}: {}", path.display(), error))?;
    }

    fs::create_dir(path).await
}
