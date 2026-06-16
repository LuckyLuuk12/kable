// The simple profile management (rename/delete/list)

use crate::features::profiles::kable_profile::{load_profiles, save_profiles};
use api_types::profiles::KableProfile;

pub async fn update_profile(old: KableProfile, new: KableProfile) -> Result<KableProfile, String> {
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

pub async fn list_profiles() -> Result<Vec<KableProfile>, String> {
    load_profiles().await
}
