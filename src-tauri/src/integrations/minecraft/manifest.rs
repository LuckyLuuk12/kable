use crate::integrations::minecraft::versions::{Arg, Arguments, McVersionManifest};
use std::collections::HashSet;

pub async fn resolve_manifest_chain(start: McVersionManifest) -> Result<McVersionManifest, String> {
    let mut chain = Vec::new();
    let mut visited = HashSet::new();
    let mut current = start;

    loop {
        if !visited.insert(current.id.clone()) {
            return Err(format!("Circular Minecraft manifest inheritance detected at '{}'", current.id));
        }

        let Some(parent_id) = current.inherits_from.clone() else {
            chain.push(current);
            break;
        };

        let path = crate::system::fs::mc_dir()?.join(crate::constants::VERSIONS_DIR).join(&parent_id).join(format!("{parent_id}.json"));

        let raw = crate::system::fs::read_str(path)
            .await
            .map_err(|e| format!("Failed to read parent manifest '{}' inherited by '{}': {}", parent_id, current.id, e))?;

        let parent = crate::integrations::minecraft::versions::load_version_manifest(raw)
            .await
            .map_err(|e| format!("Failed to parse parent manifest '{}' inherited by '{}': {}", parent_id, current.id, e))?;

        chain.push(current);
        current = parent;
    }

    merge_manifest_chain(chain)
}

fn merge_manifest_chain(mut chain: Vec<McVersionManifest>) -> Result<McVersionManifest, String> {
    if chain.is_empty() {
        return Err("Cannot merge an empty Minecraft manifest chain".to_string());
    }

    // The chain is collected child-first. Merge parent-first.
    chain.reverse();

    let mut iter = chain.into_iter();

    let mut base = iter.next().ok_or_else(|| "Cannot merge an empty Minecraft manifest chain".to_string())?;

    for child in iter {
        // Preserve the identity and type of the most-derived manifest.
        base.id = child.id.clone();

        if !child.r#type.is_empty() {
            base.r#type = child.r#type.clone();
        }

        // Child-provided values override inherited values.
        if child.main_class.is_some() {
            base.main_class = child.main_class.clone();
        }

        if child.time.is_some() {
            base.time = child.time.clone();
        }

        if child.release_time.is_some() {
            base.release_time = child.release_time.clone();
        }

        if child.minimum_launcher_version.is_some() {
            base.minimum_launcher_version = child.minimum_launcher_version;
        }

        if child.assets.is_some() {
            base.assets = child.assets.clone();
        }

        if let Some(child_asset_index) = &child.asset_index {
            base.asset_index = Some(child_asset_index.clone());
        }

        if child.compliance_level.is_some() {
            base.compliance_level = child.compliance_level;
        }

        if child.java_version.is_some() {
            base.java_version = child.java_version.clone();
        }

        if let Some(child_logging) = &child.logging {
            base.logging = Some(child_logging.clone());
        }

        if child.comment.is_some() {
            base.comment = child.comment.clone();
        }

        // Merge client/server downloads independently so a child
        // providing one artifact does not remove the other.
        if let Some(child_downloads) = &child.downloads {
            let downloads =
                base.downloads.get_or_insert(crate::integrations::minecraft::versions::Downloads { client: None, server: None });

            if child_downloads.client.is_some() {
                downloads.client = child_downloads.client.clone();
            }

            if child_downloads.server.is_some() {
                downloads.server = child_downloads.server.clone();
            }
        }

        // Preserve parent libraries and append child libraries.
        // Artifact conflicts still need to be handled by the library resolver.
        if let Some(child_libraries) = &child.libraries {
            base.libraries.get_or_insert_with(Vec::new).extend(child_libraries.clone());
        }

        // Merge argument sets without silently losing either side.
        match (&mut base.arguments, &child.arguments) {
            (Some(base_args), Some(child_args)) => {
                merge_arguments(base_args, child_args)
                    .map_err(|e| format!("Failed to merge arguments for manifest '{}': {}", child.id, e))?;
            }

            (None, Some(child_args)) => {
                base.arguments = Some(child_args.clone());
            }

            (_, None) => {}
        }

        // Unknown manifest fields are retained. Child values take precedence.
        base.extra.extend(child.extra.clone());
    }

    // The result represents a fully resolved manifest.
    base.inherits_from = None;

    Ok(base)
}

fn merge_arguments(base: &mut Arguments, child: &Arguments) -> Result<(), String> {
    // Take ownership of the current arguments so variants can be safely
    // converted without borrowing their contents while replacing the enum.
    let existing = std::mem::replace(base, Arguments::Mixed(Vec::new()));

    let merged = match (existing, child) {
        (Arguments::Structured(mut base_args), Arguments::Structured(child_args)) => {
            if let Some(jvm) = &child_args.jvm {
                base_args.jvm.get_or_insert_with(Vec::new).extend(jvm.clone());
            }

            if let Some(game) = &child_args.game {
                base_args.game.get_or_insert_with(Vec::new).extend(game.clone());
            }

            if let Some(default_user_jvm) = &child_args.default_user_jvm {
                base_args.default_user_jvm.get_or_insert_with(Vec::new).extend(default_user_jvm.clone());
            }

            Arguments::Structured(base_args)
        }

        (Arguments::Mixed(mut base_args), Arguments::Mixed(child_args)) => {
            base_args.extend(child_args.iter().cloned());
            Arguments::Mixed(base_args)
        }

        (Arguments::Flat(mut base_args), Arguments::Flat(child_args)) => {
            base_args.extend(child_args.iter().cloned());
            Arguments::Flat(base_args)
        }

        (Arguments::Flat(base_values), Arguments::Mixed(child_args)) => {
            let mut merged: Vec<Arg> = base_values.into_iter().map(Arg::String).collect();

            merged.extend(child_args.iter().cloned());

            Arguments::Mixed(merged)
        }

        (Arguments::Mixed(mut base_args), Arguments::Flat(child_values)) => {
            base_args.extend(child_values.iter().cloned().map(Arg::String));
            Arguments::Mixed(base_args)
        }

        (original, _) => {
            // Restore the original value if the representations are
            // incompatible. Do not leave the placeholder in `base`.
            *base = original;

            return Err(
                "Cannot merge structured JVM/game arguments with a legacy argument array without knowing the target argument category"
                    .to_string(),
            );
        }
    };

    *base = merged;
    Ok(())
}
