use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::projects::{KableProject, ProjectType};
/// Represents a Kable Minecraft Profile, which contains more information than a standard Minecraft Profile:
/// ```rs
/// pub struct KableProfile {
///     pub id: String,
///     pub name: String,
///     pub icon: Option<String>,
///     pub version: ProfileVersion,
///     pub created: String,
///     pub last_used: String,
///     pub java_args: Vec<String>,
///     pub dedicated_mods_folder: Option<String>,
///     pub dedicated_resource_pack_folder: Option<String>,
///     pub dedicated_shaders_folder: Option<String>,
///     pub dedicated_config_folder: Option<String>,
///     pub favorite: bool,
///     pub total_time_played_ms: u32,
///     pub parameters_map: HashMap<String, String>,
///     pub description: Option<String>,
///     pub times_launched: u32,
///     #[serde(default)]
///     pub enable_pack_merging: bool,
///     #[serde(default)]
///     pub pack_order: Vec<String>,
///     #[serde(default)]
///     pub merged_packs: Vec<String>,
/// }
/// ```
// #[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
// pub struct KableProfile {
//     pub id: String,
//     pub name: String,
//     pub icon: Option<String>,
//     pub version: ProfileVersion,
//     pub created: String,
//     pub last_used: String,
//     pub java_args: Vec<String>,
//     pub dedicated_mods_folder: Option<String>,
//     pub dedicated_resource_pack_folder: Option<String>,
//     pub dedicated_shaders_folder: Option<String>,
//     pub dedicated_config_folder: Option<String>,
//     pub favorite: bool,
//     pub total_time_played_ms: u32,
//     pub parameters_map: HashMap<String, String>,
//     pub description: Option<String>,
//     pub times_launched: u32,
//     #[serde(default)]
//     pub enable_pack_merging: bool,
//     #[serde(default)]
//     pub pack_order: Vec<String>,
//     #[serde(default)]
//     pub merged_packs: Vec<String>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
// pub struct ProfileVersion {
//     /// Raw version ID from the profile, e.g. "1.19.2-forge-43.2.0"
//     pub id: String,
//     pub display_name: String,
//     /// Vanilla, Fabric, Forge, NeoForge, Quilt, etc.
//     pub loader: LoaderKind,
//     /// Optional Minecraft version, e.g. "1.19.2", note that since 2026 minecraft versioning is <year>.<drop>.<patch> (e.g. 26.2.1)
//     pub minecraft_version: Option<String>,
//     /// Optional loader version, e.g. "43.2.0" for forge or "0.14.19" for fabric
//     pub loader_version: Option<String>,
//     /// Release, Snapshot, OldBeta, OldAlpha
//     pub version_type: Option<ProfileVersionType>,
//     /// Whether this version is marked as stable in the profile, note that this is not necessarily the same as version_type == Release
//     pub stable: Option<bool>,
//     /// Extra metadata that may be present from the version manifest
//     pub release_time: Option<String>,
//     pub updated_time: Option<String>,
//     pub url: Option<String>,
//     pub sha1: Option<String>,
//     pub compliance_level: Option<u32>,
//     pub recommended: Option<bool>,
// }
// #[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
// pub struct Versions(pub Vec<ProfileVersion>);

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy, Hash, facet::Facet, specta::Type)]
// #[facet(rename_all = "snake_case")]
// #[serde(rename_all = "snake_case")]
// #[repr(u8)]
// pub enum LoaderKind {
//     Vanilla,
//     Fabric,
//     IrisFabric,
//     Forge,
//     NeoForge,
//     Quilt,
// }

// impl std::fmt::Display for LoaderKind {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             LoaderKind::Vanilla => write!(f, "vanilla"),
//             LoaderKind::Fabric => write!(f, "fabric"),
//             LoaderKind::IrisFabric => write!(f, "iris-fabric"),
//             LoaderKind::Forge => write!(f, "forge"),
//             LoaderKind::NeoForge => write!(f, "neoforge"),
//             LoaderKind::Quilt => write!(f, "quilt"),
//         }
//     }
// }

// #[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
// #[facet(rename_all = "snake_case")]
// #[serde(rename_all = "snake_case")]
// #[repr(u8)]
// pub enum ProfileVersionType {
//     Release,
//     Snapshot,
//     OldBeta,
//     OldAlpha,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, facet::Facet, specta::Type)]
// pub struct ModJarInfo {
//     pub file_name: String,
//     pub mod_name: Option<String>,
//     pub mod_version: Option<String>,
//     pub loader: Option<String>,
//     pub disabled: bool,
// }

//?----------------------------------------------------------------------
//? Kable Profile types (refactored)
//?----------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
pub struct KableProfile {
    pub id: String,
    pub version: ProfileVersion,
    pub metadata: KableProfileMetadata,
    pub settings: KableProfileSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
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
    pub version_type: Option<ProfileVersionType>,
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
#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
pub struct Versions(pub Vec<ProfileVersion>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy, Hash, facet::Facet, specta::Type)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum ProfileVersionType {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, facet::Facet, specta::Type)]
pub struct KableProfileMetadata {
    pub name: String,
    pub icon: Option<String>,
    pub created: String,
    pub last_used: String,
    pub favorite: bool,
    pub total_time_played_ms: u32,
    pub description: Option<String>,
    pub times_launched: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, facet::Facet, specta::Type)]
