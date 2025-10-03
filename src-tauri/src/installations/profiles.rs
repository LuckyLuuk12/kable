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

// We never need to write to official launcher_profiles.json, so this function is commented out.
// pub fn write_launcher_profiles(profiles: &[LauncherProfile]) -> Result<(), String> {
//     let mc_dir = crate::get_default_minecraft_dir()?;
//     let path = mc_dir.join("launcher_profiles.json");
//     let json = serde_json::to_string_pretty(profiles)
//         .map_err(|e| format!("Failed to serialize launcher profiles: {}", e))?;
//     fs::write(&path, json).map_err(|e| format!("Failed to write launcher_profiles.json: {}", e))
// }
