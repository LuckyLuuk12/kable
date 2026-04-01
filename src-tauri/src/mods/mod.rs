#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackSelectionGroup {
    pub enabled: Vec<String>,
    pub optional: Vec<String>,
    pub disabled: Vec<String>,
    #[serde(default)]
    pub overwrite_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackSelection {
    pub mods: ModpackSelectionGroup,
    pub resourcepacks: ModpackSelectionGroup,
    pub shaderpacks: ModpackSelectionGroup,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackContext {
    pub mrpack_path: String,
    pub extracted_dir: String,
    pub installation_dir: String,
    pub provider: ProviderKind,
    pub mod_id: String,
    pub version_id: Option<String>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DownloadOrPrepareResponse {
    ModInstalled {
        success: bool,
    },
    Modpack {
        modpack: crate::installations::mrpack::MrPackDetailed,
        context: ModpackContext,
    },
}

/// Unified mod/modpack download/prepare logic for frontend API
pub async fn download_or_prepare_mod(
    provider: ProviderKind,
    mod_id: &str,
    version_id: Option<&str>,
    installation: &crate::installations::kable_profiles::KableInstallation,
) -> Result<DownloadOrPrepareResponse, String> {
    fn sanitize_for_path(input: &str) -> String {
        input
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }

    // For now, only Modrinth is supported for modpack detection
    match provider {
        ProviderKind::Modrinth => {
            // Fetch version info to check if it's a modpack
            let versions = crate::mods::modrinth::get_mod_versions(mod_id).await?;
            let version = if let Some(vid) = version_id {
                versions.iter().find(|v| v.id == vid).cloned()
            } else {
                versions.first().cloned()
            };
            let Some(version) = version else {
                return Err("Mod version not found".to_string());
            };
            let is_modpack = version.files.iter().any(|f| f.url.ends_with(".mrpack"));
            if is_modpack {
                // Download .mrpack to temp, extract, diff, return manifest/context
                let mrpack_file = version
                    .files
                    .iter()
                    .find(|f| f.url.ends_with(".mrpack"))
                    .unwrap();
                let installation_root = crate::get_default_minecraft_dir()
                    .map_err(|e| format!("Failed to resolve installation root: {e}"))?;

                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| format!("Failed to generate temp id: {e}"))?
                    .as_millis();
                let temp_modpack_id = format!(
                    "{}-{}-{}",
                    sanitize_for_path(mod_id),
                    sanitize_for_path(version_id.unwrap_or("latest")),
                    timestamp
                );
                let temp_dir = get_temp_dir(&installation.id, &temp_modpack_id)?;
                std::fs::create_dir_all(&temp_dir)
                    .map_err(|e| format!("Failed to create temp dir: {e}"))?;
                let mrpack_path = temp_dir.join(&mrpack_file.filename);
                crate::mods::modrinth::download_mod_file(&mrpack_file.url, &mrpack_path).await?;
                // Use detailed manifest for modal
                let extracted_dir = temp_dir.join("extracted");
                std::fs::create_dir_all(&extracted_dir)
                    .map_err(|e| format!("Failed to create extract dir: {e}"))?;
                crate::installations::mrpack::extract_mrpack(&mrpack_path, &extracted_dir)
                    .map_err(|e| format!("Failed to extract mrpack: {e}"))?;
                let index = crate::installations::mrpack::load_index(&extracted_dir)
                    .map_err(|e| format!("Failed to load modrinth.index.json: {e}"))?;
                let detailed = crate::installations::mrpack::list_pack_files_detailed(
                    &index,
                    Some(&installation_root),
                );
                let context = ModpackContext {
                    mrpack_path: mrpack_path.to_string_lossy().to_string(),
                    extracted_dir: extracted_dir.to_string_lossy().to_string(),
                    installation_dir: installation_root.to_string_lossy().to_string(),
                    provider,
                    mod_id: mod_id.to_string(),
                    version_id: version_id.map(|s| s.to_string()),
                };
                Ok(DownloadOrPrepareResponse::Modpack {
                    modpack: detailed,
                    context,
                })
            } else {
                // Download as normal mod
                crate::mods::download_mod(provider, mod_id, version_id, installation).await?;
                Ok(DownloadOrPrepareResponse::ModInstalled { success: true })
            }
        }
        ProviderKind::CurseForge => {
            // TODO: Add modpack detection for CurseForge if needed
            crate::mods::download_mod(provider, mod_id, version_id, installation).await?;
            Ok(DownloadOrPrepareResponse::ModInstalled { success: true })
        }
    }
}
use crate::installations::mrpack::{self};
use serde::{Deserialize, Serialize};

use crate::get_temp_dir;

/// Apply modpack selection: takes a KableInstallation and ModpackSelection, moves files accordingly
pub async fn apply_modpack_selection(
    installation: crate::installations::kable_profiles::KableInstallation,
    selection: ModpackSelection,
    context: ModpackContext,
) -> Result<(), String> {
    use sha1::Sha1;
    use sha2::{Digest, Sha512};
    use std::collections::{HashMap, HashSet};

    fn verify_bytes(
        bytes: &[u8],
        hashes: &std::collections::HashMap<String, String>,
    ) -> Result<(), String> {
        if let Some(expected) = hashes.get("sha1") {
            let mut hasher = Sha1::new();
            hasher.update(bytes);
            let result = hex::encode(hasher.finalize());
            if &result != expected {
                return Err("SHA1 hash mismatch".to_string());
            }
        }

        if let Some(expected) = hashes.get("sha512") {
            let mut hasher = Sha512::new();
            hasher.update(bytes);
            let result = hex::encode(hasher.finalize());
            if &result != expected {
                return Err("SHA512 hash mismatch".to_string());
            }
        }

        Ok(())
    }

    async fn install_file(
        client: &reqwest::Client,
        file: &mrpack::MrpackFile,
        target: &std::path::Path,
        overwrite: bool,
    ) -> Result<(), String> {
        if target.exists() && !overwrite {
            return Ok(());
        }

        let mut last_error: Option<String> = None;
        let mut verified_bytes: Option<Vec<u8>> = None;
        for url in &file.downloads {
            let response = match client.get(url).send().await {
                Ok(resp) => resp,
                Err(e) => {
                    last_error = Some(format!("request failed at {url}: {e}"));
                    continue;
                }
            };

            if !response.status().is_success() {
                last_error = Some(format!("HTTP {} at {}", response.status(), url));
                continue;
            }

            let bytes = match response.bytes().await {
                Ok(b) => b.to_vec(),
                Err(e) => {
                    last_error = Some(format!("failed to read body at {url}: {e}"));
                    continue;
                }
            };

            match verify_bytes(&bytes, &file.hashes) {
                Ok(()) => {
                    verified_bytes = Some(bytes);
                    break;
                }
                Err(e) => {
                    last_error = Some(format!("hash verification failed at {url}: {e}"));
                    continue;
                }
            }
        }

        let bytes = verified_bytes.ok_or_else(|| {
            format!(
                "Download failed for {} ({} URLs tried): {}",
                file.path,
                file.downloads.len(),
                last_error.unwrap_or_else(|| "unknown error".to_string())
            )
        })?;

        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create target folder: {e}"))?;
        }
        tokio::fs::write(target, &bytes)
            .await
            .map_err(|e| format!("Failed to write {}: {e}", target.display()))
    }

    fn parse_modrinth_ids(downloads: &[String]) -> Option<(String, String)> {
        for url in downloads {
            let clean = url.split('?').next().unwrap_or(url.as_str());
            let parts: Vec<&str> = clean.split('/').collect();

            let data_idx = parts.iter().position(|p| *p == "data")?;
            let versions_idx = parts.iter().position(|p| *p == "versions")?;

            let project_id = parts.get(data_idx + 1)?;
            let version_id = parts.get(versions_idx + 1)?;

            if !project_id.is_empty() && !version_id.is_empty() {
                return Some(((*project_id).to_string(), (*version_id).to_string()));
            }
        }

        None
    }

    #[derive(Debug, Deserialize)]
    struct ModrinthVersionDetails {
        version_number: String,
    }

    async fn resolve_modrinth_version_number(
        client: &reqwest::Client,
        version_id: &str,
        cache: &mut HashMap<String, Option<String>>,
    ) -> Option<String> {
        if let Some(cached) = cache.get(version_id) {
            return cached.clone();
        }

        let url = format!("https://api.modrinth.com/v2/version/{version_id}");
        let result = match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<ModrinthVersionDetails>().await {
                    Ok(details) => Some(details.version_number),
                    Err(_) => None,
                }
            }
            _ => None,
        };

        cache.insert(version_id.to_string(), result.clone());
        result
    }

    async fn remove_old_mod_versions_by_project_id(
        mods_dir: &std::path::Path,
        project_id: &str,
    ) -> Result<(), String> {
        for dir in [mods_dir.to_path_buf(), mods_dir.join("disabled")] {
            if !dir.exists() {
                continue;
            }

            let mut entries = tokio::fs::read_dir(&dir)
                .await
                .map_err(|e| format!("Failed to read mods folder {}: {e}", dir.display()))?;

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| format!("Failed to iterate mods folder {}: {e}", dir.display()))?
            {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("jar") {
                    continue;
                }

                let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };

                let metadata_path = dir.join(format!("{}.kable_metadata.json", file_name));
                if !metadata_path.exists() {
                    continue;
                }

                let metadata_content = match tokio::fs::read_to_string(&metadata_path).await {
                    Ok(content) => content,
                    Err(_) => continue,
                };

                let metadata = match serde_json::from_str::<ModMetadata>(&metadata_content) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                if metadata.project_id == project_id {
                    let _ = tokio::fs::remove_file(&path).await;
                    let _ = tokio::fs::remove_file(&metadata_path).await;
                }
            }
        }

        Ok(())
    }

    async fn install_group(
        group: &ModpackSelectionGroup,
        subfolder: &str,
        target_root: &std::path::Path,
        mods_root: &std::path::Path,
        files_by_path: &HashMap<String, mrpack::MrpackFile>,
        client: &reqwest::Client,
        version_cache: &mut HashMap<String, Option<String>>,
    ) -> Result<(), String> {
        let overwrite_set: HashSet<&str> =
            group.overwrite_paths.iter().map(String::as_str).collect();
        let mut removed_projects: HashSet<String> = HashSet::new();

        #[allow(clippy::too_many_arguments)]
        async fn install_paths(
            paths: &[String],
            subfolder: &str,
            target_root: &std::path::Path,
            mods_root: &std::path::Path,
            files_by_path: &HashMap<String, mrpack::MrpackFile>,
            overwrite_set: &HashSet<&str>,
            disabled: bool,
            client: &reqwest::Client,
            version_cache: &mut HashMap<String, Option<String>>,
            removed_projects: &mut HashSet<String>,
        ) -> Result<(), String> {
            for selected_path in paths {
                let file = files_by_path.get(selected_path).ok_or_else(|| {
                    format!("Selected file not found in manifest: {selected_path}")
                })?;

                let rel_path = std::path::Path::new(&file.path)
                    .strip_prefix(subfolder)
                    .unwrap_or_else(|_| std::path::Path::new(&file.path));
                let target = if disabled {
                    target_root.join("disabled").join(rel_path)
                } else {
                    target_root.join(rel_path)
                };

                let identity = if subfolder == "mods" {
                    parse_modrinth_ids(&file.downloads)
                } else {
                    None
                };

                if let Some((project_id, _version_id)) = &identity {
                    if !removed_projects.contains(project_id) {
                        remove_old_mod_versions_by_project_id(mods_root, project_id).await?;
                        removed_projects.insert(project_id.clone());
                    }
                }

                let overwrite = overwrite_set.contains(file.path.as_str()) || identity.is_some();
                install_file(client, file, &target, overwrite).await?;

                if let Some((project_id, version_id)) = identity {
                    if let Some(file_name) = target.file_name().and_then(|n| n.to_str()) {
                        let metadata_dir = target.parent().unwrap_or(mods_root);
                        let version_number =
                            resolve_modrinth_version_number(client, &version_id, version_cache)
                                .await
                                .unwrap_or_else(|| version_id.clone());

                        // Best-effort metadata write so installed detection can be identity-based.
                        let _ = crate::mods::modrinth::save_mod_metadata(
                            metadata_dir,
                            file_name,
                            &project_id,
                            &version_number,
                            &version_id,
                        )
                        .await;
                    }
                }
            }
            Ok(())
        }

        install_paths(
            &group.enabled,
            subfolder,
            target_root,
            mods_root,
            files_by_path,
            &overwrite_set,
            false,
            client,
            version_cache,
            &mut removed_projects,
        )
        .await?;
        install_paths(
            &group.optional,
            subfolder,
            target_root,
            mods_root,
            files_by_path,
            &overwrite_set,
            false,
            client,
            version_cache,
            &mut removed_projects,
        )
        .await?;
        install_paths(
            &group.disabled,
            subfolder,
            target_root,
            mods_root,
            files_by_path,
            &overwrite_set,
            true,
            client,
            version_cache,
            &mut removed_projects,
        )
        .await
    }

    if context.provider != ProviderKind::Modrinth {
        return Err("Only Modrinth modpack installation is supported right now".to_string());
    }

    let extracted_dir = std::path::PathBuf::from(&context.extracted_dir);
    if !extracted_dir.exists() {
        return Err("Extracted modpack temp folder was not found".to_string());
    }

    let index = mrpack::load_index(&extracted_dir)
        .map_err(|e| format!("Failed to load modrinth.index.json: {e}"))?;
    let files_by_path: HashMap<String, mrpack::MrpackFile> = index
        .files
        .iter()
        .cloned()
        .map(|file| (file.path.clone(), file))
        .collect();

    let installation_root = std::path::PathBuf::from(&context.installation_dir);
    let mods_root = installation.find_mods_dir()?;
    let resourcepacks_root = installation.find_resourcepacks_dir()?;
    let shaderpacks_root = installation.find_shaderpacks_dir()?;

    let client = reqwest::Client::builder()
        .user_agent("kable/1.0 (+https://github.com/LuckyLuuk12/kable)")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;
    let mut version_cache: HashMap<String, Option<String>> = HashMap::new();

    install_group(
        &selection.mods,
        "mods",
        &mods_root,
        &mods_root,
        &files_by_path,
        &client,
        &mut version_cache,
    )
    .await?;
    install_group(
        &selection.resourcepacks,
        "resourcepacks",
        &resourcepacks_root,
        &mods_root,
        &files_by_path,
        &client,
        &mut version_cache,
    )
    .await?;
    install_group(
        &selection.shaderpacks,
        "shaderpacks",
        &shaderpacks_root,
        &mods_root,
        &files_by_path,
        &client,
        &mut version_cache,
    )
    .await?;

    let mut managed_project_ids: HashSet<String> = HashSet::new();
    for selected_path in selection
        .mods
        .enabled
        .iter()
        .chain(selection.mods.optional.iter())
        .chain(selection.mods.disabled.iter())
    {
        if let Some(file) = files_by_path.get(selected_path) {
            if let Some((project_id, _)) = parse_modrinth_ids(&file.downloads) {
                managed_project_ids.insert(project_id);
            }
        }
    }

    if !managed_project_ids.is_empty() {
        let mut managed_project_ids: Vec<String> = managed_project_ids.into_iter().collect();
        managed_project_ids.sort();

        upsert_modpack_source_record(
            &installation,
            context.provider,
            &context.mod_id,
            context.version_id.clone(),
            Some(index.name.clone()),
            Some(index.version_id.clone()),
            managed_project_ids,
        )
        .await?;
    }

    mrpack::extract_overrides_to_profile(&extracted_dir, &installation_root)
        .map_err(|e| format!("Failed to apply modpack overrides: {e}"))?;

    // Best-effort temp cleanup after successful install.
    let mrpack_file = std::path::PathBuf::from(&context.mrpack_path);

    if extracted_dir.exists() {
        let _ = std::fs::remove_dir_all(&extracted_dir);
    }
    if mrpack_file.exists() {
        let _ = std::fs::remove_file(&mrpack_file);
    }

    let extracted_parent = extracted_dir.parent().map(std::path::Path::to_path_buf);
    let mrpack_parent = mrpack_file.parent().map(std::path::Path::to_path_buf);

    let mut cleanup_targets = Vec::<std::path::PathBuf>::new();
    if let Some(p) = extracted_parent {
        cleanup_targets.push(p);
    }
    if let Some(p) = mrpack_parent {
        cleanup_targets.push(p);
    }

    cleanup_targets.sort();
    cleanup_targets.dedup();

    for target in cleanup_targets {
        if target.exists() {
            if let Err(e) = std::fs::remove_dir_all(&target) {
                eprintln!(
                    "[Modpack] Temp cleanup failed for {}: {}",
                    target.display(),
                    e
                );
            }
        }
    }

    Ok(())
}
pub mod cache;
pub mod curseforge;
pub mod manager;
pub mod modrinth;
pub mod modrinth_versions_cache;

