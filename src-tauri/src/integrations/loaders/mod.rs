use crate::constants::{FABRIC_META_URL, FORGE_MAVEN_METADATA_URL, MINECRAFT_VERSION_MANIFEST_URL, NEOFORGE_VERSION_URL, QUILT_META_URL};
use api_types::profiles::{LoaderKind, ProfileVersion, Versions};
use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;
use kable_macros::persistent_cache;
use serde_json::Value;

fn loader_manifest_url(loader: LoaderKind) -> Result<String, String> {
    match loader {
        // Fetch: https://launchermeta.mojang.com/mc/game/version_manifest.json
        LoaderKind::Vanilla => Ok(MINECRAFT_VERSION_MANIFEST_URL.to_string()),
        // Fetch: https://meta.fabricmc.net/v2/versions/loader
        LoaderKind::Fabric => Ok(format!("{}/v2/versions/loader", FABRIC_META_URL)),
        // IrisFabric is basically just fabric with IrisShaders mod installed, so we can use the same manifest as Fabric
        LoaderKind::IrisFabric => Ok(format!("{}/v2/versions/loader", FABRIC_META_URL)),
        // Fetch: https://meta.quiltmc.org/v3/versions/loader
        LoaderKind::Quilt => Ok(format!("{}/v3/versions/loader", QUILT_META_URL)),
        // Fetch: https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml
        LoaderKind::Forge => Ok(FORGE_MAVEN_METADATA_URL.to_string()),
        // Fetch: https://maven.neoforged.net/api/maven/versions/releases/net%2Fneoforged%2Fneoforge
        LoaderKind::NeoForge => Ok(NEOFORGE_VERSION_URL.to_string()),
    }
}

/// Return /v2/versions/game routes for supporting loaders
/// ```
/// [
///  {
///    "version": "26.2-rc-2",
///    "stable": false
///  },
///  ...
/// ]
/// ```
fn game_manifest_url(loader: LoaderKind) -> Result<String, String> {
    match loader {
        LoaderKind::Fabric => Ok(format!("{}/v2/versions/game", FABRIC_META_URL)),
        LoaderKind::IrisFabric => Ok(format!("{}/v2/versions/game", FABRIC_META_URL)),
        LoaderKind::Quilt => Ok(format!("{}/v3/versions/game", QUILT_META_URL)),
        _ => Err("Unsupported loader".into()),
    }
}

/// Get all versions of all loaders merged
pub async fn get_versions() -> Result<Versions, String> {
    // 1. We fetch do all fetches in parallel and then merge the results, this is much faster than fetching them sequentially
    let loaders = vec![
        LoaderKind::Vanilla,
        LoaderKind::Fabric,
        LoaderKind::IrisFabric,
        LoaderKind::Quilt,
        LoaderKind::Forge,
        LoaderKind::NeoForge,
    ];
    let mut futures = FuturesUnordered::new();
    for loader in loaders {
        futures.push(get_versions_of(loader));
    }

    let mut all_versions = Vec::new();

    while let Some(result) = futures.next().await {
        match result {
            Ok(v) => all_versions.extend(v.0),
            Err(_) => {
                // log / ignore
            }
        }
    }

    // TODO: we might want to "enrich" the data of non-vanilla versions with url, releaseTime, etc from the corresponding vanilla version if we can find it??

    Ok(Versions(all_versions))
}

/// Parse loader manifest as Versions (List of ProfileVersion), display name needs fancy / custom formatting per loader e.g., "iris-fabric-loader-0.18.4-1.21.11"
pub async fn get_versions_of(loader: LoaderKind) -> Result<Versions, String> {
    match loader {
        LoaderKind::Vanilla => get_vanilla_versions().await,
        LoaderKind::Fabric => get_meta_versions(LoaderKind::Fabric).await,
        LoaderKind::IrisFabric => get_meta_versions(LoaderKind::IrisFabric).await,
        LoaderKind::Quilt => get_meta_versions(LoaderKind::Quilt).await,
        LoaderKind::Forge => get_forge_versions().await,
        LoaderKind::NeoForge => get_neoforge_versions().await,
    }
}

