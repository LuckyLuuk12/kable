use super::{LaunchContext, LaunchResult, Launchable};
use crate::launcher::utils::build_variable_map;
use async_trait::async_trait;
use reqwest::Client;
/// Loads and merges a Fabric manifest, recursively resolving `inheritsFrom` and merging libraries and arguments.
/// Returns the fully merged manifest as serde_json::Value.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FabricManifest {
    #[serde(default, rename = "inheritsFrom")]
    inherits_from: Option<String>,
    #[serde(default, rename = "mainClass")]
    main_class: Option<String>,
    #[serde(default)]
    libraries: Vec<FabricLibrary>,
    #[serde(default)]
    arguments: Option<FabricArguments>,
    #[serde(default, rename = "assetIndex")]
    asset_index: Option<Value>,
    #[serde(default)]
    assets: Option<String>,
    #[serde(default)]
    downloads: Option<Value>,
    #[serde(default)]
    logging: Option<Value>,
    #[serde(default, rename = "javaVersion")]
    java_version: Option<Value>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    time: Option<String>,
    #[serde(default, rename = "releaseTime")]
    release_time: Option<String>,
    #[serde(default, rename = "type")]
    r#type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FabricLibrary {
    #[serde(default)]
    name: Option<String>,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FabricArguments {
    #[serde(default)]
    jvm: Vec<Value>,
    #[serde(default)]
    game: Vec<Value>,
}

/// Loads a Fabric manifest from disk and parses as FabricManifest.
fn load_fabric_manifest(minecraft_dir: &str, version_id: &str) -> Result<FabricManifest, String> {
    let manifest_path = PathBuf::from(minecraft_dir)
        .join("versions")
        .join(version_id)
        .join(format!("{}.json", version_id));
    let mut file = File::open(&manifest_path)
        .map_err(|e| format!("Failed to open manifest {}: {}", manifest_path.display(), e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read manifest {}: {}", manifest_path.display(), e))?;
    serde_json::from_str(&contents).map_err(|e| format!("Failed to parse manifest JSON: {}", e))
}

/// Recursively loads and merges Fabric manifests, returning a single FabricManifest.
fn load_and_merge_fabric_manifest_struct(
    minecraft_dir: &str,
    version_id: &str,
) -> Result<FabricManifest, String> {
    let mut manifest = load_fabric_manifest(minecraft_dir, version_id)?;
    if let Some(parent_id) = &manifest.inherits_from {
        let parent = load_and_merge_fabric_manifest_struct(minecraft_dir, parent_id)?;
        // Merge libraries: parent first, then child
        let mut merged_libraries = parent.libraries.clone();
        merged_libraries.extend(manifest.libraries.clone());
        manifest.libraries = merged_libraries;
        // Merge arguments: parent first, then child (child can override/add)
        let merged_args = match (parent.arguments, manifest.arguments) {
            (Some(mut p), Some(c)) => {
                p.jvm.extend(c.jvm);
                p.game.extend(c.game);
                Some(p)
            }
            (Some(p), None) => Some(p),
            (None, Some(c)) => Some(c),
            (None, None) => None,
        };
        manifest.arguments = merged_args;
        // Merge other fields if not present in child
        if manifest.main_class.is_none() {
            manifest.main_class = parent.main_class;
        }
        if manifest.asset_index.is_none() {
            manifest.asset_index = parent.asset_index;
        }
        if manifest.assets.is_none() {
            manifest.assets = parent.assets;
        }
        if manifest.downloads.is_none() {
            manifest.downloads = parent.downloads;
        }
        if manifest.logging.is_none() {
            manifest.logging = parent.logging;
        }
        if manifest.java_version.is_none() {
            manifest.java_version = parent.java_version;
        }
        if manifest.r#type.is_none() {
            manifest.r#type = parent.r#type;
        }
        if manifest.id.is_none() {
            manifest.id = parent.id;
        }
        if manifest.time.is_none() {
            manifest.time = parent.time;
        }
        if manifest.release_time.is_none() {
            manifest.release_time = parent.release_time;
        }
    }
    Ok(manifest)
}

/// Converts FabricManifest to a generic manifest (serde_json::Value) compatible with utils.
impl From<FabricManifest> for Value {
    fn from(f: FabricManifest) -> Self {
        let mut map = serde_json::Map::new();
        if let Some(inherits) = f.inherits_from {
            map.insert("inheritsFrom".to_string(), json!(inherits));
        }
        if let Some(main_class) = f.main_class {
            map.insert("mainClass".to_string(), json!(main_class));
        }
        // Convert libraries to vanilla-like array
        let libs: Vec<Value> = f
            .libraries
            .into_iter()
            .map(|l| {
                let mut m = serde_json::Map::new();
                if let Some(name) = l.name {
                    m.insert("name".to_string(), json!(name));
                }
                for (k, v) in l.extra {
                    m.insert(k, v);
                }
                Value::Object(m)
            })
            .collect();
        map.insert("libraries".to_string(), Value::Array(libs));
        // Convert arguments to vanilla-like object
        if let Some(args) = f.arguments {
            let mut args_map = serde_json::Map::new();
            args_map.insert("jvm".to_string(), Value::Array(args.jvm));
            args_map.insert("game".to_string(), Value::Array(args.game));
            map.insert("arguments".to_string(), Value::Object(args_map));
        }
        if let Some(asset_index) = f.asset_index {
            map.insert("assetIndex".to_string(), asset_index);
        }
        if let Some(assets) = f.assets {
            map.insert("assets".to_string(), json!(assets));
        }
        if let Some(downloads) = f.downloads {
            map.insert("downloads".to_string(), downloads);
        }
        if let Some(logging) = f.logging {
            map.insert("logging".to_string(), logging);
        }
        if let Some(java_version) = f.java_version {
            map.insert("javaVersion".to_string(), java_version);
        }
        if let Some(id) = f.id {
            map.insert("id".to_string(), json!(id));
        }
        if let Some(time) = f.time {
            map.insert("time".to_string(), json!(time));
        }
        if let Some(release_time) = f.release_time {
            map.insert("releaseTime".to_string(), json!(release_time));
        }
        if let Some(t) = f.r#type {
            map.insert("type".to_string(), json!(t));
        }
        Value::Object(map)
    }
}

