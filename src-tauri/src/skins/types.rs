use serde::{Deserialize, Serialize};

/// Represents the skin model type in Minecraft
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum SkinModel {
    /// Classic model (Steve) - 4px wide arms
    Classic,
    /// Slim model (Alex) - 3px wide arms
    Slim,
}

impl SkinModel {
    /// Convert to the string format expected by Mojang API
    pub fn to_api_string(&self) -> &'static str {
        match self {
            SkinModel::Classic => "classic",
            SkinModel::Slim => "slim",
        }
    }

    /// Parse from API response string
    pub fn from_api_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "classic" | "steve" => Some(SkinModel::Classic),
            "slim" | "alex" => Some(SkinModel::Slim),
            _ => None,
        }
    }
}

impl Default for SkinModel {
    fn default() -> Self {
        SkinModel::Classic
    }
}

/// Configuration for skin upload operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinUploadConfig {
    pub model: SkinModel,
    pub file_path: String,
}

/// Response from skin upload operation
#[derive(Debug, Serialize, Deserialize)]
pub struct SkinUploadResponse {
    pub success: bool,
    pub message: String,
    pub model_used: SkinModel,
}

/// Current skin information from Mojang API
#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentSkin {
    pub model: SkinModel,
    pub url: Option<String>,
    pub has_skin: bool,
}

/// Account skin from Microsoft/Mojang skin history
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountSkin {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
    pub model: SkinModel,
    pub is_current: bool,
    pub uploaded_date: Option<i64>, // Unix timestamp
}
