use std::{
    fs,
    path::{Path, PathBuf},
};

use api_types::profiles::{
    CreateProfileRequest, KableProfile, KableProfileMetadata, KableProfileSettings, LoaderKind, ModpackSource, ProfileMetadataOverrides,
    ProfileSource, ProfileVersion, Projects,
};

use crate::{
    features::{
        launcher::runtime::ensure_runtime_dir,
        modpack::mrpack_import::{download_mrpack_archive, import_mrpack, resolve_mrpack_version_id},
        profiles::kable_profile::load_profiles,
    },
    integrations::loaders::{get_version_data, get_versions},
    Logger,
};

/// Create a profile from a Minecraft version and optional metadata.
pub async fn from_loader_version(
    loader: LoaderKind,
    version: String,
    name: Option<String>,
    icon: Option<String>,
    description: Option<String>,
) -> Result<KableProfile, String> {
    let version_data = get_version_data(loader, version.clone(), false).await?;
    let now = chrono::Utc::now().to_rfc3339();

    Ok(KableProfile {
        id: uuid::Uuid::new_v4().to_string(),
        metadata: KableProfileMetadata {
            name: name.unwrap_or_else(|| format!("{:?}-{}", loader, version)),
            icon,
            description,
            created: now.clone(),
            last_used: now,
            favorite: false,
            total_time_played_ms: 0,
            times_launched: 0,
        },
        version: version_data,
        settings: KableProfileSettings {
            launch_with_account: None,
            java_args: Vec::new(),
            parameters_map: std::collections::HashMap::new(),
            enable_pack_merging: false,
            pack_order: Vec::new(),
            merged_packs: Vec::new(),
            mods: Projects::default(),
            resourcepacks: Projects::default(),
            shaders: Projects::default(),
        },
    })
}

/// Create a profile by resolving and merging the requested sources.
///
/// Fetch version metadata at most once for this operation. Sources that do not
/// require version metadata can still be resolved without fetching it.
pub async fn create_profile(request: CreateProfileRequest) -> Result<KableProfile, String> {
    Logger::debug(&format!("[create_profile] Creating profile with request: {:?}", request), None);

    let CreateProfileRequest { base, additional_sources, version_override, metadata, update_projects } = request;

    let mut sources = Vec::with_capacity(additional_sources.len() + usize::from(base.is_some()));

    if let Some(base) = base {
        sources.push(base);
    }

    sources.extend(additional_sources);

    if sources.is_empty() {
        return Err("At least one profile source must be provided".to_string());
    }

    let needs_versions = version_override.is_some()
        || sources.iter().any(|source| matches!(source, ProfileSource::Version { .. } | ProfileSource::ModrinthModpack { .. }));

    let versions = if needs_versions { Some(get_versions().await?.0) } else { None };

    let versions = versions.as_deref();

    let mut sources = sources.into_iter();
    let first_source = sources.next().ok_or_else(|| "At least one profile source must be provided".to_string())?;

    let target_profile_id = generate_unique_profile_id().await?;

    let mut profile = resolve_source(first_source, &target_profile_id, versions).await?;

    // Keep the target identity stable if a merge copies identity fields from
    // an incoming profile.
    profile.id.clone_from(&target_profile_id);

    // Some project-management operations require the profile to be persisted.
    ensure_profile_exists(&profile).await?;

    for source in sources {
        let mut incoming_profile = resolve_source(source, &target_profile_id, versions).await?;

        profile = crate::features::profiles::merge::merge_profiles(&mut profile, &mut incoming_profile)?;

        profile.id.clone_from(&target_profile_id);
    }

    if let Some(version_id) = version_override {
        let versions = versions.ok_or_else(|| "Version metadata was not loaded for the requested version override".to_string())?;

        apply_version_override(&mut profile, &version_id, versions, update_projects).await?;
    }

    apply_metadata_overrides(&mut profile, metadata);

    Logger::debug(
        &format!(
            "[create_profile] Resolved profile id: {}, version_id: {}, loader: {:?}",
            profile.id, profile.version.id, profile.version.loader
        ),
        None,
    );

    persist_profile(profile).await
}

