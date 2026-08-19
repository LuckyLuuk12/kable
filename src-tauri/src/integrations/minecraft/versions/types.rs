// use serde::{Deserialize, Serialize};
// use std::collections::HashMap;

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct McVersionManifest {
//     pub id: String,
//     #[serde(rename = "type")]
//     pub r#type: String,

//     pub inherits_from: Option<String>,

//     pub main_class: Option<String>,

//     pub time: Option<String>,
//     pub release_time: Option<String>,

//     pub minimum_launcher_version: Option<i64>,

//     pub assets: Option<String>,
//     pub asset_index: Option<AssetIndex>,

//     pub compliance_level: Option<i64>,

//     pub java_version: Option<JavaVersion>,

//     pub logging: Option<Logging>,

//     pub arguments: Option<Arguments>,

//     pub downloads: Option<Downloads>,

//     pub libraries: Option<Vec<Library>>,
//     // Use flatten to capture any other fields (like Forge's *comment*)
//     #[serde(flatten)]
//     pub extra: HashMap<String, serde_json::Value>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct AssetIndex {
//     pub id: Option<String>,
//     pub sha1: Option<String>,
//     pub size: Option<i64>,
//     pub total_size: Option<i64>,
//     pub url: Option<String>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct JavaVersion {
//     pub component: String,
//     pub major_version: i64,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Logging {
//     pub client: Option<LoggingClient>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct LoggingClient {
//     pub argument: Option<String>,
//     pub r#type: Option<String>,
//     pub file: Option<AssetIndex>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Downloads {
//     pub client: Option<DownloadArtifact>,
//     pub server: Option<DownloadArtifact>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct DownloadArtifact {
//     pub sha1: Option<String>,
//     pub sha256: Option<String>,
//     pub sha512: Option<String>,
//     pub md5: Option<String>,

//     pub size: Option<i64>,
//     pub url: Option<String>,
//     pub path: Option<String>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Library {
//     pub name: Option<String>,
//     // For Vanilla/NeoForge style
//     pub downloads: Option<LibraryDownloads>,
//     // For Fabric/Quilt style (top-level fields)
//     pub url: Option<String>,
//     pub sha1: Option<String>,
//     pub sha256: Option<String>,
//     pub sha512: Option<String>,
//     pub md5: Option<String>,
//     pub size: Option<i64>,
//     pub rules: Option<Vec<LibraryRule>>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct LibraryDownloads {
//     pub artifact: Option<DownloadArtifact>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct LibraryRule {
//     pub action: Option<String>,
//     pub os: Option<OsRule>,
//     pub features: Option<HashMap<String, bool>>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct OsRule {
//     pub name: Option<String>,
//     pub version: Option<String>,
//     pub arch: Option<String>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum Arguments {
//     Structured(StructuredArguments),
//     Mixed(Vec<Arg>),
//     Flat(Vec<String>),
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct StructuredArguments {
//     pub game: Option<Vec<Arg>>,
//     pub jvm: Option<Vec<Arg>>,
//     #[serde(rename = "default-user-jvm")]
//     pub default_user_jvm: Option<Vec<Arg>>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum Arg {
//     String(String),
//     Rule(RuleArg),
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct RuleArg {
//     pub rules: Option<Vec<Rule>>,
//     pub value: Option<Vec<String>>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Rule {
//     pub action: Option<String>,
//     pub os: Option<OsRule>,
//     pub features: Option<HashMap<String, bool>>,
// }
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McVersionManifest {
    pub id: String,
    pub r#type: String,
    pub inherits_from: Option<String>,
    pub main_class: Option<String>,
    pub time: Option<String>,
    pub release_time: Option<String>,
    pub libraries: Option<Vec<Library>>,
    pub arguments: Option<Arguments>,
    pub asset_index: Option<AssetIndex>,
    pub assets: Option<String>,
    pub java_version: Option<JavaVersion>,
    pub downloads: Option<Downloads>,
    pub logging: Option<Logging>,
    #[serde(rename = " *comment* ")]
    pub comment: Option<Vec<String>>,
    pub compliance_level: Option<i32>,
    pub minimum_launcher_version: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<i64>,
    pub total_size: Option<i64>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logging {
    pub client: Option<LoggingClient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingClient {
    pub argument: Option<String>,
    pub r#type: Option<String>,
    pub file: Option<AssetIndex>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Downloads {
    pub client: Option<DownloadArtifact>,
    pub server: Option<DownloadArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadArtifact {
    pub path: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub sha512: Option<String>,
    pub md5: Option<String>,
    pub size: Option<i64>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub name: Option<String>,
    pub downloads: Option<LibraryDownloads>,
    pub natives: Option<HashMap<String, String>>,
    // Top-level fields for Fabric/Quilt style manifests
    pub url: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub sha512: Option<String>,
    pub md5: Option<String>,
    pub size: Option<i64>,
    pub rules: Option<Vec<LibraryRule>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<DownloadArtifact>,
    pub classifiers: Option<HashMap<String, DownloadArtifact>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryRule {
    pub action: Option<String>,
    pub os: Option<OsRule>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OsRule {
    pub name: Option<String>,
    pub version: Option<String>,
    pub version_range: Option<VersionRange>,
    pub arch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRange {
    pub min: Option<String>,
    pub max: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Arguments {
    Structured(StructuredArguments),
    Mixed(Vec<Arg>),
    Flat(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredArguments {
    pub game: Option<Vec<Arg>>,
    pub jvm: Option<Vec<Arg>>,
    #[serde(rename = "default-user-jvm")]
    pub default_user_jvm: Option<Vec<Arg>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Arg {
    String(String),
    Rule(RuleArg),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleArg {
    pub rules: Option<Vec<Rule>>,
    pub value: Option<StringOrList>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrList {
    Single(String),
    List(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub action: Option<String>,
    pub os: Option<OsRule>,
    pub features: Option<HashMap<String, bool>>,
}
