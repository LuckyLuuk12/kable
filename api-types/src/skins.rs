use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkinModel {
    #[default]
    Classic,
    Slim,
}

impl SkinModel {
    pub fn to_api_string(&self) -> &'static str {
        match self {
            SkinModel::Classic => "classic",
            SkinModel::Slim => "slim",
        }
    }

    pub fn from_api_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "classic" | "steve" => Some(SkinModel::Classic),
            "slim" | "alex" => Some(SkinModel::Slim),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinUploadConfig {
    pub model: SkinModel,
    pub file_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkinUploadResponse {
    pub success: bool,
    pub message: String,
    pub model_used: SkinModel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentSkin {
    pub model: SkinModel,
    pub url: Option<String>,
    pub has_skin: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountSkin {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
    pub model: SkinModel,
    pub is_current: bool,
    pub uploaded_date: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountCape {
    pub id: String,
    pub state: String,
    pub url: Option<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerProfile {
    pub id: String,
    pub name: String,
    pub skins: Vec<AccountSkin>,
    pub capes: Vec<AccountCape>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomSkinsRoot {
    #[serde(rename = "customSkins")]
    pub custom_skins: HashMap<String, CustomSkinEntry>,
    pub version: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomSkinEntry {
    #[serde(default, rename = "capeId")]
    pub cape_id: String,
    #[serde(default, rename = "created")]
    pub created: String,
    #[serde(default, rename = "id")]
    pub id: String,
    #[serde(default, rename = "modelImage")]
    pub model_image: String,
    #[serde(default, rename = "name")]
    pub name: String,
    #[serde(default, rename = "skinImage")]
    pub skin_image: String,
    #[serde(default, rename = "slim")]
    pub slim: bool,
    #[serde(default, rename = "textureId")]
    pub texture_id: String,
    #[serde(default, rename = "updated")]
    pub updated: String,
}