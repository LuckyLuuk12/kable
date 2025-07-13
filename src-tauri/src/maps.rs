use serde::{Deserialize, Serialize};

// Map/World management structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorldDownload {
    pub id: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub download_url: String,
    pub thumbnail: Option<String>,
    pub tags: Vec<String>,
    pub minecraft_version: String,
    pub size_mb: u64,
    pub rating: f32,
    pub downloads: u64,
    pub source: WorldSource,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WorldSource {
    PlanetMinecraft,
    MinecraftMaps,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalWorld {
    pub id: String,
    pub name: String,
    pub folder_name: String,
    pub game_mode: GameMode,
    pub difficulty: Difficulty,
    pub version: String,
    pub size_mb: u64,
    pub last_played: i64,
    pub created: i64,
    pub seed: Option<String>,
    pub icon: Option<String>,
    pub backup_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

// TODO: Implement map/world management functions
// - Search PlanetMinecraft for worlds
// - Download and extract world files
// - Manage local worlds (backup, restore, delete)
// - Import/export worlds
// - World thumbnail generation
