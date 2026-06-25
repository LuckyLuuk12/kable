// List, Remove, Add, Enable/Disable mod projects

// for now I implement all mods features here download/install, remove, enable/disable, update, list, check for updates, etc. Later we can split them into separate files if needed
use crate::integrations::modrinth::client::{download_project, search_mods};
use crate::system::fs;
use api_types::mods::{KableMod, Project, ProjectSearch};
use api_types::profiles::KableProfile;
use futures::{stream, StreamExt, TryStreamExt};

// ? LIST MODS

async fn list_enabled_mods(profile: &KableProfile) -> Result<Vec<KableMod>, String> {
    let mods_folder = profile.dedicated_mods_folder.as_ref().ok_or("invalid mods folder path")?;
    let metadata_dir = fs::read_dir(mods_folder).await?;
    let mut enabled_mods = Vec::new();
    for pathbuf in metadata_dir {
        if let Some(filename) = pathbuf.file_name() {
            if let Some(filename_str) = filename.to_str() {
                if filename_str.ends_with(".json") {
                    let metadata_path = std::path::Path::new(mods_folder).join(filename_str);
                    let serialized_project = fs::read_str(&metadata_path).await?;
                    let project: KableMod = serde_json::from_str(&serialized_project).map_err(|e| e.to_string())?;
                    enabled_mods.push(project);
                }
            }
        }
    }
    Ok(enabled_mods)
}

async fn list_disabled_mods(profile: &KableProfile) -> Result<Vec<KableMod>, String> {
    let mods_folder = profile.dedicated_mods_folder.as_ref().ok_or("invalid mods folder path")?;
    let disabled_folder = std::path::Path::new(mods_folder).join("disabled");
    let mut disabled_mods = Vec::new();
    if disabled_folder.exists() {
        let metadata_dir = fs::read_dir(disabled_folder).await?;
        for pathbuf in metadata_dir {
            if let Some(filename) = pathbuf.file_name() {
                if let Some(filename_str) = filename.to_str() {
                    if filename_str.ends_with(".json") {
                        let metadata_path = std::path::Path::new(mods_folder).join("disabled").join(filename_str);
                        let serialized_project = fs::read_str(&metadata_path).await?;
                        let project: KableMod = serde_json::from_str(&serialized_project).map_err(|e| e.to_string())?;
                        disabled_mods.push(project);
                    }
                }
            }
        }
    }
    Ok(disabled_mods)
}

/// Given profile, list all KableMods
pub async fn list_mods(profile: KableProfile) -> Result<Vec<KableMod>, String> {
    // do async-join on both list_enabled_mods and list_disabled_mods, then combine the results into a single Vec<KableMod>
    let (enabled_mods, disabled_mods) = tokio::join!(list_enabled_mods(&profile), list_disabled_mods(&profile));
    let mut mods = Vec::new();
    mods.extend(enabled_mods?);
    mods.extend(disabled_mods?);
    Ok(mods)
}

// ? REMOVE MOD

/// Given profile and KableMod, remove the jar and json from wherever they are (enabled or disabled), and return the removed KableMod
/// NOTE: metadata filename is always same as jar filename, but with .json extension instead of .jar, which is KableMod.filename
pub async fn remove_mod(profile: KableProfile, kable_mod: KableMod) -> Result<KableMod, String> {
    let mods_folder = profile.dedicated_mods_folder.as_ref().ok_or("invalid mods folder path")?;
    let enabled_metadata_path = std::path::Path::new(mods_folder).join(&kable_mod.filename).with_extension("json");
    let disabled_metadata_path = std::path::Path::new(mods_folder).join("disabled").join(&kable_mod.filename).with_extension("json");

    if enabled_metadata_path.exists() {
        fs::remove_file(&enabled_metadata_path).await?;
        let jar_path = std::path::Path::new(mods_folder).join(&kable_mod.filename);
        if jar_path.exists() {
            fs::remove_file(&jar_path).await?;
        }
    } else if disabled_metadata_path.exists() {
        fs::remove_file(&disabled_metadata_path).await?;
        let jar_path = std::path::Path::new(mods_folder).join("disabled").join(&kable_mod.filename);
        if jar_path.exists() {
            fs::remove_file(&jar_path).await?;
        }
    } else {
        return Err(format!("Mod {} not found in enabled or disabled folders", kable_mod.filename));
    }

    Ok(kable_mod)
}

// ? DOWNLOAD + METADATA

/// Given a profile,and Project, add/download the mod
async fn add_mod(profile: KableProfile, project: Project, version_id: Option<&str>) -> Result<KableMod, String> {
    // 1. get profile mod dir
    let mods_folder = profile.dedicated_mods_folder.as_ref().ok_or("invalid mods folder path")?;
    // 2. download the mod files to the mods folder
    let filenames = download_project(&project.clone(), version_id, std::path::PathBuf::from(mods_folder)).await?;
    // 3. assume 1 single file is downladed: the mod, and get the filename
    let filename = filenames.get(0).ok_or("No files downloaded")?.to_string();
    // 4. construct KableMod { project, version_id || project.latest_version unwrapped, filename, enabled: true }
    Ok(KableMod {
        project: project.clone(),
        version_id: version_id
            .or_else(|| project.latest_version.as_deref())
            .expect(&format!("No version id was associated with project {}", project.project_id).to_string())
            .to_string(),
        filename,
        enabled: true,
    })
}

