use api_types::{
    mrpack::{ModpackFile, ModpackManifest},
    profiles::{KableProfile, ProfileVersion},
    projects::ProjectType,
};
use sha1::Sha1;
use sha2::{Digest, Sha512};
use std::{
    fs::{self, File},
    io::Write,
    path::{Component, Path, PathBuf},
    time::Duration,
};

use crate::{
    features::{
        launcher::runtime::ensure_runtime_dir,
        projects::management::{add_project, get_installed_project},
    },
    integrations::modrinth::{ferinth_client::get_project, mrpack},
    Logger,
};

/// Import a Modrinth modpack into a candidate Kable profile.
///
/// The caller must construct a candidate profile with its target ID and
/// resolved Minecraft/loader version before calling this function.
///
/// This function installs manifest files and client overrides into the
/// candidate profile's runtime directory and returns the modified profile
/// without saving its profile settings. The caller owns profile persistence.
///
/// Files matching a primary Modrinth CDN project file are registered with
/// Kable's project manager. Other files are downloaded directly into the
/// profile runtime and are not tracked as Kable projects.
pub async fn import_mrpack(archive_path: impl AsRef<Path>, mut profile: KableProfile) -> Result<KableProfile, String> {
    let archive_path = archive_path.as_ref();

    let manifest = mrpack::read_manifest(archive_path)?;

    validate_profile_dependencies(&profile, &manifest)?;

    if profile.metadata.name.trim().is_empty() {
        profile.metadata.name = manifest.name.clone();
    }

    if profile.metadata.description.is_none() {
        profile.metadata.description = manifest.summary.clone();
    }

    // This must use the target profile ID supplied by create_profile.
    let runtime_dir = ensure_runtime_dir(&profile.id).await?;

    Logger::debug_global(&format!("Importing Modrinth modpack '{}' into profile {}", manifest.name, profile.id), Some(profile.id.as_str()));

    // Install manifest files first. Overrides are layered over them afterward,
    // as required by the Modrinth format.
    for file in &manifest.files {
        if !mrpack::is_client_file(file) {
            continue;
        }

        install_manifest_file(&mut profile, file, &runtime_dir).await?;
    }

    mrpack::extract_client_overrides(archive_path, &runtime_dir)?;

    Logger::debug_global(
        &format!("Imported Modrinth modpack '{}' into unsaved profile {}", manifest.name, profile.id),
        Some(profile.id.as_str()),
    );

    Ok(profile)
}

/// Ensure the candidate profile was constructed for the dependencies declared
/// by the pack. Profile creation remains responsible for resolving and
/// installing the actual Minecraft/loader version.
fn validate_profile_dependencies(profile: &KableProfile, manifest: &ModpackManifest) -> Result<(), String> {
    let manifest_minecraft = manifest
        .dependencies
        .get("minecraft")
        .ok_or_else(|| "Modpack is missing its Minecraft version dependency".to_string())?;

    if profile.version.minecraft_version.as_deref() != Some(manifest_minecraft.as_str()) {
        return Err(format!(
            "Profile Minecraft version {:?} does not match modpack Minecraft version {}",
            profile.version.minecraft_version, manifest_minecraft
        ));
    }

    let loader_dependencies = [("fabric-loader", "fabric"), ("quilt-loader", "quilt"), ("forge", "forge"), ("neoforge", "neoforge")];

    let requested_loaders = loader_dependencies
        .iter()
        .filter_map(|(dependency, loader)| manifest.dependencies.get(*dependency).map(|version| (*loader, version.as_str())))
        .collect::<Vec<_>>();

    if requested_loaders.len() > 1 {
        return Err(format!(
            "Modpack declares multiple mod loaders: {}",
            requested_loaders.iter().map(|(loader, _)| *loader).collect::<Vec<_>>().join(", ")
        ));
    }

    let (expected_loader, expected_loader_version) = requested_loaders.first().copied().unwrap_or(("vanilla", ""));

    let actual_loader = normalize_loader(&profile.version.loader.to_string());

    if actual_loader != expected_loader {
        return Err(format!("Profile loader {} does not match modpack loader {}", actual_loader, expected_loader));
    }

    if !expected_loader_version.is_empty() && profile.version.loader_version.as_deref() != Some(expected_loader_version) {
        return Err(format!(
            "Profile loader version {:?} does not match modpack loader version {}",
            profile.version.loader_version, expected_loader_version
        ));
    }

    Ok(())
}

