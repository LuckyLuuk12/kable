use serde::{Deserialize, Serialize};
use std::fs;
use tokio::fs as async_fs;
use tokio::task;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherProfile {
    pub created: Option<String>,
    pub game_dir: Option<String>,
    pub icon: Option<String>,
    pub java_args: Option<String>,
    pub last_used: Option<String>,
    pub last_version_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub profile_type: String,
}

pub fn read_launcher_profiles() -> Result<Vec<LauncherProfile>, String> {
    // Synchronous version for compatibility
    let mc_dir = crate::get_default_minecraft_dir()?;
    let path = mc_dir.join("launcher_profiles.json");
    let data = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read launcher_profiles.json: {}", e))?;
    parse_launcher_profiles_json(&data)
}

fn parse_launcher_profiles_json(data: &str) -> Result<Vec<LauncherProfile>, String> {
    let json: serde_json::Value = serde_json::from_str(data)
        .map_err(|e| format!("Failed to parse launcher_profiles.json: {}", e))?;
    let profiles = json
        .get("profiles")
        .and_then(|p| p.as_object())
        .ok_or("No 'profiles' object found in launcher_profiles.json")?;
    let mut result = Vec::new();
    for profile_value in profiles.values() {
        let profile: LauncherProfile = serde_json::from_value(profile_value.clone())
            .map_err(|e| format!("Failed to parse a profile: {}", e))?;
        result.push(profile);
    }
    Ok(result)
}

pub async fn read_launcher_profiles_async() -> Result<Vec<LauncherProfile>, String> {
    let mc_dir = crate::get_default_minecraft_dir()?;
    let path = mc_dir.join("launcher_profiles.json");
    let data = async_fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read launcher_profiles.json: {}", e))?;
    // Parse JSON in a blocking thread
    task::spawn_blocking(move || parse_launcher_profiles_json(&data))
        .await
        .unwrap()
}

/// Removes a profile from launcher_profiles.json that matches the given criteria
/// This is used during deletion to prevent re-importing official launcher profiles
pub async fn remove_launcher_profile_by_match(
    name: &str,
    version_id: &str,
    created: &str,
) -> Result<(), String> {
    let mc_dir = crate::get_default_minecraft_dir()?;
    let path = mc_dir.join("launcher_profiles.json");

    // If file doesn't exist, nothing to remove
    if !path.exists() {
        return Ok(());
    }

    // Read the current file
    let data = async_fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read launcher_profiles.json: {}", e))?;

    let mut json: serde_json::Value = task::spawn_blocking(move || {
        serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse launcher_profiles.json: {}", e))
    })
    .await
    .unwrap()?;

    // Get the profiles object
    let profiles = json
        .get_mut("profiles")
        .and_then(|p| p.as_object_mut())
        .ok_or("No 'profiles' object found in launcher_profiles.json")?;

    // Find and remove matching profile(s)
    let mut found_key: Option<String> = None;
    for (key, profile_value) in profiles.iter() {
        if let Ok(profile) = serde_json::from_value::<LauncherProfile>(profile_value.clone()) {
            let profile_created = profile
                .created
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
            if profile.name == name
                && profile.last_version_id == version_id
                && profile_created == created
            {
                found_key = Some(key.clone());
                break;
            }
        }
    }

    // Remove the profile if found
    if let Some(key) = found_key {
        profiles.remove(&key);

        // Write back the modified JSON
        let json_owned = json.clone();
        let json_str = task::spawn_blocking(move || {
            serde_json::to_string_pretty(&json_owned)
                .map_err(|e| format!("Failed to serialize launcher profiles: {}", e))
        })
        .await
        .unwrap()?;

        // Use atomic write to avoid corruption
        crate::write_file_atomic_async(&path, json_str.as_bytes())
            .await
            .map_err(|e| format!("Failed to write launcher_profiles.json: {}", e))?;

        crate::logging::Logger::debug_global(
            &format!("Removed profile '{}' from launcher_profiles.json", name),
            None,
        );
    }

    Ok(())
}
