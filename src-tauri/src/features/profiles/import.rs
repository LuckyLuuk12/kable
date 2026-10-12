use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{copy, Read, Write},
    path::{Component, Path, PathBuf},
};

use api_types::{
    profiles::KableProfile,
    projects::{KableProject, ProjectType},
};
use serde::{Deserialize, Serialize};
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

use crate::{
    constants::{EXPORT_FILENAME, EXPORT_FORMAT_VERSION, RUNTIME_PREFIX},
    features::{
        launcher::runtime::ensure_runtime_dir,
        projects::management::{ensure_profile_projects, list_profile_projects},
    },
    integrations::loaders::get_versions,
    system::fs as kable_fs,
    Logger,
};

const PROJECT_TYPES: [ProjectType; 3] = [ProjectType::Mod, ProjectType::Resourcepack, ProjectType::Shader];

#[derive(Debug, Serialize, Deserialize)]
struct ExportManifest {
    format_version: u32,
    profile: KableProfile,
}

/// Export a profile and its managed project files to a self-contained ZIP.
///
/// The archive contains:
/// - `kable.export.json`: the profile and export-format version.
/// - `mods/`, `resourcepacks/`, `shaderpacks/`: managed project files and metadata.
/// - `runtime/`: regular profile-specific runtime files.
///
/// Files are streamed into the ZIP rather than loaded entirely into memory.
/// Runtime symlinks are skipped.
///
/// The archive is first written to a temporary file in the destination
/// directory. The ZIP writer and underlying file are closed before the
/// completed archive is moved to its final destination.
pub async fn export_profile(profile: &KableProfile, destination: impl AsRef<Path>) -> Result<PathBuf, String> {
    let requested_destination = destination.as_ref();

    let destination = if requested_destination.is_dir() {
        requested_destination.join(format!("{}.zip", safe_archive_name(&profile.metadata.name)))
    } else {
        requested_destination.to_path_buf()
    };

    let parent = destination.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));

    kable_fs::create_dir(parent).await?;

    let filename = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Export destination has an invalid filename".to_string())?;

    let temporary_destination = parent.join(format!(".{filename}.{}.tmp", uuid::Uuid::new_v4()));

    let result = write_profile_archive(profile, &temporary_destination).await;

    if let Err(error) = result {
        if kable_fs::exists(&temporary_destination).await.unwrap_or(false) {
            let _ = kable_fs::remove_file(&temporary_destination).await;
        }

        return Err(error);
    }

    // Do not replace directories or symbolic links with an exported archive.
    match fs::symlink_metadata(&destination) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                let _ = kable_fs::remove_file(&temporary_destination).await;

                return Err(format!("Export destination is not a regular file: {}", destination.display()));
            }

            // The filesystem abstraction's rename operation does not
            // guarantee replacement of an existing file on Windows.
            if let Err(error) = kable_fs::remove_file(&destination).await {
                let _ = kable_fs::remove_file(&temporary_destination).await;

                return Err(format!("Failed to replace existing export {}: {error}", destination.display()));
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            let _ = kable_fs::remove_file(&temporary_destination).await;

            return Err(format!("Failed to inspect export destination {}: {error}", destination.display()));
        }
    }

    if let Err(error) = kable_fs::rename(&temporary_destination, &destination).await {
        let _ = kable_fs::remove_file(&temporary_destination).await;

        return Err(format!("Failed to move completed export to {}: {error}", destination.display()));
    }

    Logger::debug_global(&format!("Exported profile {} to {}", profile.id, destination.display()), Some(profile.id.as_str()));

    Ok(destination)
}

