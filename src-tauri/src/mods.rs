use serde::{Deserialize, Serialize};

// Mod management structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModProject {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub client_side: Compatibility,
    pub server_side: Compatibility,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub source: ModSource,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Compatibility {
    Required,
    Optional,
    Unsupported,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModSource {
    Modrinth,
    CurseForge,
    Local,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModVersion {
    pub id: String,
    pub version_number: String,
    pub version_type: VersionType,
    pub minecraft_versions: Vec<String>,
    pub mod_loaders: Vec<String>,
    pub date_published: String,
    pub downloads: u64,
    pub changelog: Option<String>,
    pub files: Vec<ModFile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum VersionType {
    Release,
    Beta,
    Alpha,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModFile {
    pub url: String,
    pub filename: String,
    pub size: u64,
    pub sha1: String,
    pub primary: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledMod {
    pub id: String,
    pub name: String,
    pub version: String,
    pub source: ModSource,
    pub source_id: String,
    pub file_path: String,
    pub minecraft_version: String,
    pub mod_loader: ModLoader,
    pub enabled: bool,
    pub dependencies: Vec<String>,
    pub auto_update: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModLoader {
    Fabric,
    Forge,
    Quilt,
    NeoForge,
}

// TODO: Implement mod management functions
// - Search Modrinth and CurseForge APIs
// - Download and install mods
// - Manage mod versions and updates
// - Handle mod dependencies
// - Create and manage modpacks
// - Import/export modpack configurations
