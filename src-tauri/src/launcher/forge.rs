use crate::launcher::launchables::{LaunchContext, LaunchResult, Launchable};
use crate::launcher::utils::{
    build_classpath_from_manifest_with_instance, build_jvm_and_game_args_with_instance,
    build_variable_map, spawn_and_log_process,
};
use async_trait::async_trait;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

/// Loads a Forge manifest from disk as serde_json::Value
fn load_forge_manifest(minecraft_dir: &str, version_id: &str) -> Result<Value, String> {
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

#[derive(Default)]
pub struct ForgeLaunchable;

#[async_trait]
impl Launchable for ForgeLaunchable {
    async fn prepare(&self, _context: &LaunchContext) -> Result<(), String> {
        // For most Forge versions, if the manifest and jar exist, nothing to do
        // If you want to support auto-install, add logic here
        Ok(())
    }

    async fn launch(&self, context: &LaunchContext) -> Result<LaunchResult, String> {
        let version_id = &context.installation.version_id;
        let manifest = load_forge_manifest(&context.minecraft_dir, version_id)?;

        // Build classpath (all libraries + version JAR)
        let libraries_path = PathBuf::from(&context.minecraft_dir).join("libraries");
        let version_jar_path = PathBuf::from(&context.minecraft_dir)
            .join("versions")
            .join(version_id)
            .join(format!("{}.jar", version_id));
        if !version_jar_path.exists() {
            return Err(format!(
                "Version jar not found for classpath: {}",
                version_jar_path.display()
            ));
        }
        let classpath = build_classpath_from_manifest_with_instance(
            &manifest,
            &libraries_path,
            &version_jar_path,
            Some(&context.installation.id),
        );

        // Build variable map
        let variables = build_variable_map(
            context,
            Some(&manifest),
            &classpath,
            Some(&context.installation.parameters_map),
        );

        // Build JVM and game arguments
        let (jvm_args_vec, game_args_vec) = build_jvm_and_game_args_with_instance(
            &manifest,
            &variables,
            Some(&context.installation.id),
        );

        // Remove any -cp or -classpath and their following value from jvm_args_vec
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

        // Prepend installation-specific JVM args if present
        if !context.installation.java_args.is_empty() {
            cleaned_jvm_args.splice(0..0, context.installation.java_args.clone());
        }

        // Add/overwrite with parameters_map (for --key style)
        for (k, v) in &context.installation.parameters_map {
            if k.starts_with("--") {
                cleaned_jvm_args.push(k.clone());
                if !v.is_empty() {
                    cleaned_jvm_args.push(v.clone());
                }
            }
        }

        // Forge does not use --fabric.modDir, so remove if present
        let mut final_game_args_vec = game_args_vec.clone();
        final_game_args_vec.retain(|arg| arg != "--fabric.modDir");

        // Ensure --gameDir is present and set to the correct path
        let game_dir_flag_index = final_game_args_vec
            .iter()
            .position(|arg| arg == "--gameDir");
        if let Some(idx) = game_dir_flag_index {
            // If --gameDir is present, ensure the next argument is correct
            if idx + 1 < final_game_args_vec.len() {
                final_game_args_vec[idx + 1] = context.minecraft_dir.clone();
            } else {
                final_game_args_vec.push(context.minecraft_dir.clone());
            }
        } else {
            // If --gameDir is missing, add it
            final_game_args_vec.push("--gameDir".to_string());
            final_game_args_vec.push(context.minecraft_dir.clone());
        }

        // Build command: main class for Forge
        let java_path = context
            .settings
            .general
            .java_path
            .clone()
            .unwrap_or_else(|| "java".to_string());
        let main_class = manifest
            .get("mainClass")
            .and_then(|v| v.as_str())
            .unwrap_or("cpw.mods.modlauncher.Launcher");
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
        spawn_and_log_process(
            cmd,
            &context.minecraft_dir,
            &context.installation.id,
            &manifest,
            &installation_json,
        )
        .await
    }
}
