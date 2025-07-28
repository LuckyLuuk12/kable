// launcher/vanilla.rs

use super::{Launchable, LaunchContext, LaunchResult};
use async_trait::async_trait;
use std::process::Command;
use std::fs;
use std::path::{Path, PathBuf};
use reqwest::Client;
use tokio::io::AsyncWriteExt;

pub struct FabricLaunchable;

impl FabricLaunchable {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FabricLaunchable {
    fn default() -> Self {
        Self::new()
    }
}

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
        use crate::launcher::utils::{substitute_variables, process_arguments, build_variable_map};
        use std::fs;
        use serde_json::Value;
        use std::path::PathBuf;

        // 1. Load manifest
        let version_id = &context.installation.version_id;
        let manifest_path = PathBuf::from(&context.minecraft_dir)
            .join("versions").join(version_id).join(format!("{}.json", version_id));
        let manifest_str = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        let manifest: Value = serde_json::from_str(&manifest_str)
            .map_err(|e| format!("Failed to parse manifest: {}", e))?;

        // 2. Build classpath
        let libraries_path = PathBuf::from(&context.minecraft_dir).join("libraries");
        let version_jar_path = PathBuf::from(&context.minecraft_dir)
            .join("versions").join(version_id).join(format!("{}.jar", version_id));
        // Use vanilla's build_classpath helper
        fn build_classpath(libraries: &[Value], libraries_path: &std::path::Path, version_jar_path: &std::path::Path) -> String {
            let mut entries = Vec::new();
            for lib in libraries {
                if let Some(obj) = lib.as_object() {
                    if let Some(downloads) = obj.get("downloads") {
                        if let Some(artifact) = downloads.get("artifact") {
                            if let Some(path) = artifact.get("path").and_then(|v| v.as_str()) {
                                let jar_path = libraries_path.join(path);
                                entries.push(jar_path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
            entries.push(version_jar_path.to_string_lossy().to_string());
            let sep = if cfg!(windows) { ";" } else { ":" };
            entries.join(sep)
        }
        let classpath = build_classpath(
            manifest.get("libraries").and_then(|v| v.as_array()).unwrap_or(&vec![]),
            &libraries_path,
            &version_jar_path,
        );

        // 3. Build variable map
        let variables = build_variable_map(
            context,
            Some(&manifest),
            &classpath,
            Some(&context.installation.parameters_map),
        );

        // 4. Build JVM and game arguments
        let arguments = manifest.get("arguments").and_then(|v| v.as_object()).ok_or("No arguments in manifest")?;
        let empty_vec = Vec::new();
        let jvm_args = arguments.get("jvm").and_then(|v| v.as_array()).unwrap_or(&empty_vec);
        let game_args = arguments.get("game").and_then(|v| v.as_array()).unwrap_or(&empty_vec);
        let mut jvm_args_vec = process_arguments(jvm_args, &variables);
        let mut game_args_vec = process_arguments(game_args, &variables);

        // 5. Add/overwrite with parameters_map (for --key style)
        for (k, v) in &context.installation.parameters_map {
            if k.starts_with("--") {
                jvm_args_vec.push(k.clone());
                if !v.is_empty() {
                    jvm_args_vec.push(v.clone());
                }
            }
        }

        // 6. Handle mods folder override for Fabric
        if let Some(mods_folder) = &context.installation.dedicated_mods_folder {
            // Fabric uses --gameDir or --fabric.modDir (see https://fabricmc.net/wiki/documentation:fabric_loader_arguments)
            // We'll use --fabric.modDir if present, else fallback to --gameDir
            // If mods_folder is relative, resolve from .minecraft/kable/mods/
            let mods_path = {
                let p = PathBuf::from(mods_folder);
                if p.is_absolute() {
                    p
                } else {
                    PathBuf::from(&context.minecraft_dir).join("kable").join("mods").join(p)
                }
            };
            // Remove any previous --fabric.modDir or --gameDir from game_args_vec
            game_args_vec.retain(|arg| arg != "--fabric.modDir" && arg != "--gameDir");
            // Insert the override
            game_args_vec.push("--fabric.modDir".to_string());
            game_args_vec.push(mods_path.to_string_lossy().to_string());
        }

        // 7. Build command
        let java_path = context.settings.general.java_path.clone().unwrap_or_else(|| "java".to_string());
        let main_class = manifest.get("mainClass").and_then(|v| v.as_str()).unwrap_or("net.fabricmc.loader.impl.launch.knot.KnotClient");
        let mut cmd = Command::new(&java_path);
        cmd.args(&jvm_args_vec);
        cmd.arg(main_class);
        cmd.args(&game_args_vec);
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