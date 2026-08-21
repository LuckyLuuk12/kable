use std::path::PathBuf;

use ferinth::{
    structures::{
        project::ProjectType as FerinthProjectType,
        search::{Facet as FerinthFacet, Sort as FerinthSort},
        version::{
            AdditionalFileType as FerinthFileType, Dependency as FerinthDependency, DependencyType as FerinthDependencyType,
            RequestedStatus as FerinthRequestedStatus, Status as FerinthStatus, Version as FerinthVersion,
            VersionFile as FerinthVersionFile, VersionType as FerinthVersionType,
        },
    },
    Ferinth,
};

use crate::Logger;
use api_types::projects::{
    ClientSide, DependencyType, Facet, FacetField, FacetGroup, FacetOperator, FileType, ModrinthResults, Project, ProjectSearch,
    ProjectType, ProjectVersion, SearchIndex, ServerSide, Status, VersionDependency, VersionFile, VersionFileHashes, VersionType,
};
use futures::future::join_all;

// ============================================================================
// CLIENT
// ============================================================================

fn client() -> Ferinth<()> {
    Ferinth::<()>::new("Kable", Some(env!("CARGO_PKG_VERSION")), Some("https://kable.kablan.nl"))
}

// ============================================================================
// FACETS
// ============================================================================

fn convert_facet(facet: &Facet) -> Option<FerinthFacet> {
    let operation = match facet.operator {
        FacetOperator::Eq => "=",
        FacetOperator::NotEq => "!=",
        FacetOperator::Greater => ">",
        FacetOperator::GreaterEq => ">=",
        FacetOperator::Less => "<",
        FacetOperator::LessEq => "<=",
    };

    let field = match facet.field {
        FacetField::ProjectType => "project_type",
        FacetField::Categories => "categories",
        FacetField::Version => "versions",
        FacetField::OpenSource => "open_source",
        FacetField::License => "license",
        FacetField::ClientSide => "client_side",
        FacetField::ServerSide => "server_side",
        FacetField::Title => "title",
        FacetField::Author => "author",
        FacetField::Follows => "follows",
        FacetField::ProjectId => "project_id",
        FacetField::Downloads => "downloads",
        FacetField::Color => "color",
        FacetField::CreatedTimestamp => "created_timestamp",
        FacetField::ModifiedTimestamp => "modified_timestamp",
        FacetField::DateCreated => "date_created",
        FacetField::DateModified => "date_modified",
    };

    Some(FerinthFacet::Custom { _type: field.to_string(), operation: operation.to_string(), value: facet.value.clone() })
}

fn convert_facets(groups: &[FacetGroup]) -> Vec<Vec<FerinthFacet>> {
    groups
        .iter()
        .map(|group| group.facets.iter().filter_map(convert_facet).collect())
        .filter(|group: &Vec<FerinthFacet>| !group.is_empty())
        .collect()
}

// ============================================================================
// SORT
// ============================================================================

fn convert_sort(index: SearchIndex) -> FerinthSort {
    match index {
        SearchIndex::Relevance => FerinthSort::Relevance,
        SearchIndex::Downloads => FerinthSort::Downloads,
        SearchIndex::Follows => FerinthSort::Follows,
        SearchIndex::Newest => FerinthSort::Newest,
        SearchIndex::Updated => FerinthSort::Updated,
    }
}

// ============================================================================
// SEARCH
// ============================================================================

async fn search(project_type: Option<String>, project_search: ProjectSearch) -> Result<ModrinthResults, String> {
    let mut facet_groups = project_search.facets.clone();

    if let Some(project_type) = project_type.clone() {
        facet_groups.insert(
            0,
            FacetGroup {
                facets: vec![Facet {
                    field: FacetField::ProjectType,
                    operator: api_types::projects::FacetOperator::Eq,
                    value: project_type,
                }],
            },
        );
    }

    let facets = convert_facets(&facet_groups);

    let query = project_search.query.clone().unwrap_or_default();

    let sort = convert_sort(project_search.index.unwrap_or_default());

    let offset = project_search.offset.unwrap_or(0);

    let limit = project_search.limit.unwrap_or(20);

    Logger::debug_global(&format!("Searching on Modrinth for {project_type:?} with sort {sort:?}, limit {limit}, offset {offset}"), None);

    let response = client()
        .search_paged(query, sort, usize::try_from(limit).unwrap_or(20), usize::try_from(offset).unwrap_or(0), facets)
        .await
        .map_err(|e| format!("Failed to search Modrinth: {e}"))?;

    Ok(ModrinthResults {
        hits: join_all(response.hits.into_iter().map(convert_search_hit)).await,

        offset: i32::try_from(response.offset).unwrap_or(0),

        limit: i32::try_from(response.limit).unwrap_or(20),

        total_hits: i32::try_from(response.total_hits).unwrap_or(0),
    })
}