/// Build and close an export archive at a temporary destination.
async fn write_profile_archive(profile: &KableProfile, destination: &Path) -> Result<(), String> {
    let project_root = kable_fs::projects_dir()?;
    let runtime_dir = ensure_runtime_dir(&profile.id).await?;

    // create_new prevents accidentally truncating an existing file.
    // The current filesystem abstraction does not expose this operation.
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|e| format!("Failed to create temporary export archive {}: {e}", destination.display()))?;

    let mut archive = ZipWriter::new(file);

    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let manifest = ExportManifest { format_version: EXPORT_FORMAT_VERSION, profile: profile.clone() };

    let manifest_json = serde_json::to_vec_pretty(&manifest).map_err(|e| format!("Failed to serialize export manifest: {e}"))?;

    archive.start_file(EXPORT_FILENAME, options).map_err(|e| format!("Failed to create manifest entry: {e}"))?;

    archive.write_all(&manifest_json).map_err(|e| format!("Failed to write export manifest: {e}"))?;

    // These lists are independent of the project type, so fetch each only once.
    let enabled_projects = list_profile_projects(profile.clone(), true).await?;
    let disabled_projects = list_profile_projects(profile.clone(), false).await?;

    for project_type in PROJECT_TYPES {
        let projects_dir = project_root.join(project_type_folder(project_type));
        let mut exported_filenames = HashSet::new();

        for projects_by_state in [&enabled_projects, &disabled_projects] {
            let Some(projects) = projects_by_state.get(&project_type) else {
                continue;
            };

            for project in projects {
                if !exported_filenames.insert(project.filename.clone()) {
                    continue;
                }

                validate_filename(&project.filename)?;

                let source_file = projects_dir.join(&project.filename);
                let source_metadata = source_file.with_extension("json");

                if !kable_fs::is_file(&source_file).await? {
                    return Err(format!("Project file is missing while exporting {}: {}", profile.metadata.name, source_file.display()));
                }

                if !kable_fs::is_file(&source_metadata).await? {
                    return Err(format!(
                        "Project metadata is missing while exporting {}: {}",
                        profile.metadata.name,
                        source_metadata.display()
                    ));
                }

                let archive_folder = project_type_folder(project_type);

                add_file_to_archive(&mut archive, &source_file, &format!("{archive_folder}/{}", project.filename), options).await?;

                add_file_to_archive(
                    &mut archive,
                    &source_metadata,
                    &format!("{archive_folder}/{}", project.filename_with_json_extension()),
                    options,
                )
                .await?;
            }
        }
    }

    add_runtime_files(&mut archive, &runtime_dir, Path::new(""), options).await?;

    // finish() consumes the ZIP writer and returns the underlying file.
    // Sync and drop the handle before the temporary archive is renamed.
    let finished_file = archive.finish().map_err(|e| format!("Failed to finalize export archive: {e}"))?;

    finished_file.sync_all().map_err(|e| format!("Failed to flush export archive to disk: {e}"))?;

    drop(finished_file);

    Ok(())
}

/// Import an exported profile from a directory containing `kable.export.json`
/// or from a ZIP archive.
///
/// The caller generates and supplies the target profile ID. This function
/// never generates an ID or saves the profile to the profile list.
///
/// Runtime files are restored beneath the target profile's runtime directory.
/// Managed project files are restored into shared Kable project storage.
pub async fn import_profile(source: impl AsRef<Path>, target_profile_id: &str) -> Result<KableProfile, String> {
    let source = source.as_ref();

    if target_profile_id.trim().is_empty() {
        return Err("Target profile ID cannot be empty".to_string());
    }

    validate_filename(target_profile_id)?;

    let mut imported_profile;
    let mut archived_projects = Vec::new();
    let mut runtime_entries: Vec<(PathBuf, Vec<u8>)> = Vec::new();

    if source.is_dir() {
        let manifest_path = source.join(EXPORT_FILENAME);

        if !manifest_path.is_file() {
            return Err(format!("Export directory does not contain {}: {}", EXPORT_FILENAME, source.display()));
        }

        let manifest_bytes = kable_fs::read(&manifest_path).await?;

        imported_profile = parse_export_manifest(&manifest_bytes)?;

        collect_directory_projects(source, &mut archived_projects).await?;
    } else if source.is_file() {
        let file = File::open(source).map_err(|e| format!("Failed to open export archive: {e}"))?;

        let mut archive = ZipArchive::new(file).map_err(|e| format!("Invalid profile ZIP archive: {e}"))?;

        let mut manifest_bytes: Option<Vec<u8>> = None;

        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).map_err(|e| format!("Failed to read ZIP entry {index}: {e}"))?;

            let entry_name = entry.name().to_string();
            let relative_path = validate_archive_path(&entry_name)?;

            if entry.unix_mode().is_some_and(|mode| mode & 0o170000 == 0o120000) {
                return Err(format!("Profile archive contains a symlink: {entry_name}"));
            }

            if entry.is_dir() {
                continue;
            }

            let mut contents = Vec::new();

            entry.read_to_end(&mut contents).map_err(|e| format!("Failed to read ZIP entry {entry_name}: {e}"))?;

            if relative_path == Path::new(EXPORT_FILENAME) {
                if manifest_bytes.replace(contents).is_some() {
                    return Err(format!("Profile archive contains multiple {EXPORT_FILENAME} entries"));
                }

                continue;
            }

            if let Some((project_type, filename, is_metadata)) = parse_project_archive_path(&relative_path)? {
                archived_projects.push(ArchivedProject { project_type, filename, is_metadata, contents });
            } else if relative_path.starts_with(Path::new("runtime")) {
                let runtime_path = relative_path.strip_prefix("runtime").map_err(|_| "Invalid runtime archive path".to_string())?;

                if runtime_path.as_os_str().is_empty() {
                    continue;
                }

                validate_relative_path(runtime_path)?;

                runtime_entries.push((runtime_path.to_path_buf(), contents));
            }
        }

        let manifest_bytes = manifest_bytes.ok_or_else(|| format!("Profile archive does not contain {EXPORT_FILENAME}"))?;

        imported_profile = parse_export_manifest(&manifest_bytes)?;
    } else {
        return Err(format!("Profile export source does not exist or is not a file/directory: {}", source.display()));
    }

    // Never reuse the source profile ID.
    imported_profile.id = target_profile_id.to_string();

    // Reject unsupported versions before writing runtime or project files.
    let all_versions = get_versions().await?.0;

    if !all_versions.iter().any(|version| version.id == imported_profile.version.id) {
        return Err(format!("Imported profile version {} is not compatible with the current Kable version", imported_profile.version.id));
    }

    let runtime_dir = ensure_runtime_dir(&imported_profile.id).await?;

    if source.is_file() {
        for (relative_path, contents) in runtime_entries {
            write_runtime_file(&runtime_dir, &relative_path, &contents).await?;
        }
    } else {
        let runtime_source = source.join("runtime");

        if runtime_source.is_dir() {
            copy_runtime_directory(&runtime_source, &runtime_dir, Path::new("")).await?;
        }
    }

    restore_archived_projects(&archived_projects).await?;

    ensure_profile_projects(imported_profile.clone()).await?;

    Logger::debug_global(
        &format!("Imported profile {} with target ID {}", imported_profile.metadata.name, imported_profile.id),
        Some(imported_profile.id.as_str()),
    );

    // create_profile owns persistence after all sources have been merged
    // and metadata/version overrides have been applied.
    Ok(imported_profile)
}

