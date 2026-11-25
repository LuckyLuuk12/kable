use crate::constants::KABLE_PROFILES_FILE;
use crate::integrations::loaders::{get_versions, get_versions_of};
use crate::integrations::minecraft::profiles::parse_launcher_profiles;
use crate::system::cache::invalidate_no_args;
use crate::system::fs::{launcher_dir, read_str, write_str};
use crate::Logger;

use api_types::profiles::LauncherProfiles;
use api_types::profiles::{KableProfile, Projects};
use kable_macros::persistent_cache;

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const LATEST_VERSION_REFRESH_INTERVAL: Duration = Duration::from_secs(15 * 60);

static LAST_LATEST_VERSION_REFRESH: Mutex<Option<Instant>> = Mutex::new(None);

/// A way to convert an official launcher profile into a KableProfile,
/// which is the internal representation of a profile in Kable.
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

        let Some(mut version_data) = version_data.cloned() else {
            continue;
        };

        // Preserve special aliases so they can be refreshed later.
        match version_id {
            Some("latest-release") => {
                version_data.id = "latest-release".to_string();
            }
            Some("latest-snapshot") => {
                version_data.id = "latest-snapshot".to_string();
            }
            _ => {}
        }

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
                launch_with_account: None,
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

/// Merge official launcher profiles with Kable profiles.
///
/// Kable profiles take precedence when profile IDs overlap.
/// Version data is fetched only when new official launcher profiles need importing.
async fn merge_profiles(launcher_profiles: LauncherProfiles, kable_profiles: Vec<KableProfile>) -> Result<Vec<KableProfile>, String> {
    let kable_profile_ids: HashSet<&str> = kable_profiles.iter().map(|profile| profile.id.as_str()).collect();

    let new_launcher_profiles: HashMap<_, _> =
        launcher_profiles.profiles.into_iter().filter(|(id, _)| !kable_profile_ids.contains(id.as_str())).collect();

    if new_launcher_profiles.is_empty() {
        return Ok(kable_profiles);
    }

    let all_versions = get_versions().await?.0;

    let mut merged_profiles = into(LauncherProfiles { profiles: new_launcher_profiles, ..launcher_profiles }, &all_versions)?;

    merged_profiles.extend(kable_profiles);

    Ok(merged_profiles)
}

/// Schedule a background refresh for latest-release and latest-snapshot profiles.
///
/// Refreshes are throttled to avoid repeatedly requesting the Vanilla version
/// manifest whenever profiles are loaded.
fn schedule_latest_version_refresh() {
    let should_schedule = {
        let mut last_refresh = LAST_LATEST_VERSION_REFRESH.lock().unwrap_or_else(|error| error.into_inner());

        let now = Instant::now();

        match *last_refresh {
            Some(previous) if now.duration_since(previous) < LATEST_VERSION_REFRESH_INTERVAL => false,
            _ => {
                *last_refresh = Some(now);
                true
            }
        }
    };

    if !should_schedule {
        return;
    }

    tokio::spawn(async {
        if let Err(error) = refresh_latest_version_profiles().await {
            Logger::debug_global(&format!("[profiles] Failed to refresh latest Minecraft versions: {}", error), None);

            // Allow another attempt after a failed refresh.
            let mut last_refresh = LAST_LATEST_VERSION_REFRESH.lock().unwrap_or_else(|error| error.into_inner());

            *last_refresh = None;
        }
    });
}

/// Fetch the latest Vanilla versions and update persisted alias profiles.
///
/// The network request happens before acquiring the profile file lock.
/// Once the lock is acquired, profiles are reloaded from disk to ensure
/// concurrent profile mutations are not overwritten.
async fn refresh_latest_version_profiles() -> Result<(), String> {
    let vanilla_versions = get_versions_of(api_types::profiles::LoaderKind::Vanilla).await?.0;

    crate::system::fs::with_file_lock(KABLE_PROFILES_FILE, || async move {
        let mut profiles = parse_kable_profiles().await?;

        let updated = update_latest_version_profiles(&mut profiles, &vanilla_versions);

        if !updated {
            return Ok(());
        }

        save_profiles(&profiles).await
    })
    .await
}

/// Load profiles from the official launcher and Kable.
///
/// Existing profiles are returned without waiting for the Vanilla version
/// manifest. Special latest-version aliases are refreshed asynchronously.
#[persistent_cache(parent = "kable_profiles")]
pub async fn load_profiles() -> Result<Vec<KableProfile>, String> {
    let launcher_profiles_future = parse_launcher_profiles();
    let kable_profiles_future = parse_kable_profiles();

    let (launcher_profiles_result, kable_profiles_result) = tokio::join!(launcher_profiles_future, kable_profiles_future);

    let launcher_profiles = launcher_profiles_result?;
    let kable_profiles = kable_profiles_result?;

    let mut profiles = merge_profiles(launcher_profiles, kable_profiles).await?;

    let has_latest_profile = profiles.iter().any(|profile| matches!(profile.version.id.as_str(), "latest-release" | "latest-snapshot"));

    if has_latest_profile {
        schedule_latest_version_refresh();
    }

    profiles.sort_by(|a, b| b.metadata.last_used.cmp(&a.metadata.last_used));

    Ok(profiles)
}

/// Persist Kable profiles.
///
/// Profiles are sorted by last_used before saving.
pub async fn save_profiles(profiles: &[KableProfile]) -> Result<(), String> {
    let launcher_dir = launcher_dir()?;
    let profiles_file = launcher_dir.join(KABLE_PROFILES_FILE);

    let mut profiles = profiles.to_vec();

    profiles.sort_by(|a, b| b.metadata.last_used.cmp(&a.metadata.last_used));

    let content = serde_json::to_string_pretty(&profiles).map_err(|e| format!("Failed to serialize profiles: {e}"))?;
    invalidate_no_args("kable_profiles", "save_profiles").await?;
    write_str(&profiles_file, &content, false).await?;

    Logger::info(format!("Saved profiles to {}", profiles_file.display()).as_str(), None);

    Ok(())
}

/// Update profiles that use latest-release or latest-snapshot.
///
/// Returns true if at least one profile was updated.
fn update_latest_version_profiles(profiles: &mut [KableProfile], all_versions: &[api_types::profiles::ProfileVersion]) -> bool {
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

    let mut updated = false;

    for profile in profiles {
        match profile.version.id.as_str() {
            "latest-release" => {
                if let Some(version) = latest_release {
                    profile.version = version.clone();
                    profile.version.id = "latest-release".to_string();
                    updated = true;
                }
            }
            "latest-snapshot" => {
                if let Some(version) = latest_snapshot {
                    profile.version = version.clone();
                    profile.version.id = "latest-snapshot".to_string();
                    updated = true;
                }
            }
            _ => {}
        }
    }

    updated
}
