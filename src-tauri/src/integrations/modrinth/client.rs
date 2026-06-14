// implementation of the Modrinth API: https://docs.modrinth.com/api/
// Previously I implemented this myself but I found this crate: https://crates.io/crates/modrinth-api

//  pub struct ProjectSearch {
//     pub query: Option<String>,
//     pub facets: Vec<FacetGroup>,
//     pub index: SearchIndex,
//     pub offset: i32,
//     pub limit: i32,
// }
use api_types::mods::ProjectSearch;
use modrinth_api::{
    apis::projects_api::search_projects, apis::Configuration, models::SearchResults,
};

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
pub const MODRINTH_CONFIGURATION: Configuration = Configuration {
    user_agent: Some("minecraft-launcher/Kable".to_owned()),
    ..Configuration::default()
};
// With this custom config we now "extend" the modrinth_api crate as it has poor types for things like facets (filters) and such.

/**
 * We automatically put the filter in for mods and then wrap the project_search parameter into:
 * search_projects(configuration: &configuration::Configuration, query: Option<&str>, facets: Option<&str>, index: Option<&str>, offset: Option<i32>, limit: Option<i32>)
 * from modrinth_api crate.
 */
async fn search(
    project_type: Option<String>,
    project_search: ProjectSearch,
) -> Result<SearchResults, String> {
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
    search_projects(
        &MODRINTH_CONFIGURATION,
        project_search.query.as_deref(),
        facets,
        project_search.index.as_ref().map(|i| i.as_str()),
        project_search.offset,
        project_search.limit,
    )
    .await
    .map_err(|e| format!("Failed to search for: {}", e))
}

pub async fn search_mods(project_search: ProjectSearch) -> Result<SearchResults, String> {
    search(Some("mod".to_string()), project_search).await
}

pub async fn search_resourcepacks(project_search: ProjectSearch) -> Result<SearchResults, String> {
    search(Some("resourcepack".to_string()), project_search).await
}