/// Resolve a source into the candidate profile identity.
async fn resolve_source(
    source: ProfileSource,
    target_profile_id: &str,
    versions: Option<&[ProfileVersion]>,
) -> Result<KableProfile, String> {
    match source {
        ProfileSource::Version { version_id } => {
            let versions = require_versions(versions)?;
            new_profile(version_id, target_profile_id, versions)
        }

        ProfileSource::ExistingProfile { profile_id } => {
            let existing_profiles = load_profiles().await?;

            let mut profile = existing_profiles
                .into_iter()
                .find(|profile| profile.id == profile_id)
                .ok_or_else(|| format!("Existing profile with ID '{}' was not found", profile_id))?;

            let original_profile_id = profile.id.clone();

            // Copy runtime data before replacing the ID. Files are copied into
            // the target runtime, not shared with the original profile.
            copy_profile_runtime(&original_profile_id, target_profile_id).await?;

            profile.id = target_profile_id.to_string();

            Ok(profile)
        }

        ProfileSource::ExportedProfile { path } => {
            crate::features::profiles::import::import_profile(PathBuf::from(path), target_profile_id).await
        }

        ProfileSource::ExternalProfile { launcher, path } => {
            Err(format!("Importing profiles from external launcher '{:?}' is not implemented yet (path: '{}')", launcher, path))
        }

        ProfileSource::ModrinthModpack { source } => {
            let versions = require_versions(versions)?;
            resolve_modrinth_modpack(source, target_profile_id, versions).await
        }

        ProfileSource::CurseForgeModpack { source } => {
            Err(format!("CurseForge modpack import is not implemented yet (source: {:?})", source))
        }
    }
}

/// Resolve a Modrinth modpack archive into the shared candidate profile ID.
async fn resolve_modrinth_modpack(
    source: ModpackSource,
    target_profile_id: &str,
    versions: &[ProfileVersion],
) -> Result<KableProfile, String> {
    match source {
        ModpackSource::File { path } => import_mrpack_from_path(PathBuf::from(path), target_profile_id, versions).await,

        ModpackSource::Url { url } => {
            let archive_path = download_mrpack_archive(&url).await?;

            let result = import_mrpack_from_path(archive_path.clone(), target_profile_id, versions).await;

            if let Err(error) = tokio::fs::remove_file(&archive_path).await {
                Logger::warn_global(&format!("Failed to remove temporary modpack archive {}: {}", archive_path.display(), error), None);
            }

            result
        }

        ModpackSource::Project { project_id, version_id } => Err(format!(
            "Modrinth project-based modpack resolution is not implemented yet: project '{}', version {:?}",
            project_id, version_id
        )),
    }
}

/// Read a modpack manifest, create a candidate profile with the target ID,
/// register it if necessary, and return the in-memory profile after import.
async fn import_mrpack_from_path(
    archive_path: PathBuf,
    target_profile_id: &str,
    versions: &[ProfileVersion],
) -> Result<KableProfile, String> {
    let manifest = crate::integrations::modrinth::mrpack::read_manifest(&archive_path)?;

    // Reuse the version list already fetched by create_profile().
    let version_id = resolve_mrpack_version_id(&manifest, versions).await?;

    let mut profile = new_profile(version_id, target_profile_id, versions)?;

    profile.metadata.name = manifest.name.clone();
    profile.metadata.description = manifest.summary.clone();

    // Some project-management operations may require the profile to exist.
    ensure_profile_exists(&profile).await?;

    // Keep the returned in-memory profile, including its enabled-project states.
    profile = import_mrpack(&archive_path, profile).await?;

    Ok(profile)
}

/// Ensure a profile with this ID exists in persistent storage.
///
/// This is intentionally insert-only when the profile already exists.
async fn ensure_profile_exists(profile: &KableProfile) -> Result<(), String> {
    let profiles = load_profiles().await?;

    if profiles.iter().any(|existing| existing.id == profile.id) {
        return Ok(());
    }

    super::management::save_profile(profile.clone()).await?;

    Ok(())
}

