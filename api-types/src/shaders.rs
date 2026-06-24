use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, facet::Facet, specta::Type)]
pub struct ShaderFilterFacets {
    pub query: Option<String>,
    pub loaders: Option<Vec<(String, String)>>,
    pub categories: Option<Vec<(String, String)>>,
    pub game_versions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
pub struct ShaderPack {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: Option<String>,
    pub file_path: String,
    pub file_name: String,
    pub file_size: u32,
    pub compatible_versions: Vec<String>,
    pub enabled: bool,
    pub source_url: Option<String>,
    pub thumbnail: Option<String>,
    pub shader_loader: ShaderLoader,
    pub installed_date: i32,
    pub last_used: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum ShaderLoader {
    Canvas,
    Iris,
    OptiFine,
    Vanilla,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
pub struct ShaderSettings {
    pub quality: ShaderQuality,
    pub shadows: bool,
    pub shadow_resolution: u32,
    pub anti_aliasing: bool,
    pub bloom: bool,
    pub motion_blur: bool,
    pub custom_settings: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum ShaderQuality {
    Low,
    Medium,
    High,
    Ultra,
    Custom,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
pub struct ShaderDownload {
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
    pub shader_loader: ShaderLoader,
    pub rating: f32,
    pub downloads: u32,
    pub size_mb: u32,
    pub source: ShaderSource,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum ShaderSource {
    Modrinth,
    CurseForge,
    Other(String),
}
