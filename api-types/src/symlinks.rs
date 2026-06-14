use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet)]
pub struct CustomSymlink {
    pub id: String,
    pub source_path: String,
    pub target_path: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, facet::Facet)]
pub struct CustomSymlinksConfig {
    pub symlinks: Vec<CustomSymlink>,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet)]
pub struct SymlinkInfo {
    pub source_path: String,
    pub target_path: String,
    pub enabled: bool,
}
