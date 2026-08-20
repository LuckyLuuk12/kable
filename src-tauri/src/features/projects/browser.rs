// Placeholder: mod browser backend - searching, next page, filtering, etc.
use crate::integrations::modrinth::ferinth_client::{search_mods, search_resourcepacks, search_shaderpacks};
use api_types::profiles::KableProfile;
use api_types::projects::{Facet, FacetField, FacetGroup, FacetOperator, ModrinthResults, ProjectSearch, ProjectType};
use kable_macros::persistent_cache;

/// Given an installation search for mods, apply version and loader filters and return the results
#[persistent_cache(parent = "projects", ttl_secs = 302400)] // cache for 3.5 days
pub async fn browse(
    profile: KableProfile,
    search: ProjectSearch,
    smart_filter: bool,
    project_type: ProjectType,
) -> Result<ModrinthResults, String> {
    let mut search_with_facets = search.clone();
    if smart_filter {
        // If smart_filter is enabled, we add facets to the search to filter by the profile's loader and Minecraft version
        // docs https://docs.modrinth.com/api/operations/searchprojects/ are unclear on how to filter loader-versions so we just do loader & mc version for now
        let mut facets = Vec::new();
        facets.push(Facet { field: FacetField::Categories, operator: FacetOperator::Eq, value: profile.version.loader.to_string() });
        if let Some(minecraft_version) = profile.version.minecraft_version {
            facets.push(Facet { field: FacetField::Version, operator: FacetOperator::GreaterEq, value: minecraft_version });
        }
        // add this group to the existing facets in the search
        search_with_facets.facets.push(FacetGroup { facets });
    }
    // Then we perform the search with complete VersionData included based on the project type (mods, shaderpacks, resourcepacks)
    match project_type {
        ProjectType::Mod => search_mods(search_with_facets, true).await,
        ProjectType::Resourcepack => search_resourcepacks(search_with_facets, true).await,
        ProjectType::Shader => search_shaderpacks(search_with_facets, true).await,
        _ => Err("Unsupported project type for browsing".to_string()),
    }
}
