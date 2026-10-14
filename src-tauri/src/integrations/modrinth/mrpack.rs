use api_types::mrpack::{EnvironmentSupport, ModpackManifest};
use std::{
    fs::{self, File},
    io,
    path::{Component, Path, PathBuf},
};
use zip::ZipArchive;

const MANIFEST_FILE: &str = "modrinth.index.json";
const SHARED_OVERRIDES: &str = "overrides/";
const CLIENT_OVERRIDES: &str = "client-overrides/";

/// Read and validate the manifest at the root of a Modrinth modpack archive.
///
/// This function does not create profiles, download project files, or modify
/// the destination runtime.
pub fn read_manifest(path: impl AsRef<Path>) -> Result<ModpackManifest, String> {
    let path = path.as_ref();
    let file = File::open(path).map_err(|error| format!("Failed to open modpack {}: {error}", path.display()))?;

    let mut archive = ZipArchive::new(file).map_err(|error| format!("Invalid modpack ZIP archive {}: {error}", path.display()))?;

    let manifest = {
        let mut entry = archive.by_name(MANIFEST_FILE).map_err(|_| format!("Modpack {} is missing {MANIFEST_FILE}", path.display()))?;

        if entry.is_dir() {
            return Err(format!("Modpack manifest in {} is a directory", path.display()));
        }

        let mut contents = String::new();
        io::Read::read_to_string(&mut entry, &mut contents).map_err(|error| format!("Failed to read {MANIFEST_FILE}: {error}"))?;

        serde_json::from_str::<ModpackManifest>(&contents).map_err(|error| format!("Invalid {MANIFEST_FILE}: {error}"))?
    };

    validate_manifest(&manifest)?;

    Ok(manifest)
}

/// Validate manifest-level constraints and every declared file.
///
/// This validates the format's structure and security-sensitive fields.
/// It does not guarantee that download URLs are reachable or that downloaded
/// content matches the declared hashes.
pub fn validate_manifest(manifest: &ModpackManifest) -> Result<(), String> {
    if manifest.format_version != 1 {
        return Err(format!("Unsupported Modrinth modpack format version: {}", manifest.format_version));
    }

    if manifest.game != "minecraft" {
        return Err(format!("Unsupported Modrinth modpack game: {:?}", manifest.game));
    }

    if manifest.name.trim().is_empty() {
        return Err("Modrinth modpack name cannot be empty".to_string());
    }

    if manifest.version_id.trim().is_empty() {
        return Err("Modrinth modpack version ID cannot be empty".to_string());
    }

    match manifest.dependencies.get("minecraft") {
        Some(version) if !version.trim().is_empty() => {}
        _ => {
            return Err("Modrinth modpack is missing its Minecraft version dependency".to_string());
        }
    }

    for (index, file) in manifest.files.iter().enumerate() {
        validate_relative_path(&file.path).map_err(|error| format!("Invalid path for modpack file at index {index}: {error}"))?;

        if file.downloads.is_empty() {
            return Err(format!("Modpack file {:?} has no download URLs", file.path));
        }

        for url in &file.downloads {
            validate_download_url(url).map_err(|error| format!("Invalid download URL for modpack file {:?}: {error}", file.path))?;
        }

        validate_hash(&file.hashes, "sha1", 40, &file.path)?;
        validate_hash(&file.hashes, "sha512", 128, &file.path)?;

        if let Some(environment) = &file.env {
            // The enum's Deserialize implementation already restricts these
            // values to required, optional, or unsupported.
            let _ = (environment.client, environment.server);
        }
    }

    Ok(())
}

fn validate_hash(
    hashes: &std::collections::HashMap<String, String>,
    algorithm: &str,
    expected_length: usize,
    file_path: &str,
) -> Result<(), String> {
    let hash = hashes
        .get(algorithm)
        .ok_or_else(|| format!("Modpack file {:?} is missing its required {} hash", file_path, algorithm))?;

    if hash.len() != expected_length || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("Modpack file {:?} has an invalid {} hash", file_path, algorithm));
    }

    Ok(())
}

/// Validate a URL before it is passed to the download layer.
///
/// Modrinth's format uses HTTPS download URLs. This check deliberately does
/// not fetch the URL or follow redirects.
fn validate_download_url(url: &str) -> Result<(), String> {
    if url.trim() != url || url.chars().any(char::is_whitespace) {
        return Err("URL contains whitespace".to_string());
    }

    let Some(authority_and_path) = url.strip_prefix("https://") else {
        return Err("Only HTTPS download URLs are accepted".to_string());
    };

    let authority = authority_and_path.split('/').next().unwrap_or_default();

    if authority.is_empty() || authority.starts_with(':') {
        return Err("URL has no valid host".to_string());
    }

    if authority.contains('@') {
        return Err("URLs containing embedded credentials are not accepted".to_string());
    }

    Ok(())
}

/// Validate a portable relative path from a manifest or archive.
///
/// Both slash styles are checked explicitly so that a path cannot become
/// traversal when moved between Unix and Windows.
fn validate_relative_path(path: &str) -> Result<PathBuf, String> {
    if path.is_empty() {
        return Err("Path cannot be empty".to_string());
    }

    if path.starts_with('/') || path.starts_with('\\') || path.contains('\\') || path.contains(':') || path.contains('\0') {
        return Err(format!("Path is absolute or contains unsafe characters: {path:?}"));
    }

    let mut result = PathBuf::new();

    for component in Path::new(path).components() {
        match component {
            Component::Normal(part) => result.push(part),
            _ => return Err(format!("Path is not a safe relative path: {path:?}")),
        }
    }

    if result.as_os_str().is_empty() {
        return Err(format!("Path cannot be empty: {path:?}"));
    }

    Ok(result)
}

