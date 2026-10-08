use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone,  specta::Type)]
pub struct SoundpackMetadata {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub file_path: Option<String>,
}