/// Insert a new profile or update an existing profile.
///
/// Project entries written by import or project-management operations may
/// already exist on disk. Preserve persisted entries that are absent from the
/// candidate, while allowing the candidate's explicit states to take
/// precedence when the same project appears in both.
async fn persist_profile(mut profile: KableProfile) -> Result<KableProfile, String> {
    let profiles = load_profiles().await?;

    if let Some(existing_profile) = profiles.into_iter().find(|existing| existing.id == profile.id) {
        preserve_persisted_project_entries(&mut profile, &existing_profile);

        super::management::modify_profile(existing_profile, profile).await
    } else {
        super::management::save_profile(profile).await
    }
}

/// Preserve project states recorded on disk when they are missing from the
/// candidate profile. Candidate states win when both profiles contain the
/// same project, maintaining the merge policy used by merge_profiles().
fn preserve_persisted_project_entries(candidate: &mut KableProfile, persisted: &KableProfile) {
    for project_type in [
        api_types::projects::ProjectType::Mod,
        api_types::projects::ProjectType::Resourcepack,
        api_types::projects::ProjectType::Shader,
    ] {
        let mut persisted_settings = persisted.settings.clone();
        let persisted_projects = persisted_settings.from(project_type).clone();

        let candidate_projects = candidate.settings.from(project_type).clone();

        let merged_projects = merge_project_states(persisted_projects, candidate_projects);

        candidate.settings.set_projects(project_type, merged_projects);
    }
}

/// Merge project states with the second configuration taking precedence.
fn merge_project_states(first: Projects, second: Projects) -> Projects {
    let mut enabled = first.enabled;
    let mut disabled = first.disabled;

    for project in second.enabled {
        disabled.remove(&project);
        enabled.insert(project);
    }

    for project in second.disabled {
        enabled.remove(&project);
        disabled.insert(project);
    }

    Projects { enabled, disabled }
}

/// Apply the final version override and refresh project compatibility.
///
/// The cached version list is reused; this function does not fetch it again.
async fn apply_version_override(
    profile: &mut KableProfile,
    version_id: &str,
    versions: &[ProfileVersion],
    update_projects: bool,
) -> Result<(), String> {
    profile.version = versions
        .iter()
        .find(|version| version.id == version_id)
        .cloned()
        .ok_or_else(|| format!("Version data for '{}' not found", version_id))?;

    // These functions may require the profile to be present in persistent
    // storage, so make sure it has been registered before invoking them.
    ensure_profile_exists(profile).await?;

    if update_projects {
        for project_type in [
            api_types::projects::ProjectType::Mod,
            api_types::projects::ProjectType::Resourcepack,
            api_types::projects::ProjectType::Shader,
        ] {
            crate::features::projects::management::update_all_projects(profile.clone(), project_type).await?;
        }

        crate::features::projects::management::check_incompatible_projects(profile.clone(), true).await?;
    }
    // Refresh from disk in case compatibility operations changed the profile.
    *profile = super::management::get_profile(&profile.id).await?;

    Ok(())
}

/// Apply only metadata fields explicitly supplied by the request.
fn apply_metadata_overrides(profile: &mut KableProfile, overrides: ProfileMetadataOverrides) {
    if let Some(name) = overrides.name {
        profile.metadata.name = name;
    }

    if let Some(icon) = overrides.icon {
        profile.metadata.icon = Some(icon);
    }

    if let Some(description) = overrides.description {
        profile.metadata.description = Some(description);
    }
}

/// Return the cached version slice, or an error if the operation needs version
/// metadata but none was loaded.
fn require_versions(versions: Option<&[ProfileVersion]>) -> Result<&[ProfileVersion], String> {
    versions.ok_or_else(|| "Version metadata is required for this profile source but was not loaded".to_string())
}

/// Allocate a profile ID that does not collide with a currently saved profile.
async fn generate_unique_profile_id() -> Result<String, String> {
    let existing_profiles = load_profiles().await?;

    loop {
        let id = uuid::Uuid::new_v4().to_string();

        if !existing_profiles.iter().any(|profile| profile.id == id) {
            return Ok(id);
        }
    }
}

