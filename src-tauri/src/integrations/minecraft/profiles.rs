use crate::constants::LAUNCHER_PROFILES_FILE;
use crate::system::fs::{mc_dir, read_str, write_str};
use crate::Logger;
use api_types::profiles::LauncherProfiles;

//?----------------------------------------------------------------------
//? Load and convert launcher_profiles.json into Profile structs
//?----------------------------------------------------------------------

/// Parse the .minecraft/launcher_profiles.json file into LauncherProfiles type:
pub async fn parse_launcher_profiles() -> Result<LauncherProfiles, String> {
    let launcher_profiles = mc_dir()?.join(LAUNCHER_PROFILES_FILE);
    let content = read_str(&launcher_profiles).await?;
    // if content is empty return LauncherProfiles::default() and write this default to the file, otherwise parse the content into LauncherProfiles
    if content.trim().is_empty() {
        let default_profiles = LauncherProfiles::default();
        let default_content =
            serde_json::to_string_pretty(&default_profiles).map_err(|e| format!("Failed to serialize default launcher profiles: {e}"))?;
        write_str(&launcher_profiles, default_content.as_str(), false)
            .await
            .map_err(|e| format!("Failed to write default launcher profiles: {e}"))?;
        return Ok(default_profiles);
    }
    let launcher_profiles: LauncherProfiles =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse launcher profiles: {e}\n\n{content}"))?;
    Logger::debug_global(format!("Parsed {} launcher profiles", launcher_profiles.profiles.len()).as_str(), None);
    Ok(launcher_profiles)
}
