use serde::{Deserialize, Serialize};

// Shader management structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShaderPack {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: Option<String>,
    pub file_path: String,
    pub compatible_versions: Vec<String>,
    pub enabled: bool,
    pub source_url: Option<String>,
    pub thumbnail: Option<String>,
    pub shader_loader: ShaderLoader,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ShaderLoader {
    OptiFine,
    Iris,
    Sodium,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShaderSettings {
    pub quality: ShaderQuality,
    pub shadows: bool,
    pub shadow_resolution: u32,
    pub anti_aliasing: bool,
    pub bloom: bool,
    pub motion_blur: bool,
    pub custom_settings: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ShaderQuality {
    Low,
    Medium,
    High,
    Ultra,
    Custom,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShaderDownload {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub download_url: String,
    pub thumbnail: Option<String>,
    pub version: String,
    pub compatible_loaders: Vec<ShaderLoader>,
    pub minecraft_versions: Vec<String>,
    pub downloads: u64,
    pub rating: f32,
}

// TODO: Implement shader management functions
// - Download and install shader packs
// - Manage shader settings and presets
// - Handle OptiFine/Iris compatibility
// - Shader pack browsing and searching
// - Performance optimization recommendations
