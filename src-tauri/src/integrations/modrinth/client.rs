// implementation of the Modrinth API: https://docs.modrinth.com/api/
// Previously I implemented this myself but I found this crate: https://crates.io/crates/modrinth-api

//  pub struct ProjectSearch {
//     pub query: Option<String>,
//     pub facets: Vec<FacetGroup>,
//     pub index: SearchIndex,
//     pub offset: i32,
//     pub limit: i32,
// }
use api_types::mods::{ModrinthResults, Project, ProjectSearch, ProjectVersion};
use modrinth_api::{apis::projects_api::search_projects, apis::Configuration, models::SearchResults};

/**
 * Configuration {
 *     base_path: "https://api.modrinth.com/v2".to_owned(),
 *     user_agent: Some("OpenAPI-Generator/v2.7.0/15cf3fc/rust".to_owned()),
 *     client: reqwest::Client::new(),
 *     basic_auth: None,
 *     oauth_access_token: None,
 *     bearer_access_token: None,
 *     api_key: None,
 * }
 */
pub const MODRINTH_CONFIGURATION: Configuration =
    Configuration { user_agent: Some("minecraft-launcher/Kable".to_owned()), ..Configuration::default() };
// With this custom config we now "extend" the modrinth_api crate as it has poor types for things like facets (filters) and such.

//?----------------------------------------------------------------------
//? SEARCH PROJECTS
//?----------------------------------------------------------------------

/**
 * We automatically put the filter in for mods and then wrap the project_search parameter into:
 * search_projects(configuration: &configuration::Configuration, query: Option<&str>, facets: Option<&str>, index: Option<&str>, offset: Option<i32>, limit: Option<i32>)
 * from modrinth_api crate.
 */
async fn search(project_type: Option<String>, project_search: ProjectSearch, with_version_data: bool) -> Result<ModrinthResults, String> {
    let mut facets_str = String::new();

    if let Some(pt) = project_type {
        facets_str.push_str(&format!("project_type:{}", pt));
    }

    let extra = project_search.facets.as_str();
    if !extra.is_empty() {
        if !facets_str.is_empty() {
            facets_str.push(',');
        }
        facets_str.push_str(extra);
    }

    let facets = Some(facets_str).as_deref();
    let projects = search_projects(
        &MODRINTH_CONFIGURATION,
        project_search.query.as_deref(),
        facets,
        project_search.index.as_ref().map(|i| i.as_str()),
        project_search.offset,
        project_search.limit,
    )
    .await
    .map_err(|e| format!("Failed to search for: {}", e))?;

    // TODO: I kinda dislike how big this function got and that we do 2 requests in 1 function, we definitely want caching here!
    // Now we make a versions request to fill our custom Project struct with Vec<ProjectVersion> instead of Vec<String> for the versions field.
    // pub async fn get_project_versions(configuration: &configuration::Configuration, id_pipe_slug: &str, loaders: Option<&str>, game_versions: Option<&str>, featured: Option<bool>) -> Result<Vec<models::Version>, Error<GetProjectVersionsError>>
    // info like loaders and game_versions will be taken from the ProjectSearch parameter:
    if with_version_data {
        let loaders = project_search.facets.get("loaders").map(|v| v.join(",")).as_deref();
        let game_versions = project_search.facets.get("versions").map(|v| v.join(",")).as_deref();
        let featured = project_search.facets.get("featured").and_then(|v| v.first()).map(|s| s == "true");
    }
    // Now for each project we get the versions and fill the Project struct with Vec<ProjectVersion> instead of Vec<String> for the versions field.
    let mut results = Vec::new();
    for project in projects.hits {
        let versions = Vec::new();
        if with_version_data {
            versions = modrinth_api::apis::versions_api::get_project_versions(
                &MODRINTH_CONFIGURATION,
                &project.project_id,
                loaders,
                game_versions,
                featured,
            )
            .await
            .map_err(|e| format!("Failed to get versions for project {}: {}", project.project_id, e))?;
        }
        let project_with_versions = Project { versions: versions.into_iter().map(|v| v.into()).collect(), ..project.into_iter().collect() };

        results.push(project_with_versions);
    }
    Ok(ModrinthResults { hits: results, offset: projects.offset, limit: projects.limit, total_hits: projects.total_hits })
}

pub async fn search_mods(project_search: ProjectSearch, with_version_data: bool) -> Result<ModrinthResults, String> {
    search(Some("mod".to_string()), project_search, with_version_data).await
}

pub async fn search_resourcepacks(project_search: ProjectSearch, with_version_data: bool) -> Result<ModrinthResults, String> {
    search(Some("resourcepack".to_string()), project_search, with_version_data).await
}

pub async fn search_shaderpacks(project_search: ProjectSearch, with_version_data: bool) -> Result<ModrinthResults, String> {
    search(Some("shader".to_string()), project_search, with_version_data).await
}

pub async fn search_modpacks(project_search: ProjectSearch, with_version_data: bool) -> Result<ModrinthResults, String> {
    search(Some("modpack".to_string()), project_search, with_version_data).await
}

//?----------------------------------------------------------------------
//? DOWNLOAD PROJECT FILES
//?----------------------------------------------------------------------

pub async fn download_project(project: &Project, version_id: Option<&str>, parent_folder: PathBuf) -> Result<(), String> {
    const CLIENT: reqwest::Client = reqwest::Client::new();
    const VERSION: ProjectVersion = project
        .versions
        .iter()
        .find(|v| v.id == version_id.unwrap_or_default())
        .ok_or_else(|| format!("Version ID {} not found in project {}", version_id.unwrap_or_default(), project.project_id))?;
    VERSION.files.iter().for_each(async |file| {
        let url = &file.url;
        let filename = &file.filename;
        let path = parent_folder.join(filename);
        let response = CLIENT.get(url).send().await.map_err(|e| format!("Failed to download file {}: {}", filename, e))?;
        let mut file = std::fs::File::create(&path).map_err(|e| format!("Failed to create file {}: {}", path.display(), e))?;
        std::io::copy(&mut response.bytes().await.map_err(|e| format!("Failed to read response for file {}: {}", filename, e))?, &mut file)
            .map_err(|e| format!("Failed to write to file {}: {}", path.display(), e))?;
    });
    Ok(())
}

//?----------------------------------------------------------------------
//? DOWNLOAD PROJECT FILES
//?----------------------------------------------------------------------
