// Implementation of the Modrinth API: https://docs.modrinth.com/api/
// Previously I implemented this myself but I found this crate: https://crates.io/crates/modrinth-api
// We still do some conversion / wrapping of types to make it more ergonomic to use in our codebase, but this crate does a lot of the heavy lifting for us.
use std::path::PathBuf;

use super::as_string::AsString;
use api_types::projects::{FacetField, ModrinthResults, Project, ProjectSearch, ProjectVersion};
use modrinth_api::apis::{configuration::Configuration, projects_api::search_projects};

pub fn modrinth_configuration() -> Configuration {
    Configuration { user_agent: Some("minecraft-launcher/Kable".to_string()), ..Configuration::default() }
}
// With this custom config we now "extend" the modrinth_api crate as it has poor types for things like facets (filters) and such.

//?----------------------------------------------------------------------
//? SEARCH PROJECTS
//?----------------------------------------------------------------------

async fn search(project_type: Option<String>, project_search: ProjectSearch) -> Result<ModrinthResults, String> {
    let mut facets_str = String::new();

    if let Some(pt) = project_type {
        facets_str.push_str(&format!("project_type:{}", pt));
    }

    let extra = project_search.facets.as_string();
    if !extra.is_empty() {
        if !facets_str.is_empty() {
            facets_str.push(',');
        }
        facets_str.push_str(extra.as_str());
    }

    let facets = Some(facets_str.as_str());
    let projects = search_projects(
        &modrinth_configuration(),
        project_search.query.as_deref(),
        facets,
        Some(project_search.index.unwrap_or_default().as_string().as_str()),
        project_search.offset,
        project_search.limit,
    )
    .await
    .map_err(|e| format!("Failed to search for: {}", e))?;

    Ok(ModrinthResults {
        hits: projects.hits.into_iter().map(|h| h.into()).collect(),
        offset: projects.offset,
        limit: projects.limit,
        total_hits: projects.total_hits,
    })
}

async fn add_version_data(projects: Vec<Project>, project_search: ProjectSearch) -> Result<Vec<Project>, String> {
    let mut results = Vec::new();
    for mut project in projects {
        // loaders are under the categories facet and formatted as ["categories:fabric", "categories:forge"] for example, so we have to extract the value after "categories:" and before the next ":" if it exists.
        let loaders = project_search.facets.iter().find_map(|fg| {
            fg.facets.iter().find_map(|f| {
                if f.field == FacetField::Categories {
                    Some(f.value.split(':').nth(1).unwrap_or_default().to_string())
                } else {
                    None
                }
            })
        });
        // Similar for versions: ["versions:1.21.1", "versions:1.20.1"] for example, so we have to extract the value after "versions:" and before the next ":" if it exists.
        let game_versions = project_search.facets.iter().find_map(|fg| {
            fg.facets.iter().find_map(|f| {
                if f.field == FacetField::Version {
                    Some(f.value.split(':').nth(1).unwrap_or_default().to_string())
                } else {
                    None
                }
            })
        });
        // for featured I could not find good docs on how to use it so we just do None

        let versions = modrinth_api::apis::versions_api::get_project_versions(
            &modrinth_configuration(),
            &project.project_id,
            loaders.as_deref(),
            game_versions.as_deref(),
            None, // featured is not used for now
        )
        .await
        .map_err(|e| format!("Failed to get versions for project {}: {}", project.project_id, e))?;

        project.versions = versions.into_iter().map(|v| v.into()).collect();

        results.push(project);
    }
    Ok(results)
}

pub async fn search_mods(project_search: ProjectSearch, with_version_data: bool) -> Result<ModrinthResults, String> {
    let results = search(Some("mod".to_string()), project_search.clone()).await?;
    if with_version_data {
        add_version_data(results.hits, project_search).await.map(|hits| ModrinthResults { hits, ..results })
    } else {
        Ok(results)
    }
}

pub async fn search_resourcepacks(project_search: ProjectSearch, with_version_data: bool) -> Result<ModrinthResults, String> {
    let results = search(Some("resourcepack".to_string()), project_search.clone()).await?;
    if with_version_data {
        add_version_data(results.hits, project_search).await.map(|hits| ModrinthResults { hits, ..results })
    } else {
        Ok(results)
    }
}

pub async fn search_shaderpacks(project_search: ProjectSearch, with_version_data: bool) -> Result<ModrinthResults, String> {
    let results = search(Some("shader".to_string()), project_search.clone()).await?;
    if with_version_data {
        add_version_data(results.hits, project_search).await.map(|hits| ModrinthResults { hits, ..results })
    } else {
        Ok(results)
    }
}

pub async fn search_modpacks(project_search: ProjectSearch, with_version_data: bool) -> Result<ModrinthResults, String> {
    let results = search(Some("modpack".to_string()), project_search.clone()).await?;
    if with_version_data {
        add_version_data(results.hits, project_search).await.map(|hits| ModrinthResults { hits, ..results })
    } else {
        Ok(results)
    }
}

//?----------------------------------------------------------------------
//? DOWNLOAD PROJECT FILES
//?----------------------------------------------------------------------

/// Downloads the files of a mod project to the specified parent folder, falls back to project's latest version if version_id is None, returns a list of the downloaded filenames
/// Normally this returns a single file
pub async fn download_project(project: &Project, version_id: Option<&str>, parent_folder: PathBuf) -> Result<Vec<String>, String> {
    let client: reqwest::Client = reqwest::Client::new();
    let version: &ProjectVersion = project
        .versions
        .iter()
        .find(|v| v.id == version_id.or_else(|| project.latest_version.as_deref()).unwrap_or_default())
        .ok_or_else(|| format!("Version ID {} not found in project {}", version_id.unwrap_or_default(), project.project_id))?;

    let mut filenames = Vec::new();
    for file in &version.files {
        let url = &file.url;
        let filename = &file.filename;
        let path = parent_folder.join(filename);
        let response = client.get(url).send().await.map_err(|e| format!("Failed to download file {}: {}", filename, e))?;
        let bytes = response.bytes().await.map_err(|e| format!("Failed to read response for file {}: {}", filename, e))?;
        crate::system::fs::write(path.as_path(), &bytes, true).await?;
        filenames.push(filename.clone());
    }
    Ok(filenames)
}

//?----------------------------------------------------------------------
//? GET DEPENDENCIES - or do this in kable-specific code module(s)
//?----------------------------------------------------------------------