fn normalize_loader(loader: &str) -> String {
    match loader.trim().to_ascii_lowercase().replace(['_', '-', ' '], "").as_str() {
        "irisfabric" | "irisfabricloader" | "fabric" | "quiltedfabric" => "fabric".to_string(),
        "quilt" => "quilt".to_string(),
        "neoforge" => "neoforge".to_string(),
        "forge" => "forge".to_string(),
        "vanilla" => "vanilla".to_string(),
        other => other.to_string(),
    }
}

/// Install a manifest file either through Kable's project manager or directly
/// into the target profile runtime.
async fn install_manifest_file(profile: &mut KableProfile, file: &ModpackFile, runtime_dir: &Path) -> Result<(), String> {
    // Project integration is only appropriate when the manifest path maps
    // directly to a managed project file and the manifest URL matches the
    // primary file of the exact declared project version.
    if let Some((project_id, version_id)) = file.downloads.iter().find_map(|url| parse_modrinth_cdn_url(url)) {
        if try_install_managed_project(profile, file, &project_id, &version_id).await? {
            return Ok(());
        }
    }

    install_direct_file(file, runtime_dir).await
}

/// Extract project/version IDs from:
/// https://cdn.modrinth.com/data/{project_id}/versions/{version_id}/...
fn parse_modrinth_cdn_url(url: &str) -> Option<(String, String)> {
    let rest = url.strip_prefix("https://cdn.modrinth.com/")?;
    let path = rest.split(['?', '#']).next()?;
    let segments = path.split('/').collect::<Vec<_>>();

    if segments.len() < 5 || segments[0] != "data" || segments[2] != "versions" {
        return None;
    }

    let project_id = segments[1];
    let version_id = segments[3];

    if project_id.is_empty() || version_id.is_empty() || segments[4].is_empty() {
        return None;
    }

    Some((project_id.to_string(), version_id.to_string()))
}

/// Attempt to register a manifest file as a Kable-managed project.
///
/// Returns Ok(false) when the file cannot safely be represented by the
/// existing project-management API, allowing the caller to install it
/// directly at the manifest path instead.
async fn try_install_managed_project(
    profile: &mut KableProfile,
    file: &ModpackFile,
    project_id: &str,
    version_id: &str,
) -> Result<bool, String> {
    let Some((project_type, filename)) = managed_project_path(&file.path) else {
        return Ok(false);
    };

    let project = match get_project(project_id).await {
        Ok(project) => project,
        Err(error) => {
            Logger::debug_global(
                &format!("Could not resolve Modrinth project {} for {:?}; installing it directly: {}", project_id, file.path, error),
                Some(profile.id.as_str()),
            );

            return Ok(false);
        }
    };

    if project.project_id != project_id || project.project_type != project_type {
        return Ok(false);
    }

    let Some(version) = project.versions.iter().find(|version| version.id == version_id) else {
        return Ok(false);
    };

    // add_project() installs the primary version file. Only use it if that is
    // exactly the file declared in the manifest.
    let matching_primary_file = version.files.iter().find(|project_file| {
        project_file.primary
            && project_file.filename == filename
            && file.downloads.iter().any(|url| url == &project_file.url)
            && manifest_hashes_match(file, &project_file.hashes)
    });

    if matching_primary_file.is_none() {
        return Ok(false);
    }

    // Reuse an identical installation when it already exists.
    if let Some(installed) = get_installed_project(project_type, &filename).await? {
        if installed.project.project_id == project_id && installed.version_id == version_id {
            enable_profile_project(profile, project_type, &installed.filename);

            return Ok(true);
        }
    }

    let installed = add_project(project, Some(version_id)).await?;

    // The project manager's primary filename must match the path from the
    // manifest. Otherwise this file is not a safe fit for managed installation.
    if installed.filename != filename {
        return Err(format!(
            "Modrinth project {} installed as {:?}, but the modpack expects {:?}",
            project_id, installed.filename, filename
        ));
    }

    enable_profile_project(profile, project_type, &installed.filename);

    Ok(true)
}

