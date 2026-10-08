use serde::{Deserialize, Serialize};
use url::Url;

/// Represents a project in a profile's dedicated mods folder, including its metadata and whether it is enabled or disabled
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct KableProject {
    /// The project itself, containing all of its metadata
    pub project: Project,
    /// Should match a ProjectVersion.id and indicate what version of the mod is installed in the profile's dedicated mods folder
    pub version_id: String,
    /// The filename of the mod jar file in the profile's dedicated mods folder, should match a ProjectVersion.files.filename, and is used to locate the mod jar file in the profile's dedicated mods folder
    pub filename: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct UpdateMap {
    pub kable_project: KableProject,
    pub update: Option<Project>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, specta::Type)]
pub struct ProjectSearch {
    pub query: Option<String>,
    pub facets: Vec<FacetGroup>,
    pub index: Option<SearchIndex>,
    pub offset: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, specta::Type)]
pub struct FacetGroup {
    pub facets: Vec<Facet>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, specta::Type)]
pub struct Facet {
    pub field: FacetField,
    pub operator: FacetOperator,
    pub value: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, specta::Type)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum FacetOperator {
    Eq,
    NotEq,
    Greater,
    #[default]
    GreaterEq,
    Less,
    LessEq,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, specta::Type)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum FacetField {
    ProjectType,
    /// Categories also contains loaders: "fabric", "forge", "quilt", "neo-forge", etc.
    Categories,
    Version,
    ClientSide,
    ServerSide,
    OpenSource,
    #[default]
    Title,
    Author,
    Follows,
    ProjectId,
    License,
    Downloads,
    Color,
    CreatedTimestamp,
    ModifiedTimestamp,
    DateCreated,
    DateModified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, specta::Type)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum SearchIndex {
    #[default]
    Relevance,
    Downloads,
    Follows,
    Newest,
    Updated,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct ModrinthResults {
    /// The list of results
    #[serde(rename = "hits")]
    pub hits: Vec<Project>,
    /// The number of results that were skipped by the query
    #[serde(rename = "offset")]
    pub offset: i32,
    /// The number of results that were returned by the query
    #[serde(rename = "limit")]
    pub limit: i32,
    /// The total number of results that match the query
    #[serde(rename = "total_hits")]
    pub total_hits: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct Project {
    /// The slug of a project, used for vanity URLs. Regex: ```^[\\w!@$()`.+,\"\\-']{3,64}$```
    #[serde(rename = "slug")]
    pub slug: String,
    /// The title or name of the project
    #[serde(rename = "title")]
    pub title: String,
    /// A short description of the project
    #[serde(rename = "description")]
    pub description: String,
    /// A list of the categories that the project has
    // #[serde(rename = "categories", skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
    /// The client side support of the project
    #[serde(rename = "client_side")]
    pub client_side: ClientSide,
    /// The server side support of the project
    #[serde(rename = "server_side")]
    pub server_side: ServerSide,
    /// The project type of the project
    #[serde(rename = "project_type")]
    pub project_type: ProjectType,
    /// The total number of downloads of the project
    #[serde(rename = "downloads")]
    pub downloads: i32,
    /// The URL of the project's icon
    // #[serde(rename = "icon_url", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    // #[specta(type = specta_serde::Phased<Option<Option<String>>, Option<Option<String>>>)]
    pub icon_url: Option<Option<String>>,
    /// The RGB color of the project, automatically generated from the project icon
    // #[serde(rename = "color", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    // #[specta(type = specta_serde::Phased<Option<Option<i32>>, Option<Option<i32>>>)]
    pub color: Option<Option<i32>>,
    /// The ID of the moderation thread associated with this project
    // #[serde(rename = "thread_id", skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    // #[serde(rename = "monetization_status", skip_serializing_if = "Option::is_none")]
    pub monetization_status: Option<MonetizationStatus>,
    /// The ID of the project
    #[serde(rename = "project_id")]
    pub project_id: String,
    /// The username of the project's author
    #[serde(rename = "author")]
    pub author: String,
    /// A list of the categories that the project has which are not secondary
    // #[serde(rename = "display_categories", skip_serializing_if = "Option::is_none")]
    pub display_categories: Option<Vec<String>>,
    /// A list of the minecraft versions supported by the project
    #[serde(rename = "versions")]
    pub versions: Vec<ProjectVersion>,
    /// The total number of users following the project
    #[serde(rename = "follows")]
    pub follows: i32,
    /// The date the project was added to search
    #[serde(rename = "date_created")]
    pub date_created: String,
    /// The date the project was last modified
    #[serde(rename = "date_modified")]
    pub date_modified: String,
    /// The latest version of minecraft that this project supports
    // #[serde(rename = "latest_version", skip_serializing_if = "Option::is_none")]
    pub latest_version: Option<String>,
    /// The SPDX license ID of a project
    #[serde(rename = "license")]
    pub license: String,
    /// All gallery images attached to the project
    // #[serde(rename = "gallery", skip_serializing_if = "Option::is_none")]
    pub gallery: Vec<Url>,
    /// The featured gallery image of the project
    // #[serde(rename = "featured_gallery", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    // #[specta(type = specta_serde::Phased<Option<Option<String>>, Option<Option<String>>>)]
    pub featured_gallery: Option<Url>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum ClientSide {
    #[serde(rename = "required")]
    Required,
    #[serde(rename = "optional")]
    Optional,
    #[serde(rename = "unsupported")]
    Unsupported,
}

/// The server side support of the project
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum ServerSide {
    #[serde(rename = "required")]
    Required,
    #[serde(rename = "optional")]
    Optional,
    #[serde(rename = "unsupported")]
    Unsupported,
}

/// The project type of the project
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum ProjectType {
    #[serde(rename = "mod")]
    Mod,
    #[serde(rename = "modpack")]
    Modpack,
    #[serde(rename = "resourcepack")]
    Resourcepack,
    #[serde(rename = "shader")]
    Shader,
    #[serde(rename = "plugin")]
    Plugin,
    #[serde(rename = "datapack")]
    Datapack,
    #[serde(rename = "minecraft-java-server")]
    MinecraftJavaServer,
    #[serde(rename = "other")]
    Other,
}

/// The monetization status of the project
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum MonetizationStatus {
    #[serde(rename = "monetized")]
    Monetized,
    #[serde(rename = "demonetized")]
    Demonetized,
    #[serde(rename = "force-demonetized")]
    ForceDemonetized,
    #[serde(rename = "other")]
    Other,
}

//?---------------------------------------------------------------------
//? PROJECT VERSION STRUCTURES - from modrinth_api::models::version.rs
//?---------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct ProjectVersion {
    /// The name of this version
    // #[serde(rename = "name")]
    pub name: String,
    /// The version number. Ideally will follow semantic versioning
    // #[serde(rename = "version_number")]
    pub version_number: String,
    /// The changelog for this version
    // #[serde(rename = "changelog", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    // #[specta(type = specta_serde::Phased<Option<Option<String>>, Option<Option<String>>>)]
    pub changelog: Option<Option<String>>,
    /// A list of specific versions of projects that this version depends on
    // #[serde(rename = "dependencies", skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<VersionDependency>>,
    /// A list of versions of Minecraft that this version supports
    // #[serde(rename = "game_versions")]
    pub game_versions: Vec<String>,
    /// The release channel for this version
    // #[serde(rename = "version_type")]
    pub version_type: VersionType,
    /// The mod loaders that this version supports
    // #[serde(rename = "loaders")]
    pub loaders: Vec<String>,
    /// Whether the version is featured or not
    // #[serde(rename = "featured")]
    pub featured: bool,
    // #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    // #[serde(rename = "requested_status", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    // #[specta(type = specta_serde::Phased<Option<Option<RequestedStatus>>, Option<Option<RequestedStatus>>>)]
    pub requested_status: Option<Option<RequestedStatus>>,
    /// The ID of the version, encoded as a base62 string
    // #[serde(rename = "id")]
    pub id: String,
    /// The ID of the project this version is for
    // #[serde(rename = "project_id")]
    pub project_id: String,
    /// The ID of the author who published this version
    // #[serde(rename = "author_id")]
    pub author_id: String,
    // #[serde(rename = "date_published")]
    pub date_published: String,
    /// The number of times this version has been downloaded
    // #[serde(rename = "downloads")]
    pub downloads: i32,
    /// A link to the changelog for this version. Always null, only kept for legacy compatibility.
    // #[serde(rename = "changelog_url", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    // #[specta(type = specta_serde::Phased<Option<Option<String>>, Option<Option<String>>>)]
    pub changelog_url: Option<Option<String>>,
    /// A list of files available for download for this version
    // #[serde(rename = "files")]
    pub files: Vec<VersionFile>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct VersionDependency {
    /// The ID of the version that this version depends on
    // #[serde(rename = "version_id", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    // #[specta(type = specta_serde::Phased<Option<Option<String>>, Option<Option<String>>>)]
    pub version_id: Option<Option<String>>,
    /// The ID of the project that this version depends on
    // #[serde(rename = "project_id", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    // #[specta(type = specta_serde::Phased<Option<Option<String>>, Option<Option<String>>>)]
    pub project_id: Option<Option<String>>,
    /// The file name of the dependency, mostly used for showing external dependencies on modpacks
    // #[serde(rename = "file_name", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    // #[specta(type = specta_serde::Phased<Option<Option<String>>, Option<Option<String>>>)]
    pub file_name: Option<Option<String>>,
    /// The type of dependency that this version has
    // #[serde(rename = "dependency_type")]
    pub dependency_type: DependencyType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum DependencyType {
    #[serde(rename = "required")]
    Required,
    #[serde(rename = "optional")]
    Optional,
    #[serde(rename = "incompatible")]
    Incompatible,
    #[serde(rename = "embedded")]
    Embedded,
    #[serde(rename = "other")]
    Other,
}

/// The release channel for this version
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum VersionType {
    #[serde(rename = "release")]
    Release,
    #[serde(rename = "beta")]
    Beta,
    #[serde(rename = "alpha")]
    Alpha,
    /// This one is used when converting String to ProjectVersion as they lack a lot of info but the alternative is dropping all version info on converting ProjectResults...
    #[serde(rename = "incomplete")]
    Incomplete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Status {
    #[serde(rename = "listed")]
    Listed,
    #[serde(rename = "archived")]
    Archived,
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "unlisted")]
    Unlisted,
    #[serde(rename = "scheduled")]
    Scheduled,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum RequestedStatus {
    #[serde(rename = "listed")]
    Listed,
    #[serde(rename = "archived")]
    Archived,
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "unlisted")]
    Unlisted,
    #[serde(rename = "other")]
    Other,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct VersionFile {
    #[serde(rename = "hashes")]
    pub hashes: VersionFileHashes,
    /// A direct link to the file
    #[serde(rename = "url")]
    pub url: String,
    /// The name of the file
    #[serde(rename = "filename")]
    pub filename: String,
    /// Whether this file is the primary one for its version. Only a maximum of one file per version will have this set to true. If there are not any primary files, it can be inferred that the first file is the primary one.
    #[serde(rename = "primary")]
    pub primary: bool,
    /// The size of the file in bytes
    #[serde(rename = "size")]
    pub size: i32,
    /// The type of the additional file, used mainly for adding resource packs to datapacks
    // #[serde(rename = "file_type", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    // #[specta(type = specta_serde::Phased<Option<Option<FileType>>, Option<Option<FileType>>>)]
    pub file_type: Option<Option<FileType>>,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct VersionFileHashes {
    // #[serde(rename = "sha512", skip_serializing_if = "Option::is_none")]
    pub sha512: Option<String>,
    // #[serde(rename = "sha1", skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum FileType {
    #[serde(rename = "required-resource-pack")]
    RequiredResourcePack,
    #[serde(rename = "optional-resource-pack")]
    OptionalResourcePack,
    #[serde(rename = "sources-jar")]
    SourcesJar,
    #[serde(rename = "dev-jar")]
    DevJar,
    #[serde(rename = "javadoc-jar")]
    JavadocJar,
    #[serde(rename = "signature")]
    Signature,
    #[serde(rename = "unknown")]
    Unknown,
}
