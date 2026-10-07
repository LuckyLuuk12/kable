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
/// This includes mods, resource packs, and shader packs.
///
/// These files are owned by Kable and are treated as immutable project
/// artifacts while they are installed. Runtime directories should therefore
/// symlink to these files rather than copying them.
///
/// Example:
///
///     .kable/projects/mods/sodium.jar
///         -> runtime/<profile-id>/mods/sodium.jar
///
/// The project manager is responsible for ensuring that project files are not
/// deleted while they are still referenced by profiles. Minecraft itself only
/// consumes these files and should never modify the Kable project storage.
///
/// ## 2. Shared Minecraft data
///
/// Some files and directories can safely be shared between runtimes. These
/// should be symlinked from the original `.minecraft` directory instead of
/// being copied for every profile.
///
/// Typical examples are immutable or effectively read-only installation data,
/// such as:
///
///     assets/
///     libraries/
///     versions/
///     downloads/
///     - any folder except from the ones specified below
///
/// Other data may also be intentionally shared when its semantics require
/// global access. Shared mutable data must only be added to this category when
/// concurrent access from multiple Minecraft processes is known to be safe.
///
/// A symlink means that the runtime and the original Minecraft installation
/// refer to the same underlying data. Consequently, modifications made through
/// the runtime will modify the original data as well.
///
/// ## 3. Runtime-owned data
///
/// Files and directories that Minecraft or its mods may modify and which must
/// be independent between profiles belong directly inside the runtime.
///
/// Examples include:
///
///     options.txt
///     optionsof.txt
///     servers.dat
///     config/
///     logs/
///     crash-reports/
///     screenshots/
///
/// These entries must not be symlinked to the original `.minecraft`
/// installation. They are created or copied into the profile runtime so that
/// two simultaneously running profiles cannot overwrite each other's state.
///
/// If initial data needs to be inherited from `.minecraft`, it should be
/// copied into the runtime before launch. Subsequent changes made by Minecraft
/// remain isolated to that runtime.
///
/// ## Project composition
///
/// Project directories such as `mods/`, `resourcepacks/`, and `shaderpacks/`
/// are composed specifically for each profile.
///
/// A runtime may contain:
///
///     - globally installed user projects from `.minecraft/<directory>/`
///     - enabled Kable-managed projects from `.kable/projects/<directory>/`
///
/// Kable-managed projects take precedence over global projects with the same
/// filename. This prevents a global project from silently overriding the
/// project explicitly selected by a profile.
///
/// These directories must therefore not be symlinked as a whole. Their
/// individual entries are assembled into the runtime instead.
///
/// ## Unknown and mod-created data
///
/// Mods may create additional files and directories that are not known to
/// Kable, for example:
///
///     .voxy/
///     <mod-specific-cache>/
///
/// Unknown data is runtime-owned by default. Kable must not automatically
/// symlink arbitrary entries from `.minecraft`, because doing so could
/// accidentally expose mutable, profile-specific or version-specific state to
/// every runtime.
///
/// Some caches may eventually be classified as intentionally shared data.
/// Such entries should be explicitly added to the shared-data policy rather
/// than being shared implicitly.
///
/// ## Isolation guarantee
///
/// The runtime preparation system must guarantee that:
///
///     - Kable project artifacts are never modified by Minecraft.
///     - Profile-specific settings are never shared between runtimes.
///     - Known shared data is deliberately shared rather than accidentally
///       shared.
///     - Unknown files are isolated by default.
///     - The original `.minecraft` installation remains usable independently
///       of Kable.
///     - Multiple Kable profiles can be launched simultaneously without
///       writing to each other's runtime-owned files.
///
/// The runtime system is responsible only for constructing this filesystem
/// view. The symlink manager owns the actual creation and tracking of
/// symlinks.
use crate::constants::{MODS_DIR, RESOURCEPACKS_DIR, SHADERPACKS_DIR};
use crate::features::advanced::symlink;
use crate::system::fs;
use api_types::profiles::KableProfile;
use api_types::projects::{KableProject, ProjectType};
use std::path::{Path, PathBuf};

/// Entries in `.minecraft` which must remain isolated per Kable runtime.
///
/// Anything not listed here is considered shared Minecraft data and will be
/// exposed to the runtime through an individual symlink.
///
/// This is intentionally a conservative list of known mutable/profile-specific
/// data. Unknown Minecraft or mod-created data is shared by default.
const RUNTIME_OWNED_FILES: &[&str] = &["options.txt", "optionsof.txt", "servers.dat"];

const RUNTIME_OWNED_DIRECTORIES: &[&str] = &["config", "logs", "crash-reports", "screenshots"];

