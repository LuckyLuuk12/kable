use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeoforgeVersionManifest {
    pub id: String,
    pub time: String,
    pub release_time: String,
    #[serde(rename = "type")]
    pub neoforge_version_manifest_type: String,
    pub main_class: String,
    pub inherits_from: String,
    pub arguments: Arguments,
    pub libraries: Vec<Library>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arguments {
    pub game: Vec<String>,
    pub jvm: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    pub downloads: Downloads,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Downloads {
    pub artifact: Artifact,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    pub sha1: String,
    pub size: i64,
    pub url: String,
    pub path: String,
}
