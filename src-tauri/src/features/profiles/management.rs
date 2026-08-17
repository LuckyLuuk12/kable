// The simple profile management (rename/delete/list)

use crate::features::profiles::kable_profile::{load_profiles, save_profiles};
use api_types::profiles::KableProfile;
use api_types::projects::KableProject;

pub async fn get_profile(profile_id: &str) -> Result<KableProfile, String> {
    let profiles = load_profiles().await?;
    profiles.into_iter().find(|p| p.id == profile_id).ok_or_else(|| format!("Profile with id {} not found", profile_id))
}

pub async fn modify_profile(old: KableProfile, new: KableProfile) -> Result<KableProfile, String> {
    let mut profiles = load_profiles().await?;
    // check if both have same id, otherwise error:
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

pub async fn list_profiles() -> Result<Vec<KableProfile>, String> {
    load_profiles().await
}

/// Modifies the profile.settings.<project.type> with its toggle(str) function, then uses the modify_profile function to save the updated profile to disk. This is used to enable/disable a project for a profile.
pub async fn toggle_project(profile: KableProfile, project: KableProject) -> Result<KableProfile, String> {
    let mut updated_profile = profile.clone();
    let toggled = updated_profile.settings.toggle(project.project.project_type, &project.filename);
    // if we toggled from disabled to enabled, we might have to download the project again as removing projects is possible from global projects folder if no profiles have it enabled:
    if !toggled {
        crate::features::projects::management::add_project(project.project, Some(&project.version_id)).await?;
    }
    modify_profile(profile, updated_profile).await
}

pub async fn is_project_enabled(profile: KableProfile, project: KableProject) -> Result<bool, String> {
    let updated_profile = profile.clone();
    Ok(updated_profile.settings.is_enabled(project))
}
