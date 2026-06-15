// use serde::{Deserialize, Serialize};
// use std::fmt;

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
// pub struct FilterFacets {
//     pub query: Option<String>,
//     pub categories: Option<Vec<(String, String)>>,
//     pub client_side: Option<(String, String)>,
//     pub server_side: Option<(String, String)>,
//     pub open_source: Option<bool>,
//     pub license: Option<(String, String)>,
//     pub downloads: Option<(String, u64)>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct ModrinthInfo {
//     #[serde(rename = "project_id")]
//     pub project_id: String,
//     pub project_type: String,
//     pub slug: String,
//     pub title: String,
//     pub description: String,
//     pub author: String,
//     pub categories: Vec<String>,
//     #[serde(default)]
//     pub display_categories: Vec<String>,
//     #[serde(default)]
//     pub versions: Vec<String>,
//     pub downloads: u64,
//     #[serde(rename = "follows")]
//     pub followers: Option<u64>,
//     #[serde(rename = "icon_url")]
//     pub icon_url: Option<String>,
//     #[serde(rename = "date_created")]
//     pub date_created: Option<String>,
//     #[serde(rename = "date_modified")]
//     pub date_modified: Option<String>,
//     #[serde(rename = "latest_version")]
//     pub latest_version: Option<String>,
//     pub license: Option<String>,
//     pub client_side: Option<String>,
//     pub server_side: Option<String>,
//     pub gallery: Option<Vec<String>>,
//     #[serde(rename = "featured_gallery")]
//     pub featured_gallery: Option<String>,
//     pub color: Option<u32>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub body: Option<String>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub additional_categories: Option<Vec<String>>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub issues_url: Option<String>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub source_url: Option<String>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub wiki_url: Option<String>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub discord_url: Option<String>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct DonationUrl {
//     pub id: String,
//     pub platform: String,
//     pub url: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct ModrinthLicense {
//     pub id: String,
//     pub name: String,
//     pub url: Option<String>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct ModerationMessage {
//     pub message: String,
//     pub body: Option<String>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct ModrinthVersion {
//     pub id: String,
//     pub name: String,
//     pub version_number: String,
//     pub changelog: Option<String>,
//     pub files: Vec<ModrinthFile>,
//     pub game_versions: Vec<String>,
//     pub loaders: Vec<String>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct ModrinthFile {
//     pub url: String,
//     pub filename: String,
//     pub primary: bool,
//     pub hashes: std::collections::HashMap<String, String>,
//     pub size: u64,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct ProjectDependencies {
//     pub projects: Vec<DependencyProject>,
//     pub versions: Vec<ModrinthVersion>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct DependencyProject {
//     #[serde(rename = "id", alias = "project_id")]
//     pub project_id: String,
//     #[serde(default)]
//     pub slug: String,
//     #[serde(default)]
//     pub title: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct VersionDependency {
//     pub version_id: Option<String>,
//     pub project_id: Option<String>,
//     pub file_name: Option<String>,
//     pub dependency_type: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub enum ProviderKind {
//     Modrinth,
//     CurseForge,
// }

