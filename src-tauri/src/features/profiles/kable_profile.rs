use crate::constants::KABLE_PROFILES_FILE;
use crate::integrations::loaders::get_versions;
use crate::integrations::minecraft::profiles::parse_launcher_profiles;
use crate::system::cache::invalidate_no_args;
use crate::system::fs::{launcher_dir, read_str, write_str};
use crate::Logger;
use api_types::profiles::LauncherProfiles;
use api_types::profiles::{KableProfile, Projects};
use kable_macros::persistent_cache;
use std::collections::{HashMap, HashSet};

/// A way to convert a official launcher profile into a KableProfile, which is the internal representation of a profile in Kable
fn into(launcher_profiles: LauncherProfiles, all_versions: &[api_types::profiles::ProfileVersion]) -> Result<Vec<KableProfile>, String> {
    let mut kable_profiles = Vec::new();

    let latest_release = all_versions
        .iter()
        .filter(|v| v.loader == api_types::profiles::LoaderKind::Vanilla && v.stable == Some(true))
        .max_by(|a, b| a.release_time.cmp(&b.release_time));

    let latest_snapshot = all_versions
        .iter()
        .filter(|v| {
            v.loader == api_types::profiles::LoaderKind::Vanilla
                && v.version_type == Some(api_types::profiles::ProfileVersionType::Snapshot)
        })
        .max_by(|a, b| a.release_time.cmp(&b.release_time));

    for (id, profile) in launcher_profiles.profiles {
        let version_id = profile.last_version_id.as_deref();

        let version_data = match version_id {
            Some("latest-release") => latest_release,
            Some("latest-snapshot") => latest_snapshot,
            Some(version_id) => all_versions.iter().find(|v| v.id == version_id),
            None => None,
        };

        let Some(version_data) = version_data.cloned() else {
            continue;
        };

        let kable_profile = KableProfile {
            id,
            version: version_data,
            metadata: api_types::profiles::KableProfileMetadata {
                name: profile.name.filter(|name| !name.is_empty()).unwrap_or_else(|| match version_id {
                    Some("latest-release") => "latest-release".to_string(),
                    Some("latest-snapshot") => "latest-snapshot".to_string(),
                    _ => String::new(),
                }),
                icon: profile.icon,
                created: profile.created.unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
                last_used: profile.last_used.unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
                favorite: false,
                total_time_played_ms: 0,
                description: None,
                times_launched: 0,
            },
            settings: api_types::profiles::KableProfileSettings {
                java_args: profile.java_args.map(|args| args.split_whitespace().map(|s| s.to_string()).collect()).unwrap_or_else(Vec::new),
                parameters_map: HashMap::new(),
                enable_pack_merging: true,
                pack_order: Vec::new(),
                merged_packs: Vec::new(),
                mods: Projects::default(),
                resourcepacks: Projects::default(),
                shaders: Projects::default(),
            },
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
async fn merge_profiles(
    launcher_profiles: LauncherProfiles,
    kable_profiles: Vec<KableProfile>,
    all_versions: &[api_types::profiles::ProfileVersion],
) -> Result<Vec<KableProfile>, String> {
    let mut merged_profiles = into(launcher_profiles, all_versions)?;

    let mut kable_profiles_map: HashMap<String, KableProfile> = HashMap::new();

    for profile in kable_profiles {
        kable_profiles_map.insert(profile.id.clone(), profile);
    }

    merged_profiles.extend(kable_profiles_map.into_values());

    let mut seen = HashSet::new();
    merged_profiles.retain(|profile| seen.insert(profile.id.clone()));

    Ok(merged_profiles)
}

/// async execute loading of the 2 different profile sources then join / await completion of both in async, save the profiles, and return the merged result
/// After sorting by lastUsed such that the most recently used profile is first in the list
#[persistent_cache(parent = "profiles", ttl_secs = 3600)] // cache for 1 hour
pub async fn load_profiles() -> Result<Vec<KableProfile>, String> {
    let launcher_profiles_future = parse_launcher_profiles();
    let kable_profiles_future = parse_kable_profiles();
    let versions_future = get_versions();

    let (launcher_profiles_result, kable_profiles_result, versions_result) =
        tokio::join!(launcher_profiles_future, kable_profiles_future, versions_future);

    let launcher_profiles = launcher_profiles_result?;
    let kable_profiles = kable_profiles_result?;
    let all_versions = versions_result?.0;

    let mut profiles = merge_profiles(launcher_profiles, kable_profiles, &all_versions).await?;

    update_latest_version_profiles(&mut profiles, &all_versions);

    profiles.sort_by(|a, b| b.metadata.last_used.cmp(&a.metadata.last_used));

    save_profiles(&profiles).await?;
    Ok(profiles)
}

pub async fn save_profiles(profiles: &[KableProfile]) -> Result<(), String> {
    let launcher_dir = launcher_dir()?;
    let profiles_file = launcher_dir.join(KABLE_PROFILES_FILE);
    // Ensure profiles are sorted by last_used before saving
    let mut profiles = profiles.to_vec();
    profiles.sort_by(|a, b| b.metadata.last_used.cmp(&a.metadata.last_used));

    let content = serde_json::to_string_pretty(&profiles).map_err(|e| format!("Failed to serialize profiles: {e}"))?;

    write_str(&profiles_file, &content, false).await?;
    Logger::debug_global(format!("Saved profiles to {}", profiles_file.display()).as_str(), None);
    invalidate_no_args("profiles", "load_profiles").await
}

fn update_latest_version_profiles(profiles: &mut [KableProfile], all_versions: &[api_types::profiles::ProfileVersion]) {
    let latest_release = all_versions
        .iter()
        .filter(|v| v.loader == api_types::profiles::LoaderKind::Vanilla && v.stable == Some(true))
        .max_by(|a, b| a.release_time.cmp(&b.release_time));

    let latest_snapshot = all_versions
        .iter()
        .filter(|v| {
            v.loader == api_types::profiles::LoaderKind::Vanilla
                && v.version_type == Some(api_types::profiles::ProfileVersionType::Snapshot)
        })
        .max_by(|a, b| a.release_time.cmp(&b.release_time));

    for profile in profiles {
        match profile.version.id.as_str() {
            "latest-release" => {
                if let Some(version) = latest_release {
                    profile.version = version.clone();
                    profile.version.id = "latest-release".to_string();
                }
            }
            "latest-snapshot" => {
                if let Some(version) = latest_snapshot {
                    profile.version = version.clone();
                    profile.version.id = "latest-snapshot".to_string();
                }
            }
            _ => {}
        }
    }
}
