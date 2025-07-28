/// Loads and merges a Fabric manifest, recursively resolving `inheritsFrom` and merging libraries and arguments.
/// Returns the fully merged manifest as serde_json::Value.
use serde_json::Value;
use std::fs::File;
use std::io::Read;

/// Loads a manifest from disk given a version id and minecraft_dir.
fn load_manifest(minecraft_dir: &str, version_id: &str) -> Result<Value, String> {
    let manifest_path = PathBuf::from(minecraft_dir)
        .join("versions").join(version_id).join(format!("{}.json", version_id));
    let mut file = File::open(&manifest_path)
        .map_err(|e| format!("Failed to open manifest {}: {}", manifest_path.display(), e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read manifest {}: {}", manifest_path.display(), e))?;
    serde_json::from_str(&contents).map_err(|e| format!("Failed to parse manifest JSON: {}", e))
}

/// Recursively merges manifests for Fabric, handling `inheritsFrom`.
fn load_and_merge_fabric_manifest(minecraft_dir: &str, version_id: &str) -> Result<Value, String> {
    let mut manifest = load_manifest(minecraft_dir, version_id)?;
    // If inheritsFrom, merge parent manifest
    if let Some(parent_id) = manifest.get("inheritsFrom").and_then(|v| v.as_str()) {
        let parent = load_and_merge_fabric_manifest(minecraft_dir, parent_id)?;
        // Merge libraries: parent first, then child (child can override)
        let mut merged_libraries = vec![];
        if let Some(parent_libs) = parent.get("libraries").and_then(|v| v.as_array()) {
            merged_libraries.extend(parent_libs.clone());
        }
        if let Some(child_libs) = manifest.get("libraries").and_then(|v| v.as_array()) {
            merged_libraries.extend(child_libs.clone());
        }
        manifest["libraries"] = Value::Array(merged_libraries);
        // Merge arguments: parent first, then child (child can override/add)
        if let Some(parent_args) = parent.get("arguments").and_then(|v| v.as_object()) {
            let mut merged_args = parent_args.clone();
            if let Some(child_args) = manifest.get("arguments").and_then(|v| v.as_object()) {
                for (k, v) in child_args {
                    merged_args.insert(k.clone(), v.clone());
                }
            }
            manifest["arguments"] = Value::Object(merged_args);
        }
        // Merge other fields (mainClass, etc.) if not present in child
        for key in ["mainClass", "assetIndex", "assets", "downloads", "logging", "javaVersion", "type"] {
            if manifest.get(key).is_none() {
                if let Some(val) = parent.get(key) {
                    manifest[key] = val.clone();
                }
            }
        }
    }
    Ok(manifest)
}
// launcher/vanilla.rs

use super::{Launchable, LaunchContext, LaunchResult};
use async_trait::async_trait;
use std::process::Command;
use std::fs;
use std::path::PathBuf;
use reqwest::Client;
use tokio::io::AsyncWriteExt;
use crate::{launcher::utils::{
    build_classpath_from_manifest_with_instance, build_jvm_and_game_args_with_instance, build_variable_map, load_and_merge_manifest_with_instance
}, Logger};

#[derive(Default)]
pub struct FabricLaunchable;

#[async_trait]
impl Launchable for FabricLaunchable {
    async fn prepare(&self, context: &LaunchContext) -> Result<(), String> {
        // 1. Get installer Maven coordinates from version.extra
        use crate::installations::get_version;
        let version_id = &context.installation.version_id;
        let version_data = get_version(version_id.clone()).await
            .ok_or("Could not find version data for installation's version_id")?;
        let extra = &version_data.extra;
        let maven = extra.get("installer_maven")
            .ok_or("No 'installer_maven' key in version.extra for Fabric")?
            .as_str().ok_or("'installer_maven' must be a string")?;

        // 2. Parse maven: group:artifact:version
        let mut parts = maven.split(':');
        let group = parts.next().ok_or("Invalid maven: missing group")?;
        let artifact = parts.next().ok_or("Invalid maven: missing artifact")?;
        let version = parts.next().ok_or("Invalid maven: missing version")?;
        let group_path = group.replace('.', "/");
        let jar_name = format!("{}-{}.jar", artifact, version);
        let url = format!("https://maven.fabricmc.net/{}/{}/{}/{}", group_path, artifact, version, jar_name);

        // 3. Download to cache dir if not present
        let cache_dir = PathBuf::from(&context.minecraft_dir).join("fabric-installer-cache");
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir).map_err(|e| format!("Failed to create cache dir: {e}"))?;
        }
        let jar_path = cache_dir.join(&jar_name);
        if !jar_path.exists() {
            let client = Client::new();
            let resp = client.get(&url).send().await.map_err(|e| format!("Failed to download Fabric installer: {e}"))?;
            if !resp.status().is_success() {
                return Err(format!("Failed to download Fabric installer: HTTP {}", resp.status()));
            }
            let bytes = resp.bytes().await.map_err(|e| format!("Failed to read Fabric installer bytes: {e}"))?;
            let mut file = tokio::fs::File::create(&jar_path).await.map_err(|e| format!("Failed to create installer jar: {e}"))?;
            file.write_all(&bytes).await.map_err(|e| format!("Failed to write installer jar: {e}"))?;
        }

        // 4. Run installer in headless mode if manifest/jar not present
        let versions_dir = PathBuf::from(&context.minecraft_dir).join("versions");
        let fabric_json = versions_dir.join(version_id).join(format!("{}.json", version_id));
        let fabric_jar = versions_dir.join(version_id).join(format!("{}.jar", version_id));
        if !fabric_json.exists() || !fabric_jar.exists() {
            // Ensure version subdir exists
            let version_subdir = versions_dir.join(version_id);
            if !version_subdir.exists() {
                fs::create_dir_all(&version_subdir).map_err(|e| format!("Failed to create version dir: {e}"))?;
            }
            // Build java command: java -jar <installer> server|client -dir <mcdir> -mcversion <mcver> -noprofile -downloadMinecraft
            let java_path = context.settings.general.java_path.clone().unwrap_or_else(|| "java".to_string());
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
            let status = cmd.status().map_err(|e| format!("Failed to run Fabric installer: {e}"))?;
            if !status.success() {
                return Err(format!("Fabric installer failed with status: {}", status));
            }
        }
        Ok(())
    }

    async fn launch(&self, context: &LaunchContext) -> Result<LaunchResult, String> {
        // 1. Load merged manifest (with inheritance, Fabric-aware)
        let version_id = &context.installation.version_id;
        let manifest = load_and_merge_fabric_manifest(
            &context.minecraft_dir,
            version_id
        )?;

        // 2. Build classpath (all libraries + version JAR)
        let libraries_path = PathBuf::from(&context.minecraft_dir).join("libraries");
        let version_jar_path = PathBuf::from(&context.minecraft_dir)
            .join("versions").join(version_id).join(format!("{}.jar", version_id));
        let classpath = crate::launcher::utils::build_classpath_from_manifest_with_instance(
            &manifest,
            &libraries_path,
            &version_jar_path,
            Some(&context.installation.id)
        );

        // 3. Build variable map
        let variables = build_variable_map(
            context,
            Some(&manifest),
            &classpath,
            Some(&context.installation.parameters_map),
        );

        // 4. Build JVM and game arguments
        let (mut jvm_args_vec, game_args_vec) = crate::launcher::utils::build_jvm_and_game_args_with_instance(
            &manifest,
            &variables,
            Some(&context.installation.id)
        );

        // 5. Prepend installation-specific JVM args (Vec<String>) if present
        if !context.installation.java_args.is_empty() {
            jvm_args_vec.splice(0..0, context.installation.java_args.clone());
        }

        // 6. Add/overwrite with parameters_map (for --key style)
        for (k, v) in &context.installation.parameters_map {
            if k.starts_with("--") {
                jvm_args_vec.push(k.clone());
                if !v.is_empty() {
                    jvm_args_vec.push(v.clone());
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
                    PathBuf::from(&context.minecraft_dir).join("kable").join("mods").join(p)
                }
            };
            final_game_args_vec.retain(|arg| arg != "--fabric.modDir");
            final_game_args_vec.push("--fabric.modDir".to_string());
            final_game_args_vec.push(mods_path.to_string_lossy().to_string());
        }

        // 8. Build command: exactly like vanilla (single -cp, correct order)
        let java_path = context.settings.general.java_path.clone().unwrap_or_else(|| "java".to_string());
        let main_class = manifest.get("mainClass").and_then(|v| v.as_str()).unwrap_or("net.fabricmc.loader.impl.launch.knot.KnotClient");
        let mut cmd = Command::new(&java_path);
        cmd.args(&jvm_args_vec);
        cmd.arg("-cp");
        cmd.arg(&classpath);
        cmd.arg(main_class);
        cmd.args(&final_game_args_vec);
        cmd.current_dir(&context.minecraft_dir);

        // Use spawn_and_log_process utility
        let installation_json = serde_json::to_value(&context.installation)
            .map_err(|e| format!("Failed to serialize installation: {}", e))?;
        crate::launcher::utils::spawn_and_log_process(
            cmd,
            &context.minecraft_dir,
            version_id,
            &manifest,
            &installation_json,
        ).await
    }
}