#[derive(Debug)]
struct ArchivedProject {
    project_type: ProjectType,
    filename: String,
    is_metadata: bool,
    contents: Vec<u8>,
}

fn parse_export_manifest(bytes: &[u8]) -> Result<KableProfile, String> {
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| format!("Failed to parse export manifest JSON: {e}"))?;

    // Support the versioned export format and legacy raw KableProfile JSON.
    if value.get("profile").is_some() {
        let manifest: ExportManifest = serde_json::from_value(value).map_err(|e| format!("Invalid export manifest: {e}"))?;

        if manifest.format_version != EXPORT_FORMAT_VERSION {
            return Err(format!("Unsupported Kable export format version: {}", manifest.format_version));
        }

        Ok(manifest.profile)
    } else {
        serde_json::from_value(value).map_err(|e| format!("Failed to parse exported profile: {e}"))
    }
}

fn project_type_folder(project_type: ProjectType) -> &'static str {
    match project_type {
        ProjectType::Mod => "mods",
        ProjectType::Resourcepack => "resourcepacks",
        ProjectType::Shader => "shaderpacks",
        _ => "unsupported",
    }
}

fn validate_filename(filename: &str) -> Result<(), String> {
    if filename.is_empty()
        || filename == "."
        || filename == ".."
        || filename.contains('/')
        || filename.contains('\\')
        || Path::new(filename).components().count() != 1
        || !matches!(Path::new(filename).components().next(), Some(Component::Normal(_)))
    {
        return Err(format!("Invalid filename: {filename}"));
    }

    Ok(())
}

fn validate_relative_path(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(format!("Invalid relative path: {}", path.display()));
    }

    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            _ => {
                return Err(format!("Unsafe path in profile archive: {}", path.display()));
            }
        }
    }

    Ok(())
}

fn validate_archive_path(path: &str) -> Result<PathBuf, String> {
    if path.contains('\\') || path.contains('\0') {
        return Err(format!("ZIP entry uses an invalid path: {path}"));
    }

    let path = PathBuf::from(path);
    validate_relative_path(&path)?;

    Ok(path)
}