/// Directories which Kable composes itself and therefore must never be shared
/// wholesale from `.minecraft`.
const KABLE_COMPOSED_DIRECTORIES: &[&str] = &[MODS_DIR, RESOURCEPACKS_DIR, SHADERPACKS_DIR];

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
/// The returned `game_dir` should be supplied to the Minecraft version
/// manifest as its `gameDir`.
pub async fn prepare(profile: &KableProfile) -> Result<MinecraftRuntime, String> {
    let game_dir = ensure_runtime_dir(&profile.id).await?;

    // Remove temporary links left over from a previous preparation of this
    // profile. Runtime-owned files are not touched.
    symlink::cleanup_profile(&profile.id).await?;

    prepare_shared_data(profile, &game_dir).await?;
    prepare_runtime_owned_data(&game_dir).await?;
    prepare_projects(profile, &game_dir).await?;

    Ok(MinecraftRuntime::new(profile.id.clone(), game_dir))
}

/// Prepare shared Minecraft data.
///
/// Every entry in the user's real `.minecraft` directory is considered shared
/// unless it belongs to one of the explicit exclusion categories:
///
/// - `.kable`
/// - Kable-composed project directories
/// - explicitly runtime-owned files
/// - explicitly runtime-owned directories
///
/// Entries are linked individually rather than linking `.minecraft` itself.
/// This keeps Kable's own storage outside of the runtime.
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

const KABLE_DIRECTORIES: &[&str] = &[".kable", ".kable-dev"];

/// Determine whether a top-level `.minecraft` entry must not be shared.
///
/// `.kable` is always excluded because the runtime must never expose Kable's
/// internal storage. Project directories are handled separately so they can
/// be composed per profile.
fn is_shared_data_excluded(filename: &str) -> bool {
    if KABLE_DIRECTORIES.contains(&filename) {
        return true;
    }

    if KABLE_COMPOSED_DIRECTORIES.contains(&filename) {
        return true;
    }

    if RUNTIME_OWNED_FILES.contains(&filename) {
        return true;
    }

    if RUNTIME_OWNED_DIRECTORIES.contains(&filename) {
        return true;
    }

    false
}

/// Prepare directories and files which belong exclusively to this runtime.
///
/// Runtime-owned entries are deliberately created directly in the runtime
/// rather than linked to the user's original `.minecraft`.
async fn prepare_runtime_owned_data(game_dir: &Path) -> Result<(), String> {
    for directory in RUNTIME_OWNED_DIRECTORIES {
        fs::create_dir(game_dir.join(directory)).await?;
    }

    Ok(())
}

/// Prepare the profile-specific project directories.
///
/// Project directories are merged rather than linked as a whole:
///
/// - enabled Kable projects are linked into the runtime first
/// - global `.minecraft` projects are linked afterwards
/// - a global project with the same filename is therefore ignored when a
///   Kable-managed project already occupies that filename
///
/// This makes Kable-managed projects take precedence over global projects.
async fn prepare_projects(profile: &KableProfile, game_dir: &Path) -> Result<(), String> {
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

    // Kable projects are deliberately prepared first. This gives them
    // precedence over global projects with the same filename.
    prepare_kable_projects(profile, project_type, runtime_directory.as_path(), projects).await?;

    prepare_global_projects(profile, directory, runtime_directory.as_path()).await?;

    Ok(())
}

/// Make the enabled Kable projects available to the runtime.
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

        if !source.exists() {
            return Err(format!("Project file does not exist: {}", source.display()));
        }

        create_temporary_symlink(&profile.id, source, destination).await?;
    }

    Ok(())
}

/// Make projects which already exist in the user's `.minecraft` directory
/// available to this runtime.
///
/// These are deliberately linked individually instead of linking the whole
/// directory, because the runtime directory also contains profile-specific
/// Kable projects.
///
/// Kable projects are prepared before this function, so a matching filename
/// already present in the runtime is intentionally left untouched.
async fn prepare_global_projects(profile: &KableProfile, directory: &str, runtime_directory: &Path) -> Result<(), String> {
    let minecraft_directory = fs::mc_dir()?.join(directory);

    if !minecraft_directory.exists() {
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
/// Destination checks use symlink metadata rather than `Path::exists()` so
/// dangling symlinks are handled correctly.
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
/// This removes all runtime-owned and composed data as well as any temporary
/// symlinks associated with the profile. The directory is recreated empty.
pub async fn reset_runtime_dir(profile_id: &str) -> Result<PathBuf, String> {
    symlink::cleanup_profile(profile_id).await?;

    let path = fs::runtime_dir(profile_id)?;

    if destination_exists(&path).await? {
        fs::remove_dir_all(&path).await.map_err(|e| format!("remove runtime failed {}: {}", path.display(), e))?;
    }

    fs::create_dir(&path).await
}
