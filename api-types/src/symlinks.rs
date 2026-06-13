use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomSymlink {
    pub id: String,
    pub source_path: String,
    pub target_path: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CustomSymlinksConfig {
    pub symlinks: Vec<CustomSymlink>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SymlinkInfo {
    pub source_path: String,
    pub target_path: String,
    pub enabled: bool,
}