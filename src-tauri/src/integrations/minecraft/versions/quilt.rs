use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuiltVersionManifest {
    pub id: String,
    pub inherits_from: String,
    #[serde(rename = "type")]
    pub quilt_version_manifest_type: String,
    pub main_class: String,
    pub arguments: Arguments,
    pub libraries: Vec<Library>,
    pub release_time: String,
    pub time: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arguments {
    pub game: Vec<Option<serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    pub url: String,
}
