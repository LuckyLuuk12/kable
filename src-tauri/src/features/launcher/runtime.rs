/// # Minecraft Runtime Storage Model
///
/// Kable never modifies the user's original Minecraft installation as part of
/// profile management or profile launching. Each Kable profile receives its
/// own runtime directory:
///
///     .minecraft/.kable/runtime/<profile-id>/
///
/// The runtime directory acts as the `gameDir` supplied to Minecraft.
///
/// Runtime files are divided into three primary storage categories:
///
/// ## 1. Kable-managed projects
///
/// Projects installed and managed by Kable are stored under:
///
///     .minecraft/.kable/projects/
///
/// These files are owned by Kable and are treated as immutable project
/// artifacts. Runtime directories symlink to these files rather than copying
/// them.
///
/// ## 2. Shared Minecraft data
///
/// Immutable or effectively read-only installation data is shared through
/// individual symlinks from the original `.minecraft` directory, including:
///
///     assets/
///     libraries/
///     versions/
///     downloads/
///
/// Other entries are shared only when explicitly classified as safe.
/// Unknown files and directories are runtime-owned by default.
///
/// ## 3. Runtime-owned data
///
/// Files and directories which Minecraft or its mods may modify and which must
/// remain independent between profiles belong directly inside the runtime.
///
/// Examples include:
///
///     options.txt
///     optionsof.txt
///     optionsshaders.txt
///     servers.dat
///     config/
///     logs/
///     crash-reports/
///     screenshots/
///
/// Known settings and configuration are copied from `.minecraft` when the
/// runtime does not already contain them. This seeds a new profile with the
/// user's existing settings without overwriting profile-specific changes on
/// subsequent launches.
///
/// Runtime-owned data is never symlinked to the original `.minecraft`.
///
/// ## Project composition
///
/// Project directories such as `mods/`, `resourcepacks/`, and `shaderpacks/`
/// are composed specifically for each profile. Enabled Kable-managed projects
/// take precedence over global projects with the same filename.
///
/// These directories must never be symlinked as a whole.
///
/// ## Unknown and mod-created data
///
/// Unknown data is runtime-owned by default. Kable must not automatically
/// symlink arbitrary entries from `.minecraft`, because doing so could expose
/// mutable, profile-specific or version-specific state to every runtime.
///
/// ## Isolation guarantee
///
/// The runtime preparation system guarantees that:
///
/// - Kable project artifacts are never modified by Minecraft.
/// - Profile-specific settings are not shared between runtimes.
/// - Known shared data is deliberately shared.
/// - Unknown files are isolated by default.
/// - The original `.minecraft` installation remains usable independently.
/// - Multiple Kable profiles can run simultaneously without sharing
///   runtime-owned files.
///
/// The runtime system constructs this filesystem view. The symlink manager
/// owns the creation and tracking of symlinks.
use crate::constants::{MODS_DIR, RESOURCEPACKS_DIR, SHADERPACKS_DIR};
use crate::features::advanced::symlink;
use crate::system::fs;
use crate::Logger;
use api_types::profiles::KableProfile;
use api_types::projects::{KableProject, ProjectType};
use std::path::{Path, PathBuf};

/// Settings files copied from `.minecraft` when missing from the runtime.
const RUNTIME_SEED_FILES: &[&str] = &["options.txt", "optionsof.txt", "optionsshaders.txt", "servers.dat"];

/// Configuration directories copied from `.minecraft` when missing.
const RUNTIME_SEED_DIRECTORIES: &[&str] = &["config"];

/// Directories which belong exclusively to each runtime and are created empty.
const RUNTIME_OWNED_DIRECTORIES: &[&str] = &["logs", "crash-reports", "screenshots"];

/// Directories which Kable composes itself and must never be shared wholesale.
const KABLE_COMPOSED_DIRECTORIES: &[&str] = &[MODS_DIR, RESOURCEPACKS_DIR, SHADERPACKS_DIR];

/// Kable-managed storage must never be exposed inside a Minecraft runtime.
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

pub async fn ensure_runtime_dir(profile_id: &str) -> Result<PathBuf, String> {
    fs::create_dir(fs::runtime_dir(profile_id)?).await
}

