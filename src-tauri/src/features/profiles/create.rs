use crate::integrations::minecraft::versions::get_version_data;
use crate::integrations::modrinth::client::download_project;

/*
 *  create profile functions:
 * - new profile from loader, version and some (optional) metadata (name, icon, description)
 * - new profile by import from .minecraft folder
 * - new profile by import from existing profile (not a copy but from exported profile folder with mods, resourcepacks, etc.)
 * - new profile with modpack(s) (mrpack) as basis
 */

pub async fn from_loader_version(
    loader: api_types::profiles::LoaderKind,
    version: String,
    name: Option<String>,
    icon: Option<String>,
    description: Option<String>,
) -> Result<api_types::profiles::KableProfile, String> {
    let version_data = get_version_data(loader, version.clone(), false).await?;

    let profile_name = name.unwrap_or_else(|| format!("{:?}-{}", loader, version));
    let profile_icon = icon; // TODO: Make a default icon for kable profiles based on the loader?
    let profile_description = description;

    Ok(api_types::profiles::KableProfile {
        id: uuid::Uuid::new_v4().to_string(),
        name: profile_name,
        icon: profile_icon,
        version: version_data,
        created: chrono::Utc::now().to_string(),
        last_used: chrono::Utc::now().to_string(),
        java_args: Vec::new(), // TODO : add default java args just like the official launcher does
        dedicated_mods_folder: Some(format!("{}/{}", crate::constants::MODS_DIR, uuid::Uuid::new_v4())),
        dedicated_config_folder: Some(format!("{}/{}", crate::constants::CONFIG_DIR, uuid::Uuid::new_v4())),
        dedicated_resource_pack_folder: Some(format!("{}/{}", crate::constants::RESOURCEPACKS_DIR, uuid::Uuid::new_v4())),
        dedicated_shaders_folder: Some(format!("{}/{}", crate::constants::SHADERPACKS_DIR, uuid::Uuid::new_v4())),
        favorite: false,
        total_time_played_ms: 0,
        parameters_map: std::collections::HashMap::new(),
        description: profile_description,
        times_launched: 0,
        enable_pack_merging: false,
        pack_order: Vec::new(),
        merged_packs: Vec::new(),
    })
}