/// Extract shared and client-specific overrides into a destination directory.
///
/// Extraction order is deterministic:
/// 1. `overrides/`
/// 2. `client-overrides/`
///
/// Files in `client-overrides/` overwrite files from `overrides/`.
/// `server-overrides/` and all unrelated archive entries are ignored.
///
/// The destination is supplied by the caller. This function does not know
/// about Kable profile IDs or runtime directory layout.
pub fn extract_client_overrides(archive_path: impl AsRef<Path>, destination: impl AsRef<Path>) -> Result<(), String> {
    let archive_path = archive_path.as_ref();
    let destination = destination.as_ref();

    let file = File::open(archive_path).map_err(|error| format!("Failed to open modpack {}: {error}", archive_path.display()))?;

    let mut archive = ZipArchive::new(file).map_err(|error| format!("Invalid modpack ZIP archive {}: {error}", archive_path.display()))?;

    // Validate the manifest before extracting anything.
    let manifest = {
        let mut entry = archive.by_name(MANIFEST_FILE).map_err(|_| format!("Modpack is missing {MANIFEST_FILE}"))?;

        let mut contents = String::new();
        io::Read::read_to_string(&mut entry, &mut contents).map_err(|error| format!("Failed to read {MANIFEST_FILE}: {error}"))?;

        serde_json::from_str::<ModpackManifest>(&contents).map_err(|error| format!("Invalid {MANIFEST_FILE}: {error}"))?
    };

    validate_manifest(&manifest)?;

    extract_override_layer(&mut archive, SHARED_OVERRIDES, destination)?;
    extract_override_layer(&mut archive, CLIENT_OVERRIDES, destination)?;

    Ok(())
}

fn extract_override_layer<R: io::Read + io::Seek>(archive: &mut ZipArchive<R>, prefix: &str, destination: &Path) -> Result<(), String> {
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| format!("Failed to read modpack archive entry: {error}"))?;

        let archive_name = entry.name().to_string();

        if !archive_name.starts_with(prefix) {
            continue;
        }

        let relative_name = &archive_name[prefix.len()..];

        // The root directory entry itself has no content to extract.
        if relative_name.is_empty() {
            continue;
        }

        let relative_path =
            validate_relative_path(relative_name).map_err(|error| format!("Unsafe path in modpack override archive: {error}"))?;

        // ZIP entries should contain regular files and directories, not links
        // or special files. Reject symbolic links where Unix mode metadata
        // identifies them.
        if let Some(mode) = entry.unix_mode() {
            let file_type = mode & 0o170000;

            if file_type == 0o120000 {
                return Err(format!("Symbolic links are not allowed in modpack overrides: {archive_name}"));
            }

            if file_type != 0 && file_type != 0o100000 && file_type != 0o040000 {
                return Err(format!("Unsupported special file in modpack overrides: {archive_name}"));
            }
        }

        let target = destination.join(&relative_path);

        if entry.is_dir() {
            ensure_directory_without_symlinks(&target)?;
            continue;
        }

        let parent = target.parent().ok_or_else(|| format!("Override path has no parent: {}", target.display()))?;

        ensure_directory_without_symlinks(parent)?;
        ensure_target_is_not_symlink(&target)?;

        let mut output = File::create(&target).map_err(|error| format!("Failed to create override file {}: {error}", target.display()))?;

        io::copy(&mut entry, &mut output).map_err(|error| format!("Failed to extract override file {}: {error}", target.display()))?;
    }

    Ok(())
}

/// Create each directory component individually, rejecting existing symlinks
/// and non-directory components.
fn ensure_directory_without_symlinks(path: &Path) -> Result<(), String> {
    let mut current = PathBuf::new();

    for component in path.components() {
        current.push(component.as_os_str());

        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(format!("Refusing to extract through symbolic link: {}", current.display()));
                }

                if !metadata.is_dir() {
                    return Err(format!("Expected a directory during override extraction: {}", current.display()));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                fs::create_dir(&current)
                    .map_err(|create_error| format!("Failed to create override directory {}: {create_error}", current.display()))?;
            }
            Err(error) => {
                return Err(format!("Failed to inspect override directory {}: {error}", current.display()));
            }
        }
    }

    Ok(())
}

/// Reject an existing symbolic link or directory at a file destination.
/// Existing regular files may be overwritten by a later override layer.
fn ensure_target_is_not_symlink(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(format!("Refusing to overwrite symbolic link during override extraction: {}", path.display()))
        }
        Ok(metadata) if metadata.is_dir() => Err(format!("Override file conflicts with an existing directory: {}", path.display())),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("Failed to inspect override destination {}: {error}", path.display())),
    }
}

/// Return whether a manifest file should be installed on the client.
///
/// Missing `env` means the file is not explicitly restricted by the manifest.
/// Optional files are included by default; a UI can offer a selection step
/// before installation if desired.
pub fn is_client_file(manifest_file: &api_types::mrpack::ModpackFile) -> bool {
    manifest_file.env.as_ref().map(|environment| environment.client != EnvironmentSupport::Unsupported).unwrap_or(true)
}
