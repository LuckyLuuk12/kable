use crate::features::profiles::kable_profile::{load_profiles, save_profiles};
use crate::system::cache::invalidate_no_args;
use crate::system::fs;
use api_types::profiles::KableProfile;
use api_types::projects::KableProject;

const PROFILES_CACHE_NAME: &str = "profiles";
const PROFILES_CACHE_FUNCTION: &str = "load_profiles";

async fn invalidate_profiles_cache() {
    // The profile mutation has already been persisted. Cache invalidation
    // must not turn a successful mutation into an apparent failure.
    let _ = invalidate_no_args(PROFILES_CACHE_NAME, PROFILES_CACHE_FUNCTION).await;
}

async fn save_profiles_and_invalidate(profiles: &[KableProfile]) -> Result<(), String> {
    save_profiles(profiles).await?;
    invalidate_profiles_cache().await;
    Ok(())
}

async fn modify_current_profile_unlocked<F>(profile_id: &str, modify: F) -> Result<KableProfile, String>
where
    F: FnOnce(&mut KableProfile),
{
    let mut profiles = load_profiles().await?;

    let profile = profiles
        .iter_mut()
        .find(|profile| profile.id == profile_id)
        .ok_or_else(|| format!("Profile with id {} not found", profile_id))?;

    modify(profile);

    let updated_profile = profile.clone();

    save_profiles_and_invalidate(&profiles).await?;

    Ok(updated_profile)
}

pub async fn save_profile(profile: KableProfile) -> Result<KableProfile, String> {
    fs::with_file_lock(crate::constants::KABLE_PROFILES_FILE, || async {
        let mut profiles = load_profiles().await?;

        if profiles.iter().any(|existing| existing.id == profile.id) {
            return Err(format!("Profile with id {} already exists", profile.id));
        }

        profiles.push(profile.clone());
        save_profiles_and_invalidate(&profiles).await?;

        Ok(profile)
    })
    .await
}

pub async fn get_profile(profile_id: &str) -> Result<KableProfile, String> {
    let profiles = load_profiles().await?;

    profiles
        .into_iter()
        .find(|profile| profile.id == profile_id)
        .ok_or_else(|| format!("Profile with id {} not found", profile_id))
}

async fn modify_profile_unlocked(old: KableProfile, new: KableProfile) -> Result<KableProfile, String> {
    if old.id != new.id {
        return Err(format!("Cannot update profile: old id {} does not match new id {}", old.id, new.id));
    }

    let mut profiles = load_profiles().await?;

    let position = profiles
        .iter()
        .position(|profile| profile.id == old.id)
        .ok_or_else(|| format!("Profile with id {} not found", old.id))?;

    profiles[position] = new.clone();

    save_profiles_and_invalidate(&profiles).await?;

    Ok(new)
}

pub async fn modify_profile(old: KableProfile, new: KableProfile) -> Result<KableProfile, String> {
    fs::with_file_lock(crate::constants::KABLE_PROFILES_FILE, || async move { modify_profile_unlocked(old, new).await }).await
}

pub async fn modify_current_profile<F>(profile_id: &str, modify: F) -> Result<KableProfile, String>
where
    F: FnOnce(&mut KableProfile),
{
    let profile_id = profile_id.to_owned();

    fs::with_file_lock(crate::constants::KABLE_PROFILES_FILE, || async move { modify_current_profile_unlocked(&profile_id, modify).await })
        .await
}

pub async fn delete_profile(profile_id: &str) -> Result<(), String> {
    let profile_id = profile_id.to_owned();

    fs::with_file_lock(crate::constants::KABLE_PROFILES_FILE, || async move {
        let mut profiles = load_profiles().await?;

        let position = profiles
            .iter()
            .position(|profile| profile.id == profile_id)
            .ok_or_else(|| format!("Profile with id {} not found", profile_id))?;

        profiles.remove(position);

        save_profiles_and_invalidate(&profiles).await
    })
    .await
}

pub async fn toggle_favorite(profile: KableProfile) -> Result<KableProfile, String> {
    modify_current_profile(&profile.id, |current_profile| {
        current_profile.metadata.favorite = !current_profile.metadata.favorite;
    })
    .await
}

pub async fn update_last_used(profile: KableProfile) -> Result<KableProfile, String> {
    modify_current_profile(&profile.id, |current_profile| {
        current_profile.metadata.last_used = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    })
    .await
}

/// Sets a project's enabled state to the requested value.
///
/// This operation is idempotent and is serialized with other profile mutations.
pub async fn set_project_enabled(profile_id: &str, project: KableProject, enabled: bool) -> Result<KableProfile, String> {
    let profile_id = profile_id.to_owned();

    modify_current_profile(&profile_id, move |current_profile| {
        let currently_enabled = current_profile.settings.is_enabled(project.clone());

        if currently_enabled != enabled {
            current_profile.settings.toggle(project.project.project_type, &project.filename);
        }
    })
    .await
}

/// Toggles a project's enabled state atomically against the current persisted profile.
///
/// Prefer `set_project_enabled` when the caller already knows the desired state.
pub async fn toggle_project(profile: KableProfile, project: KableProject) -> Result<KableProfile, String> {
    let profile_id = profile.id.clone();

    modify_current_profile(&profile_id, move |current_profile| {
        current_profile.settings.toggle(project.project.project_type, &project.filename);
    })
    .await
}

pub async fn is_project_enabled(profile: KableProfile, project: KableProject) -> Result<bool, String> {
    let current_profile = get_profile(&profile.id).await?;
    Ok(current_profile.settings.is_enabled(project))
}

pub async fn list_profiles(force: bool) -> Result<Vec<KableProfile>, String> {
    if force {
        invalidate_profiles_cache().await;
    }

    load_profiles().await
}

pub async fn increment_times_launched(profile: KableProfile) -> Result<KableProfile, String> {
    modify_current_profile(&profile.id, |current_profile| {
        current_profile.metadata.times_launched = current_profile.metadata.times_launched.saturating_add(1);
    })
    .await
}

pub async fn add_play_time(profile: KableProfile, duration_ms: u32) -> Result<KableProfile, String> {
    modify_current_profile(&profile.id, |current_profile| {
        current_profile.metadata.total_time_played_ms = current_profile.metadata.total_time_played_ms.saturating_add(duration_ms);
    })
    .await
}
