use std::path::PathBuf;

use api_types::profiles::{KableProfile, KableProfileMetadata, KableProfileSettings, LoaderKind, Projects};

use crate::{
    features::{profiles::kable_profile::load_profiles, projects::management::ensure_profile_projects},
    integrations::loaders::{get_version_data, get_versions},
    system::fs::profiles_dir,
};

/*
 *  create profile functions:
 * - new profile from loader, version and some (optional) metadata (name, icon, description)
 * - new profile by import from .minecraft folder
 * - new profile by import from existing profile (not a copy but from exported profile folder with mods, resourcepacks, etc.)
 * - new profile with modpack(s) (mrpack) as basis
 */

pub async fn from_loader_version(
    loader: LoaderKind,
    version: String,
    name: Option<String>,
    icon: Option<String>,
    description: Option<String>,
) -> Result<KableProfile, String> {
    let version_data = get_version_data(loader, version.clone(), false).await?;

    let profile_name = name.unwrap_or_else(|| format!("{:?}-{}", loader, version));
    let profile_icon = icon; // TODO: Make a default icon for kable profiles based on the loader?
    let profile_description = description;

    Ok(KableProfile {
        id: uuid::Uuid::new_v4().to_string(),
        metadata: KableProfileMetadata {
            name: profile_name,
            icon: profile_icon,
            description: profile_description,
            created: chrono::Utc::now().to_string(),
            last_used: chrono::Utc::now().to_string(),
            favorite: false,
            total_time_played_ms: 0,
            times_launched: 0,
        },
        version: version_data,
        settings: KableProfileSettings {
            java_args: Vec::new(), // TODO : add default java args just like the official launcher does
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

/// The create_profile() function that wraps all ways of creating a profile so:
/// - from version_data (this data should include the version id and loader)
/// - from an (older) profile with newer/other version_data
/// - from exported profile data (should be a zip with all info required to recreate the profile exactly, including mods, resource packs, etc. This is for profile sharing)
/// - from 1 (or more) mrpack's
/// - from any combination of the above, where the user can choose which data to take from each source (e.g. version data from version_data, mods from mrpack, etc.)
///
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
    } else if let Some(exported) = exported_profile.as_ref() {
        import(exported.clone()).await?
    } else if let Some(mrpack) = mrpack.as_ref() {
        crate::features::modpack::into(mrpack.clone()).await?
    } else if let Some(version_id) = version_id.as_ref() {
        new_profile(version_id.clone()).await?
    } else {
        return Err("At least one of version_id, base_profile, exported_profile, or mrpack must be provided".to_string());
    };
    // ! Now we should merge the other sources into the new_profile, if they exist, and then return it
    if let Some(exported) = exported_profile {
        let mut imported_profile = import(exported).await?;
        // Merge the imported_profile into new_profile, with imported_profile taking precedence over new_profile
        new_profile = crate::features::profiles::merge::merge_profiles(&mut new_profile, &mut imported_profile)?;
    }
    if let Some(mrpack) = mrpack {
        let mut mrpack_profile = crate::features::modpack::into(mrpack).await?;
        // Merge the mrpack_profile into new_profile, with mrpack_profile taking precedence over new_profile
        new_profile = crate::features::profiles::merge::merge_profiles(&mut new_profile, &mut mrpack_profile)?;
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
        // ! here we should try to update all projects in the profile, incompatible ones should be disabled...
        for project_type in [
            api_types::projects::ProjectType::Mod,
            api_types::projects::ProjectType::Resourcepack,
            api_types::projects::ProjectType::Shader,
        ] {
            let _updated_projects = crate::features::projects::management::update_all_projects(new_profile.clone(), project_type).await?;
        }
        // Now we disabled any incompatible projects.
        let _incompatible_projects = crate::features::projects::management::check_incompatible_projects(new_profile.clone(), true).await?;
    }
    Ok(new_profile)
}

/// The pathbuf should point to a folder/zip that contains a kable.export.json file which contains a KableProfile in JSON format.
/// This function will do all steps to ensure the profile is imported and working, including:
/// - Checking if the profile id already exists, and if so generating a new unique id for the imported profile
/// - Checking if the version_id of the imported profile is compatible with the current kable version
/// - installing any missing mods, resource packs, shaders, etc. that are required by the imported profile
///
/// ! NOTE: this function does NOT save the imported profile to disk, it just returns the imported profile as a KableProfile struct. The caller should then call save_profiles() to save it to disk.
async fn import(exported_profile: PathBuf) -> Result<KableProfile, String> {
    // ? We need to parse the exported profile and return it as a KableProfile
    let profile_path = exported_profile.join("kable.export.json");
    if !profile_path.exists() {
        return Err(format!("Exported profile does not contain kable.export.json at path: {:?}", profile_path));
    }
    let profile_json = std::fs::read_to_string(profile_path).map_err(|e| format!("Failed to read exported profile: {}", e))?;
    let mut imported_profile: KableProfile =
        serde_json::from_str(&profile_json).map_err(|e| format!("Failed to parse exported profile JSON: {}", e))?;
    // ? Now we need to check if the imported profile id already exists in the current profiles, and if so generate a new unique id for the imported profile
    let existing_profiles = load_profiles().await?;
    if existing_profiles.iter().any(|p| p.id == imported_profile.id) {
        imported_profile.id = uuid::Uuid::new_v4().to_string();
    }
    // ? Now we need to check if the version_id of the imported profile is compatible with the current kable version, and if not we error
    let all_versions = get_versions().await?.0;
    let version_data = all_versions.iter().find(|v| v.id == imported_profile.version.id);
    if version_data.is_none() {
        return Err(format!("Imported profile version_id {} is not compatible with current kable version", imported_profile.version.id));
    }
    // ? Now we need to check if the imported profile has any missing mods, resource packs, shaders, etc. that are required by the imported profile, and if so we should install them
    ensure_profile_projects(imported_profile.clone()).await?;
    // ? Finally, we have to copy any remaining files from the folder/zip into the profiles_dir/<profile_id>/ folder
    let target_dir = profiles_dir()?.join(&imported_profile.id);
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir).map_err(|e| format!("Failed to create target profile directory: {}", e))?;
    }
    // ? Now we copy all files from the exported_profile folder into the target_dir, except for the kable.export.json file which we already parsed
    for entry in std::fs::read_dir(&exported_profile).map_err(|e| format!("Failed to read exported profile directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read exported profile directory entry: {}", e))?;
        let path = entry.path();
        if path.is_file() && path.file_name().unwrap() != "kable.export.json" {
            let file_name = path.file_name().unwrap();
            let target_file = target_dir.join(file_name);
            std::fs::copy(&path, &target_file).map_err(|e| format!("Failed to copy file {:?} to target directory: {}", path, e))?;
        }
    }
    Ok(imported_profile)
}

/// This only make a new profile instance (not saved to disk) with given version_id, unique id and default fields. This is used for creating a new profile from scratch,
/// or for creating a new profile from an existing profile with a different version_id.
/// ! NOTE: this function does NOT save the new profile to disk, it just returns the new profile as a KableProfile struct. The caller should then call save_profiles() to save it to disk.
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
        metadata: KableProfileMetadata {
            name: format!("KableProfile ({})", version_id),
            icon: None,
            description: Some(format!(
                "Profile created from version `{}` at `{}` with unique ID `{}` through the Kable Launcher",
                version_id,
                chrono::Utc::now().to_rfc3339(),
                id
            )),
            created: chrono::Utc::now().to_string(),
            last_used: chrono::Utc::now().to_string(),
            favorite: false,
            total_time_played_ms: 0,
            times_launched: 0,
        },
        version: version_data,
        settings: KableProfileSettings {
            java_args: Vec::new(), // TODO : add default java args just like the official launcher does
            parameters_map: std::collections::HashMap::new(),
            enable_pack_merging: true,
            pack_order: Vec::new(),
            merged_packs: Vec::new(),
            mods: Projects::default(),
            resourcepacks: Projects::default(),
            shaders: Projects::default(),
        },
    };
    Ok(new_profile)
}