pub async fn get_version(id: String) -> Result<ProfileVersion, String> {
    let versions = get_versions().await?;

    versions.0.into_iter().find(|v| v.id == id).ok_or_else(|| format!("Version not found: {}", id))
}

pub async fn get_version_data(loader: LoaderKind, version_id: String, include_unstable: bool) -> Result<ProfileVersion, String> {
    let versions = get_versions_of(loader).await?;

    let version_data = versions.0.into_iter().find(|v| v.id == version_id && (include_unstable || v.stable.unwrap_or(false)));

    version_data.ok_or_else(|| format!("Version data not found for {:?} {}", loader, version_id))
}

/// Obviously we also want to know the vanilla versions, example:
/// `{"latest": {"release": "26.1.2", "snapshot": "26.2-rc-2"}, "versions": [{"id": "26.2-rc-2", "type": "snapshot", "url": "https://piston-meta.mojang.com/v1/packages/9c01b04a6ffd22f6ef4c1dfa8fab9850648fb9dd/26.2-rc-2.json", "time": "2026-06-12T11:41:39+00:00", "releaseTime": "2026-06-12T11:32:28+00:00"}`
#[persistent_cache(parent = "loader-versions", ttl_secs = 604800)] // cache for 1 week
async fn get_vanilla_versions() -> Result<Versions, String> {
    let manifest = reqwest::get(MINECRAFT_VERSION_MANIFEST_URL)
        .await
        .map_err(|e| e.to_string())?
        .json::<Value>()
        .await
        .map_err(|e| e.to_string())?;
    // Parse the json:
    let mut versions = Vec::new();
    if let Some(versions_arr) = manifest.get("versions").and_then(|v| v.as_array()) {
        for v in versions_arr {
            let id = v.get("id").and_then(|i| i.as_str()).unwrap_or_default().to_string();
            let release_type = v.get("type").and_then(|t| t.as_str()).unwrap_or_default();

            versions.push(ProfileVersion {
                id: id.clone(),
                display_name: id.clone(),
                loader: LoaderKind::Vanilla,
                minecraft_version: Some(id),
                loader_version: None,
                version_type: match release_type {
                    "release" => Some(api_types::profiles::ProfileVersionType::Release),
                    "snapshot" => Some(api_types::profiles::ProfileVersionType::Snapshot),
                    "old_beta" => Some(api_types::profiles::ProfileVersionType::OldBeta),
                    "old_alpha" => Some(api_types::profiles::ProfileVersionType::OldAlpha),
                    _ => None,
                },
                stable: Some(release_type == "release"),
                release_time: v.get("releaseTime").and_then(|t| t.as_str()).map(|s| s.to_string()),
                updated_time: v.get("updateTime").and_then(|t| t.as_str()).map(|s| s.to_string()),
                url: v.get("url").and_then(|u| u.as_str()).map(|s| s.to_string()),
                sha1: v.get("sha1").and_then(|s| s.as_str()).map(|s| s.to_string()),
                compliance_level: None,
                recommended: None,
            });
        }
    }
    Ok(Versions(versions))
}