#[derive(Default)]
pub struct FabricLaunchable;

#[async_trait]
impl Launchable for FabricLaunchable {
    // TODO: Implement proper prepare logic... This is untested and may need adjustments
    async fn prepare(&self, context: &LaunchContext) -> Result<(), String> {
        // 1. Check if manifest and jar already exist; if so, skip installer
        let version_id = &context.installation.version_id;
        let versions_dir = PathBuf::from(&context.minecraft_dir).join("versions");
        let fabric_json = versions_dir
            .join(version_id)
            .join(format!("{}.json", version_id));
        let fabric_jar = versions_dir
            .join(version_id)
            .join(format!("{}.jar", version_id));
        if !fabric_json.exists() {
            crate::logging::Logger::debug_global(
                &format!("Fabric manifest not found: {}", fabric_json.display()),
                Some(&context.installation.id),
            );
        }
        if !fabric_jar.exists() {
            crate::logging::Logger::debug_global(
                &format!("Fabric jar not found: {}", fabric_jar.display()),
                Some(&context.installation.id),
            );
        }
        if fabric_json.exists() && fabric_jar.exists() {
            // Already installed, nothing to do
            return Ok(());
        }

        // 2. Get installer Maven coordinates from version.extra (only if needed)
        use crate::installations::get_version;
        let version_data = get_version(version_id.clone()).await;
        if version_data.is_none() {
            crate::logging::Logger::debug_global(
                &format!(
                    "Could not find version data for installation's version_id: {}",
                    version_id
                ),
                Some(&context.installation.id),
            );
        }
        let version_data =
            version_data.ok_or("Could not find version data for installation's version_id")?;
        let extra = &version_data.extra;
        let maven = extra.get("installer_maven");
        if maven.is_none() {
            crate::logging::Logger::debug_global(&format!("No 'installer_maven' key in version.extra for Fabric (and install is required) for version_id: {}", version_id), Some(&context.installation.id));
        }
        let maven = maven
            .ok_or(
                "No 'installer_maven' key in version.extra for Fabric (and install is required)",
            )?
            .as_str()
            .ok_or("'installer_maven' must be a string")?;

        // 3. Parse maven: group:artifact:version
        let mut parts = maven.split(':');
        let group = parts.next().ok_or("Invalid maven: missing group")?;
        let artifact = parts.next().ok_or("Invalid maven: missing artifact")?;
        let version = parts.next().ok_or("Invalid maven: missing version")?;
        let group_path = group.replace('.', "/");
        let jar_name = format!("{}-{}.jar", artifact, version);
        let url = format!(
            "https://maven.fabricmc.net/{}/{}/{}/{}",
            group_path, artifact, version, jar_name
        );

        // 4. Download to cache dir if not present
        let cache_dir = PathBuf::from(&context.minecraft_dir).join("fabric-installer-cache");
        if !cache_dir.exists() {
            crate::logging::Logger::debug_global(
                &format!(
                    "Creating Fabric installer cache dir: {}",
                    cache_dir.display()
                ),
                Some(&context.installation.id),
            );
            crate::ensure_folder_sync(&cache_dir)
                .map_err(|e| format!("Failed to create cache dir: {e}"))?;
        }
        let jar_path = cache_dir.join(&jar_name);
        if !jar_path.exists() {
            crate::logging::Logger::debug_global(
                &format!(
                    "Fabric installer jar not found, will download: {}",
                    jar_path.display()
                ),
                Some(&context.installation.id),
            );
            let client = Client::new();
            let resp = client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("Failed to download Fabric installer: {e}"))?;
            if !resp.status().is_success() {
                crate::logging::Logger::debug_global(
                    &format!(
                        "Failed to download Fabric installer: HTTP {}",
                        resp.status()
                    ),
                    Some(&context.installation.id),
                );
                return Err(format!(
                    "Failed to download Fabric installer: HTTP {}",
                    resp.status()
                ));
            }
            let bytes = resp
                .bytes()
                .await
                .map_err(|e| format!("Failed to read Fabric installer bytes: {e}"))?;
            let mut file = tokio::fs::File::create(&jar_path)
                .await
                .map_err(|e| format!("Failed to create installer jar: {e}"))?;
            file.write_all(&bytes)
                .await
                .map_err(|e| format!("Failed to write installer jar: {e}"))?;
        }