pub async fn get_version_data(version_ids: Vec<String>) -> Result<Vec<ProjectVersion>, String> {
    if version_ids.is_empty() {
        return Ok(Vec::new());
    }

    let ids: Vec<&str> = version_ids.iter().map(String::as_str).collect();

    let versions = client().version_get_multiple(&ids).await.map_err(|e| format!("Failed to get Modrinth versions: {}", e))?;

    Ok(versions.into_iter().map(convert_version).collect())
}

// ============================================================================
// SEARCH HIT CONVERSION
// ============================================================================

async fn convert_search_hit(value: ferinth::structures::search::SearchHit) -> Project {
    Project {
        slug: value.slug.unwrap_or_default(),

        title: value.title,

        description: value.description,

        categories: Some(value.categories),

        client_side: convert_client_side(value.client_side),

        server_side: convert_server_side(value.server_side),

        project_type: convert_project_type(value.project_type),

        downloads: i32::try_from(value.downloads).unwrap_or(0),

        icon_url: Some(value.icon_url.map(|c| c.to_string())),

        color: Some(value.color.map(|c| i32::try_from(c).unwrap_or(0))),

        thread_id: None,

        monetization_status: None,

        project_id: value.project_id,

        author: value.author,

        display_categories: Some(value.display_categories),

        versions: get_version_data(value.game_versions).await.unwrap_or_default(),

        follows: i32::try_from(value.follows).unwrap_or(0),

        date_created: value.date_created.to_string(),

        date_modified: value.date_modified.to_string(),

        latest_version: Some(value.latest_version),

        license: value.license,

        gallery: None,

        featured_gallery: None,
    }
}

// ============================================================================
// VERSION DATA
// ============================================================================

fn extract_facet_values(search: &ProjectSearch, field: FacetField) -> Vec<String> {
    search
        .facets
        .iter()
        .flat_map(|group| group.facets.iter())
        .filter(|facet| facet.field == field)
        .filter_map(|facet| facet.value.split(':').nth(1).map(str::to_owned))
        .collect()
}

async fn add_version_data(projects: Vec<Project>, project_search: ProjectSearch) -> Result<Vec<Project>, String> {
    let loaders = extract_facet_values(&project_search, FacetField::Categories);

    let game_versions = extract_facet_values(&project_search, FacetField::Version);

    let loader_refs: Vec<&str> = loaders.iter().map(String::as_str).collect();

    let game_version_refs: Vec<&str> = game_versions.iter().map(String::as_str).collect();

    let loaders = if loader_refs.is_empty() { None } else { Some(loader_refs.as_slice()) };

    let game_versions = if game_version_refs.is_empty() { None } else { Some(game_version_refs.as_slice()) };

    let client = client();

    let mut results = Vec::with_capacity(projects.len());

    for mut project in projects {
        let versions = client
            .version_list_filtered(&project.project_id, loaders, game_versions, None)
            .await
            .map_err(|e| format!("Failed to get versions for project {}: {}", project.project_id, e))?;

        project.versions = versions.into_iter().map(convert_version).collect();
        Logger::debug_global(&format!("Added version data for project {}: {} versions", project.project_id, project.versions.len()), None);
        results.push(project);
    }

    Logger::debug_global(&format!("Added version data for {} projects", results.len()), None);

    Ok(results)
}

// ============================================================================
// PUBLIC SEARCH API
// ============================================================================

