use crate::constants::{CONFIG_DIR, KABLE_PROFILES_FILE, MODS_DIR, RESOURCEPACKS_DIR, SHADERPACKS_DIR};
use crate::integrations::loaders::get_versions;
use crate::integrations::minecraft::profiles::parse_launcher_profiles;
use crate::system::fs::{launcher_dir, read_str, write_str};
use api_types::profiles::LauncherProfiles;
use api_types::profiles::{KableProfile, Profile};
use std::collections::HashMap;
use std::ops::Deref;

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
            name: profile.name.unwrap_or(format!("{}", id.clone())),
            icon: profile.icon,
            version: version_data,
            created: profile.created.unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            last_used: profile.last_used.unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            java_args: profile.java_args.map(|args| args.split_whitespace().map(|s| s.to_string()).collect()).unwrap_or_else(|| Vec::new()),
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

/// async execute loading of the 2 different profile sources then join / await completion of both in async and return the merged result
pub async fn load_profiles() -> Result<Vec<KableProfile>, String> {
    let launcher_profiles_future = parse_launcher_profiles();
    let kable_profiles_future = parse_kable_profiles();

    let (launcher_profiles_result, kable_profiles_result) = tokio::join!(launcher_profiles_future, kable_profiles_future);

    let launcher_profiles = launcher_profiles_result?;
    let kable_profiles = kable_profiles_result?;

    merge_profiles(launcher_profiles, kable_profiles).await
}

pub async fn save_profiles(profiles: &[KableProfile]) -> Result<(), String> {
    let launcher_dir = launcher_dir()?;
    let profiles_file = launcher_dir.join(KABLE_PROFILES_FILE);
    let content = serde_json::to_string_pretty(profiles).map_err(|e| format!("Failed to serialize profiles: {e}"))?;

    write_str(&profiles_file, &content, false).await?;

    Ok(())
}
