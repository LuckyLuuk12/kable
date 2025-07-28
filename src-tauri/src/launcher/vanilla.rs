// launcher/vanilla.rs

use super::{Launchable, LaunchContext, LaunchResult};
use crate::launcher::utils::{build_variable_map};
use async_trait::async_trait;
use std::process::Command;
use std::path::PathBuf;


#[derive(Default)]
pub struct VanillaLaunchable;

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
        // 1. Load merged manifest (with inheritance)
        let version_id = &context.installation.version_id;
        let manifest = crate::launcher::utils::load_and_merge_manifest_with_instance(
            &context.minecraft_dir,
            version_id,
            Some(&context.installation.id)
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
        cmd.arg("-cp");
        cmd.arg(&classpath);
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