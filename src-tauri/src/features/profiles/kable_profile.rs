use crate::constants::{CONFIG_DIR, KABLE_PROFILES_FILE, MODS_DIR, RESOURCEPACKS_DIR, SHADERPACKS_DIR};
use crate::integrations::loaders::get_versions;
use crate::integrations::minecraft::profiles::parse_launcher_profiles;
use crate::system::fs::{launcher_dir, read_str, write_str};
use api_types::profiles::KableProfile;
use api_types::profiles::LauncherProfiles;
use std::collections::HashMap;
use std::path::PathBuf;

/// A way to convert a official launcher profile into a KableProfile, which is the internal representation of a profile in Kable
async fn into(launcher_profiles: LauncherProfiles) -> Result<Vec<KableProfile>, String> {
    let mut kable_profiles = Vec::new();
    let all_versions = get_versions().await?.0;
    for (id, profile) in launcher_profiles.profiles {
        let version_data = all_versions.iter().find(|v| v.id == profile.last_version_id.clone().unwrap_or_default());
        let version_data = match version_data {
            Some(v) => v.clone(),
            None => continue,
        };
        let kable_profile = KableProfile {
            id: id.clone(),
            name: profile.name.unwrap_or(id.clone().to_string()),
            icon: profile.icon,
            version: version_data,
            created: profile.created.unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            last_used: profile.last_used.unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            java_args: profile.java_args.map(|args| args.split_whitespace().map(|s| s.to_string()).collect()).unwrap_or_else(Vec::new),
            dedicated_mods_folder: Some(format!("{}/{}", MODS_DIR, id)),
            dedicated_config_folder: Some(format!("{}/{}", CONFIG_DIR, id)),
            dedicated_resource_pack_folder: Some(format!("{}/{}", RESOURCEPACKS_DIR, id)),
            dedicated_shaders_folder: Some(format!("{}/{}", SHADERPACKS_DIR, id)),
            favorite: false,
            total_time_played_ms: 0,
            parameters_map: HashMap::new(),
            description: None,
            times_launched: 0,
            enable_pack_merging: true,
            pack_order: Vec::new(),
            merged_packs: Vec::new(),
        };
        kable_profiles.push(kable_profile);
    }
    Ok(kable_profiles)
}

async fn parse_kable_profiles() -> Result<Vec<KableProfile>, String> {
    let launcher_dir = launcher_dir()?;
    let profiles_file = launcher_dir.join(KABLE_PROFILES_FILE);
    if !profiles_file.exists() {
        return Ok(Vec::new());
    }
    let content = read_str(&profiles_file).await?;
    let kable_profiles: Vec<KableProfile> = serde_json::from_str(&content).map_err(|e| format!("Failed to parse kable profiles: {e}"))?;
    Ok(kable_profiles)
}

/// This will take the official launcher profiles and the, if existing, kable profiles and merge them into a single list of KableProfiles
/// with the kable profiles taking precedence over the official launcher profiles in case of duplicate profile IDs
async fn merge_profiles(launcher_profiles: LauncherProfiles, kable_profiles: Vec<KableProfile>) -> Result<Vec<KableProfile>, String> {
    let mut merged_profiles = Vec::new();
    let mut kable_profiles_map: HashMap<String, KableProfile> = HashMap::new();

    // Insert kable profiles into the map
    for profile in kable_profiles {
        kable_profiles_map.insert(profile.id.clone(), profile);
    }

    merged_profiles.extend(into(launcher_profiles).await?);

    // Add all kable profiles to the merged list
    merged_profiles.extend(kable_profiles_map.into_values());

    Ok(merged_profiles)
}

/// async execute loading of the 2 different profile sources then join / await completion of both in async, save the profiles, and return the merged result
/// After sorting by lastUsed such that the most recently used profile is first in the list
pub async fn load_profiles() -> Result<Vec<KableProfile>, String> {
    let launcher_profiles_future = parse_launcher_profiles();
    let kable_profiles_future = parse_kable_profiles();

    let (launcher_profiles_result, kable_profiles_result) = tokio::join!(launcher_profiles_future, kable_profiles_future);

    let launcher_profiles = launcher_profiles_result?;
    let kable_profiles = kable_profiles_result?;

    let mut profiles = merge_profiles(launcher_profiles, kable_profiles).await?;
    profiles.sort_by(|a, b| b.last_used.cmp(&a.last_used));

    save_profiles(&profiles).await?;
    Ok(profiles)
}

pub async fn save_profiles(profiles: &[KableProfile]) -> Result<(), String> {
    let launcher_dir = launcher_dir()?;
    let profiles_file = launcher_dir.join(KABLE_PROFILES_FILE);
    // Ensure profiles are sorted by last_used before saving
    let mut profiles = profiles.to_vec();
    profiles.sort_by(|a, b| b.last_used.cmp(&a.last_used));

    let content = serde_json::to_string_pretty(&profiles).map_err(|e| format!("Failed to serialize profiles: {e}"))?;

    write_str(&profiles_file, &content, false).await?;

    Ok(())
}