// impl fmt::Display for ProviderKind {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             ProviderKind::Modrinth => write!(f, "Modrinth"),
//             ProviderKind::CurseForge => write!(f, "CurseForge"),
//         }
//     }
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
// pub enum ModInfoKind {
//     Modrinth(ModrinthInfo),
//     CurseForge(CurseForgeInfo),
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub enum ModFilter {
//     Modrinth(FilterFacets),
//     CurseForge(CurseForgeFilter),
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct ModpackSelectionGroup {
//     pub enabled: Vec<String>,
//     pub optional: Vec<String>,
//     pub disabled: Vec<String>,
//     #[serde(default)]
//     pub overwrite_paths: Vec<String>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ModpackSelection {
//     pub mods: ModpackSelectionGroup,
//     pub resourcepacks: ModpackSelectionGroup,
//     pub shaderpacks: ModpackSelectionGroup,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ModpackContext {
//     pub mrpack_path: String,
//     pub extracted_dir: String,
//     pub installation_dir: String,
//     pub provider: ProviderKind,
//     pub mod_id: String,
//     pub version_id: Option<String>,
// }

// #[allow(clippy::large_enum_variant)]
// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum DownloadOrPrepareResponse {
//     ModInstalled {
//         success: bool,
//     },
//     Modpack {
//         modpack: crate::profiles::MrPackDetailed,
//         context: ModpackContext,
//     },
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct CurseForgeFilter {
//     pub query: Option<String>,
//     pub category_id: Option<u32>,
//     pub game_version: Option<String>,
//     pub mod_loader_type: Option<ModLoaderType>,
//     pub sort_field: Option<ModsSearchSortField>,
//     pub sort_order: Option<SortOrder>,
// }

// #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
// #[repr(u32)]
// pub enum ModLoaderType {
//     Any = 0,
//     Forge = 1,
//     Cauldron = 2,
//     LiteLoader = 3,
//     Fabric = 4,
//     Quilt = 5,
//     NeoForge = 6,
// }

// #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
// #[repr(u32)]
// pub enum ModsSearchSortField {
//     Featured = 1,
//     Popularity = 2,
//     LastUpdated = 3,
//     Name = 4,
//     Author = 5,
//     TotalDownloads = 6,
//     Category = 7,
//     GameVersion = 8,
//     EarlyAccess = 9,
//     FeaturedReleased = 10,
//     ReleasedDate = 11,
//     Rating = 12,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub enum SortOrder {
//     #[serde(rename = "asc")]
//     Ascending,
//     #[serde(rename = "desc")]
//     Descending,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
// pub struct CurseForgeInfo {
//     pub id: u32,
//     #[serde(rename = "gameId")]
//     pub game_id: u32,
//     pub name: String,
//     pub slug: String,
//     #[serde(default)]
//     pub links: ModLinks,
//     pub summary: String,
//     pub status: u32,
//     #[serde(rename = "downloadCount")]
//     pub download_count: u64,
//     #[serde(rename = "isFeatured")]
//     pub is_featured: bool,
//     #[serde(rename = "primaryCategoryId")]
//     pub primary_category_id: u32,
//     #[serde(default)]
//     pub categories: Vec<Category>,
//     #[serde(rename = "classId")]
//     pub class_id: Option<u32>,
//     #[serde(default)]
//     pub authors: Vec<ModAuthor>,
//     pub logo: Option<ModAsset>,
//     #[serde(default)]
//     pub screenshots: Vec<ModAsset>,
//     #[serde(rename = "mainFileId")]
//     pub main_file_id: u32,
//     #[serde(rename = "latestFiles", default)]
//     pub latest_files: Vec<CurseForgeFile>,
//     #[serde(rename = "latestFilesIndexes", default)]
//     pub latest_files_indexes: Vec<FileIndex>,
//     #[serde(rename = "dateCreated")]
//     pub date_created: String,
//     #[serde(rename = "dateModified")]
//     pub date_modified: String,
//     #[serde(rename = "dateReleased")]
//     pub date_released: Option<String>,
//     #[serde(rename = "allowModDistribution")]
//     pub allow_mod_distribution: Option<bool>,
//     #[serde(rename = "gamePopularityRank")]
//     pub game_popularity_rank: u32,
//     #[serde(rename = "isAvailable")]
//     pub is_available: bool,
//     #[serde(rename = "thumbsUpCount")]
//     pub thumbs_up_count: u32,
//     pub rating: Option<f64>,
// }

// #[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct ModLinks {
//     #[serde(rename = "websiteUrl")]
//     pub website_url: Option<String>,
//     #[serde(rename = "wikiUrl")]
//     pub wiki_url: Option<String>,
//     #[serde(rename = "issuesUrl")]
//     pub issues_url: Option<String>,
//     #[serde(rename = "sourceUrl")]
//     pub source_url: Option<String>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct Category {
//     pub id: u32,
//     #[serde(rename = "gameId")]
//     pub game_id: u32,
//     pub name: String,
//     pub slug: String,
//     pub url: String,
//     #[serde(rename = "iconUrl")]
//     pub icon_url: String,
//     #[serde(rename = "dateModified")]
//     pub date_modified: String,
//     #[serde(rename = "isClass")]
//     pub is_class: bool,
//     #[serde(rename = "classId")]
//     pub class_id: Option<u32>,
//     #[serde(rename = "parentCategoryId")]
//     pub parent_category_id: Option<u32>,
//     #[serde(rename = "displayIndex")]
//     pub display_index: u32,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct ModAuthor {
//     pub id: u32,
//     pub name: String,
//     pub url: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct ModAsset {
//     pub id: u32,
//     #[serde(rename = "modId")]
//     pub mod_id: u32,
//     pub title: String,
//     pub description: String,
//     #[serde(rename = "thumbnailUrl")]
//     pub thumbnail_url: String,
//     pub url: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct CurseForgeFile {
//     pub id: u32,
//     #[serde(rename = "gameId")]
//     pub game_id: u32,
//     #[serde(rename = "modId")]
//     pub mod_id: u32,
//     #[serde(rename = "isAvailable")]
//     pub is_available: bool,
//     #[serde(rename = "displayName")]
//     pub display_name: String,
//     #[serde(rename = "fileName")]
//     pub file_name: String,
//     #[serde(rename = "releaseType")]
//     pub release_type: u32,
//     #[serde(rename = "fileStatus")]
//     pub file_status: u32,
//     #[serde(default)]
//     pub hashes: Vec<FileHash>,
//     #[serde(rename = "fileDate")]
//     pub file_date: String,
//     #[serde(rename = "fileLength")]
//     pub file_length: u64,
//     #[serde(rename = "downloadCount")]
//     pub download_count: u64,
//     #[serde(rename = "downloadUrl")]
//     pub download_url: String,
//     #[serde(rename = "gameVersions", default)]
//     pub game_versions: Vec<String>,
//     #[serde(default)]
//     pub dependencies: Vec<FileDependency>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct FileHash {
//     pub value: String,
//     pub algo: u32,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct FileDependency {
//     #[serde(rename = "modId")]
//     pub mod_id: u32,
//     #[serde(rename = "relationType")]
//     pub relation_type: u32,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct FileIndex {
//     #[serde(rename = "gameVersion")]
//     pub game_version: String,
//     #[serde(rename = "fileId")]
//     pub file_id: u32,
//     pub filename: String,
//     #[serde(rename = "releaseType")]
//     pub release_type: u32,
//     #[serde(rename = "gameVersionTypeId")]
//     pub game_version_type_id: Option<u32>,
//     #[serde(rename = "modLoader")]
//     pub mod_loader: u32,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SearchModsResponse {
//     pub data: Vec<CurseForgeInfo>,
//     pub pagination: Pagination,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Pagination {
//     pub index: u32,
//     #[serde(rename = "pageSize")]
//     pub page_size: u32,
//     #[serde(rename = "resultCount")]
//     pub result_count: u32,
//     #[serde(rename = "totalCount")]
//     pub total_count: u32,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ModFilesResponse {
//     pub data: Vec<CurseForgeFile>,
//     pub pagination: Pagination,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
// pub struct CurseForgeProvider {
//     pub limit: usize,
//     pub filter: CurseForgeFilter,
//     pub cache_path: std::path::PathBuf,
// }
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, facet::Facet)]
pub struct ProjectSearch {
    pub query: Option<String>,
    pub facets: Vec<FacetGroup>,
    pub index: Option<SearchIndex>,
    pub offset: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, facet::Facet)]
