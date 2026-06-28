use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, facet::Facet, specta::Type)]
pub struct Symlink {
    pub id: String, // unique identifier being string combo of source+destination.
    pub source: PathBuf,
    pub destination: PathBuf,
    pub is_temporary: bool,  // if true, this symlink is temporary and should be removed on exit
    pub enabled: bool,       // if false, this symlink is disabled and should not be created on startup
    pub from_launcher: bool, // if true, this symlink was created by the launcher (either temporary or custom), if false, it was imported from existing symlinks
}

impl Symlink {
    pub fn new(source: PathBuf, destination: PathBuf, is_temporary: bool, enabled: bool, from_launcher: bool) -> Self {
        let id = format!("{}-{}", source.display(), destination.display());
        Self { id, source, destination, is_temporary, enabled, from_launcher }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, facet::Facet, specta::Type)]
pub struct SymlinkCreateRequest {
    pub source: String,
    pub destination: String,
}

impl From<SymlinkCreateRequest> for Symlink {
    fn from(req: SymlinkCreateRequest) -> Self {
        let source = PathBuf::from(req.source);
        let destination = PathBuf::from(req.destination);
        Self::new(source, destination, false, true, true)
    }
}
