use std::collections::{HashMap, HashSet};

use crate::integrations::modrinth::ferinth_client::{get_project, get_version_data};
use crate::Logger;

use api_types::profiles::KableProfile;
use api_types::projects::{DependencyType, KableProject, Project, ProjectType, ResolvedDependency};

use crate::features::projects::management::get_installed_project;
use crate::features::projects::smart::{is_version_compatible, resolve_version_id};

/// Resolve a project's dependencies recursively without installing anything.
///
/// Dependencies that already exist in the profile are excluded regardless
/// of whether they are enabled or disabled. Project identity is determined
/// by `(ProjectType, project_id)`, never by filename or version.
pub async fn resolve_project_dependencies(
    profile: KableProfile,
    project: Project,
    version_id: &str,
) -> Result<Vec<ResolvedDependency>, String> {
    let current_profile = crate::features::profiles::management::get_profile(&profile.id).await?;

    let root_version = project
        .versions
        .iter()
        .find(|version| version.id == version_id)
        .ok_or_else(|| format!("Version {} was not found for project {}", version_id, project.project_id))?;

    if !is_version_compatible(&current_profile, root_version, project.project_type) {
        return Err(format!(
            "Version {} of project {} is not compatible with profile {}",
            version_id, project.project_id, current_profile.metadata.name
        ));
    }

    type VersionKey = (String, String);
    type ProjectKey = (ProjectType, String);

    struct DependencyNode {
        project: KableProject,
        dependency_type: DependencyType,
        children: Vec<usize>,
    }

    struct WorkItem {
        project: Project,
        version_id: String,
        parent_node: Option<usize>,
        ancestry: HashSet<VersionKey>,
    }

    // Build a set of all projects already referenced by this profile.
    // Include disabled projects so they are not offered as new dependencies.
    let mut existing_projects: HashSet<ProjectKey> = HashSet::new();

    for project_type in [ProjectType::Mod, ProjectType::Resourcepack, ProjectType::Shader] {
        let mut filenames = current_profile.settings.by_project_type(project_type, true);

        filenames.extend(current_profile.settings.by_project_type(project_type, false));

        for filename in filenames {
            if let Some(installed_project) = get_installed_project(project_type, &filename).await? {
                existing_projects.insert((installed_project.project.project_type, installed_project.project.project_id));
            }
        }
    }

    let root_key = (project.project_id.clone(), version_id.to_string());
    let mut root_ancestry = HashSet::new();
    root_ancestry.insert(root_key);

    let mut pending =
        vec![WorkItem { project: project.clone(), version_id: version_id.to_string(), parent_node: None, ancestry: root_ancestry }];

    let mut project_cache: HashMap<String, Project> = HashMap::new();
    project_cache.insert(project.project_id.clone(), project.clone());

    let mut selected_versions: HashMap<String, String> = HashMap::new();
    selected_versions.insert(project.project_id.clone(), version_id.to_string());

    let mut nodes: Vec<DependencyNode> = Vec::new();
    let mut root_nodes: Vec<usize> = Vec::new();

    while let Some(work) = pending.pop() {
        let parent_version = work.project.versions.iter().find(|version| version.id == work.version_id).ok_or_else(|| {
            format!("Version {} was not found while resolving dependencies for {}", work.version_id, work.project.project_id)
        })?;

        for dependency in parent_version.dependencies.clone().unwrap_or_default() {
            let dependency_type = dependency.dependency_type;

            match dependency_type {
                DependencyType::Incompatible | DependencyType::Embedded => continue,
                DependencyType::Required | DependencyType::Optional | DependencyType::Other => {}
            }

            let dependency_version_id =
                dependency.version_id.as_ref().and_then(|value| value.as_deref()).filter(|id| !id.trim().is_empty()).map(str::to_owned);

            let mut dependency_project_id =
                dependency.project_id.as_ref().and_then(|value| value.as_deref()).filter(|id| !id.trim().is_empty()).map(str::to_owned);

            // Some dependencies specify only a version ID. Resolve its
            // project ID before checking whether the profile already has it.
            if dependency_project_id.is_none() {
                let Some(exact_version_id) = dependency_version_id.as_ref() else {
                    continue;
                };

                let versions = get_version_data(vec![exact_version_id.clone()]).await.map_err(|error| {
                    format!("Failed to resolve dependency version {} required by {}: {}", exact_version_id, work.project.title, error)
                })?;

                let version = versions.into_iter().find(|version| version.id == *exact_version_id).ok_or_else(|| {
                    format!("Dependency version {} required by {} was not found on Modrinth", exact_version_id, work.project.title)
                })?;

                dependency_project_id = Some(version.project_id);
            }

            let Some(dependency_project_id) = dependency_project_id else {
                continue;
            };

            let dependency_project = if let Some(cached) = project_cache.get(&dependency_project_id) {
                cached.clone()
            } else {
                let fetched = get_project(&dependency_project_id).await.map_err(|error| {
                    format!("Failed to fetch dependency project {} required by {}: {}", dependency_project_id, work.project.title, error)
                })?;

                project_cache.insert(dependency_project_id.clone(), fetched.clone());

                fetched
            };

            // An existing project satisfies the dependency regardless of
            // enabled state or installed version.
            let project_key = (dependency_project.project_type, dependency_project.project_id.clone());

            if existing_projects.contains(&project_key) {
                Logger::debug_global(
                    &format!(
                        "Skipping dependency {} ({:?}): already present in profile {}",
                        dependency_project.title, dependency_project.project_type, current_profile.metadata.name
                    ),
                    Some(current_profile.id.as_str()),
                );

                continue;
            }

            let selected_version_id = match resolve_version_id(&current_profile, &dependency_project, dependency_version_id.as_deref()) {
                Ok(version_id) => version_id.to_string(),

                Err(err) if dependency_type == DependencyType::Optional => {
                    Logger::debug_global(
                        &format!(
                            "Skipping unavailable optional dependency {} ({}) required by {}: {}",
                            dependency_project.title, dependency_project.project_id, work.project.title, err,
                        ),
                        Some(current_profile.id.as_str()),
                    );

                    continue;
                }

                Err(err) => {
                    return Err(format!(
                        "Failed to resolve {} dependency {} required by {}: {}",
                        match dependency_type {
                            DependencyType::Required => "required",
                            DependencyType::Other => "unknown-type",
                            _ => "other",
                        },
                        dependency_project.title,
                        work.project.title,
                        err,
                    ));
                }
            };

            // Reject conflicting version requirements reached through
            // different dependency branches.
            if let Some(existing_version_id) = selected_versions.get(&dependency_project_id) {
                if existing_version_id != &selected_version_id {
                    return Err(format!(
                        "Conflicting dependency versions for project {}: {} and {}",
                        dependency_project.title, existing_version_id, selected_version_id
                    ));
                }
            } else {
                selected_versions.insert(dependency_project_id.clone(), selected_version_id.clone());
            }

            let selected_version =
                dependency_project.versions.iter().find(|version| version.id == selected_version_id).ok_or_else(|| {
                    format!("Selected version {} was not found for dependency {}", selected_version_id, dependency_project.title)
                })?;

            let primary_files: Vec<_> = selected_version.files.iter().filter(|file| file.primary).collect();

            if primary_files.len() != 1 {
                return Err(format!(
                    "Version {} of dependency {} must have exactly one primary file; found {}",
                    selected_version.version_number,
                    dependency_project.title,
                    primary_files.len()
                ));
            }

            let candidate = KableProject {
                project: dependency_project.clone(),
                version_id: selected_version_id.clone(),
                filename: primary_files[0].filename.clone(),
            };

            let node_index = nodes.len();

            nodes.push(DependencyNode { project: candidate, dependency_type, children: Vec::new() });

            if let Some(parent_index) = work.parent_node {
                nodes[parent_index].children.push(node_index);
            } else {
                root_nodes.push(node_index);
            }

            let child_key = (dependency_project_id, selected_version_id.clone());

            if work.ancestry.contains(&child_key) {
                Logger::debug_global(
                    &format!(
                        "Detected dependency cycle at {} version {}; stopping expansion of this branch",
                        dependency_project.title, selected_version_id
                    ),
                    Some(current_profile.id.as_str()),
                );

                continue;
            }

            let mut child_ancestry = work.ancestry.clone();
            child_ancestry.insert(child_key);

            pending.push(WorkItem {
                project: dependency_project,
                version_id: selected_version_id,
                parent_node: Some(node_index),
                ancestry: child_ancestry,
            });
        }
    }

    fn build_tree(index: usize, nodes: &[DependencyNode]) -> ResolvedDependency {
        let node = &nodes[index];

        ResolvedDependency {
            project: node.project.clone(),
            dependency_type: node.dependency_type,
            dependencies: node.children.iter().map(|&child_index| build_tree(child_index, nodes)).collect(),
        }
    }

    let resolved_projects = root_nodes.iter().map(|&index| build_tree(index, &nodes)).collect::<Vec<_>>();

    Logger::debug_global(
        &format!(
            "Resolved {} top-level dependencies ({} dependency occurrences) for project {} version {}",
            resolved_projects.len(),
            nodes.len(),
            project.title,
            version_id
        ),
        Some(current_profile.id.as_str()),
    );

    Ok(resolved_projects)
}
