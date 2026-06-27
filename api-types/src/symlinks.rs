use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, facet::Facet, specta::Type)]
pub struct Symlink {
    pub source: PathBuf,
    pub destination: PathBuf,
}

impl Symlink {
    pub fn new(source: PathBuf, destination: PathBuf) -> Self {
        Self { source, destination }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, facet::Facet, specta::Type)]
pub struct SymlinkCreateRequest {
    pub source: PathBuf,
    pub destination_parent: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, facet::Facet, specta::Type)]
pub struct SymlinkUpdateRequest {
    pub old: Symlink,
    pub new: Symlink,
}
