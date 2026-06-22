use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet)]
pub struct KableProfile {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub version: ProfileVersion,
    pub created: String,
    pub last_used: String,
    pub java_args: Vec<String>,
    pub dedicated_mods_folder: Option<String>,
    pub dedicated_resource_pack_folder: Option<String>,
    pub dedicated_shaders_folder: Option<String>,
    pub dedicated_config_folder: Option<String>,
    pub favorite: bool,
    pub total_time_played_ms: u64,
    pub parameters_map: HashMap<String, String>,
    pub description: Option<String>,
    pub times_launched: u32,
    #[serde(default)]
    pub enable_pack_merging: bool,
    #[serde(default)]
    pub pack_order: Vec<String>,
    #[serde(default)]
    pub merged_packs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet)]
pub struct ProfileVersion {
    /// Raw version ID from the profile, e.g. "1.19.2-forge-43.2.0"
    pub id: String,
    pub display_name: String,
    /// Vanilla, Fabric, Forge, NeoForge, Quilt, etc.
    pub loader: LoaderKind,
    /// Optional Minecraft version, e.g. "1.19.2", note that since 2026 minecraft versioning is <year>.<drop>.<patch> (e.g. 26.2.1)
    pub minecraft_version: Option<String>,
    /// Optional loader version, e.g. "43.2.0" for forge or "0.14.19" for fabric
    pub loader_version: Option<String>,
    /// Release, Snapshot, OldBeta, OldAlpha
    pub version_type: Option<VersionType>,
    /// Whether this version is marked as stable in the profile, note that this is not necessarily the same as version_type == Release
    pub stable: Option<bool>,
    /// Extra metadata that may be present from the version manifest
    pub release_time: Option<String>,
    pub updated_time: Option<String>,
    pub url: Option<String>,
    pub sha1: Option<String>,
    pub compliance_level: Option<u32>,
    pub recommended: Option<bool>,
}
#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet)]
pub struct Versions(pub Vec<ProfileVersion>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum LoaderKind {
    Vanilla,
    Fabric,
    IrisFabric,
    Forge,
    NeoForge,
    Quilt,
}

impl std::fmt::Display for LoaderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderKind::Vanilla => write!(f, "vanilla"),
            LoaderKind::Fabric => write!(f, "fabric"),
            LoaderKind::IrisFabric => write!(f, "iris-fabric"),
            LoaderKind::Forge => write!(f, "forge"),
            LoaderKind::NeoForge => write!(f, "neoforge"),
            LoaderKind::Quilt => write!(f, "quilt"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum VersionType {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, facet::Facet)]
pub struct ModJarInfo {
    pub file_name: String,
    pub mod_name: Option<String>,
    pub mod_version: Option<String>,
    pub loader: Option<String>,
    pub disabled: bool,
}

//?----------------------------------------------------------------------
//? .minecraft launcher_profiles.json types
//?----------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct LauncherProfiles {
    // Profile name is key to profile object
    pub profiles: HashMap<String, Profile>,
    pub settings: OfficialLauncherSettings,
    pub version: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: Option<String>,
    pub created: Option<String>,
    pub icon: Option<String>,
    #[serde(rename = "javaArgs")]
    pub java_args: Option<String>,
    #[serde(rename = "lastUsed")]
    pub last_used: Option<String>,
    #[serde(rename = "lastVersionId")]
    pub last_version_id: Option<String>,
    #[serde(rename = "type")]
    pub profile_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OfficialLauncherSettings {
    #[serde(rename = "crashAssistance")]
    pub crash_assistance: bool,
    #[serde(rename = "enableAdvanced")]
    pub enable_advanced: bool,
    #[serde(rename = "enableAnalytics")]
    pub enable_analytics: bool,
    #[serde(rename = "enableHistorical")]
    pub enable_historical: bool,
    #[serde(rename = "enableReleases")]
    pub enable_releases: bool,
    #[serde(rename = "enableSnapshots")]
    pub enable_snapshots: bool,
    #[serde(rename = "keepLauncherOpen")]
    pub keep_launcher_open: bool,
    #[serde(rename = "profileSorting")]
    pub profile_sorting: String,
    #[serde(rename = "showGameLog")]
    pub show_game_log: bool,
    #[serde(rename = "showMenu")]
    pub show_menu: bool,
    #[serde(rename = "soundOn")]
    pub sound_on: bool,
}

//?----------------------------------------------------------------------
//? impl blocks
//?----------------------------------------------------------------------

impl std::fmt::Display for VersionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionType::Release => write!(f, "release"),
            VersionType::Snapshot => write!(f, "snapshot"),
            VersionType::OldBeta => write!(f, "old_beta"),
            VersionType::OldAlpha => write!(f, "old_alpha"),
        }
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, facet::Facet)]
// pub struct VersionData {
//     pub version_id: String,
//     pub loader: LoaderKind,
//     pub display_name: String,
//     pub is_stable: bool,
//     pub extra: Option<HashMap<String, String>>,
// }

// #[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, facet::Facet)]
// pub struct Versions(pub Vec<VersionData>);

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, facet::Facet)]
// pub struct ResourcePackInfo {
//     pub file_name: String,
//     pub name: Option<String>,
//     pub description: Option<String>,
//     pub disabled: bool,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, facet::Facet)]
// pub struct ShaderPackInfo {
//     pub file_name: String,
//     pub name: Option<String>,
//     pub description: Option<String>,
//     pub disabled: bool,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, facet::Facet)]
// pub struct PackFileDetailedGroup {
//     pub disabled: Vec<PackFileInfo>,
//     pub optional: Vec<PackFileInfo>,
//     pub to_be_installed: Vec<PackFileInfo>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, facet::Facet)]
// pub struct MrPackDetailed {
//     pub mods: PackFileDetailedGroup,
//     pub resourcepacks: PackFileDetailedGroup,
//     pub shaderpacks: PackFileDetailedGroup,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, facet::Facet)]
// pub struct PackFileInfo {
//     pub path: String,
//     pub file_size: u64,
//     pub hashes: std::collections::HashMap<String, String>,
//     pub downloads: Vec<String>,
//     pub env: Option<MrpackEnv>,
//     pub already_installed: bool,
//     pub overwrite: bool,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, facet::Facet)]
// pub struct PackFileGroups {
//     pub mods: Vec<PackFileInfo>,
//     pub resourcepacks: Vec<PackFileInfo>,
//     pub shaderpacks: Vec<PackFileInfo>,
//     pub others: Vec<PackFileInfo>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, facet::Facet)]
// pub struct MrpackIndex {
//     pub name: String,
//     pub version_id: String,
//     pub format_version: u32,
//     pub files: Vec<MrpackFile>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, facet::Facet)]
// #[facet(rename_all = "snake_case")]
// #[serde(rename_all = "snake_case")]
// pub struct MrpackFile {
//     pub path: String,
//     pub file_size: u64,
//     pub hashes: std::collections::HashMap<String, String>,
//     pub downloads: Vec<String>,
//     pub env: Option<MrpackEnv>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, facet::Facet)]
// #[facet(rename_all = "snake_case")]
// #[serde(rename_all = "snake_case")]
// pub struct MrpackEnv {
//     pub client: Option<String>,
//     pub server: Option<String>,
// }