/// The create_profile() function that wraps all ways of creating a profile so:
/// - from version_data (this data should include the version id and loader)
/// - from an (older) profile with newer/other version_data
/// - from exported profile data (should be a zip with all info required to recreate the profile exactly, including mods, resource packs, etc. This is for profile sharing)
/// - from 1 (or more) mrpack's
/// - from any combination of the above, where the user can choose which data to take from each source (e.g. version data from version_data, mods from mrpack, etc.)
/// This method will try to optimistically merge all the data sources into a single profile, and will return an error if it cannot do so.
pub async fn create_profile(
    version_id: Option<String>,
    base_profile: Option<KableProfile>,
    exported_profile: Option<PathBuf>,
    mrpack: Option<String>,
) -> Result<KableProfile, String> {
    // ! Now in order we do: base_profile copy, exported_profile merge, mrpack merge, version_id update/downgrade
    // If there is no base_profile though we just shift it such that the exported_profile becomes the base_profile,
    // and if that is None then the mrpack becomes the base_profile, and if that is None then the version_id becomes the base_profile and we create one from scratch essentially
    let mut new_profile = if let Some(base) = base_profile {
        base
    } else if let Some(exported) = exported_profile {
        import(exported).await?
    } else if let Some(mrpack) = mrpack {
        crate::features::modpack::into(mrpack).await?
    } else if let Some(version_id) = version_id {
        new_profile(version_id).await?
    } else {
        return Err("At least one of version_id, base_profile, exported_profile, or mrpack must be provided".to_string());
    };
    // ! Now we should merge the other sources into the new_profile, if they exist, and then return it
    if let Some(exported) = exported_profile {
        let imported_profile = import(exported).await?;
        // Check if this is different from base at all, if not then we don't need to merge it
        if imported_profile != new_profile {
            // Merge the imported_profile into new_profile, with imported_profile taking precedence over new_profile
            new_profile = crate::features::profiles::merge::merge_profiles(new_profile, imported_profile)?;
        }
    }
    if let Some(mrpack) = mrpack {
        let mrpack_profile = crate::features::modpack::into(mrpack).await?;
        // Check if this is different from base at all, if not then we don't need to merge it
        if mrpack_profile != new_profile {
            // Merge the mrpack_profile into new_profile, with mrpack_profile taking precedence over new_profile
            new_profile = crate::features::profiles::merge::merge_profiles(new_profile, mrpack_profile)?;
        }
    }
    if let Some(version_id) = version_id {
        // Update the version_id of new_profile to the provided version_id, and update the version_data accordingly,
        // and if the version_id is different from the current one then we should also update all data (mods, resource packs, etc.) to be compatible with the new version_id
        new_profile.version = get_versions()
            .await?
            .0
            .into_iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| format!("Version data for {} not found", version_id))?;
        // TODO: Now use modrinth to get the latest compatible versions of mods, resource packs, etc. for the new version_id and update the new_profile accordingly
    }
    todo!()
}

async fn import(exported_profile: PathBuf) -> Result<KableProfile, String> {
    // ! We need to parse the exported profile and return it as a KableProfile
    todo!()
}

/// This only make a new profile instance (not saved to disk) with given version_id, unique id and default fields. This is used for creating a new profile from scratch,
/// or for creating a new profile from an existing profile with a different version_id.
async fn new_profile(version_id: String) -> Result<KableProfile, String> {
    let all_versions = get_versions().await?.0;
    let version_data = all_versions.iter().find(|v| v.id == version_id);
    let version_data = match version_data {
        Some(v) => v.clone(),
        None => return Err(format!("Version data for {} not found", version_id)),
    };
    let mut id = uuid::Uuid::new_v4().to_string();
    // check if id is not already used in existing profiles, if so generate a new one until it is unique
    let existing_profiles = load_profiles().await?;
    while existing_profiles.iter().any(|p| p.id == id) {
        id = uuid::Uuid::new_v4().to_string();
    }
    let new_profile = KableProfile {
        id: id.clone(),
        name: format!("KableProfile ({})", version_id),
        icon: None,
        version: version_data,
        created: chrono::Utc::now().to_rfc3339(),
        last_used: chrono::Utc::now().to_rfc3339(),
        java_args: Vec::new(),
        dedicated_mods_folder: Some(format!("{}/{}", MODS_DIR, uuid::Uuid::new_v4())),
        dedicated_config_folder: Some(format!("{}/{}", CONFIG_DIR, uuid::Uuid::new_v4())),
        dedicated_resource_pack_folder: Some(format!("{}/{}", RESOURCEPACKS_DIR, uuid::Uuid::new_v4())),
        dedicated_shaders_folder: Some(format!("{}/{}", SHADERPACKS_DIR, uuid::Uuid::new_v4())),
        favorite: false,
        total_time_played_ms: 0,
        parameters_map: HashMap::new(),
        description: Some(format!(
            "Profile created from version `{}` at `{}` with unique ID `{}` through the Kable Launcher",
            version_id,
            chrono::Utc::now().to_rfc3339(),
            id
        )),
        times_launched: 0,
        enable_pack_merging: true,
        pack_order: Vec::new(),
        merged_packs: Vec::new(),
    };
    Ok(new_profile)
}