        // 5. Run installer in headless mode
        if !fabric_json.exists() || !fabric_jar.exists() {
            // Ensure version subdir exists
            let version_subdir = versions_dir.join(version_id);
            if !version_subdir.exists() {
                crate::logging::Logger::debug_global(
                    &format!("Creating version subdir: {}", version_subdir.display()),
                    Some(&context.installation.id),
                );
                crate::ensure_folder_sync(&version_subdir)
                    .map_err(|e| format!("Failed to create version dir: {e}"))?;
            }
            // Build java command: java -jar <installer> server|client -dir <mcdir> -mcversion <mcver> -noprofile -downloadMinecraft
            let java_path = context
                .settings
                .general
                .java_path
                .clone()
                .unwrap_or_else(|| "java".to_string());
            let mc_version = version_id;
            let mut cmd = Command::new(&java_path);
            cmd.arg("-jar");
            cmd.arg(&jar_path);
            cmd.arg("client");
            cmd.arg("-dir");
            cmd.arg(&context.minecraft_dir);
            cmd.arg("-mcversion");
            cmd.arg(mc_version);
            cmd.arg("-noprofile");
            cmd.arg("-downloadMinecraft");
            crate::logging::Logger::debug_global(
                &format!("Running Fabric installer: {:?}", cmd),
                Some(&context.installation.id),
            );
            let status = cmd
                .status()
                .map_err(|e| format!("Failed to run Fabric installer: {e}"))?;
            if !status.success() {
                crate::logging::Logger::debug_global(
                    &format!("Fabric installer failed with status: {}", status),
                    Some(&context.installation.id),
                );
                return Err(format!("Fabric installer failed with status: {}", status));
            }
        }
        Ok(())
    }

    async fn launch(&self, context: &LaunchContext) -> Result<LaunchResult, String> {
        println!("FABRIC::launch() -> {}", context.installation.name);
        // 1. Load and merge Fabric manifest as struct, then convert to generic manifest
        let version_id = &context.installation.version_id;
        let fabric_manifest_struct =
            match load_and_merge_fabric_manifest_struct(&context.minecraft_dir, version_id) {
                Ok(m) => m,
                Err(e) => {
                    crate::logging::Logger::debug_global(
                        &format!("Failed to load/merge Fabric manifest: {}", e),
                        Some(&context.installation.id),
                    );
                    return Err(e);
                }
            };
        let manifest: Value = fabric_manifest_struct.into();

        // 2. Build classpath (all libraries + version JAR)
        let libraries_path = PathBuf::from(&context.minecraft_dir).join("libraries");
        let version_jar_path = PathBuf::from(&context.minecraft_dir)
            .join("versions")
            .join(version_id)
            .join(format!("{}.jar", version_id));
        if !version_jar_path.exists() {
            crate::logging::Logger::debug_global(
                &format!(
                    "Version jar not found for classpath: {}",
                    version_jar_path.display()
                ),
                Some(&context.installation.id),
            );
        }
        let classpath = crate::launcher::utils::build_classpath_from_manifest_with_instance(
            &manifest,
            &libraries_path,
            &version_jar_path,
            Some(&context.installation.id),
        );

        // Clear natives folder to prevent version conflicts from previous launches
        let natives_dir = PathBuf::from(&context.minecraft_dir).join("natives");
        if natives_dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(&natives_dir) {
                crate::logging::Logger::warn_global(
                    &format!("Failed to clear natives directory (will continue): {}", e),
                    Some(&context.installation.id),
                );
            } else {
                crate::logging::Logger::debug_global(
                    "Cleared natives directory to prevent version conflicts",
                    Some(&context.installation.id),
                );
            }
        }
        // Recreate empty natives directory
        if let Err(e) = crate::ensure_folder_sync(&natives_dir) {
            crate::logging::Logger::warn_global(
                &format!("Failed to recreate natives directory: {}", e),
                Some(&context.installation.id),
            );
        }

        // Extract native libraries from the manifest
        if let Some(libs_array) = manifest.get("libraries").and_then(|v| v.as_array()) {
            let libraries: Vec<crate::launcher::utils::Library> = libs_array
                .iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
            if let Err(e) = crate::launcher::utils::extract_natives(
                &libraries,
                &libraries_path,
                &natives_dir,
                Some(&context.installation.id),
            ) {
                crate::logging::Logger::warn_global(
                    &format!("Failed to extract natives: {}", e),
                    Some(&context.installation.id),
                );
            }
        }

        // 3. Build variable map
        let variables = build_variable_map(
            context,
            Some(&manifest),
            &classpath,
            Some(&context.installation.parameters_map),
        );

        // 4. Build JVM and game arguments
        let (jvm_args_vec, game_args_vec) =
            crate::launcher::utils::build_jvm_and_game_args_with_instance(
                &manifest,
                &variables,
                Some(&context.installation.id),
            );

        // Remove any -cp or -classpath and their following value from jvm_args_vec (defensive, in case manifest or parameters injects it)
        let mut cleaned_jvm_args = Vec::new();
        let mut skip_next = false;
        for arg in jvm_args_vec.into_iter() {
            if skip_next {
                skip_next = false;
                continue;
            }
            if arg == "-cp" || arg == "-classpath" {
                skip_next = true;
                continue;
            }
            cleaned_jvm_args.push(arg);
        }

        // 5. Prepend installation-specific JVM args (Vec<String>) if present
        if !context.installation.java_args.is_empty() {
            cleaned_jvm_args.splice(0..0, context.installation.java_args.clone());
        }

        // 6. Add/overwrite with parameters_map (for --key style)
        for (k, v) in &context.installation.parameters_map {
            if k.starts_with("--") {
                cleaned_jvm_args.push(k.clone());
                if !v.is_empty() {
                    cleaned_jvm_args.push(v.clone());
                }
            }
        }

        // 7. Handle mods folder override for Fabric (optional, only if needed)
        let mut final_game_args_vec = game_args_vec.clone();
        if let Some(mods_folder) = &context.installation.dedicated_mods_folder {
            let mods_path = {
                let p = PathBuf::from(mods_folder);
                if p.is_absolute() {
                    p
                } else {
                    PathBuf::from(&context.minecraft_dir)
                        .join("kable")
                        .join("mods")
                        .join(p)
                }
            };
            final_game_args_vec.retain(|arg| arg != "--fabric.modDir");
            final_game_args_vec.push("--fabric.modDir".to_string());
            final_game_args_vec.push(mods_path.to_string_lossy().to_string());
        }

        // 8. Build command: exactly like vanilla (single -cp, correct order)
        let java_path = context
            .settings
            .general
            .java_path
            .clone()
            .unwrap_or_else(|| "java".to_string());
        let main_class = manifest
            .get("mainClass")
            .and_then(|v| v.as_str())
            .unwrap_or("net.fabricmc.loader.impl.launch.knot.KnotClient");
        let mut cmd = Command::new(&java_path);
        cmd.args(&cleaned_jvm_args);
        cmd.arg("-cp");
        cmd.arg(&classpath);
        cmd.arg(main_class);
        cmd.args(&final_game_args_vec);
        cmd.current_dir(&context.minecraft_dir);

        // Use spawn_and_log_process utility
        let mut installation_json = serde_json::to_value(&context.installation)
            .map_err(|e| format!("Failed to serialize installation: {}", e))?;
        if let Some(obj) = installation_json.as_object_mut() {
            obj.insert(
                "path".to_string(),
                serde_json::json!(context.installation_path().to_string_lossy().to_string()),
            );
        }
        crate::launcher::utils::spawn_and_log_process(
            cmd,
            &context.minecraft_dir,
            &context.installation.id,
            &manifest,
            &installation_json,
        )
        .await
    }
}
