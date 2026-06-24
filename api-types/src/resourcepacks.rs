use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, facet::Facet, specta::Type)]
pub struct ResourcePackFilterFacets {
    pub query: Option<String>,
    pub categories: Option<Vec<(String, String)>>,
    pub game_versions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
pub struct ResourcePack {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: Option<String>,
    pub file_path: String,
    pub file_name: String,
    pub file_size: u32,
    pub compatible_versions: Vec<String>,
    pub pack_format: u32,
    pub enabled: bool,
    pub source_url: Option<String>,
    pub thumbnail: Option<String>,
    pub installed_date: i32,
    pub last_used: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
pub struct ResourcePackDownload {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub download_url: String,
    pub thumbnail: Option<String>,
    pub gallery: Option<Vec<String>>,
    pub featured_gallery: Option<String>,
    pub tags: Vec<String>,
    pub minecraft_versions: Vec<String>,
    pub resolution: Option<String>,
    pub rating: f32,
    pub downloads: u32,
    pub size_mb: u32,
    pub source: ResourcePackSource,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum ResourcePackSource {
    Modrinth,
    CurseForge,
    Other(String),
}
