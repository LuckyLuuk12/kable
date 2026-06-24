use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, facet::Facet, specta::Type)]
pub struct UpdateData {
    pub version: String,
    pub date: Option<String>,
    pub body: String,
    pub current_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, facet::Facet, specta::Type)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub prerelease: bool,
    pub draft: bool,
    #[allow(dead_code)]
    #[serde(default)]
    pub body: String,
}