/// Only direct files in the conventional project folders are eligible for
/// managed installation. Nested or arbitrary manifest paths remain direct
/// runtime files.
fn managed_project_path(path: &str) -> Option<(ProjectType, String)> {
    let mut components = Path::new(path).components();

    let first = match components.next()? {
        Component::Normal(value) => value.to_str()?,
        _ => return None,
    };

    let filename = match components.next()? {
        Component::Normal(value) => value.to_str()?,
        _ => return None,
    };

    if components.next().is_some() {
        return None;
    }

    let project_type = match first {
        "mods" => ProjectType::Mod,
        "resourcepacks" => ProjectType::Resourcepack,
        "shaderpacks" => ProjectType::Shader,
        _ => return None,
    };

    Some((project_type, filename.to_string()))
}

fn manifest_hashes_match(file: &ModpackFile, project_hashes: &api_types::projects::VersionFileHashes) -> bool {
    let Some(manifest_sha1) = file.hashes.get("sha1") else {
        return false;
    };

    let Some(manifest_sha512) = file.hashes.get("sha512") else {
        return false;
    };

    project_hashes.sha1.as_deref().is_some_and(|hash| manifest_sha1.eq_ignore_ascii_case(hash))
        && project_hashes.sha512.as_deref().is_some_and(|hash| manifest_sha512.eq_ignore_ascii_case(hash))
}

/// Build a download client that refuses redirects away from HTTPS.
fn https_download_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.url().scheme() != "https" {
                return attempt.error("Refusing an HTTPS-to-HTTP redirect");
            }

            reqwest::redirect::Policy::default().redirect(attempt)
        }))
        .build()
        .map_err(|error| format!("Failed to create modpack download client: {error}"))
}

/// Download a manifest file, verify its declared SHA-1 and SHA-512 hashes, and
/// write it to the path declared by the manifest inside the target runtime.
async fn install_direct_file(file: &ModpackFile, runtime_dir: &Path) -> Result<(), String> {
    let relative_path = validate_relative_path(&file.path)?;
    let target = runtime_dir.join(relative_path);

    let client = https_download_client()?;
    let mut last_error = None;

    // Try the declared mirrors in order. A failed download or hash check on
    // one URL does not prevent trying the next URL.
    for url in &file.downloads {
        let result = async {
            let parsed_url = reqwest::Url::parse(url).map_err(|error| format!("Invalid download URL: {error}"))?;

            if parsed_url.scheme() != "https" {
                return Err("Modpack file downloads must use HTTPS".to_string());
            }

            let response = client
                .get(parsed_url)
                .send()
                .await
                .map_err(|error| format!("Download request failed: {error}"))?
                .error_for_status()
                .map_err(|error| format!("Download returned an unsuccessful status: {error}"))?;

            let bytes = response.bytes().await.map_err(|error| format!("Failed to read downloaded file: {error}"))?;

            if let Some(expected_size) = file.file_size {
                if bytes.len() as u64 != expected_size {
                    return Err(format!("File {:?} has size {}, expected {}", file.path, bytes.len(), expected_size));
                }
            }

            verify_hashes(file, &bytes)?;
            write_runtime_file(runtime_dir, &target, &bytes)?;

            Ok::<(), String>(())
        }
        .await;

        match result {
            Ok(()) => {
                Logger::debug_global(&format!("Installed modpack file {:?}", file.path), None);

                return Ok(());
            }
            Err(error) => last_error = Some(format!("{url}: {error}")),
        }
    }

    Err(format!(
        "Failed to install modpack file {:?}: {}",
        file.path,
        last_error.unwrap_or_else(|| "no usable download URL".to_string())
    ))
}

fn verify_hashes(file: &ModpackFile, bytes: &[u8]) -> Result<(), String> {
    let expected_sha1 = file.hashes.get("sha1").ok_or_else(|| format!("File {:?} has no SHA-1 hash", file.path))?;

    let expected_sha512 = file.hashes.get("sha512").ok_or_else(|| format!("File {:?} has no SHA-512 hash", file.path))?;

    let actual_sha1 = format!("{:x}", Sha1::digest(bytes));
    let actual_sha512 = format!("{:x}", Sha512::digest(bytes));

    if !actual_sha1.eq_ignore_ascii_case(expected_sha1) {
        return Err(format!("SHA-1 mismatch for {:?}", file.path));
    }

    if !actual_sha512.eq_ignore_ascii_case(expected_sha512) {
        return Err(format!("SHA-512 mismatch for {:?}", file.path));
    }

    Ok(())
}

