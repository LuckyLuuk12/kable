// The simple profile management (rename/delete/list)

use crate::features::profiles::kable_profile::{load_profiles, save_profiles};
use crate::system::cache::invalidate_no_args;
use crate::system::fs;
use api_types::profiles::KableProfile;
use api_types::projects::KableProject;

pub async fn save_profile(profile: KableProfile) -> Result<KableProfile, String> {
    let mut profiles = load_profiles().await?;
    if profiles.iter().any(|p| p.id == profile.id) {
        return Err(format!("Profile with id {} already exists", profile.id));
    }
    profiles.push(profile.clone());
    save_profiles(&profiles).await?;
    Ok(profile)
}

pub async fn get_profile(profile_id: &str) -> Result<KableProfile, String> {
    let profiles = load_profiles().await?;
    profiles.into_iter().find(|p| p.id == profile_id).ok_or_else(|| format!("Profile with id {} not found", profile_id))
}

async fn modify_profile_unlocked(old: KableProfile, new: KableProfile) -> Result<KableProfile, String> {
    let mut profiles = load_profiles().await?;

    if old.id != new.id {
        return Err(format!("Cannot update profile: old id {} does not match new id {}", old.id, new.id));
    }

    if let Some(pos) = profiles.iter().position(|p| p.id == old.id) {
        profiles[pos] = new.clone();
        save_profiles(&profiles).await?;
        Ok(new)
    } else {
        Err(format!("Profile with id {} not found", old.id))
    }
}

pub async fn modify_profile(old: KableProfile, new: KableProfile) -> Result<KableProfile, String> {
    fs::with_file_lock(crate::constants::KABLE_PROFILES_FILE, || async move { modify_profile_unlocked(old, new).await }).await
}

pub async fn modify_current_profile<F>(profile_id: &str, modify: F) -> Result<KableProfile, String>
where
    F: FnOnce(&mut KableProfile),
{
    fs::with_file_lock(crate::constants::KABLE_PROFILES_FILE, || async move {
        let mut profiles = load_profiles().await?;

        let profile = profiles
            .iter_mut()
            .find(|profile| profile.id == profile_id)
            .ok_or_else(|| format!("Profile with id {} not found", profile_id))?;

        modify(profile);

        let updated_profile = profile.clone();

        save_profiles(&profiles).await?;

        Ok(updated_profile)
    })
    .await
}

pub async fn delete_profile(profile_id: &str) -> Result<(), String> {
    let mut profiles = load_profiles().await?;
    if let Some(pos) = profiles.iter().position(|p| p.id == profile_id) {
        profiles.remove(pos);
        save_profiles(&profiles).await?;
        Ok(())
    } else {
        Err(format!("Profile with id {} not found", profile_id))
    }
}

pub async fn toggle_favorite(profile: KableProfile) -> Result<KableProfile, String> {
    let mut profiles = load_profiles().await?;
    if let Some(pos) = profiles.iter().position(|p| p.id == profile.id) {
        let mut updated_profile = profile.clone();
        updated_profile.metadata.favorite = !profile.metadata.favorite;
        profiles[pos] = updated_profile.clone();
        save_profiles(&profiles).await?;
        Ok(updated_profile)
    } else {
        Err(format!("Profile with id {} not found", profile.id))
    }
}

/// Ensures lastUsed in the profile is updated in ISO 8601 format (e.g. "2024-12-23T18:23:42.000Z") to the current time, and saves the updated profile list
pub async fn update_last_used(profile: KableProfile) -> Result<KableProfile, String> {
    let mut profiles = load_profiles().await?;
    if let Some(pos) = profiles.iter().position(|p| p.id == profile.id) {
        let mut updated_profile = profile.clone();
        updated_profile.metadata.last_used = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        profiles[pos] = updated_profile.clone();
        save_profiles(&profiles).await?;
        Ok(updated_profile)
    } else {
        Err(format!("Profile with id {} not found", profile.id))
    }
}

pub async fn list_profiles(force: bool) -> Result<Vec<KableProfile>, String> {
    if force {
        invalidate_no_args("profiles", "load_profiles").await?;
    }
    load_profiles().await
}

/// Modifies the profile.settings.<project.type> with its toggle(str) function, then uses the modify_profile function to save the updated profile to disk. This is used to enable/disable a project for a profile.
pub async fn toggle_project(profile: KableProfile, project: KableProject) -> Result<KableProfile, String> {
    let profile_id = profile.id.clone();
    let project_type = project.project.project_type;
    let filename = project.filename.clone();
    let project_data = project.project.clone();
    let version_id = project.version_id.clone();

    let updated_profile = modify_current_profile(&profile_id, |current_profile| {
        current_profile.settings.toggle(project_type, &filename);
    })
    .await?;

    // If the project was enabled, it may need to be downloaded again
    // if its files were removed while no profiles had it enabled.
    if updated_profile.settings.is_enabled(project) {
        crate::features::projects::management::add_project(project_data, Some(&version_id)).await?;
    }

    Ok(updated_profile)
}

pub async fn is_project_enabled(profile: KableProfile, project: KableProject) -> Result<bool, String> {
    let current_profile = get_profile(&profile.id).await?;
    Ok(current_profile.settings.is_enabled(project))
}
