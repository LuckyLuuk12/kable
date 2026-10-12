// use std::collections::HashMap;

// use serde::{Deserialize, Serialize};

// #[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
// #[serde(rename_all = "camelCase")]
// pub struct MrpackIndex {
//     pub format_version: u32,
//     pub game: String,
//     pub version_id: String,
//     pub name: String,
//     pub summary: Option<String>,
//     pub files: Vec<MrpackFile>,
//     pub dependencies: HashMap<String, String>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
// pub struct MrpackFile {
//     pub path: String,
//     pub hashes: MrpackHashes,
//     pub env: Option<MrpackEnvironment>,
//     pub downloads: Vec<String>,
//     pub file_size: u64,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
// pub struct MrpackHashes {
//     pub sha1: String,
//     pub sha512: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
// pub struct MrpackEnvironment {
//     pub client: MrpackEnvironmentType,
//     pub server: MrpackEnvironmentType,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
// #[serde(rename_all = "lowercase")]
// #[repr(u8)]
// pub enum MrpackEnvironmentType {
//     Required,
//     Optional,
//     Unsupported,
// }
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The `modrinth.index.json` manifest stored at the root of an `.mrpack` archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackManifest {
    /// Currently supported format version is 1.
    pub format_version: u32,

    /// The game this modpack targets. Currently, Modrinth supports "minecraft".
    pub game: String,

    /// Unique identifier for this specific modpack version.
    pub version_id: String,

    /// Human-readable modpack name.
    pub name: String,

    /// Optional short description of the modpack.
    #[serde(default)]
    pub summary: Option<String>,

    /// Files to download into the instance directory.
    #[serde(default)]
    pub files: Vec<ModpackFile>,

    /// Required Minecraft and mod-loader versions.
    pub dependencies: HashMap<String, String>,
}

/// A downloadable file listed in `modrinth.index.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackFile {
    /// Destination path relative to the instance directory.
    pub path: String,

    /// Content hashes keyed by algorithm, e.g. "sha1" and "sha512".
    ///
    /// The specification requires both SHA-1 and SHA-512 hashes.
    pub hashes: HashMap<String, String>,

    /// Client/server compatibility. Absent in older or non-conforming manifests.
    #[serde(default)]
    pub env: Option<ModpackFileEnvironment>,

    /// Candidate download URLs, in preference order.
    pub downloads: Vec<String>,

    /// Expected file size in bytes, when provided.
    #[serde(default)]
    pub file_size: Option<u64>,
}

/// Indicates whether a file is required, optional, or unsupported
/// for a particular environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackFileEnvironment {
    pub client: EnvironmentSupport,
    pub server: EnvironmentSupport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnvironmentSupport {
    Required,
    Optional,
    Unsupported,
}