fn parse_project_archive_path(path: &Path) -> Result<Option<(ProjectType, String, bool)>, String> {
    let mut components = path.components();

    let Some(Component::Normal(folder)) = components.next() else {
        return Ok(None);
    };

    let folder = folder.to_str().ok_or_else(|| format!("Invalid project folder in archive path: {}", path.display()))?;

    let project_type = match folder {
        "mods" => ProjectType::Mod,
        "resourcepacks" => ProjectType::Resourcepack,
        "shaderpacks" => ProjectType::Shader,
        _ => return Ok(None),
    };

    let Some(Component::Normal(filename)) = components.next() else {
        return Err(format!("Invalid project path in archive: {}", path.display()));
    };

    if components.next().is_some() {
        return Err(format!("Project archive entries must be direct children of their project folder: {}", path.display()));
    }

    let filename = filename.to_str().ok_or_else(|| format!("Invalid project filename in archive: {}", path.display()))?.to_string();

    validate_filename(&filename)?;

    let is_metadata = filename.ends_with(".json");

    Ok(Some((project_type, filename, is_metadata)))
}

async fn restore_archived_projects(archived_projects: &[ArchivedProject]) -> Result<(), String> {
    let project_root = kable_fs::projects_dir()?;

    let mut metadata = Vec::<(ProjectType, String, KableProject)>::new();
    let mut files = Vec::<(ProjectType, String, Vec<u8>)>::new();

    // Validate all metadata before making any changes.
    for project in archived_projects {
        if project.is_metadata {
            let parsed: KableProject = serde_json::from_slice(&project.contents)
                .map_err(|e| format!("Invalid Kable project metadata for {}: {e}", project.filename))?;

            if parsed.project.project_type != project.project_type {
                return Err(format!("Project type mismatch in metadata for {}", project.filename));
            }

            validate_filename(&parsed.filename)?;

            let filename_with_ext = Path::new(&parsed.filename).with_extension("json");

            let expected_metadata_name = filename_with_ext
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| "Invalid filename in project metadata".to_string())?;

            if expected_metadata_name != project.filename {
                return Err(format!(
                    "Project metadata filename mismatch: archive has {}, metadata refers to {}",
                    project.filename, parsed.filename
                ));
            }

            metadata.push((project.project_type, project.filename.clone(), parsed));
        } else {
            files.push((project.project_type, project.filename.clone(), project.contents.clone()));
        }
    }

    // Reject duplicate archive entries instead of relying on archive order.
    let mut seen_files = HashSet::new();

    for (_, filename, _) in &files {
        if !seen_files.insert(filename.clone()) {
            return Err(format!("Project archive contains duplicate project file: {filename}"));
        }
    }

    let mut seen_metadata = HashSet::new();

    for (_, filename, _) in &metadata {
        if !seen_metadata.insert(filename.clone()) {
            return Err(format!("Project archive contains duplicate metadata file: {filename}"));
        }
    }

    // Validate file/metadata conflicts before writing anything.
    for (project_type, filename, _) in &files {
        let folder = project_root.join(project_type_folder(*project_type));
        let target = folder.join(filename);
        let target_metadata = target.with_extension("json");

        if !kable_fs::exists(&target).await? {
            continue;
        }

        let matching_metadata = metadata.iter().find(|(kind, _, project)| kind == project_type && project.filename == *filename);

        let Some((_, _, archived_metadata)) = matching_metadata else {
            return Err(format!("Project file {} already exists, but the archive has no matching metadata", target.display()));
        };

        if let Some(existing) = crate::features::projects::management::get_installed_project(*project_type, filename).await? {
            if existing.project.project_id != archived_metadata.project.project_id || existing.version_id != archived_metadata.version_id {
                return Err(format!("Project filename conflict: {} is already used by another project/version", filename));
            }
        } else if kable_fs::exists(&target_metadata).await? {
            return Err(format!("Existing metadata for {} could not be loaded safely", filename));
        }
    }

    for (project_type, metadata_filename, archived_metadata) in &metadata {
        let folder = project_root.join(project_type_folder(*project_type));
        let metadata_path = folder.join(metadata_filename);

        if !kable_fs::exists(&metadata_path).await? {
            continue;
        }

        let existing_bytes = kable_fs::read(&metadata_path).await?;

        let existing: KableProject =
            serde_json::from_slice(&existing_bytes).map_err(|e| format!("Existing project metadata is invalid: {e}"))?;

        if existing.project.project_id != archived_metadata.project.project_id || existing.version_id != archived_metadata.version_id {
            return Err(format!("Project metadata conflict for {}", metadata_filename));
        }
    }

    // Validate destination directories and files against symlink traversal.
    for (project_type, filename, _) in &files {
        let folder = project_root.join(project_type_folder(*project_type));
        let target = folder.join(filename);

        ensure_directory_without_symlinks(&folder)?;
        ensure_target_is_safe_file(&target)?;
    }

    for (project_type, metadata_filename, _) in &metadata {
        let folder = project_root.join(project_type_folder(*project_type));
        let target = folder.join(metadata_filename);

        ensure_directory_without_symlinks(&folder)?;
        ensure_target_is_safe_file(&target)?;
    }

    for (project_type, filename, contents) in files {
        let folder = project_root.join(project_type_folder(project_type));
        let target = folder.join(&filename);

        // Preserve an already-installed matching project.
        if kable_fs::exists(&target).await? {
            continue;
        }

        write_managed_file(&target, &contents).await?;
    }

    for (project_type, metadata_filename, archived_metadata) in metadata {
        let folder = project_root.join(project_type_folder(project_type));
        let metadata_path = folder.join(&metadata_filename);

        if kable_fs::exists(&metadata_path).await? {
            continue;
        }

        let project_path = folder.join(&archived_metadata.filename);

        if !kable_fs::is_file(&project_path).await? {
            return Err(format!(
                "Project metadata {} has no corresponding project file {} in the archive or global storage",
                metadata_filename,
                project_path.display()
            ));
        }

        let serialized = serde_json::to_vec_pretty(&archived_metadata).map_err(|e| format!("Failed to serialize project metadata: {e}"))?;

        write_managed_file(&metadata_path, &serialized).await?;
    }

    Ok(())
}

