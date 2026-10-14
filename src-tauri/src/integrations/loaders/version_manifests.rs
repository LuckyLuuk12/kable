use api_types::profiles::{LoaderKind, ProfileVersion};
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::constants::{
    FABRIC_LOADER_PROFILE_PATH, FABRIC_META_URL, FORGE_INSTALLER_PATH, FORGE_INSTALLER_SUFFIX, FORGE_INSTALL_CLIENT_FLAG,
    FORGE_MAVEN_BASE_URL, JAVA_JAR_FLAG, LOADER_INSTALLERS_DIR, LOADER_PROFILE_JSON_PATH, MINECRAFT_VERSION_MANIFEST_URL,
    NEOFORGE_INSTALLER_PATH, NEOFORGE_INSTALLER_SUFFIX, NEOFORGE_INSTALL_CLIENT_FLAG, NEOFORGE_MAVEN_BASE_URL, QUILT_LOADER_PROFILE_PATH,
    QUILT_META_URL, VERSIONS_DIR,
};
use crate::system::{fs, java};

/// Ensures the selected version manifest and all inherited manifests exist locally
/// under `.minecraft/versions/`.
pub async fn ensure_version_installed(version: &ProfileVersion) -> Result<(), String> {
    ensure_safe_version_id(&version.id)?;

    let manifest_path = version_manifest_path(&version.id)?;

    if fs::is_file(&manifest_path).await? {
        validate_local_manifest(&manifest_path, &version.id).await?;
    } else {
        match &version.loader {
            LoaderKind::Vanilla => {
                ensure_vanilla_manifest(&version.id).await?;
            }
            LoaderKind::Fabric => {
                let url = fabric_profile_url(version)?;
                download_manifest(&url, &manifest_path, &version.id).await?;
            }
            LoaderKind::IrisFabric => {
                ensure_iris_fabric_installed(version).await?;
            }
            LoaderKind::Quilt => {
                let url = quilt_profile_url(version)?;
                download_manifest(&url, &manifest_path, &version.id).await?;
            }
            LoaderKind::Forge => {
                ensure_forge_installed(version).await?;
            }
            LoaderKind::NeoForge => {
                ensure_neoforge_installed(version).await?;
            }
        }
    }

    ensure_manifest_parents(&version.id).await
}

/// Ensures an official vanilla Minecraft version manifest exists locally.
async fn ensure_vanilla_manifest(version_id: &str) -> Result<(), String> {
    ensure_safe_version_id(version_id)?;

    let path = version_manifest_path(version_id)?;

    if fs::is_file(&path).await? {
        validate_local_manifest(&path, version_id).await?;
        return Ok(());
    }

    let response = reqwest::get(MINECRAFT_VERSION_MANIFEST_URL)
        .await
        .map_err(|e| format!("Failed to fetch Minecraft version catalog: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Minecraft version catalog request failed: {e}"))?;

    let catalog: Value = response.json().await.map_err(|e| format!("Failed to parse Minecraft version catalog: {e}"))?;

    let versions = catalog
        .get("versions")
        .and_then(Value::as_array)
        .ok_or_else(|| "Minecraft version catalog has no versions array".to_string())?;

    let entry = versions
        .iter()
        .find(|entry| entry.get("id").and_then(Value::as_str) == Some(version_id))
        .ok_or_else(|| format!("Minecraft version {version_id} was not found in the official version catalog"))?;

    let url = entry
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Minecraft version catalog entry {version_id} has no manifest URL"))?;

    download_manifest(url, &path, version_id).await
}

/// Downloads, validates, and atomically writes a version manifest.
async fn download_manifest(url: &str, path: &Path, expected_id: &str) -> Result<(), String> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to download version manifest from {url}: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Version manifest request failed for {url}: {e}"))?;

    let raw = response.text().await.map_err(|e| format!("Failed to read version manifest from {url}: {e}"))?;

    let manifest = crate::integrations::minecraft::versions::load_version_manifest(raw.clone())
        .await
        .map_err(|e| format!("Downloaded invalid version manifest from {url}: {e}"))?;

    if manifest.id != expected_id {
        return Err(format!("Downloaded version manifest ID mismatch: expected {expected_id}, got {}", manifest.id));
    }

    fs::write_str(path, &raw, false).await?;

    Ok(())
}

/// Validates that an existing local manifest is readable, parseable, and has the
/// expected ID. Does not silently overwrite an invalid existing file.
async fn validate_local_manifest(path: &Path, expected_id: &str) -> Result<(), String> {
    let raw = fs::read_str(path).await.map_err(|e| format!("Failed to read version manifest for {expected_id}: {e}"))?;

    let manifest = crate::integrations::minecraft::versions::load_version_manifest(raw)
        .await
        .map_err(|e| format!("Failed to parse local version manifest for {expected_id}: {e}"))?;

    if manifest.id != expected_id {
        return Err(format!("Local version manifest ID mismatch: expected {expected_id}, got {}", manifest.id));
    }

    Ok(())
}

