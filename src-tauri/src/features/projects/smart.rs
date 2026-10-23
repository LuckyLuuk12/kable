use api_types::profiles::KableProfile;
use api_types::projects::{Project, ProjectType, ProjectVersion};
use semver::Version;

/// Normalize loader names so profile loaders and Modrinth version metadata
/// can be compared consistently.
pub fn normalize_loader(loader: &str) -> String {
    let normalized = loader.trim().to_lowercase().replace(['_', '-', ' '], "");

    match normalized.as_str() {
        "irisfabric" | "irisfabricloader" | "fabric" | "quiltedfabric" => "fabric".to_string(),
        "quilt" => "quilt".to_string(),
        "neoforge" => "neoforge".to_string(),
        "forge" => "forge".to_string(),
        "vanilla" => "vanilla".to_string(),
        other => other.to_string(),
    }
}

/// Determine whether a project version is compatible with a profile's
/// Minecraft version and loader.
///
/// Loader compatibility is only checked for project types that depend on
/// a mod loader. Resource packs and shaders are not loader-restricted here.
pub fn is_version_compatible(profile: &KableProfile, version: &ProjectVersion, project_type: ProjectType) -> bool {
    let minecraft_compatible = profile
        .version
        .minecraft_version
        .as_ref()
        .map(|minecraft_version| version.game_versions.contains(minecraft_version))
        .unwrap_or(true);

    let loader_relevant = project_type == ProjectType::Mod || project_type == ProjectType::Modpack;

    let profile_loader = normalize_loader(&profile.version.loader.to_string());

    let loader_compatible = !loader_relevant || version.loaders.iter().any(|loader| normalize_loader(loader) == profile_loader);

    minecraft_compatible && loader_compatible
}

/// Convert a published timestamp to milliseconds since the Unix epoch.
///
/// Returns `i64::MIN` when the timestamp cannot be parsed, ensuring
/// malformed timestamps rank below valid timestamps during version selection.
fn version_timestamp(date: &str) -> i64 {
    if let Ok(date) = chrono::DateTime::parse_from_rfc3339(date) {
        return date.timestamp_millis();
    }

    let utc_date = date.strip_suffix(" UTC").unwrap_or(date);

    if let Ok(date) = chrono::NaiveDateTime::parse_from_str(utc_date, "%Y-%m-%d %H:%M:%S%.f") {
        return date.and_utc().timestamp_millis();
    }

    i64::MIN
}

fn parse_semver(version_number: &str) -> Option<Version> {
    let version_number = version_number.trim();
    let version_number = version_number.strip_prefix('v').unwrap_or(version_number);

    Version::parse(version_number).ok()
}

pub fn latest_compatible_version<'a>(profile: &KableProfile, project: &'a Project) -> Option<&'a ProjectVersion> {
    let compatible_versions: Vec<&ProjectVersion> =
        project.versions.iter().filter(|version| is_version_compatible(profile, version, project.project_type)).collect();

    if compatible_versions.is_empty() {
        return None;
    }

    // Use semantic ordering only when every compatible version parses.
    // This avoids mixing semantic ordering and date ordering in one comparator.
    let parsed_versions: Option<Vec<(&ProjectVersion, Version)>> = compatible_versions
        .iter()
        .map(|version| parse_semver(&version.version_number).map(|parsed| (*version, parsed)))
        .collect();

    if let Some(parsed_versions) = parsed_versions {
        return parsed_versions
            .into_iter()
            .max_by(|(a_version, a_semver), (b_version, b_semver)| {
                a_semver
                    .cmp(b_semver)
                    .then_with(|| version_timestamp(&a_version.date_published).cmp(&version_timestamp(&b_version.date_published)))
            })
            .map(|(version, _)| version);
    }

    // Fallback for projects whose compatible versions do not all use SemVer.
    compatible_versions.into_iter().max_by_key(|version| version_timestamp(&version.date_published))
}

/// Resolve an explicitly requested version or select the latest compatible
/// version when no version ID was provided.
///
/// Explicitly requested versions are validated rather than silently replaced.
pub fn resolve_version_id<'a>(profile: &KableProfile, project: &'a Project, version_id: Option<&str>) -> Result<&'a str, String> {
    if let Some(version_id) = version_id {
        let version = project
            .versions
            .iter()
            .find(|version| version.id == version_id)
            .ok_or_else(|| format!("Version {} was not found for project {}", version_id, project.project_id))?;

        if !is_version_compatible(profile, version, project.project_type) {
            return Err(format!(
                "Version {} of project {} is not compatible with profile {}",
                version_id, project.project_id, profile.metadata.name
            ));
        }

        return Ok(version.id.as_str());
    }

    latest_compatible_version(profile, project)
        .map(|version| version.id.as_str())
        .ok_or_else(|| format!("No compatible version found for project {} and profile {}", project.project_id, profile.metadata.name))
}
