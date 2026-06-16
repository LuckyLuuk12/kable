use std::collections::HashMap;

use crate::constants::{CONFIG_DIR, KABLE_PROFILES_FILE, LAUNCHER_PROFILES_FILE, MODS_DIR, RESOURCEPACKS_DIR, SHADERPACKS_DIR};
use crate::system::fs::{get_default_minecraft_dir, get_kable_launcher_dir, read_to_string, write_file_atomic_async};
use api_types::profiles::LauncherProfiles;
pub use api_types::profiles::{KableProfile, Profile};

/// A way to convert a official launcher profile into a KableProfile, which is the internal representation of a profile in Kable
fn into(profile: Profile) -> KableProfile {
    KableProfile {
        id: profile.id,
        name: profile.name.unwrap_or(format!("Kable-{}", profile.id)),
        icon: profile.icon,
        version: profile.version.into(),
        created: profile.created.unwrap_or_else(chrono::Utc::now),
        last_used: profile.last_used.unwrap_or_else(chrono::Utc::now),
        // Map java arg string to hashmap of key-value pairs, or empty hashmap if None
        java_args: profile.java_args.map_or(Vec::new(), |argstr| {
            argstr
                .split_whitespace()
                .filter_map(|s| {
                    let mut parts = s.splitn(2, '=');
                    let key = parts.next()?;
                    let value = parts.next().unwrap_or("");
                    Some((key.to_string(), value.to_string()))
                })
                .collect()
        }),
        dedicated_mods_folder: Some(format!("{}/{}", MODS_DIR, profile.id)),
        dedicated_config_folder: Some(format!("{}/{}", CONFIG_DIR, profile.id)),
        dedicated_resource_pack_folder: Some(format!("{}/{}", RESOURCEPACKS_DIR, profile.id)),
        dedicated_shaders_folder: Some(format!("{}/{}", SHADERPACKS_DIR, profile.id)),
        favorite: false,
        total_time_played_ms: profile.total_time_played_ms,
        parameters_map: HashMap::new(),
        description: profile.description,
        times_launched: profile.times_launched,
        enable_pack_merging: profile.enable_pack_merging,
        pack_order: profile.pack_order,
        merged_packs: profile.merged_packs,
    }
}

/// Parse the .minecraft/launcher_profiles.json file into LauncherProfiles type:
async fn parse_launcher_profiles() -> Result<LauncherProfiles, String> {
    let launcher_profiles = get_default_minecraft_dir()?.join(LAUNCHER_PROFILES_FILE);
    let content = read_to_string(&launcher_profiles).await?;
    let launcher_profiles: LauncherProfiles = serde_json::from_str(&content)?;
    Ok(launcher_profiles)
}

async fn parse_kable_profiles() -> Result<Vec<KableProfile>, String> {
    let launcher_dir = get_kable_launcher_dir()?;
    let profiles_file = launcher_dir.join(KABLE_PROFILES_FILE);
    if !profiles_file.exists() {
        return Ok(Vec::new());
    }
    let content = read_to_string(&profiles_file).await?;
    let kable_profiles: Vec<KableProfile> = serde_json::from_str(&content)?;
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

    // Iterate over launcher profiles and convert them to KableProfiles
    for (id, profile) in launcher_profiles.profiles {
        if !kable_profiles_map.contains_key(&id) {
            merged_profiles.push(into(profile));
        }
    }

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
    let launcher_dir = get_kable_launcher_dir()?;
    let profiles_file = launcher_dir.join(KABLE_PROFILES_FILE);
    let content = serde_json::to_string_pretty(profiles)?;
    write_file_atomic_async(&profiles_file, content.as_bytes()).await?;
    Ok(())
}