pub async fn search_mods(project_search: ProjectSearch, with_version_data: bool) -> Result<ModrinthResults, String> {
    search_typed("mod", project_search, with_version_data).await
}

pub async fn search_resourcepacks(project_search: ProjectSearch, with_version_data: bool) -> Result<ModrinthResults, String> {
    search_typed("resourcepack", project_search, with_version_data).await
}

pub async fn search_shaderpacks(project_search: ProjectSearch, with_version_data: bool) -> Result<ModrinthResults, String> {
    search_typed("shader", project_search, with_version_data).await
}

pub async fn search_modpacks(project_search: ProjectSearch, with_version_data: bool) -> Result<ModrinthResults, String> {
    search_typed("modpack", project_search, with_version_data).await
}

async fn search_typed(project_type: &str, project_search: ProjectSearch, with_version_data: bool) -> Result<ModrinthResults, String> {
    let results = search(Some(project_type.to_string()), project_search.clone()).await?;

    if !with_version_data {
        return Ok(results);
    }

    let hits = add_version_data(results.hits, project_search).await?;

    Ok(ModrinthResults { hits, ..results })
}

// ============================================================================
// CONVERSIONS
// ============================================================================

fn convert_project_type(value: FerinthProjectType) -> ProjectType {
    match value {
        FerinthProjectType::Mod => ProjectType::Mod,
        FerinthProjectType::Modpack => ProjectType::Modpack,
        FerinthProjectType::Resourcepack => ProjectType::Resourcepack,
        FerinthProjectType::Shader => ProjectType::Shader,
        FerinthProjectType::Plugin => ProjectType::Plugin,
        FerinthProjectType::Datapack => ProjectType::Datapack,
        FerinthProjectType::MinecraftJavaServer => ProjectType::MinecraftJavaServer,
        FerinthProjectType::Other => ProjectType::Other,
    }
}

pub fn convert_client_side(value: ferinth::structures::project::ProjectSupportRange) -> ClientSide {
    match value {
        ferinth::structures::project::ProjectSupportRange::Required => ClientSide::Required,
        ferinth::structures::project::ProjectSupportRange::Optional => ClientSide::Optional,
        ferinth::structures::project::ProjectSupportRange::Unsupported => ClientSide::Unsupported,
        ferinth::structures::project::SideType::Unknown => ClientSide::Optional,
    }
    // TODO: check if we should modify our types...
}

pub fn convert_server_side(value: ferinth::structures::project::ProjectSupportRange) -> ServerSide {
    match value {
        ferinth::structures::project::ProjectSupportRange::Required => ServerSide::Required,
        ferinth::structures::project::ProjectSupportRange::Optional => ServerSide::Optional,
        ferinth::structures::project::ProjectSupportRange::Unsupported => ServerSide::Unsupported,
        ferinth::structures::project::SideType::Unknown => ServerSide::Optional,
    }
    // TODO: check if we should modify our types...
}

// fn convert_monetization_status(value: FerinthMonetizationStatus) -> api_types::projects::MonetizationStatus {
//     match value {
//         FerinthMonetizationStatus::Monetized => api_types::projects::MonetizationStatus::Monetized,

//         FerinthMonetizationStatus::Demonetized => api_types::projects::MonetizationStatus::Demonetized,

//         FerinthMonetizationStatus::ForceDemonetized => api_types::projects::MonetizationStatus::ForceDemonetized,
//         FerinthMonetizationStatus::Other => api_types::projects::MonetizationStatus::Other,
//     }
// }

fn convert_version_type(value: FerinthVersionType) -> VersionType {
    match value {
        FerinthVersionType::Release => VersionType::Release,

        FerinthVersionType::Beta => VersionType::Beta,

        FerinthVersionType::Alpha => VersionType::Alpha,
    }
}

fn convert_dependency_type(value: FerinthDependencyType) -> DependencyType {
    match value {
        FerinthDependencyType::Required => DependencyType::Required,

        FerinthDependencyType::Optional => DependencyType::Optional,

        FerinthDependencyType::Incompatible => DependencyType::Incompatible,

        FerinthDependencyType::Embedded => DependencyType::Embedded,
        FerinthDependencyType::Other => DependencyType::Other,
    }
}

