use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet)]
pub struct SkinData {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
    pub alias: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
pub struct CapeData {
    pub id: String,
    pub state: String,
    pub url: String,
    pub alias: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
pub struct PlayerProfile {
    pub uuid: String,
    pub username: String,
    pub skin_url: Option<String>,
    pub cape_url: Option<String>,
    pub skin_model: SkinModel,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum SkinModel {
    Steve,
    Alex,
}
