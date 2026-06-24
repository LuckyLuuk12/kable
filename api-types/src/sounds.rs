use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
pub struct SoundpackMetadata {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub file_path: Option<String>,
}
