// Placeholder: mod browser backend - searching, next page, filtering, etc.
use crate::integrations::modrinth::ferinth_client::{search_mods, search_resourcepacks, search_shaderpacks};
use api_types::profiles::KableProfile;
use api_types::projects::{Facet, FacetField, FacetGroup, FacetOperator, ModrinthResults, ProjectSearch, ProjectType};
use kable_macros::persistent_cache;

/// Given an installation search for mods, apply version and loader filters and return the results
pub async fn browse(
    profile: KableProfile,
    search: ProjectSearch,
    smart_filter: bool,
    project_type: ProjectType,
) -> Result<ModrinthResults, String> {
    browse_cached(search, project_type, smart_filter, profile.version.loader.to_string(), profile.version.minecraft_version).await
}

#[persistent_cache(parent = "projects", ttl_secs = 302400)]
async fn browse_cached(
    mut search: ProjectSearch,
    project_type: ProjectType,
    smart_filter: bool,
    loader: String,
    minecraft_version: Option<String>,
) -> Result<ModrinthResults, String> {
    if smart_filter {
        let mut facets = Vec::new();

        facets.push(Facet { field: FacetField::Categories, operator: FacetOperator::Eq, value: loader });

        if let Some(minecraft_version) = minecraft_version {
            facets.push(Facet { field: FacetField::Version, operator: FacetOperator::GreaterEq, value: minecraft_version });
        }

        search.facets.push(FacetGroup { facets });
    }

    match project_type {
        ProjectType::Mod => search_mods(search, true).await,
        ProjectType::Resourcepack => search_resourcepacks(search, true).await,
        ProjectType::Shader => search_shaderpacks(search, true).await,
        _ => Err("Unsupported project type for browsing".to_string()),
    }
}