pub use self::cache::*;
pub use self::curseforge::{CurseForgeFilter, CurseForgeInfo, CurseForgeProvider};
pub use self::manager::*;
pub use self::modrinth::*;
pub use self::modrinth_versions_cache::*;

use crate::installations::kable_profiles::KableInstallation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderKind {
    Modrinth,
    CurseForge,
}

use once_cell::sync::Lazy;
use std::sync::Mutex;

static MODRINTH: Lazy<Mutex<ModrinthProvider>> =
    Lazy::new(|| Mutex::new(ModrinthProvider::default()));
static CURSEFORGE: Lazy<Mutex<CurseForgeProvider>> =
    Lazy::new(|| Mutex::new(CurseForgeProvider::default()));

pub fn set_provider_filter(
    provider: ProviderKind,
    installation: Option<&KableInstallation>,
    filter: Option<ModFilter>,
) {
    match provider {
        ProviderKind::Modrinth => {
            MODRINTH.lock().unwrap().filter(installation, filter);
        }
        ProviderKind::CurseForge => {
            CURSEFORGE.lock().unwrap().filter(installation, filter);
        }
    }
}

pub fn set_provider_limit(provider: ProviderKind, limit: usize) {
    match provider {
        ProviderKind::Modrinth => {
            MODRINTH.lock().unwrap().set_limit(limit);
        }
        ProviderKind::CurseForge => {
            CURSEFORGE.lock().unwrap().set_limit(limit);
        }
    }
}

