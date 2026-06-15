// Placeholder: mod browser backend - searching, next page, filtering, etc.
use crate::integrations::modrinth::client::search_mods;
use api_types::mods::{Facet, FacetField, FacetGroup, FacetOperator, ModrinthResults, ProjectSearch};
use api_types::profiles::KableProfile;

/// Given an installation search for mods, apply version and loader filters and return the results
pub async fn browse(profile: KableProfile, search: ProjectSearch, smart_filter: bool) -> Result<ModrinthResults, String> {
    let mut search_with_facets = search.clone();
    if smart_filter {
        // If smart_filter is enabled, we add facets to the search to filter by the profile's loader and Minecraft version
        // docs https://docs.modrinth.com/api/operations/searchprojects/ are unclear on how to filter loader-versions so we just do loader & mc version for now
        let facets: vec![Facet { field: FacetField::Categories, operator: FacetOperator::Eq, value: profile.version.loader.to_string() }];
        if let Some(minecraft_version) = profile.version.minecraft_version {
            facets.push(Facet { field: FacetField::Version, operator: FacetOperator::GreaterEq, value: minecraft_version });
        }
        // add this group to the existing facets in the search
        search_with_facets.facets.push(FacetGroup { facets });
    }
    // Then we perform the search with complete VersionData included
    search_mods(search_with_facets, true).await
}
