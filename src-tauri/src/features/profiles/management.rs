// The simple profile management (rename/delete/list)

use crate::features::profiles::kable_profile::{load_profiles, save_profiles};
use crate::system::fs::rename;
use api_types::profiles::KableProfile;
use std::path::Path;

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

pub async fn list_profiles() -> Result<Vec<KableProfile>, String> {
    load_profiles().await
}

pub async fn change_id(profile: KableProfile, new_id: &str, update_folders: bool) -> Result<KableProfile, String> {
    let mut profiles = load_profiles().await?;
    if let Some(pos) = profiles.iter().position(|p| p.id == profile.id) {
        let mut updated_profile = profile.clone();
        updated_profile.id = new_id.to_string();
        if update_folders {
            // First rename the old folders if they exist, then update the profile to point to the new folders
            updated_profile.dedicated_mods_folder =
                modify_dedicated_folder(updated_profile.dedicated_mods_folder, &format!("{}/{}", crate::constants::MODS_DIR, new_id))
                    .await?;
            updated_profile.dedicated_config_folder =
                modify_dedicated_folder(updated_profile.dedicated_config_folder, &format!("{}/{}", crate::constants::CONFIG_DIR, new_id))
                    .await?;
            updated_profile.dedicated_resource_pack_folder = modify_dedicated_folder(
                updated_profile.dedicated_resource_pack_folder,
                &format!("{}/{}", crate::constants::RESOURCEPACKS_DIR, new_id),
            )
            .await?;
            updated_profile.dedicated_shaders_folder = modify_dedicated_folder(
                updated_profile.dedicated_shaders_folder,
                &format!("{}/{}", crate::constants::SHADERPACKS_DIR, new_id),
            )
            .await?;
        }

        profiles[pos] = updated_profile.clone();
        save_profiles(&profiles).await?;
        Ok(updated_profile)
    } else {
        Err(format!("Profile with id {} not found", profile.id))
    }
}

async fn modify_dedicated_folder(folder: Option<String>, new_folder: &str) -> Result<Option<String>, String> {
    if let Some(old_folder) = folder {
        rename(&Path::new(&old_folder), &Path::new(new_folder))
            .await
            .map_err(|e| format!("Failed to rename folder from {} to {}: {}", old_folder, new_folder, e))?;
        Ok(Some(new_folder.to_string()))
    } else {
        Ok(None)
    }
}
