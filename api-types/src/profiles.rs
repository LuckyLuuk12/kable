use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KableProfile {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub version_id: String,
    pub created: String,
    pub last_used: String,
    pub java_args: Vec<String>,
    pub dedicated_mods_folder: Option<String>,
    pub dedicated_resource_pack_folder: Option<String>,
    pub dedicated_shaders_folder: Option<String>,
    pub dedicated_config_folder: Option<String>,
    pub favorite: bool,
    pub total_time_played_ms: u64,
    pub parameters_map: std::collections::HashMap<String, String>,
    pub description: Option<String>,
    pub times_launched: u32,
    #[serde(default)]
    pub enable_pack_merging: bool,
    #[serde(default)]
    pub pack_order: Vec<String>,
    #[serde(default)]
    pub merged_packs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy, Hash)]
pub enum LoaderKind {
    Vanilla,
    Fabric,
    IrisFabric,
    Forge,
    NeoForge,
    Quilt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VersionData {
    pub version_id: String,
    pub loader: LoaderKind,
    pub display_name: String,
    pub is_stable: bool,
    pub extra: serde_json::Value,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Versions(pub Vec<VersionData>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModJarInfo {
    pub file_name: String,
    pub mod_name: Option<String>,
    pub mod_version: Option<String>,
    pub loader: Option<String>,
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourcePackInfo {
    pub file_name: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShaderPackInfo {
    pub file_name: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackFileDetailedGroup {
    pub disabled: Vec<PackFileInfo>,
    pub optional: Vec<PackFileInfo>,
    pub to_be_installed: Vec<PackFileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MrPackDetailed {
    pub mods: PackFileDetailedGroup,
    pub resourcepacks: PackFileDetailedGroup,
    pub shaderpacks: PackFileDetailedGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackFileInfo {
    pub path: String,
    pub file_size: u64,
    pub hashes: std::collections::HashMap<String, String>,
    pub downloads: Vec<String>,
    pub env: Option<MrpackEnv>,
    pub already_installed: bool,
    pub overwrite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackFileGroups {
    pub mods: Vec<PackFileInfo>,
    pub resourcepacks: Vec<PackFileInfo>,
    pub shaderpacks: Vec<PackFileInfo>,
    pub others: Vec<PackFileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MrpackIndex {
    pub name: String,
    pub version_id: String,
    pub format_version: u32,
    pub files: Vec<MrpackFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MrpackFile {
    pub path: String,
    pub file_size: u64,
    pub hashes: std::collections::HashMap<String, String>,
    pub downloads: Vec<String>,
    pub env: Option<MrpackEnv>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MrpackEnv {
    pub client: Option<String>,
    pub server: Option<String>,
}