pub struct FacetGroup {
    pub facets: Vec<Facet>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, facet::Facet)]
pub struct Facet {
    pub field: FacetField,
    pub operator: FacetOperator,
    pub value: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, facet::Facet)]
#[facet(rename_all = "snake_case")]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum FacetField {
    ProjectType,
    Category,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, facet::Facet)]
#[facet(rename_all = "snake_case")]
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

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, facet::Facet)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, facet::Facet)]
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
    #[serde(rename = "categories", skip_serializing_if = "Option::is_none")]
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
    #[serde(
        rename = "icon_url",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub icon_url: Option<Option<String>>,
    /// The RGB color of the project, automatically generated from the project icon
    #[serde(
        rename = "color",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub color: Option<Option<i32>>,
    /// The ID of the moderation thread associated with this project
    #[serde(rename = "thread_id", skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(
        rename = "monetization_status",
        skip_serializing_if = "Option::is_none"
    )]
    pub monetization_status: Option<MonetizationStatus>,
    /// The ID of the project
    #[serde(rename = "project_id")]
    pub project_id: String,
    /// The username of the project's author
    #[serde(rename = "author")]
    pub author: String,
    /// A list of the categories that the project has which are not secondary
    #[serde(rename = "display_categories", skip_serializing_if = "Option::is_none")]
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
    #[serde(rename = "latest_version", skip_serializing_if = "Option::is_none")]
    pub latest_version: Option<String>,
    /// The SPDX license ID of a project
    #[serde(rename = "license")]
    pub license: String,
    /// All gallery images attached to the project
    #[serde(rename = "gallery", skip_serializing_if = "Option::is_none")]
    pub gallery: Option<Vec<String>>,
    /// The featured gallery image of the project
    #[serde(
        rename = "featured_gallery",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub featured_gallery: Option<Option<String>>,
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, facet::Facet,
)]
#[facet(rename_all = "snake_case")]
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
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, facet::Facet,
)]
#[facet(rename_all = "snake_case")]
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
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, facet::Facet,
)]
#[facet(rename_all = "snake_case")]
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
}