pub async fn get_mods(provider: ProviderKind, offset: usize) -> Result<Vec<ModInfoKind>, String> {
    match provider {
        ProviderKind::Modrinth => {
            let mut prov = {
                let prov_guard = MODRINTH.lock().unwrap();
                prov_guard.clone()
            };
            prov.get(offset).await
        }
        ProviderKind::CurseForge => {
            let mut prov = {
                let prov_guard = CURSEFORGE.lock().unwrap();
                prov_guard.clone()
            };
            prov.get(offset).await
        }
    }
}

pub async fn download_mod(
    provider: ProviderKind,
    mod_id: &str,
    version_id: Option<&str>,
    installation: &KableInstallation,
) -> Result<(), String> {
    match provider {
        ProviderKind::Modrinth => {
            let prov = {
                let prov_guard = MODRINTH.lock().unwrap();
                prov_guard.clone()
            };
            prov.download(mod_id, version_id, installation).await
        }
        ProviderKind::CurseForge => {
            let prov = {
                let prov_guard = CURSEFORGE.lock().unwrap();
                prov_guard.clone()
            };
            prov.download(mod_id, version_id, installation).await
        }
    }
}

pub fn clear_provider_cache(provider: ProviderKind) {
    match provider {
        ProviderKind::Modrinth => {
            let mut prov = MODRINTH.lock().unwrap();
            prov.cache.clear();
            let _ = prov.cache.save_to_disk(&prov.cache_path);
        }
        ProviderKind::CurseForge => {
            let mut prov = CURSEFORGE.lock().unwrap();
            prov.cache.clear();
            let _ = prov.cache.save_to_disk(&prov.cache_path);
        }
    }
}

