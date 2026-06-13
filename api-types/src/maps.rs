use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorldDownload {
    pub name: String,
    pub url: String,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WorldSource {
    Local,
    Remote,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalWorld {
    pub name: String,
    pub path: String,
    pub source: WorldSource,
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