/// Fabric & Quilt use what they call their "meta api" which provides various routes to get the info we need
#[persistent_cache(parent = "loader-versions", ttl_secs = 604800)] // cache for 1 week
async fn get_meta_versions(loader: LoaderKind) -> Result<Versions, String> {
    let game_response = reqwest::get(game_manifest_url(loader)?).await.map_err(|e| e.to_string())?;
    // for all supported minecraft versions fetch the supported loader versions and merge them into one manifest with all info:
    let game_manifest: Value = game_response.json().await.map_err(|e| e.to_string())?;
    let mut versions = Vec::new();

    // loop over objects in game_manifest, these are not in json by a key but just as array with objects with version field:
    if let Some(game_versions) = game_manifest.as_array() {
        for game_version in game_versions {
            let mc_version = game_version.get("version").and_then(|v| v.as_str());
            if let Some(mc_version) = mc_version {
                let loader_response =
                    reqwest::get(format!("{}/{}", &loader_manifest_url(loader)?, mc_version)).await.map_err(|e| e.to_string())?;

                // All manifest don't exactly match the ProfileVersion struct so we do some custom parsing here
                let manifest: Value = loader_response.json().await.map_err(|e| e.to_string())?;

                // Process the manifest and add versions to the list
                let loader_ver = manifest.get("loader").and_then(|l| l.get("version")).and_then(|i| i.as_str()).unwrap_or_default();
                let stable = manifest.get("loader").and_then(|l| l.get("stable")).and_then(|i| i.as_bool()).unwrap_or_default();
                versions.push(ProfileVersion {
                    // id is like "fabric-loader-0.18.4-1.21.11"
                    id: format!(
                        "{}-loader-{}-{}",
                        match loader {
                            LoaderKind::Fabric => "fabric",
                            LoaderKind::IrisFabric => "iris-fabric",
                            LoaderKind::Quilt => "quilt",
                            _ => "unknown",
                        },
                        loader_ver,
                        mc_version
                    ),
                    // Displayy name is like "Fabric 0.18.4 for Minecraft 1.21.11"
                    display_name: format!(
                        "{} {} for Minecraft {}",
                        match loader {
                            LoaderKind::Fabric => "Fabric",
                            LoaderKind::IrisFabric => "Iris Fabric",
                            LoaderKind::Quilt => "Quilt",
                            _ => "Unknown",
                        },
                        loader_ver,
                        mc_version
                    ),
                    loader,
                    minecraft_version: Some(mc_version.to_string()),
                    loader_version: Some(loader_ver.to_string()),
                    version_type: None,
                    stable: Some(stable),
                    release_time: None,
                    updated_time: None,
                    url: None,
                    // Quilt has a loader.hashes.sha1 field but fabric not so only set sha1 for Quilt, otherwise None
                    sha1: if loader == LoaderKind::Quilt {
                        manifest
                            .get("loader")
                            .and_then(|l| l.get("hashes"))
                            .and_then(|h| h.get("sha1"))
                            .and_then(|s| s.as_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    },
                    compliance_level: None,
                    recommended: None,
                });
            }
        }
    }

    Ok(Versions(versions))
}

/// here, https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json, forge has a
/// json object, with key being minecraft versions and then for each key a list of forge versiosn formatted as
/// mcversion-forgeversion, e.g. 1.19.2-43.2.0, we want to parse this into a list of ProfileVersion with loader = Forge and minecraft_version and loader_version set accordingly, display name can be like "Forge 43.2.0 for Minecraft 1.19.2"
#[persistent_cache(parent = "loader-versions", ttl_secs = 604800)] // cache for 1 week
async fn get_forge_versions() -> Result<Versions, String> {
    let response = reqwest::get(loader_manifest_url(LoaderKind::Forge)?).await.map_err(|e| e.to_string())?;
    let manifest: Value = response.json().await.map_err(|e| e.to_string())?;

    let mut versions = Vec::new();
    if let Some(mc_versions) = manifest.get("versions").and_then(|v| v.as_object()) {
        for (mc_version, forge_versions) in mc_versions {
            if let Some(forge_versions) = forge_versions.as_array() {
                for forge_version in forge_versions {
                    if let Some(forge_version_str) = forge_version.as_str() {
                        versions.push(ProfileVersion {
                            id: format!("{}-forge-{}", mc_version, forge_version_str),
                            display_name: format!("Forge {} for Minecraft {}", forge_version_str, mc_version),
                            loader: LoaderKind::Forge,
                            minecraft_version: Some(mc_version.to_string()),
                            loader_version: Some(forge_version_str.to_string()),
                            version_type: None,
                            stable: None,
                            release_time: None,
                            updated_time: None,
                            url: None,
                            sha1: None,
                            compliance_level: None,
                            // I don't recommend forge nor neoforge so we just default to false here..
                            recommended: Some(false),
                        });
                    }
                }
            }
        }
    }

    Ok(Versions(versions))
}

/// now neoforge is a pain in the a$$, they have an api at https://maven.neoforged.net/api/maven/versions/releases/net%2Fneoforged%2Fneoforge
/// which returns a json object with "isSnapshot" and "versions" fields, the versions field is just a list of all versions neoforge has which
/// are formatted like: a.b.c[-add] with a being the minecraft version after the the first dot (so 1.21.1 -> 21), b being after the second dot
/// (so 1.21.4 -> 4) and c being the neoforge version which increments, and sometimes there is a dash with additional info like beta, alpha.1+snapshot+1, etc...
/// THIS STOPS after minecraft 26.x, minecraft dropped their 1.b.c semver and now neoforge uses just the <full mc version>.c[-add] format.
#[persistent_cache(parent = "loader-versions", ttl_secs = 604800)] // cache for 1 week
async fn get_neoforge_versions() -> Result<Versions, String> {
    let response = reqwest::get(loader_manifest_url(LoaderKind::NeoForge)?).await.map_err(|e| e.to_string())?;
    let manifest: Value = response.json().await.map_err(|e| e.to_string())?;

    let mut versions = Vec::new();
    if let Some(is_snapshot) = manifest.get("isSnapshot").and_then(|s| s.as_bool()) {
        if let Some(neoforge_versions) = manifest.get("versions").and_then(|v| v.as_array()) {
            for version in neoforge_versions {
                if let Some(version_str) = version.as_str() {
                    let is_beta = version_str.contains("beta");
                    let is_alpha = version_str.contains("alpha");
                    let is_snapshot = version_str.contains("snapshot") || is_snapshot;
                    let looks_unstable = is_beta || is_alpha || is_snapshot;
                    if let Some(last_dot_index) = version_str.rfind('.') {
                        // split version_str into mc_version and neoforge_version by the last dot, so 21.11.41-beta -> mc_version = 1.21.11 and neoforge_version = 41-beta, but if the version is in the new format like
                        // 26.1.2.28-beta then mc_version = 26.1.2 and neoforge_version = 28-beta,
                        // we can detect this by checking if the part before the last dot starts with "1." or not, if it starts with "1." we are in the old format and need to add "1." to the mc_version,
                        // otherwise we are in the new format and can use the mc_version as is
                        let mc_version = if let Some(last_dot_index) = version_str.rfind('.') {
                            let potential_mc_version = &version_str[..last_dot_index];
                            if potential_mc_version.starts_with("1.") {
                                format!("1.{}", potential_mc_version)
                            } else {
                                potential_mc_version.to_string()
                            }
                        } else {
                            continue; // skip if there is no dot, should not happen
                        };
                        let neoforge_version = &version_str[last_dot_index + 1..];
                        versions.push(ProfileVersion {
                            // the neoforge id to run neoforge requires only neoforge-<version from manifest list>
                            id: format!("neoforge-{}", neoforge_version),
                            display_name: format!("NeoForge {} for Minecraft {}", neoforge_version, mc_version),
                            loader: LoaderKind::NeoForge,
                            minecraft_version: Some(mc_version.to_string()),
                            loader_version: Some(neoforge_version.to_string()),
                            version_type: match (is_snapshot, looks_unstable) {
                                (true, _) => Some(api_types::profiles::ProfileVersionType::Snapshot),
                                (false, true) => Some(api_types::profiles::ProfileVersionType::OldBeta), // we don't have exact version types for beta/alpha so just mark them as old beta
                                (false, false) => Some(api_types::profiles::ProfileVersionType::Release),
                            },
                            stable: Some(!is_snapshot && !looks_unstable),
                            release_time: None,
                            updated_time: None,
                            // We should likely just put a link here to the latest neoforge installer as we don't have urls for neoforge...
                            url: None,
                            sha1: None,
                            compliance_level: None,
                            // I don't recommend forge nor neoforge so we just default to false here..
                            recommended: Some(false),
                        });
                    }
                }
            }
        }
    }

    Ok(Versions(versions))
}
