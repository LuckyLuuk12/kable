// This code merges two KableProfiles together, with the second profile taking
// precedence over the first wherever values conflict.
//
// Mergeable fields are combined according to their specific merge policies.
// The result represents a new profile with a unique ID and fresh creation time.
// Filesystem operations and persistence are handled elsewhere.
use api_types::profiles::{KableProfile, Projects};
use std::collections::HashMap;

pub fn merge_profiles(first: &mut KableProfile, second: &mut KableProfile) -> Result<KableProfile, String> {
    let mut merged_profile = first.clone();

    // Merge metadata.
    // The second profile takes precedence for descriptive metadata.
    merged_profile.id = uuid::Uuid::new_v4().to_string();
    merged_profile.metadata.name = second.metadata.name.clone();
    merged_profile.metadata.icon = second.metadata.icon.clone();
    merged_profile.metadata.description = second.metadata.description.clone();
    merged_profile.metadata.favorite = first.metadata.favorite || second.metadata.favorite;
    merged_profile.metadata.last_used = std::cmp::max(first.metadata.last_used.clone(), second.metadata.last_used.clone());
    merged_profile.metadata.created = chrono::Utc::now().to_rfc3339();
    merged_profile.metadata.times_launched = first.metadata.times_launched + second.metadata.times_launched;
    merged_profile.metadata.total_time_played_ms = first.metadata.total_time_played_ms + second.metadata.total_time_played_ms;

    // The second profile determines the Minecraft version and loader.
    merged_profile.version = second.version.clone();

    // Merge settings.
    // Java arguments are concatenated, preserving the order of both profiles.
    merged_profile.settings.java_args = merge_java_args(&first.settings.java_args, &second.settings.java_args);

    // Values from the second profile overwrite duplicate keys from the first.
    merged_profile.settings.parameters_map = first.settings.parameters_map.clone();
    merged_profile.settings.parameters_map.extend(second.settings.parameters_map.clone());

    merged_profile.settings.enable_pack_merging = first.settings.enable_pack_merging || second.settings.enable_pack_merging;

    // The second profile's pack order takes precedence.
    merged_profile.settings.pack_order = second.settings.pack_order.iter().chain(first.settings.pack_order.iter()).cloned().collect();

    // Preserve both merged-pack lists, with the first profile's entries first.
    merged_profile.settings.merged_packs = first.settings.merged_packs.iter().chain(second.settings.merged_packs.iter()).cloned().collect();

    // Merge project states, ensuring enabled and disabled sets cannot conflict.
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

/// Merges two project configurations.
///
/// The second profile takes precedence when a project occurs in both
/// profiles with conflicting enabled/disabled states. Projects that occur
/// only in the first profile retain their original state.
fn merge_projects(first: Projects, second: Projects) -> Projects {
    let mut enabled = first.enabled;
    let mut disabled = first.disabled;

    // Apply the second profile's enabled projects.
    for project in second.enabled {
        disabled.remove(&project);
        enabled.insert(project);
    }

    // Apply the second profile's disabled projects.
    for project in second.disabled {
        enabled.remove(&project);
        disabled.insert(project);
    }

    Projects { enabled, disabled }
}

fn merge_java_args(first: &[String], second: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    let mut positions: HashMap<String, usize> = HashMap::new();

    let mut args = first.iter().chain(second.iter()).peekable();

    while let Some(arg) = args.next() {
        let arg = arg.as_str();

        // Options whose values are provided as the next argument.
        if java_arg_takes_separate_value(arg) {
            let value = args.next().map(String::as_str);

            let Some(value) = value else {
                // Preserve malformed or incomplete input instead of silently dropping it.
                result.push(arg.to_string());
                continue;
            };

            let key = java_arg_key(arg);

            if let Some(&index) = positions.get(&key) {
                result[index] = arg.to_string();
                result.insert(index + 1, value.to_string());

                // Rebuild positions because insertion shifts later indexes.
                positions.clear();
                for (i, existing) in result.iter().enumerate() {
                    if !java_arg_takes_separate_value(existing) {
                        positions.insert(java_arg_key(existing), i);
                    }
                }
            } else {
                positions.insert(key, result.len());
                result.push(arg.to_string());
                result.push(value.to_string());
            }

            continue;
        }

        let key = java_arg_key(arg);

        if let Some(&index) = positions.get(&key) {
            result[index] = arg.to_string();
        } else {
            positions.insert(key, result.len());
            result.push(arg.to_string());
        }
    }

    result
}

fn java_arg_takes_separate_value(arg: &str) -> bool {
    matches!(
        arg,
        "-cp"
            | "-classpath"
            | "--class-path"
            | "-p"
            | "--module-path"
            | "--upgrade-module-path"
            | "--add-modules"
            | "--limit-modules"
            | "--add-exports"
            | "--add-opens"
            | "--add-reads"
            | "--patch-module"
            | "--module"
            | "-m"
            | "--source"
            | "-encoding"
            | "-javaagent"
            | "-agentlib"
            | "-agentpath"
            | "-Xlog"
    )
}

fn java_arg_key(arg: &str) -> String {
    // Heap sizing.
    for prefix in ["-Xmx", "-Xms", "-Xmn"] {
        if arg.starts_with(prefix) {
            return prefix.to_string();
        }
    }

    // System properties: -Dname=value.
    if let Some(property) = arg.strip_prefix("-D") {
        let key = property.split_once('=').map_or(property, |(key, _)| key);
        return format!("-D{key}");
    }

    // Boolean HotSpot flags: -XX:+Name and -XX:-Name.
    for prefix in ["-XX:+", "-XX:-"] {
        if let Some(name) = arg.strip_prefix(prefix) {
            return format!("-XX:{name}");
        }
    }

    // Value-bearing HotSpot flags: -XX:Name=value.
    if let Some(option) = arg.strip_prefix("-XX:") {
        let key = option.split_once('=').map_or(option, |(key, _)| key);

        return format!("-XX:{key}");
    }

    // Logging configuration.
    if let Some(value) = arg.strip_prefix("-Xlog:") {
        let selector = value.split_once('=').map_or(value, |(key, _)| key);
        return format!("-Xlog:{selector}");
    }

    // Java agents. Preserve different agents, but replace duplicate agent paths.
    for prefix in ["-javaagent:", "-agentlib:", "-agentpath:"] {
        if let Some(value) = arg.strip_prefix(prefix) {
            let agent = value.split_once('=').map_or(value, |(key, _)| key);
            return format!("{prefix}{agent}");
        }
    }

    // Module-related options support both --option=value and --option value.
    for option in [
        "--module-path",
        "--upgrade-module-path",
        "--add-modules",
        "--limit-modules",
        "--add-exports",
        "--add-opens",
        "--add-reads",
        "--patch-module",
        "--class-path",
        "--module",
        "--source",
    ] {
        if arg == option {
            return option.to_string();
        }

        if arg.strip_prefix(option).is_some_and(|suffix| suffix.starts_with('=')) {
            return option.to_string();
        }
    }

    // Assertion options.
    for prefix in ["-ea:", "-enableassertions:", "-da:", "-disableassertions:"] {
        if let Some(scope) = arg.strip_prefix(prefix) {
            return format!("{prefix}{scope}");
        }
    }

    if matches!(arg, "-ea" | "-enableassertions" | "-da" | "-disableassertions") {
        return arg.to_string();
    }

    // Other options that can carry values.
    for prefix in [
        "-Xss",
        "-Xoss",
        "-XX:MaxRAMPercentage=",
        "-XX:MinRAMPercentage=",
        "-XX:InitialRAMPercentage=",
        "-XX:ActiveProcessorCount=",
    ] {
        if arg.starts_with(prefix) {
            if let Some((option, _)) = arg.split_once('=') {
                return option.to_string();
            }

            if prefix.ends_with('=') {
                return prefix.trim_end_matches('=').to_string();
            }

            return prefix.to_string();
        }
    }

    // Unrecognized options are deduplicated only when their complete strings match.
    arg.to_string()
}