fn convert_status(value: FerinthStatus) -> Status {
    match value {
        FerinthStatus::Listed => Status::Listed,

        FerinthStatus::Archived => Status::Archived,

        FerinthStatus::Draft => Status::Draft,

        FerinthStatus::Unlisted => Status::Unlisted,

        FerinthStatus::Scheduled => Status::Scheduled,

        FerinthStatus::Unknown => Status::Unknown,
    }
}

fn convert_requested_status(value: FerinthRequestedStatus) -> api_types::projects::RequestedStatus {
    match value {
        FerinthRequestedStatus::Listed => api_types::projects::RequestedStatus::Listed,

        FerinthRequestedStatus::Archived => api_types::projects::RequestedStatus::Archived,

        FerinthRequestedStatus::Draft => api_types::projects::RequestedStatus::Draft,

        FerinthRequestedStatus::Unlisted => api_types::projects::RequestedStatus::Unlisted,
        FerinthRequestedStatus::Other => api_types::projects::RequestedStatus::Other,
    }
}

fn convert_file_type(value: FerinthFileType) -> FileType {
    match value {
        FerinthFileType::RequiredResourcePack => FileType::RequiredResourcePack,

        FerinthFileType::OptionalResourcePack => FileType::OptionalResourcePack,

        FerinthFileType::SourcesJar => FileType::SourcesJar,
        FerinthFileType::DevJar => FileType::DevJar,
        FerinthFileType::JavadocJar => FileType::JavadocJar,
        FerinthFileType::Signature => FileType::Signature,
        FerinthFileType::Unknown => FileType::Unknown,
    }
}

fn convert_version(value: FerinthVersion) -> ProjectVersion {
    ProjectVersion {
        name: value.name,

        version_number: value.version_number,

        changelog: value.changelog.map(Some),

        dependencies: Some(value.dependencies.into_iter().map(convert_dependency).collect()),

        game_versions: value.game_versions,

        version_type: convert_version_type(value.version_type),

        loaders: value.loaders,

        featured: value.featured,

        status: value.status.map(convert_status),

        requested_status: Some(value.requested_status.map(convert_requested_status)),

        id: value.id,

        project_id: value.project_id,

        author_id: value.author_id,

        date_published: value.date_published.to_string(),

        downloads: i32::try_from(value.downloads).unwrap_or(-1),

        changelog_url: None,

        files: value.files.into_iter().map(convert_version_file).collect(),
    }
}

fn convert_dependency(value: FerinthDependency) -> VersionDependency {
    VersionDependency {
        version_id: Some(value.version_id),

        project_id: Some(value.project_id),

        file_name: Some(value.file_name),

        dependency_type: convert_dependency_type(value.dependency_type),
    }
}

fn convert_version_file(value: FerinthVersionFile) -> VersionFile {
    VersionFile {
        hashes: VersionFileHashes { sha512: Some(value.hashes.sha512), sha1: Some(value.hashes.sha1) },

        url: value.url.to_string(),

        filename: value.filename,

        primary: value.primary,

        size: i32::try_from(value.size).unwrap_or(-1),

        file_type: Some(value.file_type.map(convert_file_type)),
    }
}

// ============================================================================
// DOWNLOAD
// ============================================================================

pub async fn download_project(project: &Project, version_id: Option<&str>, parent_folder: PathBuf) -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();

    let version_id = version_id.or(project.latest_version.as_deref()).unwrap_or_default();

    let version = project
        .versions
        .iter()
        .find(|version| version.id == version_id)
        .ok_or_else(|| format!("Version ID {} not found in project {}", version_id, project.project_id))?;

    let mut filenames = Vec::new();

    for file in &version.files {
        let path = parent_folder.join(&file.filename);

        let response = client.get(&file.url).send().await.map_err(|e| format!("Failed to download file {}: {}", file.filename, e))?;

        let bytes = response.bytes().await.map_err(|e| format!("Failed to read response for file {}: {}", file.filename, e))?;

        crate::system::fs::write(path.as_path(), &bytes, true).await?;

        filenames.push(file.filename.clone());
    }

    Ok(filenames)
}
