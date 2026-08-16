// this code is responsible for merging two KableProfiles together, the second profile will take precedence over the first one.
// If fields can be merged (e.g. maps, lists, etc) they will, otherwise the second profile's field will overwrite the first one.
// This is used when creating a new profile from an existing one, or when importing a profile from a file.
// a new id will be generated just to be sure we have a unique id, and the last_used field will be set to the latest of the two

use api_types::profiles::{KableProfile, Projects};

pub fn merge_profiles(first: &mut KableProfile, second: &mut KableProfile) -> Result<KableProfile, String> {
    let mut merged_profile = first.clone();

    // Merge metadata
    merged_profile.metadata.name = second.metadata.name.clone();
    merged_profile.metadata.icon = second.metadata.icon.clone();
    merged_profile.metadata.description = second.metadata.description.clone();
    merged_profile.metadata.favorite = first.metadata.favorite || second.metadata.favorite;
    merged_profile.metadata.last_used = if first.metadata.last_used > second.metadata.last_used {
        first.metadata.last_used.clone()
    } else {
        second.metadata.last_used.clone()
    };
    merged_profile.metadata.created = chrono::Utc::now().to_string();
    merged_profile.metadata.times_launched = first.metadata.times_launched + second.metadata.times_launched;

    // Merge version
    merged_profile.version = second.version.clone();

    // Merge settings
    merged_profile.settings.java_args = first.settings.java_args.clone().into_iter().chain(second.settings.java_args.clone()).collect();
    merged_profile.settings.parameters_map =
        first.settings.parameters_map.clone().into_iter().chain(second.settings.parameters_map.clone()).collect();
    merged_profile.settings.enable_pack_merging = first.settings.enable_pack_merging || second.settings.enable_pack_merging;
    // note that pack order determines from 0..n in what order packs are merged so to make sure the second profile takes precedence we prepend it to the first profile's pack order
    merged_profile.settings.pack_order = second.settings.pack_order.clone().into_iter().chain(first.settings.pack_order.clone()).collect();
    merged_profile.settings.merged_packs =
        first.settings.merged_packs.clone().into_iter().chain(second.settings.merged_packs.clone()).collect();

    // Merge the enabled project.settings for mods, resourcepacks, and shaders
    for project_type in [
        api_types::projects::ProjectType::Mod,
        api_types::projects::ProjectType::Resourcepack,
        api_types::projects::ProjectType::Shader,
    ] {
        let first_projects = first.settings.from(project_type);
        let second_projects = second.settings.from(project_type);
        let merged_projects = merge_projects(first_projects, second_projects);
        merged_profile.settings.set_projects(project_type, merged_projects);
    }

    Ok(merged_profile)
}

/// merges the first.enabled with second.enabled hashsets and the first.disabled with second.disabled hashsets, and returns a new Projects struct with the merged enabled and disabled hashsets
fn merge_projects(first: Projects, second: Projects) -> Projects {
    let merged_enabled = first.enabled.into_iter().chain(second.enabled.into_iter()).collect();
    let merged_disabled = first.disabled.into_iter().chain(second.disabled.into_iter()).collect();

    Projects { enabled: merged_enabled, disabled: merged_disabled }
}