fn write_runtime_file(runtime_dir: &Path, target: &Path, bytes: &[u8]) -> Result<(), String> {
    ensure_directory_without_symlinks(runtime_dir)?;

    let parent = target.parent().ok_or_else(|| format!("Target has no parent directory: {}", target.display()))?;

    ensure_directory_without_symlinks(parent)?;
    ensure_target_is_not_symlink(target)?;

    let mut output = File::create(target).map_err(|error| format!("Failed to create {}: {error}", target.display()))?;

    output.write_all(bytes).map_err(|error| format!("Failed to write {}: {error}", target.display()))
}

fn validate_relative_path(path: &str) -> Result<PathBuf, String> {
    if path.is_empty()
        || path.starts_with('/')
        || path.starts_with('\\')
        || path.contains('\\')
        || path.contains(':')
        || path.contains('\0')
    {
        return Err(format!("Unsafe modpack file path: {path:?}"));
    }

    let mut result = PathBuf::new();

    for component in Path::new(path).components() {
        match component {
            Component::Normal(part) => result.push(part),
            _ => return Err(format!("Unsafe modpack file path: {path:?}")),
        }
    }

    if result.as_os_str().is_empty() {
        return Err(format!("Empty modpack file path: {path:?}"));
    }

    Ok(result)
}

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

fn ensure_target_is_not_symlink(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(format!("Refusing to overwrite symbolic link: {}", path.display())),
        Ok(metadata) if metadata.is_dir() => Err(format!("File target conflicts with directory: {}", path.display())),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("Failed to inspect file target {}: {error}", path.display())),
    }
}

/// Find the exact Kable version matching the Minecraft and loader dependencies
/// declared by a Modrinth modpack.
///
/// Uses the version list supplied by the caller, avoiding an additional
/// get_versions() call during profile creation.
///
/// Returns the Kable version ID. Does not create or save a profile.
pub async fn resolve_mrpack_version_id(manifest: &ModpackManifest, versions: &[ProfileVersion]) -> Result<String, String> {
    let minecraft_version = manifest
        .dependencies
        .get("minecraft")
        .ok_or_else(|| "Modrinth modpack is missing its Minecraft version dependency".to_string())?;

    let loader_dependencies = [("fabric-loader", "fabric"), ("quilt-loader", "quilt"), ("forge", "forge"), ("neoforge", "neoforge")];

    let requested_loaders = loader_dependencies
        .iter()
        .filter_map(|(dependency, loader)| manifest.dependencies.get(*dependency).map(|version| (*loader, version.as_str())))
        .collect::<Vec<_>>();

    if requested_loaders.len() > 1 {
        let loaders = requested_loaders.iter().map(|(loader, _)| *loader).collect::<Vec<_>>().join(", ");

        return Err(format!("Modrinth modpack declares multiple loaders: {loaders}"));
    }

    let (expected_loader, expected_loader_version) = requested_loaders.first().copied().unwrap_or(("vanilla", ""));

    let mut candidates = versions
        .iter()
        .filter(|version| {
            let version_id = version.id.to_ascii_lowercase();

            version_id != "iris-fabric"
                && !version_id.starts_with("iris-fabric-")
                && version.minecraft_version.as_deref() == Some(minecraft_version.as_str())
                && normalize_mrpack_loader(&version.loader.to_string()) == expected_loader
                && (expected_loader_version.is_empty() || version.loader_version.as_deref() == Some(expected_loader_version))
        })
        .collect::<Vec<_>>();

    match candidates.len() {
        0 => Err(format!(
            "No compatible Kable version found for Minecraft {} with loader {}{}",
            minecraft_version,
            expected_loader,
            if expected_loader_version.is_empty() { String::new() } else { format!(" {}", expected_loader_version) }
        )),
        1 => Ok(candidates.remove(0).id.clone()),
        count => Err(format!(
            "Found {} matching Kable versions for Minecraft {} with loader {}{}, namely: {}",
            count,
            minecraft_version,
            expected_loader,
            if expected_loader_version.is_empty() { String::new() } else { format!(" {}", expected_loader_version) },
            candidates.iter().map(|version| version.id.as_str()).collect::<Vec<_>>().join(", ")
        )),
    }
}

