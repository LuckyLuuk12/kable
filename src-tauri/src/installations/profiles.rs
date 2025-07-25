use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherProfile {
    pub created: String,
    pub icon: Option<String>,
    pub java_args: Option<String>,
    pub last_used: String,
    pub last_version_id: String,
    pub name: String,
    pub profile_type: String,
}

pub fn read_launcher_profiles() -> Result<Vec<LauncherProfile>, String> {
    let mc_dir = crate::get_default_minecraft_dir()?;
    let path = mc_dir.join("launcher_profiles.json");
    let data = fs::read_to_string(&path).map_err(|e| format!("Failed to read launcher_profiles.json: {}", e))?;
    serde_json::from_str::<Vec<LauncherProfile>>(&data)
        .map_err(|e| format!("Failed to parse launcher_profiles.json: {}", e))
}

// We never need to write to official launcher_profiles.json, so this function is commented out.
// pub fn write_launcher_profiles(profiles: &[LauncherProfile]) -> Result<(), String> {
//     let mc_dir = crate::get_default_minecraft_dir()?;
//     let path = mc_dir.join("launcher_profiles.json");
//     let json = serde_json::to_string_pretty(profiles)
//         .map_err(|e| format!("Failed to serialize launcher profiles: {}", e))?;
//     fs::write(&path, json).map_err(|e| format!("Failed to write launcher_profiles.json: {}", e))
// }