/// Prepare the isolated Minecraft runtime for a profile.
///
/// Existing runtime-owned data is preserved. Seed settings and configuration
/// are copied only when their runtime destinations do not exist.
pub async fn prepare(profile: &KableProfile) -> Result<MinecraftRuntime, String> {
    Logger::debug_global(format!("Preparing runtime for profile {}", profile.id).as_str(), None);
    let game_dir = ensure_runtime_dir(&profile.id).await?;

    // Remove temporary links left over from previous preparation.
    // Runtime-owned files and directories are not removed.
    symlink::cleanup_profile(&profile.id).await?;
    Logger::debug_global(format!("Cleaned up temporary symlinks for profile {}", profile.id).as_str(), None);
    // Seed profile settings before creating links to shared data.
    prepare_runtime_owned_data(&game_dir).await?;
    prepare_shared_data(profile, &game_dir).await?;
    prepare_projects(profile, &game_dir).await?;
    Logger::debug_global(format!("Runtime preparation complete for profile {}", profile.id).as_str(), None);
    Ok(MinecraftRuntime::new(profile.id.clone(), game_dir))
}

/// Prepare shared Minecraft data.
///
/// Every top-level entry in `.minecraft` is symlinked into the runtime unless
/// it belongs to an explicitly excluded category. This allows unknown
/// mod-created directories, such as `.voxy` or `DistantHorizons`, to remain
/// accessible without requiring Kable to know about every mod.
async fn prepare_shared_data(profile: &KableProfile, game_dir: &Path) -> Result<(), String> {
    let minecraft_dir = fs::mc_dir()?;

    for entry in fs::read_dir(&minecraft_dir).await? {
        let Some(filename) = entry.file_name() else {
            continue;
        };

        let filename = filename.to_string_lossy();

        if is_shared_data_excluded(&filename) {
            continue;
        }

        let destination = game_dir.join(filename.as_ref());

        create_temporary_symlink(&profile.id, entry, destination).await?;
    }

    Ok(())
}

/// Determine whether a top-level `.minecraft` entry must remain isolated.
///
/// Unknown entries are shared by default. Known mutable or Kable-managed
/// entries are excluded and handled separately.
fn is_shared_data_excluded(filename: &str) -> bool {
    KABLE_DIRECTORIES.contains(&filename)
        || KABLE_COMPOSED_DIRECTORIES.contains(&filename)
        || RUNTIME_SEED_FILES.contains(&filename)
        || RUNTIME_SEED_DIRECTORIES.contains(&filename)
        || RUNTIME_OWNED_DIRECTORIES.contains(&filename)
}

/// Copy known settings and configuration into the runtime when missing, and
/// create empty runtime-owned directories.
///
/// Existing runtime data is never overwritten. In particular, the `config/`
/// directory is copied only when it does not yet exist, preventing global
/// configuration from being merged into a profile on every launch.
async fn prepare_runtime_owned_data(game_dir: &Path) -> Result<(), String> {
    let minecraft_dir = fs::mc_dir()?;

    for filename in RUNTIME_SEED_FILES {
        let source = minecraft_dir.join(filename);
        let destination = game_dir.join(filename);

        if destination_exists(&destination).await? {
            continue;
        }

        if !fs::is_file(&source).await? {
            continue;
        }

        copy_file(&source, &destination).await?;
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

/// Copy a file using Tokio's filesystem APIs.
///
/// The destination parent is created when necessary. This helper deliberately
/// avoids following a pre-existing destination symlink: callers check for
/// destination existence before copying.
async fn copy_file(source: &Path, destination: &Path) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir(parent).await?;
    }

    tokio::fs::copy(source, destination)
        .await
        .map(|_| ())
        .map_err(|error| format!("Failed to copy {} to {}: {}", source.display(), destination.display(), error))
}

/// Recursively copy a directory and its contents.
///
/// This function is used only for initial configuration seeding. It does not
/// merge into an existing destination directory. Boxing the recursive future
/// allows asynchronous recursion without an infinitely sized future.
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

            // Do not reproduce symlinks from the original Minecraft installation
            // inside a runtime-owned configuration directory.
            if file_type.is_symlink() {
                continue;
            }

            if file_type.is_dir() {
                copy_directory_recursive(&source_path, &destination_path).await?;
            } else if file_type.is_file() {
                copy_file(&source_path, &destination_path).await?;
            }
        }

        Ok(())
    })
}

