use api_types::profiles::{KableProfile, LauncherProfiles};

//?----------------------------------------------------------------------
//? Load and convert launcher_profiles.json into Profile structs
//?----------------------------------------------------------------------

/// Parse the .minecraft/launcher_profiles.json file into LauncherProfiles type:
async fn parse_launcher_profiles() -> Result<LauncherProfiles, String> {
    let launcher_profiles = get_default_minecraft_dir()?.join(LAUNCHER_PROFILES_FILE);
    let content = read_to_string(&launcher_profiles).await?;
    let launcher_profiles: LauncherProfiles =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse launcher profiles: {e}"))?;
    Ok(launcher_profiles)
}