/// Construct an unsaved profile for the requested Kable version using the
/// identity allocated by create_profile.
fn new_profile(version_id: String, target_profile_id: &str, versions: &[ProfileVersion]) -> Result<KableProfile, String> {
    let version_data = versions
        .iter()
        .find(|version| version.id == version_id)
        .cloned()
        .ok_or_else(|| format!("Version data for '{}' not found", version_id))?;

    let now = chrono::Utc::now().to_rfc3339();

    Ok(KableProfile {
        id: target_profile_id.to_string(),
        metadata: KableProfileMetadata {
            name: format!("KableProfile ({})", version_id),
            icon: None,
            description: Some(format!("Profile created from version '{}' at '{}' through the Kable Launcher", version_id, now)),
            created: now.clone(),
            last_used: now,
            favorite: false,
            total_time_played_ms: 0,
            times_launched: 0,
        },
        version: version_data,
        settings: KableProfileSettings {
            launch_with_account: None,
            java_args: Vec::new(),
            parameters_map: std::collections::HashMap::new(),
            enable_pack_merging: true,
            pack_order: Vec::new(),
            merged_packs: Vec::new(),
            mods: Projects::default(),
            resourcepacks: Projects::default(),
            shaders: Projects::default(),
        },
    })
}

/// Copy regular files from an existing profile's runtime into the new
/// candidate's runtime. Symbolic links are deliberately skipped because
/// Kable uses them for shared project files.
async fn copy_profile_runtime(source_profile_id: &str, target_profile_id: &str) -> Result<(), String> {
    let source_dir = ensure_runtime_dir(source_profile_id).await?;
    let target_dir = ensure_runtime_dir(target_profile_id).await?;

    copy_runtime_entries(&source_dir, &target_dir)
}

/// Recursively copy runtime entries without following symbolic links.
fn copy_runtime_entries(source: &Path, target: &Path) -> Result<(), String> {
    for entry in fs::read_dir(source).map_err(|error| format!("Failed to read runtime directory {}: {error}", source.display()))? {
        let entry = entry.map_err(|error| format!("Failed to read runtime entry: {error}"))?;

        let source_path = entry.path();
        let target_path = target.join(entry.file_name());

        let metadata = fs::symlink_metadata(&source_path)
            .map_err(|error| format!("Failed to inspect runtime entry {}: {error}", source_path.display()))?;

        if metadata.file_type().is_symlink() {
            continue;
        }

        if metadata.is_dir() {
            ensure_target_directory(&target_path)?;
            copy_runtime_entries(&source_path, &target_path)?;
        } else if metadata.is_file() {
            ensure_target_directory(target_path.parent().ok_or_else(|| format!("Runtime file has no parent: {}", target_path.display()))?)?;

            ensure_target_file_is_not_symlink(&target_path)?;

            fs::copy(&source_path, &target_path)
                .map_err(|error| format!("Failed to copy runtime file {} to {}: {error}", source_path.display(), target_path.display()))?;
        }
    }

    Ok(())
}

/// Create a directory tree without traversing existing symbolic links.
fn ensure_target_directory(path: &Path) -> Result<(), String> {
    let mut current = PathBuf::new();

    for component in path.components() {
        current.push(component.as_os_str());

        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(format!("Refusing to write through symbolic link {}", current.display()));
                }

                if metadata.is_file() {
                    fs::remove_file(&current)
                        .map_err(|error| format!("Failed to replace file {} with directory: {error}", current.display()))?;

                    fs::create_dir(&current).map_err(|error| format!("Failed to create directory {}: {error}", current.display()))?;
                } else if !metadata.is_dir() {
                    return Err(format!("Runtime path is not a directory: {}", current.display()));
                }
            }

            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&current).map_err(|error| format!("Failed to create directory {}: {error}", current.display()))?;
            }

            Err(error) => {
                return Err(format!("Failed to inspect runtime directory {}: {error}", current.display()));
            }
        }
    }

    Ok(())
}

/// Ensure copying a runtime file cannot overwrite a symbolic link or directory.
fn ensure_target_file_is_not_symlink(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(format!("Refusing to overwrite symbolic link {}", path.display())),

        Ok(metadata) if metadata.is_dir() => {
            fs::remove_dir_all(path).map_err(|error| format!("Failed to replace runtime directory {} with a file: {error}", path.display()))
        }

        Ok(_) => Ok(()),

        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),

        Err(error) => Err(format!("Failed to inspect runtime file {}: {error}", path.display())),
    }
}