/// The monetization status of the project
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, facet::Facet,
)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum MonetizationStatus {
    #[serde(rename = "monetized")]
    Monetized,
    #[serde(rename = "demonetized")]
    Demonetized,
    #[serde(rename = "force-demonetized")]
    ForceDemonetized,
}

//?---------------------------------------------------------------------
//? PROJECT VERSION STRUCTURES - from modrinth_api::models::version.rs
//?---------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, facet::Facet)]
pub struct ProjectVersion {
    /// The name of this version
    #[serde(rename = "name")]
    pub name: String,
    /// The version number. Ideally will follow semantic versioning
    #[serde(rename = "version_number")]
    pub version_number: String,
    /// The changelog for this version
    #[serde(
        rename = "changelog",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub changelog: Option<Option<String>>,
    /// A list of specific versions of projects that this version depends on
    #[serde(rename = "dependencies", skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<VersionDependency>>,
    /// A list of versions of Minecraft that this version supports
    #[serde(rename = "game_versions")]
    pub game_versions: Vec<String>,
    /// The release channel for this version
    #[serde(rename = "version_type")]
    pub version_type: VersionType,
    /// The mod loaders that this version supports
    #[serde(rename = "loaders")]
    pub loaders: Vec<String>,
    /// Whether the version is featured or not
    #[serde(rename = "featured")]
    pub featured: bool,
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    #[serde(
        rename = "requested_status",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub requested_status: Option<Option<RequestedStatus>>,
    /// The ID of the version, encoded as a base62 string
    #[serde(rename = "id")]
    pub id: String,
    /// The ID of the project this version is for
    #[serde(rename = "project_id")]
    pub project_id: String,
    /// The ID of the author who published this version
    #[serde(rename = "author_id")]
    pub author_id: String,
    #[serde(rename = "date_published")]
    pub date_published: String,
    /// The number of times this version has been downloaded
    #[serde(rename = "downloads")]
    pub downloads: i32,
    /// A link to the changelog for this version. Always null, only kept for legacy compatibility.
    #[serde(
        rename = "changelog_url",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub changelog_url: Option<Option<String>>,
    /// A list of files available for download for this version
    #[serde(rename = "files")]
    pub files: Vec<VersionFile>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, facet::Facet)]
pub struct VersionDependency {
    /// The ID of the version that this version depends on
    #[serde(
        rename = "version_id",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub version_id: Option<Option<String>>,
    /// The ID of the project that this version depends on
    #[serde(
        rename = "project_id",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub project_id: Option<Option<String>>,
    /// The file name of the dependency, mostly used for showing external dependencies on modpacks
    #[serde(
        rename = "file_name",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub file_name: Option<Option<String>>,
    /// The type of dependency that this version has
    #[serde(rename = "dependency_type")]
    pub dependency_type: DependencyType,
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, facet::Facet,
)]
#[facet(rename_all = "snake_case")]
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
}

/// The release channel for this version
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, facet::Facet,
)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum VersionType {
    #[serde(rename = "release")]
    Release,
    #[serde(rename = "beta")]
    Beta,
    #[serde(rename = "alpha")]
    Alpha,
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, facet::Facet,
)]
#[facet(rename_all = "snake_case")]
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

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, facet::Facet,
)]
#[facet(rename_all = "snake_case")]
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
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, facet::Facet)]
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
    #[serde(
        rename = "file_type",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub file_type: Option<Option<FileType>>,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, facet::Facet)]
pub struct VersionFileHashes {
    #[serde(rename = "sha512", skip_serializing_if = "Option::is_none")]
    pub sha512: Option<String>,
    #[serde(rename = "sha1", skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, facet::Facet,
)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum FileType {
    #[serde(rename = "required-resource-pack")]
    RequiredResourcePack,
    #[serde(rename = "optional-resource-pack")]
    OptionalResourcePack,
}