/// Ensures all inherited version manifests exist locally.
async fn ensure_manifest_parents(version_id: &str) -> Result<(), String> {
    let mut current_id = version_id.to_string();
    let mut visited = HashSet::new();

    loop {
        ensure_safe_version_id(&current_id)?;

        if !visited.insert(current_id.clone()) {
            return Err(format!("Circular version manifest inheritance detected at {current_id}"));
        }

        let path = version_manifest_path(&current_id)?;

        if !fs::is_file(&path).await? {
            return Err(format!("Version manifest {current_id} is missing at {}", path.display()));
        }

        let raw = fs::read_str(&path).await.map_err(|e| format!("Failed to read version manifest for {current_id}: {e}"))?;

        let manifest = crate::integrations::minecraft::versions::load_version_manifest(raw)
            .await
            .map_err(|e| format!("Failed to parse version manifest for {current_id}: {e}"))?;

        if manifest.id != current_id {
            return Err(format!("Version manifest ID mismatch: expected {current_id}, got {}", manifest.id));
        }

        let Some(parent_id) = manifest.inherits_from else {
            return Ok(());
        };

        ensure_safe_version_id(&parent_id)?;

        let parent_path = version_manifest_path(&parent_id)?;

        if !fs::is_file(&parent_path).await? {
            ensure_vanilla_manifest(&parent_id)
                .await
                .map_err(|e| format!("Failed to ensure inherited manifest {parent_id} required by {current_id}: {e}"))?;
        } else {
            validate_local_manifest(&parent_path, &parent_id).await?;
        }

        current_id = parent_id;
    }
}

/// Builds the Fabric profile JSON URL for a selected Minecraft and Fabric version.
fn fabric_profile_url(version: &ProfileVersion) -> Result<String, String> {
    let mc_version = version.minecraft_version.as_deref().ok_or_else(|| format!("Missing Minecraft version for {}", version.id))?;

    let loader_version = version.loader_version.as_deref().ok_or_else(|| format!("Missing Fabric loader version for {}", version.id))?;

    ensure_safe_version_id(mc_version)?;
    ensure_safe_version_id(loader_version)?;

    Ok(format!("{FABRIC_META_URL}{FABRIC_LOADER_PROFILE_PATH}/{mc_version}/{loader_version}{LOADER_PROFILE_JSON_PATH}"))
}

/// Builds the Quilt profile JSON URL for a selected Minecraft and Quilt version.
fn quilt_profile_url(version: &ProfileVersion) -> Result<String, String> {
    let mc_version = version.minecraft_version.as_deref().ok_or_else(|| format!("Missing Minecraft version for {}", version.id))?;

    let loader_version = version.loader_version.as_deref().ok_or_else(|| format!("Missing Quilt loader version for {}", version.id))?;

    ensure_safe_version_id(mc_version)?;
    ensure_safe_version_id(loader_version)?;

    Ok(format!("{QUILT_META_URL}{QUILT_LOADER_PROFILE_PATH}/{mc_version}/{loader_version}{LOADER_PROFILE_JSON_PATH}"))
}

/// Returns the absolute path for a version manifest.
fn version_manifest_path(version_id: &str) -> Result<PathBuf, String> {
    ensure_safe_version_id(version_id)?;

    Ok(fs::mc_dir()?.join(VERSIONS_DIR).join(version_id).join(format!("{version_id}.json")))
}

/// Prevents version IDs from escaping the versions directory.
fn ensure_safe_version_id(version_id: &str) -> Result<(), String> {
    if version_id.is_empty() || version_id.contains(['/', '\\']) || version_id == "." || version_id == ".." {
        return Err(format!("Invalid version ID: {version_id}"));
    }

    Ok(())
}

/// Ensures the Forge installer has generated the selected profile's manifest.
async fn ensure_forge_installed(version: &ProfileVersion) -> Result<(), String> {
    let minecraft_version = version.minecraft_version.as_deref().ok_or_else(|| "Forge profile is missing minecraft_version".to_string())?;

    let forge_version = version.loader_version.as_deref().ok_or_else(|| "Forge profile is missing loader_version".to_string())?;

    ensure_safe_version_id(minecraft_version)?;
    ensure_safe_version_id(forge_version)?;

    let artifact_version = format!("{minecraft_version}-{forge_version}");

    let installer_url =
        format!("{FORGE_MAVEN_BASE_URL}{FORGE_INSTALLER_PATH}/{artifact_version}/forge-{artifact_version}{FORGE_INSTALLER_SUFFIX}");

    install_loader_from_jar(&installer_url, &version.id, FORGE_INSTALL_CLIENT_FLAG).await
}

