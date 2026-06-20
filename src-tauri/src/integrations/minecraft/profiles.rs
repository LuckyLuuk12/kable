use crate::constants::LAUNCHER_PROFILES_FILE;
use crate::system::fs::{mc_dir, read_str};
use api_types::profiles::LauncherProfiles;

//?----------------------------------------------------------------------
//? Load and convert launcher_profiles.json into Profile structs
//?----------------------------------------------------------------------

/// Parse the .minecraft/launcher_profiles.json file into LauncherProfiles type:
pub async fn parse_launcher_profiles() -> Result<LauncherProfiles, String> {
    let launcher_profiles = mc_dir()?.join(LAUNCHER_PROFILES_FILE);
    let content = read_str(&launcher_profiles).await?;
    let launcher_profiles: LauncherProfiles =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse launcher profiles: {e}"))?;
    Ok(launcher_profiles)
}
