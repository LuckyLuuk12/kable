use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McVersionManifest {
    pub id: String,

    #[serde(rename = "type", default)]
    pub r#type: String,

    #[serde(default)]
    pub inherits_from: Option<String>,

    #[serde(default)]
    pub main_class: Option<String>,

    #[serde(default)]
    pub time: Option<String>,

    #[serde(default)]
    pub release_time: Option<String>,

    #[serde(default)]
    pub minimum_launcher_version: Option<i32>,

    #[serde(default)]
    pub assets: Option<String>,

    #[serde(default)]
    pub asset_index: Option<AssetIndex>,

    #[serde(default)]
    pub compliance_level: Option<i32>,

    #[serde(default)]
    pub java_version: Option<JavaVersion>,

    #[serde(default)]
    pub logging: Option<Logging>,

    #[serde(default)]
    pub arguments: Option<Arguments>,

    #[serde(default)]
    pub downloads: Option<Downloads>,

    #[serde(default)]
    pub libraries: Option<Vec<Library>>,

    #[serde(rename = "*comment*", default)]
    pub comment: Option<Vec<String>>,

    /// Preserve manifest properties that this version of Kable
    /// does not explicitly model.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    #[serde(default)]
    pub id: Option<String>,

    #[serde(default)]
    pub sha1: Option<String>,

    #[serde(default)]
    pub size: Option<i64>,

    #[serde(default)]
    pub total_size: Option<i64>,

    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Logging {
    #[serde(default)]
    pub client: Option<LoggingClient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingClient {
    #[serde(default)]
    pub argument: Option<String>,
    #[serde(rename = "type", default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub file: Option<LoggingFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingFile {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Downloads {
    #[serde(default)]
    pub client: Option<DownloadArtifact>,

    #[serde(default)]
    pub server: Option<DownloadArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadArtifact {
    #[serde(default)]
    pub path: Option<String>,

    #[serde(default)]
    pub sha1: Option<String>,

    #[serde(default)]
    pub sha256: Option<String>,

    #[serde(default)]
    pub sha512: Option<String>,

    #[serde(default)]
    pub md5: Option<String>,

    #[serde(default)]
    pub size: Option<i64>,

    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,

    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,

    /// Maven repository URL used by some loader manifests.
    #[serde(default)]
    pub url: Option<String>,

    /// Top-level artifact metadata used by some Fabric/Quilt manifests.
    #[serde(default)]
    pub sha1: Option<String>,

    #[serde(default)]
    pub sha256: Option<String>,

    #[serde(default)]
    pub sha512: Option<String>,

    #[serde(default)]
    pub md5: Option<String>,

    #[serde(default)]
    pub size: Option<i64>,

    #[serde(default)]
    pub rules: Option<Vec<LibraryRule>>,

    /// Retain loader-specific library metadata such as extract,
    /// include, or other properties not explicitly modeled here.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDownloads {
    #[serde(default)]
    pub artifact: Option<DownloadArtifact>,

    #[serde(default)]
    pub classifiers: Option<HashMap<String, DownloadArtifact>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryRule {
    #[serde(default)]
    pub action: Option<String>,

    #[serde(default)]
    pub os: Option<OsRule>,

    #[serde(default)]
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OsRule {
    #[serde(default)]
    pub name: Option<String>,

    /// Legacy OS-version regular expression used in some manifests.
    #[serde(default)]
    pub version: Option<String>,

    /// Explicit minimum/maximum OS-version range.
    #[serde(default)]
    pub version_range: Option<VersionRange>,

    #[serde(default)]
    pub arch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionRange {
    #[serde(default)]
    pub min: Option<String>,

    #[serde(default)]
    pub max: Option<String>,
}

/// Minecraft 1.13+ structured arguments and legacy argument arrays.
///
/// Note: Mixed and Flat are retained as separate variants for
/// compatibility with the existing resolver. Both deserialize from
/// JSON arrays, so their declaration order matters to Serde.
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
    #[serde(default)]
    pub game: Option<Vec<Arg>>,

    #[serde(default)]
    pub jvm: Option<Vec<Arg>>,

    #[serde(default, rename = "default-user-jvm")]
    pub default_user_jvm: Option<Vec<Arg>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Arg {
    String(String),
    Rule(RuleArg),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleArg {
    #[serde(default)]
    pub rules: Option<Vec<Rule>>,

    #[serde(default)]
    pub value: Option<StringOrList>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrList {
    Single(String),
    List(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    #[serde(default)]
    pub action: Option<String>,

    #[serde(default)]
    pub os: Option<OsRule>,

    #[serde(default)]
    pub features: Option<HashMap<String, bool>>,
}
