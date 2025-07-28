// launcher/vanilla.rs

use super::{Launchable, LaunchContext, LaunchResult};
use crate::launcher::utils::{substitute_variables, process_arguments, build_variable_map};
use async_trait::async_trait;
use std::process::Command;
use tauri::AppHandle;
use std::fs;
use std::path::{Path, PathBuf};
use serde_json::Value;

pub struct VanillaLaunchable;

impl VanillaLaunchable {
    pub fn new() -> Self {
        Self
    }
}



fn build_classpath(libraries: &[Value], libraries_path: &Path, version_jar_path: &Path) -> String {
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

impl LaunchContext {
    pub fn installation_path(&self) -> PathBuf {
        // For now, use minecraft_dir as base, can be improved
        PathBuf::from(&self.minecraft_dir)
    }
}

#[async_trait]
impl Launchable for VanillaLaunchable {
    async fn prepare(&self, _context: &LaunchContext) -> Result<(), String> {
        // TODO: Download manifest, libraries, assets, natives, etc.
        Ok(())
    }

    async fn launch(&self, context: &LaunchContext) -> Result<LaunchResult, String> {
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
        let game_args_vec = process_arguments(game_args, &variables);

        // 5. Add/overwrite with parameters_map (for --key style)
        for (k, v) in &context.installation.parameters_map {
            if k.starts_with("--") {
                jvm_args_vec.push(k.clone());
                if !v.is_empty() {
                    jvm_args_vec.push(v.clone());
                }
            }
        }

        // 6. Build command
        let java_path = context.settings.general.java_path.clone().unwrap_or_else(|| "java".to_string());
        let main_class = manifest.get("mainClass").and_then(|v| v.as_str()).unwrap_or("net.minecraft.client.main.Main");
        let mut cmd = Command::new(&java_path);
        cmd.args(&jvm_args_vec);
        cmd.arg(main_class);
        cmd.args(&game_args_vec);
        cmd.current_dir(&context.minecraft_dir);

        // Use spawn_and_log_process utility
        // Pass manifest as profile, and installation as serde_json::Value
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