async fn collect_directory_projects(source: &Path, projects: &mut Vec<ArchivedProject>) -> Result<(), String> {
    for project_type in PROJECT_TYPES {
        let folder = source.join(project_type_folder(project_type));

        if !folder.is_dir() {
            continue;
        }

        for entry in fs::read_dir(&folder).map_err(|e| format!("Failed to read project folder: {e}"))? {
            let entry = entry.map_err(|e| format!("Failed to read project entry: {e}"))?;

            let file_type = entry.file_type().map_err(|e| format!("Failed to inspect project entry: {e}"))?;

            if !file_type.is_file() {
                continue;
            }

            let filename = entry.file_name().into_string().map_err(|_| "Project filename is not valid UTF-8".to_string())?;

            validate_filename(&filename)?;

            let contents = kable_fs::read(entry.path()).await?;

            projects.push(ArchivedProject { project_type, is_metadata: filename.ends_with(".json"), filename, contents });
        }
    }

    Ok(())
}

/// Stream a file into the archive without allocating a buffer for its contents.
///
/// `std::fs::File` is used here because the ZIP writer requires synchronous
/// `Write` access and system::fs currently does not expose a streaming reader.
async fn add_file_to_archive(
    archive: &mut ZipWriter<File>,
    source: &Path,
    archive_name: &str,
    options: SimpleFileOptions,
) -> Result<(), String> {
    archive.start_file(archive_name, options).map_err(|e| format!("Failed to add {archive_name} to archive: {e}"))?;

    let mut source_file = File::open(source).map_err(|e| format!("Failed to open file {}: {e}", source.display()))?;

    copy(&mut source_file, archive).map_err(|e| format!("Failed to stream {} into archive: {e}", source.display()))?;

    Ok(())
}

async fn add_runtime_files(
    archive: &mut ZipWriter<File>,
    runtime_root: &Path,
    relative_dir: &Path,
    options: SimpleFileOptions,
) -> Result<(), String> {
    let current_dir = runtime_root.join(relative_dir);

    if !kable_fs::is_dir(&current_dir).await? {
        return Ok(());
    }

    let entries = kable_fs::read_dir(&current_dir).await?;

    for entry in entries {
        let metadata = fs::symlink_metadata(&entry).map_err(|e| format!("Failed to inspect runtime entry {}: {e}", entry.display()))?;

        let file_type = metadata.file_type();

        if file_type.is_symlink() {
            continue;
        }

        let filename = entry.file_name().and_then(|name| name.to_str()).ok_or_else(|| "Runtime filename is not valid UTF-8".to_string())?;

        if relative_dir.as_os_str().is_empty() && ["mods", "resourcepacks", "shaderpacks"].contains(&filename) {
            continue;
        }

        let relative_path = relative_dir.join(filename);
        validate_relative_path(&relative_path)?;

        if file_type.is_dir() {
            Box::pin(add_runtime_files(archive, runtime_root, &relative_path, options)).await?;
        } else if file_type.is_file() {
            let archive_name = format!("{RUNTIME_PREFIX}{}", relative_path.to_string_lossy().replace('\\', "/"));

            add_file_to_archive(archive, &entry, &archive_name, options).await?;
        }
    }

    Ok(())
}

