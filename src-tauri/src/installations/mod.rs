pub mod kable_profiles;
pub mod profiles;
pub mod versions;

pub use self::kable_profiles::*;
pub use self::profiles::*;
pub use self::versions::*;
use once_cell::sync::Lazy;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Emitter;
use tokio::fs as async_fs;
use tokio::sync::RwLock;
use tokio::task;
use toml::Value as TomlValue;
use zip::ZipArchive;

/// Ensures that a modded installation has a dedicated mods folder set and created.
/// Returns true if the folder was set/created, false otherwise.
async fn ensure_dedicated_mods_folder(
    installation: &mut KableInstallation,
) -> Result<bool, String> {
    use crate::installations::get_version;
    let version_id = &installation.version_id;
    let version_data = get_version(version_id.clone()).await;
    let is_modded = match &version_data {
        Some(v) => v.loader != LoaderKind::Vanilla,
        None => false,
    };
    if is_modded && installation.dedicated_mods_folder.is_none() {
        // Set mods_folder to relative path format: "mods/{id}"
        installation.dedicated_mods_folder = Some(format!("mods/{}", installation.id));
        // Create .minecraft/.kable/mods/<id> if not exists
        let kable_dir = crate::get_minecraft_kable_dir()
            .map_err(|e| format!("Failed to get Kable dir: {e}"))?;
        let mods_dir = kable_dir.join("mods").join(&installation.id);
        if !mods_dir.exists() {
            crate::ensure_folder(&mods_dir)
                .await
                .map_err(|e| format!("Failed to create mods dir: {}", e))?;
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

// Internal cache for installations using RwLock for read/write access
static INSTALLATIONS_CACHE: Lazy<Arc<RwLock<Option<Vec<KableInstallation>>>>> =
    Lazy::new(|| Arc::new(RwLock::new(None)));

/// Builds the list of installations by merging kable_profiles and converted launcher_profiles.
async fn build_installations_async() -> Result<Vec<KableInstallation>, String> {
    let mut installations = kable_profiles::read_kable_profiles_async().await?;
    crate::logging::debug(&format!(
        "Read {} installations from kable_profiles.json",
        installations.len()
    ));

    // Emit initial installations immediately (sorted by last_used)
    if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
        if let Some(app_handle) = handle_guard.as_ref() {
            let _ = app_handle.emit(
                "installations-chunk-loaded",
                InstallationsChunk {
                    installations: installations.clone(),
                    is_complete: false,
                },
            );
        }
    }

    match profiles::read_launcher_profiles_async().await {
        Ok(launcher_profiles) => {
            // Use a tuple of (name, last_version_id, created) for deduplication, all as String
            let kable_keys: std::collections::HashSet<(String, String, String)> = installations
                .iter()
                .map(|i| (i.name.clone(), i.version_id.clone(), i.created.clone()))
                .collect();

            // IMPORTANT: Filter BEFORE converting to avoid expensive get_mods_folder_from_version_manifest() calls
            let mut new_converted: Vec<KableInstallation> = launcher_profiles
                .into_iter()
                .filter(|lp| {
                    let key = (
                        lp.name.clone(),
                        lp.last_version_id.clone(),
                        lp.created
                            .clone()
                            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
                    );
                    !kable_keys.contains(&key)
                })
                .map(|lp| lp.into())
                .collect();

            if !new_converted.is_empty() {
                installations.append(&mut new_converted);

                // Sort again after adding new installations
                installations.sort_by(|a, b| b.last_used.cmp(&a.last_used));

                // Only write if we actually added new installations
                kable_profiles::write_kable_profiles_async(&installations).await?;

                // Emit updated installations with new ones
                if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
                    if let Some(app_handle) = handle_guard.as_ref() {
                        let _ = app_handle.emit(
                            "installations-chunk-loaded",
                            InstallationsChunk {
                                installations: installations.clone(),
                                is_complete: false,
                            },
                        );
                    }
                }
            }
        }
        Err(e) => {
            crate::logging::Logger::warn_global(
                &format!(
                    "Failed to read launcher profiles, only kable profiles will be used. Error: {}",
                    e
                ),
                None,
            );
        }
    }

    // Emit completion event
    if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
        if let Some(app_handle) = handle_guard.as_ref() {
            let _ = app_handle.emit(
                "installations-loading-complete",
                InstallationsComplete {
                    total_count: installations.len(),
                },
            );
        }
    }

    crate::logging::debug(&format!(
        "Total installations after merging: {}",
        installations.len()
    ));
    Ok(installations)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstallationsChunk {
    pub installations: Vec<KableInstallation>,
    pub is_complete: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstallationsComplete {
    pub total_count: usize,
}

// !NOTE: Public API:

static VERSIONS_CACHE: Lazy<Arc<RwLock<Option<Versions>>>> =
    Lazy::new(|| Arc::new(RwLock::new(None)));

/// Gets all versions, either from cache or by building them, does not modify the cache
pub async fn get_versions() -> Versions {
    {
        let cache_read = VERSIONS_CACHE.read().await;
        if let Some(cached) = cache_read.as_ref() {
            return cached.clone();
        }
    }

    // Cache miss - build and cache versions (don't force refresh)
    let versions = build_versions(false).await;
    {
        let mut cache_write = VERSIONS_CACHE.write().await;
        *cache_write = Some(versions.clone());
    }
    crate::logging::debug(&format!("Built versions: {} items", versions.len()));
    versions
}

/// Gets all versions, either from cache or by building them
pub async fn get_all_versions(force: bool) -> Versions {
    if force {
        let versions = build_versions(true).await;
        crate::logging::debug(&format!("Fetched versions: {} items", versions.len()));
        {
            let mut cache_write = VERSIONS_CACHE.write().await;
            *cache_write = Some(versions.clone());
        }
        versions
    } else {
        get_versions().await
    }
}

pub async fn get_version(version_id: String) -> Option<VersionData> {
    let versions = get_versions().await;
    versions.get_version(&version_id).cloned()
}

/// Extracts the Minecraft version from a version ID (e.g. "fabric-loader-0.14.21-1.19.4" -> "1.19.4")
pub async fn get_minecraft_version(version_id: &str) -> Option<String> {
    let version_data = get_version(version_id.to_string()).await?;

    // For Vanilla, the version_id IS the minecraft version
    if version_data.loader == LoaderKind::Vanilla {
        return Some(version_data.version_id);
    }

    // For others, check extra field
    if let Some(mc_ver) = version_data.extra.get("minecraft_version") {
        if let Some(s) = mc_ver.as_str() {
            return Some(s.to_string());
        }
    }

    None
}

/// Returns all Kable installations, using cache. Ensures conversion if needed.
pub async fn get_installations() -> Result<Vec<KableInstallation>, String> {
    // Try to get from cache first
    {
        let cache_read = INSTALLATIONS_CACHE.read().await;
        if let Some(cached) = cache_read.as_ref() {
            crate::logging::debug(&format!(
                "Using cached installations: {} items",
                cached.len()
            ));
            return Ok(cached.clone());
        }
    }

    // Cache miss - build and cache installations
    let installations = build_installations_async().await?;
    {
        let mut cache_write = INSTALLATIONS_CACHE.write().await;
        *cache_write = Some(installations.clone());
    }
    crate::logging::debug(&format!(
        "Built installations: {} items",
        installations.len()
    ));
    Ok(installations)
}

/// Force rebuild installations from disk and return them, updating the cache.
pub async fn get_installations_force() -> Result<Vec<KableInstallation>, String> {
    // Always rebuild from sources
    let installations = build_installations_async().await?;
    {
        let mut cache_write = INSTALLATIONS_CACHE.write().await;
        *cache_write = Some(installations.clone());
    }
    crate::logging::debug(&format!(
        "Force-built installations: {} items",
        installations.len()
    ));
    Ok(installations)
}

/// Returns a single installation by id, using cache.
pub async fn get_installation(id: &str) -> Result<Option<KableInstallation>, String> {
    let installations = get_installations().await?;
    Ok(installations.into_iter().find(|i| i.id == id))
}
/// Deletes a KableInstallation by ID from kable_profiles.json and invalidates cache
pub async fn delete_installation(id: &str) -> Result<(), String> {
    let mut installations = kable_profiles::read_kable_profiles_async().await?;
    let orig_len = installations.len();

    // Find the installation to get its dedicated folder paths before deletion
    let installation = installations.iter().find(|i| i.id == id);
    let dedicated_folders = installation.map(|inst| {
        (
            inst.dedicated_mods_folder.clone(),
            inst.dedicated_resource_pack_folder.clone(),
            inst.dedicated_shaders_folder.clone(),
            inst.dedicated_config_folder.clone(),
        )
    });

    installations.retain(|i| i.id != id);
    if installations.len() == orig_len {
        crate::logging::Logger::warn_global(
            &format!("No Kable installation found with id: {}", id),
            None,
        );
        return Err(format!("No Kable installation found with id: {}", id));
    }

    // Delete dedicated folders if they exist
    if let Some((mods, resourcepacks, shaders, config)) = dedicated_folders {
        let kable_dir = match crate::get_minecraft_kable_dir() {
            Ok(dir) => dir,
            Err(e) => {
                crate::logging::Logger::warn_global(
                    &format!("Failed to get kable dir for cleanup: {}", e),
                    None,
                );
                std::path::PathBuf::new()
            }
        };

        // Helper to delete a dedicated folder
        let delete_folder = |folder_opt: Option<String>, folder_type: &str| {
            if let Some(folder) = folder_opt {
                let path = std::path::PathBuf::from(&folder);
                let final_path = if path.is_absolute() {
                    path
                } else {
                    // Normalize and construct path same way as get_*_directory functions
                    let normalized = folder.replace('\\', "/");
                    let cleaned = match folder_type {
                        "mods" => normalized.strip_prefix("mods/").unwrap_or(&normalized),
                        "resourcepacks" => normalized
                            .strip_prefix("resourcepacks/")
                            .unwrap_or(&normalized),
                        "shaderpacks" => normalized
                            .strip_prefix("shaderpacks/")
                            .unwrap_or(&normalized),
                        "config" => normalized.strip_prefix("config/").unwrap_or(&normalized),
                        _ => &normalized,
                    };
                    kable_dir.join(folder_type).join(cleaned)
                };

                if final_path.exists() {
                    if let Err(e) = std::fs::remove_dir_all(&final_path) {
                        crate::logging::Logger::warn_global(
                            &format!(
                                "Failed to delete {} folder {}: {}",
                                folder_type,
                                final_path.display(),
                                e
                            ),
                            None,
                        );
                    } else {
                        crate::logging::Logger::debug_global(
                            &format!("Deleted {} folder: {}", folder_type, final_path.display()),
                            None,
                        );
                    }
                }
            }
        };

        delete_folder(mods, "mods");
        delete_folder(resourcepacks, "resourcepacks");
        delete_folder(shaders, "shaderpacks");
        delete_folder(config, "config");
    }

    let result = kable_profiles::write_kable_profiles_async(&installations).await;
    {
        let mut cache_write = INSTALLATIONS_CACHE.write().await;
        *cache_write = Some(installations.clone());
    }
    match &result {
        Ok(_) => crate::logging::info(&format!("Installation '{}' deleted successfully.", id)),
        Err(e) => crate::logging::error(&format!("Failed to delete installation '{}': {}", id, e)),
    }
    result
}

/// Modifies an existing KableInstallation by ID in kable_profiles.json and invalidates cache
pub async fn modify_installation(
    id: &str,
    mut new_installation: KableInstallation,
) -> Result<(), String> {
    let mut installations = kable_profiles::read_kable_profiles_async().await?;
    let index = installations.iter().position(|i| i.id == id);
    if let Some(index) = index {
        // Ensure dedicated mods folder if needed
        let _ = ensure_dedicated_mods_folder(&mut new_installation).await?;
        installations[index] = new_installation;
        let result = kable_profiles::write_kable_profiles_async(&installations).await;
        {
            let mut cache_write = INSTALLATIONS_CACHE.write().await;
            *cache_write = Some(installations.clone());
        }
        match &result {
            Ok(_) => {
                crate::logging::info(&format!("Installation '{}' modified successfully.", id));

                // Emit event to notify frontend of installation update
                if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
                    if let Some(app_handle) = handle_guard.as_ref() {
                        let _ = app_handle.emit(
                            "installation-updated",
                            serde_json::json!({
                                "installation_id": id,
                                "installation": &installations[index]
                            }),
                        );
                    }
                }
            }
            Err(e) => {
                crate::logging::error(&format!("Failed to modify installation '{}': {}", id, e))
            }
        }
        result
    } else {
        crate::logging::Logger::warn_global(
            &format!("No Kable installation found with id: {}", id),
            None,
        );
        Err(format!("No Kable installation found with id: {}", id))
    }
}

/// Creates a new KableInstallation with the given version_id, using default settings for other fields and invalidates cache
pub async fn create_installation(version_id: &str) -> Result<KableInstallation, String> {
    let mut installations = kable_profiles::read_kable_profiles_async().await?;
    // Generate a default name (e.g., based on version_id and count)
    let base_name = version_id.to_string();
    let mut name = base_name.clone();
    let mut count = 1;
    while installations.iter().any(|i| i.name == name) {
        name = format!("{}-{}", base_name, count);
        count += 1;
    }
    let versions = get_versions().await;
    let version_data = versions
        .get_version(version_id)
        .cloned()
        .ok_or_else(|| format!("No version found for id: {}", version_id))?;
    crate::logging::info(&format!(
        "Creating new installation: name='{}', version_id='{}'",
        name, version_data.version_id
    ));
    let mut new_installation = KableInstallation {
        name,
        version_id: version_data.version_id.clone(),
        ..Default::default()
    };
    // Ensure dedicated mods folder if needed
    let _ = ensure_dedicated_mods_folder(&mut new_installation).await?;
    installations.push(new_installation.clone());
    let result = kable_profiles::write_kable_profiles_async(&installations).await;
    {
        let mut cache_write = INSTALLATIONS_CACHE.write().await;
        *cache_write = Some(installations.clone());
    }
    match &result {
        Ok(_) => crate::logging::info(&format!(
            "Installation '{}' created successfully.",
            new_installation.name
        )),
        Err(e) => crate::logging::error(&format!(
            "Failed to create installation '{}': {}",
            new_installation.name, e
        )),
    }
    result?;
    Ok(new_installation)
}

/// Creates a new KableInstallation by copying from an existing one
/// Optionally copies mods (with version updates), resource packs, and shaders
pub async fn create_installation_from_existing(
    version_id: &str,
    source_installation_id: &str,
    copy_mods: bool,
    copy_resource_packs: bool,
    copy_shaders: bool,
) -> Result<KableInstallation, String> {
    // First create the base installation
    let new_installation = create_installation(version_id).await?;

    // Get the source installation
    let source_installation = get_installation(source_installation_id)
        .await?
        .ok_or_else(|| format!("Source installation not found: {}", source_installation_id))?;

    crate::logging::info(&format!(
        "Copying from installation '{}' to '{}' (mods: {}, resourcepacks: {}, shaders: {})",
        source_installation.name,
        new_installation.name,
        copy_mods,
        copy_resource_packs,
        copy_shaders
    ));

    // Copy mods if requested
    if copy_mods {
        copy_and_update_mods(&source_installation, &new_installation).await?;
    }

    // Copy resource packs if requested
    if copy_resource_packs {
        copy_resource_packs_between_installations(&source_installation, &new_installation).await?;
    }

    // Copy shaders if requested
    if copy_shaders {
        copy_shaders_between_installations(&source_installation, &new_installation).await?;
    }

    Ok(new_installation)
}

/// Copy and update mods from source to target installation
/// Uses Modrinth API to find compatible versions for the target MC version and loader
async fn copy_and_update_mods(
    source: &KableInstallation,
    target: &KableInstallation,
) -> Result<(), String> {
    use crate::mods::modrinth;

    crate::logging::info(&format!(
        "Copying and updating mods from '{}' to '{}'",
        source.name, target.name
    ));

    // Get source mods directory
    let source_mods_dir = get_mods_directory(source).await?;
    let target_mods_dir = get_mods_directory(target).await?;

    // Read all mods from source
    let source_mod_files = async_fs::read_dir(&source_mods_dir)
        .await
        .map_err(|e| format!("Failed to read source mods directory: {}", e))?;

    let mut entries = vec![];
    let mut read_dir = source_mod_files;
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read directory entry: {}", e))?
    {
        entries.push(entry);
    }

    // Extract loader and game version from target installation
    let target_loader = extract_loader_from_version_id(&target.version_id);
    let target_game_version = extract_game_version_from_version_id(&target.version_id);

    crate::logging::info(&format!(
        "Target: loader={:?}, game_version={:?}",
        target_loader, target_game_version
    ));

    // Process each mod file
    for entry in entries {
        let path = entry.path();

        // Skip if not a .jar file
        if !matches!(path.extension(), Some(ext) if ext == "jar") {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "Invalid file name".to_string())?;

        crate::logging::info(&format!("Processing mod: {}", file_name));

        // PRIORITY 1: Check for kable_metadata.json file
        let metadata_file = source_mods_dir.join(format!("{}.kable_metadata.json", file_name));
        let project_id_from_metadata = if metadata_file.exists() {
            match async_fs::read_to_string(&metadata_file).await {
                Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(metadata) => {
                        let project_id = metadata
                            .get("project_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        if let Some(ref id) = project_id {
                            crate::logging::info(&format!(
                                "Found metadata for {}: project_id = {}",
                                file_name, id
                            ));
                        }
                        project_id
                    }
                    Err(e) => {
                        crate::logging::Logger::warn_global(
                            &format!("Failed to parse metadata for {}: {}", file_name, e),
                            None,
                        );
                        None
                    }
                },
                Err(_) => None,
            }
        } else {
            None
        };

        // PRIORITY 2: Try to get mod info from JAR if no metadata
        let mod_info = if project_id_from_metadata.is_none() {
            match get_mod_info_single(&path).await {
                Ok(Some(info)) => Some(info),
                Ok(None) => {
                    // Can't read mod info, move to disabled folder
                    let disabled_dir = target_mods_dir.join("disabled");
                    crate::ensure_folder_sync(&disabled_dir)
                        .map_err(|e| format!("Failed to create disabled directory: {}", e))?;
                    let target_path = disabled_dir.join(file_name);
                    crate::logging::Logger::warn_global(
                        &format!(
                            "Could not read mod info from {}, moving to disabled folder",
                            file_name
                        ),
                        None,
                    );
                    async_fs::copy(&path, &target_path).await.map_err(|e| {
                        format!("Failed to copy mod file to disabled folder: {}", e)
                    })?;
                    continue;
                }
                Err(e) => {
                    // Error reading mod, move to disabled folder
                    let disabled_dir = target_mods_dir.join("disabled");
                    crate::ensure_folder_sync(&disabled_dir)
                        .map_err(|e| format!("Failed to create disabled directory: {}", e))?;
                    let target_path = disabled_dir.join(file_name);
                    crate::logging::Logger::warn_global(
                        &format!(
                            "Error reading mod {}: {}, moving to disabled folder",
                            file_name, e
                        ),
                        None,
                    );
                    async_fs::copy(&path, &target_path).await.map_err(|e| {
                        format!("Failed to copy mod file to disabled folder: {}", e)
                    })?;
                    continue;
                }
            }
        } else {
            None
        };

        // Determine project_id (from metadata or search)
        let project_id = if let Some(id) = project_id_from_metadata {
            Some(id)
        } else if let Some(ref info) = mod_info {
            // PRIORITY 3: Search Modrinth by internal name and VERIFY with JAR filename
            if let Some(ref mod_name) = info.mod_name {
                crate::logging::info(&format!(
                    "No metadata found, searching Modrinth for '{}' (internal name: {})",
                    file_name, mod_name
                ));

                let search_url = format!(
                    "https://api.modrinth.com/v2/search?query={}&limit=10",
                    urlencoding::encode(mod_name)
                );

                let search_result: Result<serde_json::Value, String> = async {
                    let resp = reqwest::get(&search_url)
                        .await
                        .map_err(|e| format!("Modrinth search failed: {}", e))?;
                    resp.json()
                        .await
                        .map_err(|e| format!("Modrinth search parse failed: {}", e))
                }
                .await;

                match search_result {
                    Ok(json) => {
                        let hits = json.get("hits").and_then(|h| h.as_array());

                        let verified_project_id = if let Some(hits) = hits {
                            crate::logging::info(&format!(
                                "Search returned {} results for '{}'",
                                hits.len(),
                                mod_name
                            ));

                            // NEW: Verify each search result by checking if it has a version with matching JAR filename
                            let mut found_id = None;
                            for hit in hits {
                                let candidate_id = hit
                                    .get("project_id")
                                    .and_then(|id| id.as_str())
                                    .map(|s| s.to_string());

                                if let Some(ref candidate_id) = candidate_id {
                                    let slug =
                                        hit.get("slug").and_then(|s| s.as_str()).unwrap_or("");
                                    let title =
                                        hit.get("title").and_then(|s| s.as_str()).unwrap_or("");

                                    crate::logging::info(&format!(
                                        "Checking candidate: {} (slug: {}, title: {})",
                                        candidate_id, slug, title
                                    ));

                                    // Fetch ALL versions for this candidate (no filters, to check all possible filenames)
                                    match modrinth::get_project_versions_filtered(
                                        candidate_id,
                                        None, // No loader filter
                                        None, // No game version filter
                                    )
                                    .await
                                    {
                                        Ok(all_versions) => {
                                            // Check if any version has a file matching our current JAR filename
                                            let has_matching_filename =
                                                all_versions.iter().any(|version| {
                                                    version.files.iter().any(|file| {
                                                        file.filename
                                                            .eq_ignore_ascii_case(file_name)
                                                    })
                                                });

                                            if has_matching_filename {
                                                crate::logging::info(&format!(
                                                    "✓ VERIFIED: Project '{}' has a version with filename '{}'",
                                                    candidate_id, file_name
                                                ));

                                                // This is the correct project!
                                                found_id = Some(candidate_id.clone());
                                                break;
                                            } else {
                                                crate::logging::info(&format!(
                                                    "✗ Project '{}' does NOT have filename '{}', trying next result",
                                                    candidate_id, file_name
                                                ));
                                            }
                                        }
                                        Err(e) => {
                                            crate::logging::Logger::warn_global(
                                                &format!("Failed to fetch versions for candidate '{}': {}", candidate_id, e),
                                                None,
                                            );
                                        }
                                    }
                                }
                            }

                            if found_id.is_none() {
                                crate::logging::Logger::warn_global(
                                    &format!("No search results matched JAR filename '{}' for query '{}'", file_name, mod_name),
                                    None,
                                );
                            }

                            found_id
                        } else {
                            None
                        };

                        verified_project_id
                    }
                    Err(e) => {
                        crate::logging::Logger::warn_global(
                            &format!("Failed to search Modrinth for '{}': {}", mod_name, e),
                            None,
                        );
                        None
                    }
                }
            } else {
                crate::logging::Logger::warn_global(
                    &format!("No mod name found in {} JAR", file_name),
                    None,
                );
                None
            }
        } else {
            None
        };

        // If we have a project_id, try to download compatible version
        if let Some(project_id) = project_id {
            crate::logging::info(&format!(
                "Fetching compatible versions for project '{}' (loader: {:?}, game_version: {:?})",
                project_id, target_loader, target_game_version
            ));

            match modrinth::get_project_versions_filtered(
                &project_id,
                target_loader.clone().map(|l| vec![l]),
                target_game_version.clone().map(|v| vec![v]),
            )
            .await
            {
                Ok(versions) if !versions.is_empty() => {
                    crate::logging::info(&format!(
                        "Found {} compatible versions for project '{}'",
                        versions.len(),
                        project_id
                    ));

                    // Find the best version
                    let best_version = modrinth::find_best_version(
                        &versions,
                        target_loader.as_deref(),
                        target_game_version.as_deref(),
                    );

                    if let Some(version) = best_version {
                        crate::logging::info(&format!(
                            "Selected best version: {} ({})",
                            version.version_number, version.id
                        ));

                        // Download the compatible version
                        let primary_file = version
                            .files
                            .iter()
                            .find(|f| f.primary)
                            .or_else(|| version.files.first())
                            .ok_or_else(|| "No files found in version".to_string())?;

                        let target_path = target_mods_dir.join(&primary_file.filename);

                        crate::logging::info(&format!(
                            "Downloading {} from {}",
                            primary_file.filename, primary_file.url
                        ));

                        modrinth::download_mod_file(&primary_file.url, &target_path).await?;

                        crate::logging::info(&format!(
                            "Successfully downloaded: {}",
                            primary_file.filename
                        ));
                        continue;
                    } else {
                        crate::logging::Logger::warn_global(
                            &format!(
                                "Could not select best version for project '{}' from {} candidates",
                                project_id,
                                versions.len()
                            ),
                            None,
                        );
                    }
                }
                Ok(_) => {
                    crate::logging::Logger::warn_global(
                        &format!("No compatible versions found for project '{}' (loader: {:?}, game_version: {:?})", 
                            project_id, target_loader, target_game_version),
                        None,
                    );
                }
                Err(e) => {
                    crate::logging::Logger::warn_global(
                        &format!(
                            "Failed to fetch versions for project '{}': {}",
                            project_id, e
                        ),
                        None,
                    );
                }
            }
        } else {
            crate::logging::Logger::warn_global(
                &format!(
                    "No project ID found for {}, cannot check for updates",
                    file_name
                ),
                None,
            );
        }

        // Fallback: copy the original file to the disabled folder
        // (likely outdated/incompatible with the new version)
        let disabled_dir = target_mods_dir.join("disabled");
        crate::ensure_folder_sync(&disabled_dir)
            .map_err(|e| format!("Failed to create disabled directory: {}", e))?;

        let target_path = disabled_dir.join(file_name);
        crate::logging::Logger::warn_global(
            &format!(
                "No compatible update found for '{}', moving to disabled folder (may be outdated/incompatible)",
                file_name
            ),
            None,
        );
        async_fs::copy(&path, &target_path)
            .await
            .map_err(|e| format!("Failed to copy mod file to disabled folder: {}", e))?;
    }

    crate::logging::info("Finished copying and updating mods");
    Ok(())
}