/// Normalize Kable loader names to the identifiers used by Modrinth
/// modpack dependencies.
fn normalize_mrpack_loader(loader: &str) -> &'static str {
    match loader.trim().to_ascii_lowercase().replace(['_', '-', ' '], "").as_str() {
        "fabric" | "fabricloader" | "irisfabric" | "irisfabricloader" => "fabric",
        "quilt" | "quiltloader" => "quilt",
        "forge" | "minecraftforge" => "forge",
        "neoforge" => "neoforge",
        "vanilla" | "none" => "vanilla",
        _ => "unknown",
    }
}

/// Download an HTTPS Modrinth modpack archive to a unique temporary file.
///
/// The caller owns the returned file and is responsible for removing it
/// after importing. Partial files are removed automatically on failure.
pub async fn download_mrpack_archive(url: &str) -> Result<PathBuf, String> {
    const MAX_ARCHIVE_SIZE: u64 = 2 * 1024 * 1024 * 1024;
    const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
    const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

    let parsed_url = reqwest::Url::parse(url).map_err(|error| format!("Invalid modpack URL: {error}"))?;

    if parsed_url.scheme() != "https" {
        return Err("Modpack downloads must use HTTPS".to_string());
    }

    if parsed_url.host_str().is_none() {
        return Err("Modpack URL has no hostname".to_string());
    }

    let client = reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.url().scheme() != "https" {
                return attempt.error("Refusing an HTTPS-to-HTTP redirect");
            }

            reqwest::redirect::Policy::default().redirect(attempt)
        }))
        .build()
        .map_err(|error| format!("Failed to create modpack download client: {error}"))?;

    let response = client
        .get(parsed_url)
        .send()
        .await
        .map_err(|error| format!("Failed to download modpack: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Modpack download returned an unsuccessful status: {error}"))?;

    if let Some(content_length) = response.content_length() {
        if content_length > MAX_ARCHIVE_SIZE {
            return Err(format!("Modpack archive exceeds the maximum allowed size of {} MiB", MAX_ARCHIVE_SIZE / (1024 * 1024)));
        }
    }

    let target = std::env::temp_dir().join(format!("kable-modpack-{}.mrpack", uuid::Uuid::new_v4()));

    let download_result = async {
        use tokio::io::AsyncWriteExt;

        let mut response = response;

        let mut output = tokio::fs::File::create(&target)
            .await
            .map_err(|error| format!("Failed to create temporary modpack archive {}: {error}", target.display()))?;

        let mut total_bytes = 0_u64;

        while let Some(chunk) = response.chunk().await.map_err(|error| format!("Failed to read modpack download: {error}"))? {
            total_bytes = total_bytes.checked_add(chunk.len() as u64).ok_or_else(|| "Modpack archive size overflow".to_string())?;

            if total_bytes > MAX_ARCHIVE_SIZE {
                return Err(format!("Modpack archive exceeds the maximum allowed size of {} MiB", MAX_ARCHIVE_SIZE / (1024 * 1024)));
            }

            output.write_all(&chunk).await.map_err(|error| format!("Failed to write temporary modpack archive: {error}"))?;
        }

        output.flush().await.map_err(|error| format!("Failed to flush modpack archive: {error}"))?;

        Ok::<(), String>(())
    }
    .await;

    if let Err(error) = download_result {
        let _ = tokio::fs::remove_file(&target).await;
        return Err(error);
    }

    Ok(target)
}

/// Enable a managed project in the candidate profile's in-memory settings.
///
/// This deliberately performs no profile persistence. The caller must persist
/// the returned profile after the import has completed, rather than writing
/// profile JSON once for every managed project.
fn enable_profile_project(profile: &mut KableProfile, project_type: ProjectType, filename: &str) {
    let mut projects = profile.settings.from(project_type);
    projects.enable(filename.to_string());
    profile.settings.set_projects(project_type, projects);
}