async fn copy_runtime_directory(source_root: &Path, destination_root: &Path, relative_dir: &Path) -> Result<(), String> {
    let current_dir = source_root.join(relative_dir);

    let entries = match kable_fs::read_dir(&current_dir).await {
        Ok(entries) => entries,
        Err(_error) if !current_dir.exists() => {
            return Ok(());
        }
        Err(error) => {
            return Err(format!("Failed to read runtime source {}: {error}", current_dir.display()));
        }
    };

    for entry in entries {
        let file_type = fs::symlink_metadata(&entry).map_err(|e| format!("Failed to inspect runtime entry: {e}"))?.file_type();

        if file_type.is_symlink() {
            continue;
        }

        let filename = entry.file_name().and_then(|name| name.to_str()).ok_or_else(|| "Runtime filename is not valid UTF-8".to_string())?;

        if relative_dir.as_os_str().is_empty() && ["mods", "resourcepacks", "shaderpacks"].contains(&filename) {
            continue;
        }

        let relative_path = relative_dir.join(filename);
        validate_relative_path(&relative_path)?;

        if file_type.is_dir() {
            Box::pin(copy_runtime_directory(source_root, destination_root, &relative_path)).await?;
        } else if file_type.is_file() {
            let contents = kable_fs::read(&entry).await?;

            write_runtime_file(destination_root, &relative_path, &contents).await?;
        }
    }

    Ok(())
}

async fn write_runtime_file(runtime_root: &Path, relative_path: &Path, contents: &[u8]) -> Result<(), String> {
    validate_relative_path(relative_path)?;

    let target = runtime_root.join(relative_path);
    let parent = target.parent().ok_or_else(|| "Runtime file has no parent directory".to_string())?;

    ensure_directory_without_symlinks(runtime_root)?;
    ensure_directory_without_symlinks(parent)?;
    ensure_target_is_safe_file(&target)?;

    // Remove an existing regular file first because the abstraction's atomic
    // rename behavior may not replace an existing destination on Windows.
    if fs::symlink_metadata(&target).is_ok() {
        kable_fs::remove_file(&target).await?;
    }

    kable_fs::write(&target, contents, false).await?;

    Ok(())
}

/// Create missing directories without traversing existing symlinks or files.
fn ensure_directory_without_symlinks(path: &Path) -> Result<(), String> {
    let mut current = PathBuf::new();

    for component in path.components() {
        current.push(component.as_os_str());

        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(format!("Refusing to write through symbolic link: {}", current.display()));
                }

                if !metadata.is_dir() {
                    return Err(format!("Expected a directory: {}", current.display()));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&current)
                    .map_err(|create_error| format!("Failed to create directory {}: {create_error}", current.display()))?;
            }
            Err(error) => {
                return Err(format!("Failed to inspect directory {}: {error}", current.display()));
            }
        }
    }

    Ok(())
}

fn ensure_target_is_safe_file(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(format!("Refusing to overwrite symbolic link: {}", path.display())),
        Ok(metadata) if !metadata.is_file() => Err(format!("Target is not a regular file: {}", path.display())),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("Failed to inspect target {}: {error}", path.display())),
    }
}

async fn write_managed_file(target: &Path, contents: &[u8]) -> Result<(), String> {
    let parent = target.parent().ok_or_else(|| "Managed project file has no parent directory".to_string())?;

    ensure_directory_without_symlinks(parent)?;
    ensure_target_is_safe_file(target)?;

    if kable_fs::exists(target).await? {
        return Err(format!("Refusing to overwrite existing managed project file: {}", target.display()));
    }

    kable_fs::write(target, contents, false).await?;

    Ok(())
}

fn safe_archive_name(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|character| if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') { character } else { '_' })
        .collect::<String>();

    if sanitized.trim_matches('.').is_empty() {
        "kable-profile".to_string()
    } else {
        sanitized
    }
}

trait ProjectFilenameExt {
    fn filename_with_json_extension(&self) -> String;
}

impl ProjectFilenameExt for KableProject {
    fn filename_with_json_extension(&self) -> String {
        Path::new(&self.filename)
            .with_extension("json")
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("project.json")
            .to_string()
    }
}