/// Copy resource packs between installations
async fn copy_resource_packs_between_installations(
    source: &KableInstallation,
    target: &KableInstallation,
) -> Result<(), String> {
    crate::logging::info(&format!(
        "Copying resource packs from '{}' to '{}'",
        source.name, target.name
    ));

    let source_dir = get_resource_packs_directory(source).await?;
    let target_dir = get_resource_packs_directory(target).await?;

    copy_directory_contents(source_dir, target_dir).await?;

    crate::logging::info("Finished copying resource packs");
    Ok(())
}

/// Copy shaders between installations
async fn copy_shaders_between_installations(
    source: &KableInstallation,
    target: &KableInstallation,
) -> Result<(), String> {
    crate::logging::info(&format!(
        "Copying shaders from '{}' to '{}'",
        source.name, target.name
    ));

    let source_dir = get_shaders_directory(source).await?;
    let target_dir = get_shaders_directory(target).await?;

    copy_directory_contents(source_dir, target_dir).await?;

    crate::logging::info("Finished copying shaders");
    Ok(())
}

/// Copy all files from source directory to target directory
fn copy_directory_contents(
    source: PathBuf,
    target: PathBuf,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> {
    Box::pin(async move {
        // Create target directory if it doesn't exist
        async_fs::create_dir_all(&target)
            .await
            .map_err(|e| format!("Failed to create target directory: {}", e))?;

        let mut entries = async_fs::read_dir(&source)
            .await
            .map_err(|e| format!("Failed to read source directory: {}", e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("Failed to read directory entry: {}", e))?
        {
            let source_path = entry.path();
            let file_name = source_path
                .file_name()
                .ok_or_else(|| "Invalid file name".to_string())?;
            let target_path = target.join(file_name);

            if source_path.is_dir() {
                // Recursively copy directories
                copy_directory_contents(source_path, target_path).await?;
            } else {
                // Copy file
                async_fs::copy(&source_path, &target_path)
                    .await
                    .map_err(|e| format!("Failed to copy file: {}", e))?;
            }
        }

        Ok(())
    })
}

/// Get mods directory for an installation
pub async fn get_mods_directory(installation: &KableInstallation) -> Result<PathBuf, String> {
    if let Some(ref dedicated_folder) = installation.dedicated_mods_folder {
        let path = PathBuf::from(dedicated_folder);
        let final_path = if path.is_absolute() {
            path
        } else {
            // Normalize path separators and strip any leading "mods/" or "mods\" prefix
            let normalized = dedicated_folder.replace('\\', "/");
            let cleaned = normalized.strip_prefix("mods/").unwrap_or(&normalized);

            // The dedicated_folder is just the installation ID
            // It should be in .minecraft/.kable/mods/<id>
            let kable_dir = crate::get_minecraft_kable_dir()?;
            kable_dir.join("mods").join(cleaned)
        };
        async_fs::create_dir_all(&final_path)
            .await
            .map_err(|e| format!("Failed to create dedicated mods folder: {}", e))?;
        Ok(final_path)
    } else {
        // Use default mods folder in .minecraft directory
        let minecraft_dir = crate::get_default_minecraft_dir()?;
        let path = minecraft_dir.join("mods");
        async_fs::create_dir_all(&path)
            .await
            .map_err(|e| format!("Failed to create mods folder: {}", e))?;
        Ok(path)
    }
}

/// Get resource packs directory for an installation
async fn get_resource_packs_directory(installation: &KableInstallation) -> Result<PathBuf, String> {
    if let Some(ref dedicated_folder) = installation.dedicated_resource_pack_folder {
        let path = PathBuf::from(dedicated_folder);
        let final_path = if path.is_absolute() {
            path
        } else {
            // Normalize path separators and strip any leading "resourcepacks/" prefix
            let normalized = dedicated_folder.replace('\\', "/");
            let cleaned = normalized
                .strip_prefix("resourcepacks/")
                .unwrap_or(&normalized);

            // The dedicated_folder is just the installation ID
            // It should be in .minecraft/.kable/resourcepacks/<id>
            let kable_dir = crate::get_minecraft_kable_dir()?;
            kable_dir.join("resourcepacks").join(cleaned)
        };
        async_fs::create_dir_all(&final_path)
            .await
            .map_err(|e| format!("Failed to create dedicated resource packs folder: {}", e))?;
        Ok(final_path)
    } else {
        let minecraft_dir = crate::get_default_minecraft_dir()?;
        let path = minecraft_dir.join("resourcepacks");
        async_fs::create_dir_all(&path)
            .await
            .map_err(|e| format!("Failed to create resource packs folder: {}", e))?;
        Ok(path)
    }
}

/// Get shaders directory for an installation
async fn get_shaders_directory(installation: &KableInstallation) -> Result<PathBuf, String> {
    if let Some(ref dedicated_folder) = installation.dedicated_shaders_folder {
        let path = PathBuf::from(dedicated_folder);
        let final_path = if path.is_absolute() {
            path
        } else {
            // Normalize path separators and strip any leading "shaderpacks/" prefix
            let normalized = dedicated_folder.replace('\\', "/");
            let cleaned = normalized
                .strip_prefix("shaderpacks/")
                .unwrap_or(&normalized);

            // The dedicated_folder is just the installation ID
            // It should be in .minecraft/.kable/shaderpacks/<id>
            let kable_dir = crate::get_minecraft_kable_dir()?;
            kable_dir.join("shaderpacks").join(cleaned)
        };
        async_fs::create_dir_all(&final_path)
            .await
            .map_err(|e| format!("Failed to create dedicated shaders folder: {}", e))?;
        Ok(final_path)
    } else {
        let minecraft_dir = crate::get_default_minecraft_dir()?;
        let path = minecraft_dir.join("shaderpacks");
        async_fs::create_dir_all(&path)
            .await
            .map_err(|e| format!("Failed to create shaders folder: {}", e))?;
        Ok(path)
    }
}

/// Extract loader type from version_id
fn extract_loader_from_version_id(version_id: &str) -> Option<String> {
    let version_lower = version_id.to_lowercase();

    if version_lower.contains("fabric") {
        Some("fabric".to_string())
    } else if version_lower.contains("neoforge") {
        Some("neoforge".to_string())
    } else if version_lower.contains("forge") {
        Some("forge".to_string())
    } else if version_lower.contains("quilt") {
        Some("quilt".to_string())
    } else {
        None
    }
}

/// Extract Minecraft version from version_id
fn extract_game_version_from_version_id(version_id: &str) -> Option<String> {
    // Minecraft versions always start with "1." (e.g., 1.20.4, 1.21.4, 1.16.5)
    // This helps distinguish from loader versions like "0.16.10"
    if let Ok(mc_version_regex) = regex::Regex::new(r"\b(1\.\d+(?:\.\d+)?)\b") {
        // Find all matches and take the last one (most specific)
        let matches: Vec<_> = mc_version_regex
            .captures_iter(version_id)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        if !matches.is_empty() {
            // Return the last (most specific) match
            return Some(matches.last().unwrap().clone());
        }
    }
    None
}

/// Get mod info from a single jar file
async fn get_mod_info_single(path: &std::path::Path) -> Result<Option<ModJarInfo>, String> {
    let path_clone = path.to_path_buf();
    task::spawn_blocking(move || {
        let file =
            File::open(&path_clone).map_err(|e| format!("Failed to open mod file: {}", e))?;
        let mut archive =
            ZipArchive::new(file).map_err(|e| format!("Failed to read jar archive: {}", e))?;

        let file_name = path_clone
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        // Try to read mod info from various manifest files
        // fabric.mod.json
        if let Ok(mut fabric_file) = archive.by_name("fabric.mod.json") {
            let mut content = String::new();
            fabric_file
                .read_to_string(&mut content)
                .map_err(|e| format!("Failed to read fabric.mod.json: {}", e))?;

            let json: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse fabric.mod.json: {}", e))?;

            return Ok(Some(ModJarInfo {
                file_name: file_name.clone(),
                mod_name: json["id"]
                    .as_str()
                    .or_else(|| json["name"].as_str())
                    .map(|s| s.to_string()),
                mod_version: json["version"].as_str().map(|s| s.to_string()),
                loader: Some("fabric".to_string()),
                disabled: false,
            }));
        }

        // quilt.mod.json
        if let Ok(mut quilt_file) = archive.by_name("quilt.mod.json") {
            let mut content = String::new();
            quilt_file
                .read_to_string(&mut content)
                .map_err(|e| format!("Failed to read quilt.mod.json: {}", e))?;

            let json: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse quilt.mod.json: {}", e))?;

            let quilt_loader = &json["quilt_loader"];
            return Ok(Some(ModJarInfo {
                file_name: file_name.clone(),
                mod_name: quilt_loader["id"]
                    .as_str()
                    .or_else(|| quilt_loader["metadata"]["name"].as_str())
                    .map(|s| s.to_string()),
                mod_version: quilt_loader["version"].as_str().map(|s| s.to_string()),
                loader: Some("quilt".to_string()),
                disabled: false,
            }));
        }

        // META-INF/mods.toml (Forge/NeoForge)
        if let Ok(mut forge_file) = archive.by_name("META-INF/mods.toml") {
            let mut content = String::new();
            forge_file
                .read_to_string(&mut content)
                .map_err(|e| format!("Failed to read mods.toml: {}", e))?;

            let toml: TomlValue = toml::from_str(&content)
                .map_err(|e| format!("Failed to parse mods.toml: {}", e))?;

            if let Some(mods_array) = toml.get("mods").and_then(|v| v.as_array()) {
                if let Some(first_mod) = mods_array.first() {
                    let loader_type = if file_name.to_lowercase().contains("neoforge") {
                        "neoforge"
                    } else {
                        "forge"
                    };

                    return Ok(Some(ModJarInfo {
                        file_name: file_name.clone(),
                        mod_name: first_mod
                            .get("modId")
                            .and_then(|v| v.as_str())
                            .or_else(|| first_mod.get("displayName").and_then(|v| v.as_str()))
                            .map(|s| s.to_string()),
                        mod_version: first_mod
                            .get("version")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        loader: Some(loader_type.to_string()),
                        disabled: false,
                    }));
                }
            }
        }

        Ok(None)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
