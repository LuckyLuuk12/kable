use crate::integrations::minecraft::versions::{Arg, Arguments, McVersionManifest};

pub async fn resolve_manifest_chain(start: McVersionManifest) -> Result<McVersionManifest, String> {
    let mut chain = Vec::new();

    let mut current = start;

    while let Some(parent_id) = current.inherits_from.clone() {
        let path = crate::system::fs::mc_dir()?.join(crate::constants::VERSIONS_DIR).join(&parent_id).join(format!("{parent_id}.json"));

        let raw = crate::system::fs::read_str(path).await?;

        let parent: McVersionManifest = crate::integrations::minecraft::versions::load_version_manifest(raw).await?;

        chain.push(current);
        current = parent;
    }

    chain.push(current);

    // now merge bottom-up (important order)
    merge_manifest_chain(chain)
}

fn merge_manifest_chain(mut chain: Vec<McVersionManifest>) -> Result<McVersionManifest, String> {
    chain.reverse();

    let mut base = chain[0].clone();

    for m in chain.iter().skip(1) {
        // merge libraries
        if let Some(libs) = &m.libraries {
            let base_libs = base.libraries.get_or_insert_with(Vec::new);
            base_libs.extend(libs.clone());
        }

        // merge arguments
        match (&mut base.arguments, &m.arguments) {
            (Some(base_args), Some(new_args)) => {
                merge_arguments(base_args, new_args);
            }

            (None, Some(new_args)) => {
                base.arguments = Some(new_args.clone());
            }

            _ => {}
        }

        // overwrite fields (typical MC behavior)
        if m.asset_index.is_some() {
            base.asset_index = m.asset_index.clone();
        }

        if m.logging.is_some() {
            base.logging = m.logging.clone();
        }
    }

    Ok(base)
}

fn merge_arguments(base: &mut Arguments, new: &Arguments) {
    match new {
        Arguments::Structured(n) => match base {
            Arguments::Structured(b) => {
                if let Some(jvm) = &n.jvm {
                    b.jvm.get_or_insert_with(Vec::new).extend(jvm.clone());
                }

                if let Some(game) = &n.game {
                    b.game.get_or_insert_with(Vec::new).extend(game.clone());
                }
            }

            Arguments::Flat(_) | Arguments::Mixed(_) => {
                *base = Arguments::Structured(n.clone());
            }
        },

        Arguments::Flat(n) => match base {
            Arguments::Flat(b) => {
                b.extend(n.clone());
            }

            Arguments::Mixed(b) => {
                b.extend(n.iter().cloned().map(Arg::String));
            }

            Arguments::Structured(_) => {
                *base = Arguments::Flat(n.clone());
            }
        },

        Arguments::Mixed(n) => match base {
            Arguments::Mixed(b) => {
                b.extend(n.clone());
            }

            Arguments::Flat(b) => {
                let mut converted = b.clone();
                for arg in n {
                    if let Arg::String(s) = arg {
                        converted.push(s.clone())
                    }
                }
                *base = Arguments::Flat(converted);
            }

            Arguments::Structured(_) => {
                *base = Arguments::Mixed(n.clone());
            }
        },
    }
}
