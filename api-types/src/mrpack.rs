use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MrpackIndex {
    pub format_version: u32,
    pub game: String,
    pub version_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub files: Vec<MrpackFile>,
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
pub struct MrpackFile {
    pub path: String,
    pub hashes: MrpackHashes,
    pub env: Option<MrpackEnvironment>,
    pub downloads: Vec<String>,
    pub file_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
pub struct MrpackHashes {
    pub sha1: String,
    pub sha512: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
pub struct MrpackEnvironment {
    pub client: MrpackEnvironmentType,
    pub server: MrpackEnvironmentType,
}

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum MrpackEnvironmentType {
    Required,
    Optional,
    Unsupported,
}