async fn add_metadata(profile: KableProfile, kable_mod: KableMod) -> Result<KableMod, String> {
    // 1. get profile mod dir
    let mods_folder = profile.dedicated_mods_folder.as_ref().ok_or("invalid mods folder path")?;
    // 2. construct metadata path
    let metadata_path = std::path::Path::new(mods_folder).join(&kable_mod.filename).with_extension("json");
    // 3. serialize the KableMod and write it to the metadata file
    let serialized_mod = serde_json::to_string(&kable_mod).map_err(|e| e.to_string())?;
    fs::write_str(&metadata_path, serialized_mod.as_str(), false).await?;
    Ok(kable_mod)
}

pub async fn download_mod(profile: KableProfile, project: Project, version_id: Option<&str>) -> Result<KableMod, String> {
    let kable_mod = add_mod(profile.clone(), project, version_id).await?;
    add_metadata(profile, kable_mod.clone()).await
}

// ? ENABLE/DISABLE MOD

pub async fn enable_mod(profile: KableProfile, kable_mod: KableMod) -> Result<KableMod, String> {
    if kable_mod.enabled {
        return Err(format!("Mod {} is already enabled", kable_mod.filename));
    }
    let mods_folder = profile.dedicated_mods_folder.as_ref().ok_or("invalid mods folder path")?;
    let disabled_metadata_path = std::path::Path::new(mods_folder).join("disabled").join(&kable_mod.filename).with_extension("json");
    let disabled_jar_path = std::path::Path::new(mods_folder).join("disabled").join(&kable_mod.filename);

    if disabled_metadata_path.exists() {
        let enabled_metadata_path = std::path::Path::new(mods_folder).join(&kable_mod.filename).with_extension("json");
        fs::rename(&disabled_metadata_path, &enabled_metadata_path).await?;
    }

    if disabled_jar_path.exists() {
        let enabled_jar_path = std::path::Path::new(mods_folder).join(&kable_mod.filename);
        fs::rename(&disabled_jar_path, &enabled_jar_path).await?;
    }

    Ok(KableMod { enabled: true, ..kable_mod })
}

pub async fn disable_mod(profile: KableProfile, kable_mod: KableMod) -> Result<KableMod, String> {
    if !kable_mod.enabled {
        return Err(format!("Mod {} is already disabled", kable_mod.filename));
    }
    let mods_folder = profile.dedicated_mods_folder.as_ref().ok_or("invalid mods folder path")?;
    let enabled_metadata_path = std::path::Path::new(mods_folder).join(&kable_mod.filename).with_extension("json");
    let enabled_jar_path = std::path::Path::new(mods_folder).join(&kable_mod.filename);

    if enabled_metadata_path.exists() {
        let disabled_metadata_path = std::path::Path::new(mods_folder).join("disabled").join(&kable_mod.filename).with_extension("json");
        fs::rename(&enabled_metadata_path, &disabled_metadata_path).await?;
    }

    if enabled_jar_path.exists() {
        let disabled_jar_path = std::path::Path::new(mods_folder).join("disabled").join(&kable_mod.filename);
        fs::rename(&enabled_jar_path, &disabled_jar_path).await?;
    }

    Ok(KableMod { enabled: false, ..kable_mod })
}

pub async fn toggle_mod(profile: KableProfile, kable_mod: KableMod) -> Result<KableMod, String> {
    if kable_mod.enabled {
        disable_mod(profile, kable_mod).await
    } else {
        enable_mod(profile, kable_mod).await
    }
}

// ? UPDATE + CHECK FOR UPDATES

/// Search again for this profile and specific KableMod project id, check if the latest version is different from the current version, if so return Some(Project) with the latest version, else return None
pub async fn check_for_update(kable_mod: KableMod) -> Result<Option<Project>, String> {
    let project_search = ProjectSearch { query: Some(kable_mod.project.project_id.clone()), ..Default::default() };
    let results = search_mods(project_search, false).await?;
    if let Some(latest_project) = results.hits.into_iter().next() {
        if latest_project.latest_version != Some(kable_mod.version_id.clone()) {
            Ok(Some(latest_project))
        } else {
            Ok(None)
        }
    } else {
        Err(format!("No project found for mod {}", kable_mod.filename))
    }
}

pub async fn check_for_updates(profile: KableProfile) -> Result<Vec<(KableMod, Project)>, String> {
    let mods = list_mods(profile.clone()).await?;
    let mut updates = Vec::new();
    for kable_mod in mods {
        if let Some(latest_project) = check_for_update(kable_mod.clone()).await? {
            updates.push((kable_mod, latest_project));
        }
    }
    Ok(updates)
}

pub async fn update_mod(profile: KableProfile, kable_mod: KableMod) -> Result<KableMod, String> {
    if let Some(latest_project) = check_for_update(kable_mod.clone()).await? {
        let latest_version_id = latest_project.latest_version.as_deref();
        let updated_mod = download_mod(profile.clone(), latest_project.clone(), latest_version_id).await?;
        Ok(updated_mod)
    } else {
        Err(format!("No update available for mod {}", kable_mod.filename))
    }
}

pub async fn update_all_mods(profile: KableProfile) -> Result<Vec<KableMod>, String> {
    let updates = check_for_updates(profile.clone()).await?;

    stream::iter(updates)
        .map(|(kable_mod, _)| {
            let profile = profile.clone();
            async move { update_mod(profile, kable_mod).await }
        })
        .buffer_unordered(8) // TODO: make this configurable
        .try_collect()
        .await
}
