use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VanillaVersionManifest {
    pub arguments: Arguments,
    pub asset_index: AssetIndex,
    pub assets: String,
    pub compliance_level: i64,
    pub downloads: VanillaVersionManifestDownloads,
    pub id: String,
    pub java_version: JavaVersion,
    pub libraries: Vec<Library>,
    pub logging: Logging,
    pub main_class: String,
    pub minimum_launcher_version: i64,
    pub release_time: String,
    pub time: String,
    #[serde(rename = "type")]
    pub vanilla_version_manifest_type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Arguments {
    pub default_user_jvm: Vec<DefaultUserJvm>,
    pub game: Vec<GameElement>,
    pub jvm: Vec<JvmElement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefaultUserJvm {
    pub value: Vec<String>,
    pub rules: Option<Vec<DefaultUserJvmRule>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefaultUserJvmRule {
    pub action: Action,
    pub os: PurpleOs,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Allow,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurpleOs {
    pub name: Name,
    pub version_range: Option<VersionRange>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display)]
#[serde(rename_all = "snake_case")]
pub enum Name {
    #[strum(to_string = "linux")]
    Linux,
    #[strum(to_string = "osx")]
    Osx,
    #[strum(to_string = "windows")]
    Windows,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionRange {
    pub min: Option<String>,
    pub max: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GameElement {
    GameClass(GameClass),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameClass {
    pub rules: Vec<GameRule>,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameRule {
    pub action: Action,
    pub features: Features,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Features {
    pub is_demo_user: Option<bool>,
    pub has_custom_resolution: Option<bool>,
    pub has_quick_plays_support: Option<bool>,
    pub is_quick_play_singleplayer: Option<bool>,
    pub is_quick_play_multiplayer: Option<bool>,
    pub is_quick_play_realms: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    String(String),
    StringArray(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JvmElement {
    JvmClass(JvmClass),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JvmClass {
    pub rules: Vec<JvmRule>,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JvmRule {
    pub action: Action,
    pub os: FluffyOs,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FluffyOs {
    pub name: Option<Name>,
    pub arch: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: i64,
    pub total_size: Option<i64>,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VanillaVersionManifestDownloads {
    pub client: ServerClass,
    pub server: ServerClass,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerClass {
    pub sha1: String,
    pub size: i64,
    pub url: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Library {
    pub downloads: LibraryDownloads,
    pub name: String,
    pub rules: Option<Vec<LibraryRule>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: ServerClass,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryRule {
    pub action: Action,
    pub os: TentacledOs,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TentacledOs {
    pub name: Name,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Logging {
    pub client: LoggingClient,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoggingClient {
    pub argument: String,
    pub file: AssetIndex,
    #[serde(rename = "type")]
    pub client_type: String,
}