pub struct KableProfileSettings {
    pub parameters_map: HashMap<String, String>,
    pub java_args: Vec<String>,
    #[serde(default)]
    pub enable_pack_merging: bool,
    /// Lists the packs, by filename relative to the resource pack folder, in the order they are "stacked" when merging, if enabled.
    #[serde(default)]
    pub pack_order: Vec<String>,
    /// Lists the packs, by filename relative to the resource pack folder, which should be merged into a single pack before we launch the game
    #[serde(default)]
    pub merged_packs: Vec<String>,
    /// lists of enabled/disabled content/projects, the ToggleableContent lists contain filenames relative to the respective folders, e.g. "mods/forge-1.19.2-43.2.0.jar" or "resourcepacks/faithful-1.19.zip"
    pub mods: Projects,
    pub resourcepacks: Projects,
    pub shaders: Projects,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq, facet::Facet, specta::Type)]
pub struct Projects {
    pub enabled: HashSet<String>,
    pub disabled: HashSet<String>,
}

impl Projects {
    /// Enables a project, returns true if the project was previously disabled, false if it was already enabled
    pub fn enable(&mut self, id: String) -> bool {
        self.disabled.retain(|x| x != &id);

        if !self.enabled.contains(&id) {
            self.enabled.insert(id);
            return true;
        }
        false
    }
    /// Disables a project, returns true if the project was previously enabled, false if it was already disabled
    pub fn disable(&mut self, id: String) -> bool {
        self.enabled.retain(|x| x != &id);

        if !self.disabled.contains(&id) {
            self.disabled.insert(id);
            return true;
        }
        false
    }
    /// Toggles a project between enabled and disabled, returns true if the project is now enabled, false if it is now disabled
    pub fn toggle(&mut self, id: String) -> bool {
        if self.enabled.contains(&id) {
            self.disable(id)
        } else {
            self.enable(id)
        }
    }
}

impl KableProfileSettings {
    pub fn by_project_type(&self, project_type: ProjectType, enabled: bool) -> HashSet<String> {
        match project_type {
            ProjectType::Mod => {
                if enabled {
                    self.mods.enabled.clone()
                } else {
                    self.mods.disabled.clone()
                }
            }
            ProjectType::Resourcepack => {
                if enabled {
                    self.resourcepacks.enabled.clone()
                } else {
                    self.resourcepacks.disabled.clone()
                }
            }
            ProjectType::Shader => {
                if enabled {
                    self.shaders.enabled.clone()
                } else {
                    self.shaders.disabled.clone()
                }
            }
            _ => HashSet::new(),
        }
    }
    /// Toggles a project between enabled and disabled, returns true if the project is now enabled, false if it is now disabled
    pub fn toggle(&mut self, project_type: ProjectType, id: &str) -> bool {
        match project_type {
            ProjectType::Mod => self.mods.toggle(id.to_string()),
            ProjectType::Resourcepack => self.resourcepacks.toggle(id.to_string()),
            ProjectType::Shader => self.shaders.toggle(id.to_string()),
            _ => false,
        }
    }
    pub fn from(&mut self, project_type: ProjectType) -> Projects {
        match project_type {
            ProjectType::Mod => self.mods.clone(),
            ProjectType::Resourcepack => self.resourcepacks.clone(),
            ProjectType::Shader => self.shaders.clone(),
            _ => Projects::default(),
        }
    }
    pub fn set_projects(&mut self, project_type: ProjectType, projects: Projects) {
        match project_type {
            ProjectType::Mod => self.mods = projects,
            ProjectType::Resourcepack => self.resourcepacks = projects,
            ProjectType::Shader => self.shaders = projects,
            _ => {}
        }
    }
    pub fn is_enabled(&self, project: KableProject) -> bool {
        match project.project.project_type {
            ProjectType::Mod => self.mods.enabled.contains(&project.filename),
            ProjectType::Resourcepack => self.resourcepacks.enabled.contains(&project.filename),
            ProjectType::Shader => self.shaders.enabled.contains(&project.filename),
            _ => false,
        }
    }
}

//?----------------------------------------------------------------------
//? .minecraft launcher_profiles.json types
//?----------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, facet::Facet, specta::Type)]
pub struct LauncherProfiles {
    // Profile name is key to profile object
    pub profiles: HashMap<String, Profile>,
    pub settings: OfficialLauncherSettings,
    pub version: i32,
}

#[derive(Debug, Serialize, Deserialize, facet::Facet, specta::Type)]
pub struct Profile {
    pub created: Option<String>,
    pub icon: Option<String>,
    #[serde(rename = "javaArgs")]
    pub java_args: Option<String>,
    #[serde(rename = "lastUsed")]
    pub last_used: Option<String>,
    #[serde(rename = "lastVersionId")]
    pub last_version_id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub profile_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, facet::Facet, specta::Type)]
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

impl Default for LauncherProfiles {
    fn default() -> Self {
        Self {
            profiles: HashMap::new(),
            settings: OfficialLauncherSettings {
                crash_assistance: true,
                enable_advanced: false,
                enable_analytics: true,
                enable_historical: false,
                enable_releases: true,
                enable_snapshots: false,
                keep_launcher_open: false,
                profile_sorting: "byLastUsed".to_string(),
                show_game_log: false,
                show_menu: true,
                sound_on: true,
            },
            version: 1,
        }
    }
}

impl std::fmt::Display for ProfileVersionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileVersionType::Release => write!(f, "release"),
            ProfileVersionType::Snapshot => write!(f, "snapshot"),
            ProfileVersionType::OldBeta => write!(f, "old_beta"),
            ProfileVersionType::OldAlpha => write!(f, "old_alpha"),
        }
    }
}