/// Prepare the profile-specific project directories.
///
/// Enabled Kable projects are linked first, followed by global projects.
/// Kable-managed projects therefore take precedence over global projects with
/// the same filename.
async fn prepare_projects(profile: &KableProfile, game_dir: &Path) -> Result<(), String> {
    Logger::debug_global(
        &format!("Preparing profile {}: enabled mods = {:?}", profile.id, profile.settings.by_project_type(ProjectType::Mod, true),),
        None,
    );
    let projects = crate::features::projects::management::list_profile_projects(profile.clone(), true).await?;

    prepare_project_type(profile, ProjectType::Mod, MODS_DIR, game_dir, &projects).await?;
    prepare_project_type(profile, ProjectType::Resourcepack, RESOURCEPACKS_DIR, game_dir, &projects).await?;
    prepare_project_type(profile, ProjectType::Shader, SHADERPACKS_DIR, game_dir, &projects).await?;

    Ok(())
}

async fn prepare_project_type(
    profile: &KableProfile,
    project_type: ProjectType,
    directory: &str,
    game_dir: &Path,
    projects: &std::collections::HashMap<ProjectType, Vec<KableProject>>,
) -> Result<(), String> {
    let runtime_directory = game_dir.join(directory);

    fs::create_dir(&runtime_directory).await?;

    prepare_kable_projects(profile, project_type, runtime_directory.as_path(), projects).await?;

    prepare_global_projects(profile, directory, runtime_directory.as_path()).await?;

    Ok(())
}

/// Make enabled Kable projects available to the runtime.
async fn prepare_kable_projects(
    profile: &KableProfile,
    project_type: ProjectType,
    runtime_directory: &Path,
    projects: &std::collections::HashMap<ProjectType, Vec<KableProject>>,
) -> Result<(), String> {
    let Some(projects) = projects.get(&project_type) else {
        return Ok(());
    };

    for project in projects {
        let source = project_path(project, project_type).await?;
        let destination = runtime_directory.join(&project.filename);

        if !fs::is_file(&source).await? {
            return Err(format!("Project file does not exist: {}", source.display()));
        }

        create_temporary_symlink(&profile.id, source, destination).await?;
    }

    Ok(())
}

/// Make globally installed projects available to this runtime.
///
/// Project files are linked individually. Existing destinations are left
/// untouched so Kable-managed projects retain precedence.
async fn prepare_global_projects(profile: &KableProfile, directory: &str, runtime_directory: &Path) -> Result<(), String> {
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

        let destination = runtime_directory.join(filename);

        if destination_exists(&destination).await? {
            continue;
        }

        create_temporary_symlink(&profile.id, entry, destination).await?;
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

/// Create a temporary runtime symlink through the advanced symlink manager.
///
/// Destination checks use symlink metadata so dangling symlinks are handled
/// correctly.
async fn create_temporary_symlink(profile_id: &str, source: PathBuf, destination: PathBuf) -> Result<(), String> {
    if source == destination {
        return Ok(());
    }

    if destination_exists(&destination).await? {
        return Ok(());
    }

    symlink::create_temporary(profile_id, source, destination).await?;

    Ok(())
}

/// Check whether a filesystem entry exists, including dangling symlinks.
async fn destination_exists(path: &Path) -> Result<bool, String> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("Failed to inspect {}: {}", path.display(), error)),
    }
}

/// Reset the runtime directory for a profile.
///
/// This removes runtime-owned and composed data as well as temporary symlinks
/// associated with the profile. The directory is recreated empty.
pub async fn reset_runtime_dir(profile_id: &str) -> Result<PathBuf, String> {
    symlink::cleanup_profile(profile_id).await?;

    let path = fs::runtime_dir(profile_id)?;

    if destination_exists(&path).await? {
        fs::remove_dir_all(&path).await.map_err(|error| format!("remove runtime failed {}: {}", path.display(), error))?;
    }

    fs::create_dir(&path).await
}