pub fn purge_stale_provider_cache(provider: ProviderKind) {
    match provider {
        ProviderKind::Modrinth => {
            let mut prov = MODRINTH.lock().unwrap();
            prov.cache.purge_stale();
            let _ = prov.cache.save_to_disk(&prov.cache_path);
        }
        ProviderKind::CurseForge => {
            let mut prov = CURSEFORGE.lock().unwrap();
            prov.cache.purge_stale();
            let _ = prov.cache.save_to_disk(&prov.cache_path);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtendedModInfo {
    pub mod_jar_info: crate::ModJarInfo,
    pub page_uri: Option<String>, // URI to the mod's page, e.g., on Modrinth
    pub icon_uri: Option<String>,
    pub description: Option<String>,
    pub authors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMetadata {
    pub project_id: String,
    pub file_name: String,
    pub version_number: String,
    #[serde(default)]
    pub modrinth_version_id: Option<String>,
    pub download_time: String,
}

const MODPACK_SOURCES_FILE: &str = ".kable_modpack_sources.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackSourceRecord {
    pub provider: ProviderKind,
    pub mod_id: String,
    pub version_id: Option<String>,
    pub modpack_name: Option<String>,
    pub modpack_version: Option<String>,
    pub installed_at: String,
    pub managed_project_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ModpackSourcesState {
    #[serde(default)]
    records: Vec<ModpackSourceRecord>,
}

async fn load_modpack_sources_state(
    mods_dir: &std::path::Path,
) -> Result<ModpackSourcesState, String> {
    let state_path = mods_dir.join(MODPACK_SOURCES_FILE);
    if !state_path.exists() {
        return Ok(ModpackSourcesState::default());
    }

    let content = tokio::fs::read_to_string(&state_path)
        .await
        .map_err(|e| format!("Failed to read modpack sources file: {}", e))?;

    serde_json::from_str::<ModpackSourcesState>(&content)
        .map_err(|e| format!("Failed to parse modpack sources file: {}", e))
}

async fn save_modpack_sources_state(
    mods_dir: &std::path::Path,
    state: &ModpackSourcesState,
) -> Result<(), String> {
    let state_path = mods_dir.join(MODPACK_SOURCES_FILE);
    let content = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize modpack sources file: {}", e))?;

    tokio::fs::write(&state_path, content)
        .await
        .map_err(|e| format!("Failed to write modpack sources file: {}", e))
}

async fn upsert_modpack_source_record(
    installation: &KableInstallation,
    provider: ProviderKind,
    mod_id: &str,
    version_id: Option<String>,
    modpack_name: Option<String>,
    modpack_version: Option<String>,
    managed_project_ids: Vec<String>,
) -> Result<(), String> {
    let mods_dir = installation.find_mods_dir()?;
    let mut state = load_modpack_sources_state(&mods_dir).await?;

    let now = chrono::Utc::now().to_rfc3339();
    if let Some(existing) = state
        .records
        .iter_mut()
        .find(|r| r.provider == provider && r.mod_id == mod_id && r.version_id == version_id)
    {
        existing.modpack_name = modpack_name;
        existing.modpack_version = modpack_version;
        existing.installed_at = now;
        existing.managed_project_ids = managed_project_ids;
    } else {
        state.records.push(ModpackSourceRecord {
            provider,
            mod_id: mod_id.to_string(),
            version_id,
            modpack_name,
            modpack_version,
            installed_at: now,
            managed_project_ids,
        });
    }

    save_modpack_sources_state(&mods_dir, &state).await
}

async fn collect_installed_project_ids(
    installation: &KableInstallation,
) -> Result<std::collections::HashSet<String>, String> {
    let mods_dir = installation.find_mods_dir()?;
    let mut project_ids = std::collections::HashSet::new();

    for dir in [mods_dir.clone(), mods_dir.join("disabled")] {
        if !dir.exists() {
            continue;
        }

        let mut entries = tokio::fs::read_dir(&dir)
            .await
            .map_err(|e| format!("Failed to read mods folder {}: {}", dir.display(), e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("Failed to iterate mods folder {}: {}", dir.display(), e))?
        {
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !file_name.ends_with(".kable_metadata.json") {
                continue;
            }

            let content = match tokio::fs::read_to_string(&path).await {
                Ok(c) => c,
                Err(_) => continue,
            };

            if let Ok(meta) = serde_json::from_str::<ModMetadata>(&content) {
                project_ids.insert(meta.project_id);
            }
        }
    }

    Ok(project_ids)
}

pub async fn get_modpack_source_records(
    installation: &KableInstallation,
) -> Result<Vec<ModpackSourceRecord>, String> {
    let mods_dir = installation.find_mods_dir()?;
    let state = load_modpack_sources_state(&mods_dir).await?;
    let installed_project_ids = collect_installed_project_ids(installation).await?;

    let mut visible_records: Vec<ModpackSourceRecord> = state
        .records
        .into_iter()
        .filter_map(|mut record| {
            let active_ids: Vec<String> = record
                .managed_project_ids
                .into_iter()
                .filter(|id| installed_project_ids.contains(id))
                .collect();
            if active_ids.is_empty() {
                return None;
            }
            record.managed_project_ids = active_ids;
            Some(record)
        })
        .collect();

    visible_records.sort_by(|a, b| b.installed_at.cmp(&a.installed_at));
    Ok(visible_records)
}

pub async fn get_mod_metadata(
    installation: &KableInstallation,
    jar_filename: &str,
) -> Result<ModMetadata, String> {
    use tokio::fs;

    let mods_dir = installation.find_mods_dir()?;

    // Check active mods directory first
    let metadata_path = mods_dir.join(format!("{}.kable_metadata.json", jar_filename));

    // If not found in active directory, check disabled directory
    let metadata_path = if !metadata_path.exists() {
        let disabled_path = mods_dir
            .join("disabled")
            .join(format!("{}.kable_metadata.json", jar_filename));
        if disabled_path.exists() {
            disabled_path
        } else {
            return Err(format!(
                "No metadata file found for {} in active or disabled folder",
                jar_filename
            ));
        }
    } else {
        metadata_path
    };

    let content = fs::read_to_string(&metadata_path)
        .await
        .map_err(|e| format!("Failed to read metadata file: {}", e))?;

    let metadata: ModMetadata = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse metadata file: {}", e))?;

    Ok(metadata)
}

pub async fn get_extended_mod_info(
    mod_jar_info: crate::ModJarInfo,
) -> Result<ExtendedModInfo, String> {
    // Only hold the lock for the cache lookup
    let (found_info, mod_name, _loader) = {
        let modrinth = MODRINTH.lock().unwrap();
        let mod_name = mod_jar_info.mod_name.as_deref().unwrap_or("");
        let loader = mod_jar_info.loader.as_deref();
        let mut found_info = None;
        for entry in modrinth.cache.entries.values() {
            if let Some(found) = entry.value.iter().find(|info| {
                let name_match = info.title.eq_ignore_ascii_case(mod_name)
                    || info.slug.eq_ignore_ascii_case(mod_name);
                let loader_match = if let Some(loader) = loader {
                    info.loaders
                        .as_ref()
                        .is_some_and(|ls| ls.iter().any(|l| l.eq_ignore_ascii_case(loader)))
                } else {
                    true
                };
                name_match && loader_match
            }) {
                found_info = Some(found.clone());
                break;
            }
        }
        (
            found_info,
            mod_name.to_string(),
            loader.map(|s| s.to_string()),
        )
    };

    if let Some(found) = found_info {
        return Ok(ExtendedModInfo {
            mod_jar_info,
            icon_uri: found.icon_url.clone(),
            description: Some(found.description.clone()),
            authors: vec![found.author.clone()],
            page_uri: found.source_url.clone(),
        });
    }

    // Not found in cache, try Modrinth API search (no lock held here)
    let query = if !mod_name.is_empty() {
        &mod_name
    } else if let Some(name) = mod_jar_info.mod_name.as_deref() {
        name
    } else {
        mod_jar_info.file_name.as_str()
    };
    let url = format!(
        "https://api.modrinth.com/v2/search?query={}&limit=1",
        urlencoding::encode(query)
    );
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| format!("Modrinth API error: {e}"))?;
    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("Modrinth API read error: {e}"))?;
    if !status.is_success() {
        return Err(format!("Modrinth API HTTP error: {} - {}", status, text));
    }
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("Modrinth API parse error: {}\nResponse body: {}", e, text))?;
    if let Some(hits) = json.get("hits").and_then(|v| v.as_array()) {
        if let Some(hit) = hits.first() {
            if let Ok(info) = serde_json::from_value::<crate::modrinth::ModrinthInfo>(hit.clone()) {
                // Cache the result to avoid repeated API calls
                {
                    let mut modrinth = MODRINTH.lock().unwrap();
                    let cache_key = format!("search:{}", query);
                    // If an entry exists, update it; otherwise, insert a new Vec
                    let mut mods = if let Some(entry) = modrinth.cache.entries.get(&cache_key) {
                        entry.value.clone()
                    } else {
                        Vec::new()
                    };
                    if !mods.iter().any(|m| m.project_id == info.project_id) {
                        mods.push(info.clone());
                    }
                    modrinth.cache.insert(cache_key, mods);
                    let _ = modrinth.cache.save_to_disk(&modrinth.cache_path);
                }
                return Ok(ExtendedModInfo {
                    mod_jar_info,
                    icon_uri: info.icon_url.clone(),
                    description: Some(info.description.clone()),
                    authors: vec![info.author.clone()],
                    page_uri: info.source_url.clone(),
                });
            }
        }
    }
    Err(format!(
        "Mod '{}' not found in Modrinth cache or API",
        mod_name
    ))
}
