// use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize, Clone, facet::Facet)]
// pub struct CustomSymlink {
//     pub id: String,
//     pub source_path: String,
//     pub target_path: String,
//     pub enabled: bool,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, Default, facet::Facet)]
// pub struct CustomSymlinksConfig {
//     pub symlinks: Vec<CustomSymlink>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, facet::Facet)]
// pub struct SymlinkInfo {
//     pub source_path: String,
//     pub target_path: String,
//     pub enabled: bool,
// }

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
pub struct CreateSymlinkRequest {
    pub source: String,
    pub destination: String,
    pub installation_id: Option<String>,
    pub category: SymlinkCategory,
    pub source_type: SymlinkSource,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
pub struct SymlinkView {
    pub id: String,

    pub source: String,
    pub destination: String,

    pub installation_id: Option<String>,

    pub category: SymlinkCategory,
    pub kind: SymlinkKind,
    pub source_type: SymlinkSource,

    pub state: SymlinkState,

    pub source_exists: bool,
    pub link_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
pub struct SymlinkEntry {
    pub id: String,

    pub source: PathBuf,
    pub destination: PathBuf,

    pub installation_id: Option<String>,

    pub enabled: bool,

    pub category: SymlinkCategory,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum SymlinkKind {
    File,
    Directory,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum SymlinkCategory {
    ResourcePack,
    ShaderPack,
    World,
    Mod,
    Custom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum SymlinkSource {
    Managed,
    Custom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum SymlinkState {
    Enabled,
    Disabled,
    MissingTarget,
    MissingLink,
    Broken,
}

impl From<CreateSymlinkRequest> for SymlinkEntry {
    fn from(req: CreateSymlinkRequest) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source: PathBuf::from(req.source),
            destination: PathBuf::from(req.destination),
            installation_id: req.installation_id,
            enabled: req.enabled,
            category: req.category,
        }
    }
}

impl From<SymlinkEntry> for SymlinkView {
    fn from(entry: SymlinkEntry) -> Self {
        let kind = if entry.source.is_dir() { SymlinkKind::Directory } else { SymlinkKind::File };
        Self {
            id: entry.id,
            source: entry.source.to_string_lossy().to_string(),
            destination: entry.destination.to_string_lossy().to_string(),
            installation_id: entry.installation_id,
            category: entry.category,
            kind,
            source_type: SymlinkSource::Custom, // Assuming all created symlinks are custom for now
            state: SymlinkState::Disabled,      // Placeholder, actual state should be determined by checking the filesystem
            source_exists: false,               // Placeholder
            link_exists: false,                 // Placeholder
        }
    }
}
