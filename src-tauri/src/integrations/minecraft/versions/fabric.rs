use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabricVersionManifest {
    pub arguments: Arguments,
    #[serde(rename = "type")]
    pub fabric_version_manifest_type: String,
    pub id: String,
    pub inherits_from: String,
    pub libraries: Vec<Library>,
    pub main_class: String,
    pub release_time: String,
    pub time: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arguments {
    pub jvm: Vec<String>,
    pub game: Vec<Option<serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Library {
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub size: Option<i64>,
    pub name: String,
    pub sha512: Option<String>,
    pub url: String,
    pub md5: Option<String>,
}
