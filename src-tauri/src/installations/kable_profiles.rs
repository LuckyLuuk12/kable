use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::profiles::LauncherProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KableInstallation {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub version_id: String,
    pub created: String,
    pub last_used: String,
    pub java_args: Vec<String>,
    pub dedicated_resource_pack_folder: Option<String>,
    pub dedicated_shaders_folder: Option<String>,
    pub favorite: bool,
    pub total_time_played_ms: u64,
    /// User-overridable parameters for advanced use cases
    pub parameters_map: std::collections::HashMap<String, String>,
    /// Optional user-provided description or notes
    pub description: Option<String>,
    /// Number of times this installation has been launched
    pub times_launched: u32,
}

impl Default for KableInstallation {
    fn default() -> Self {
        KableInstallation {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            icon: None,
            version_id: String::new(),
            created: chrono::Utc::now().to_rfc3339(),
            last_used: chrono::Utc::now().to_rfc3339(),
            java_args: vec!["-Xmx2048M".to_string()],
            dedicated_resource_pack_folder: None,
            dedicated_shaders_folder: None,
            favorite: false,
            total_time_played_ms: 0,
            parameters_map: std::collections::HashMap::new(),
            description: None,
            times_launched: 0,
        }
    }
}

impl From<LauncherProfile> for KableInstallation {
    fn from(profile: LauncherProfile) -> Self {
        KableInstallation {
            id: uuid::Uuid::new_v4().to_string(),
            name: profile.name,
            icon: profile.icon,
            version_id: profile.last_version_id,
            created: profile.created,
            last_used: profile.last_used,
            java_args: profile.java_args
                .unwrap_or_default()
                .split_whitespace()
                .map(String::from)
                .collect(),
            dedicated_resource_pack_folder: None,
            dedicated_shaders_folder: None,
            favorite: false,
            total_time_played_ms: 0,
            parameters_map: std::collections::HashMap::new(),
            description: None,
            times_launched: 0,
        }
    }
}

pub fn read_kable_profiles() -> Result<Vec<KableInstallation>, String> {
    let kable_dir = crate::get_minecraft_kable_dir()?;
    let path = kable_dir.join("kable_profiles.json");
    let data = fs::read_to_string(&path).map_err(|e| format!("Failed to read kable_profiles.json: {}", e))?;
    serde_json::from_str::<Vec<KableInstallation>>(&data)
        .map_err(|e| format!("Failed to parse kable_profiles.json: {}", e))
}

pub fn write_kable_profiles(profiles: &[KableInstallation]) -> Result<(), String> {
    let kable_dir = crate::get_minecraft_kable_dir()?;
    let path = kable_dir.join("kable_profiles.json");
    let json = serde_json::to_string_pretty(profiles)
        .map_err(|e| format!("Failed to serialize kable profiles: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write kable_profiles.json: {}", e))
}