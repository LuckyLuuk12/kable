use serde::{Deserialize, Serialize};

// Profile-related structures for skins, capes, and player profiles
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerProfile {
    pub uuid: String,
    pub username: String,
    pub skin_url: Option<String>,
    pub cape_url: Option<String>,
    pub skin_model: SkinModel,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SkinModel {
    Steve,  // Classic skin model
    Alex,   // Slim skin model
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkinData {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
    pub alias: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CapeData {
    pub id: String,
    pub state: String,
    pub url: String,
    pub alias: Option<String>,
}

// TODO: Implement skin/cape management functions
// - Fetch player profile from Mojang API
// - Download and cache skins/capes
// - Apply custom skins
// - Manage cape visibility