/// Ensures the NeoForge installer has generated the selected profile's manifest.
async fn ensure_neoforge_installed(version: &ProfileVersion) -> Result<(), String> {
    let neoforge_version = version.loader_version.as_deref().ok_or_else(|| "NeoForge profile is missing loader_version".to_string())?;

    ensure_safe_version_id(neoforge_version)?;

    let installer_url = format!(
        "{NEOFORGE_MAVEN_BASE_URL}{NEOFORGE_INSTALLER_PATH}/{neoforge_version}/neoforge-{neoforge_version}{NEOFORGE_INSTALLER_SUFFIX}"
    );

    install_loader_from_jar(&installer_url, &version.id, NEOFORGE_INSTALL_CLIENT_FLAG).await
}

/// Ensures a Fabric profile manifest exists for an Iris/Fabric profile.
///
/// This only prepares the Fabric-based version manifest. Iris itself and its
/// dependencies must be installed separately as mods.
async fn ensure_iris_fabric_installed(version: &ProfileVersion) -> Result<(), String> {
    let url = fabric_profile_url(version)?;
    let path = version_manifest_path(&version.id)?;

    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch Fabric profile JSON: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Fabric metadata request failed for {url}: {e}"))?;

    let mut manifest: Value = response.json().await.map_err(|e| format!("Failed to parse Fabric profile JSON: {e}"))?;

    manifest["id"] = Value::String(version.id.clone());

    let raw = serde_json::to_string_pretty(&manifest).map_err(|e| format!("Failed to serialize Fabric profile JSON: {e}"))?;

    let parsed = crate::integrations::minecraft::versions::load_version_manifest(raw.clone())
        .await
        .map_err(|e| format!("Generated invalid Iris/Fabric manifest: {e}"))?;

    if parsed.id != version.id {
        return Err(format!("Generated Iris/Fabric manifest ID mismatch: expected {}, got {}", version.id, parsed.id));
    }

    let parent = path.parent().ok_or_else(|| "Invalid version manifest path".to_string())?;

    fs::create_dir(parent).await.map_err(|e| format!("Failed to create version directory: {e}"))?;

    fs::write_str(&path, &raw, false).await?;

    ensure_manifest_parents(&version.id).await
}

/// Downloads and runs a loader installer JAR, then verifies that the expected
/// manifest was created.
async fn install_loader_from_jar(installer_url: &str, expected_version_id: &str, install_flag: &str) -> Result<(), String> {
    ensure_safe_version_id(expected_version_id)?;

    let manifest_path = version_manifest_path(expected_version_id)?;

    if fs::is_file(&manifest_path).await? {
        validate_local_manifest(&manifest_path, expected_version_id).await?;
        return ensure_manifest_parents(expected_version_id).await;
    }

    let response = reqwest::get(installer_url)
        .await
        .map_err(|e| format!("Failed to download loader installer: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Loader installer download failed for {installer_url}: {e}"))?;

    let installer_bytes = response.bytes().await.map_err(|e| format!("Failed to read loader installer download: {e}"))?;

    let minecraft_dir = fs::mc_dir()?;
    let installer_dir = fs::kable_dir()?.join(LOADER_INSTALLERS_DIR);
    fs::create_dir(&installer_dir).await?;

    let file_name = installer_url.rsplit('/').next().filter(|name| !name.is_empty()).ok_or_else(|| "Invalid installer URL".to_string())?;

    let installer_path = installer_dir.join(file_name);

    fs::write(&installer_path, &installer_bytes, false).await.map_err(|e| format!("Failed to save loader installer: {e}"))?;

    let java_executable = java::auto_detect_java().map_err(|e| format!("Could not locate Java to run loader installer: {e}"))?;

    let java_executable = resolve_java_command(&java_executable);
    let install_flag = install_flag.to_string();

    let output = tokio::task::spawn_blocking(move || {
        Command::new(&java_executable)
            .arg(JAVA_JAR_FLAG)
            .arg(&installer_path)
            .arg(&install_flag)
            .arg(&minecraft_dir)
            .output()
            .map_err(|e| format!("Failed to execute loader installer: {e}"))
    })
    .await
    .map_err(|e| format!("Loader installer task failed: {e}"))??;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        return Err(format!(
            "Loader installer exited with status {}.\nstdout:\n{}\nstderr:\n{}",
            output.status,
            stdout.trim(),
            stderr.trim()
        ));
    }

    if !fs::is_file(&manifest_path).await? {
        return Err(format!(
            "Installer completed, but the expected version manifest was not created: {}. \
             The installer's version ID may differ from the profile ID '{}'.",
            manifest_path.display(),
            expected_version_id
        ));
    }

    validate_local_manifest(&manifest_path, expected_version_id).await?;
    ensure_manifest_parents(expected_version_id).await
}

/// Resolves a GUI-less Java executable where possible.
///
/// On Windows, `javaw.exe` suppresses console output. Prefer the sibling
/// `java.exe` so installer failures can be diagnosed.
fn resolve_java_command(java_path: &str) -> String {
    let path = Path::new(java_path);

    #[cfg(target_os = "windows")]
    {
        if path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.eq_ignore_ascii_case("javaw.exe")) {
            let java = path.with_file_name("java.exe");

            if java.is_file() {
                return java.to_string_lossy().into_owned();
            }
        }
    }

    java_path.to_string()
}
