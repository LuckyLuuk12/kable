// launcher/vanilla.rs

use super::{LaunchContext, LaunchResult, Launchable};
use crate::launcher::utils::build_variable_map;
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;

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
    // TODO: Implement proper prepare logic... This is untested and may need adjustments
    async fn prepare(&self, _context: &LaunchContext) -> Result<(), String> {
        // Download manifest and jar, and libraries
        let version_id = &_context.installation.version_id;
        let minecraft_dir = &_context.minecraft_dir;
        // ensure_version_manifest_and_jar now returns the resolved concrete version id
        let resolved =
            crate::launcher::utils::ensure_version_manifest_and_jar(version_id, minecraft_dir)
                .await?;
        // Load manifest using the resolved id
        let manifest = crate::launcher::utils::load_and_merge_manifest_with_instance(
            minecraft_dir,
            &resolved,
            Some(&_context.installation.id),
        )
        .await?;
        let libraries_path = std::path::PathBuf::from(minecraft_dir).join("libraries");
        crate::launcher::utils::ensure_libraries(&manifest, &libraries_path).await?;
        // Ensure minimal assets + sounds so UI and audio are available
        crate::launcher::utils::ensure_assets_for_manifest(
            minecraft_dir,
            &manifest,
            crate::launcher::utils::AssetMode::MinimalWithSounds,
            Some(&_context.installation.id),
        )
        .await?;
        Ok(())
    }

    async fn launch(&self, context: &LaunchContext) -> Result<LaunchResult, String> {
        println!("VANILLA::launch() -> {}", context.installation.name);
        // 1. Load merged manifest (with inheritance)
        let version_id = &context.installation.version_id;
        // Ensure manifest/jar exist and get resolved id (in case of latest-* placeholders)
        let resolved = crate::launcher::utils::ensure_version_manifest_and_jar(
            version_id,
            &context.minecraft_dir,
        )
        .await?;
        let manifest = crate::launcher::utils::load_and_merge_manifest_with_instance(
            &context.minecraft_dir,
            &resolved,
            Some(&context.installation.id),
        )
        .await?;

        // 2. Build classpath (all libraries + version JAR)
        let libraries_path = PathBuf::from(&context.minecraft_dir).join("libraries");
        let version_jar_path = PathBuf::from(&context.minecraft_dir)
            .join("versions")
            .join(&resolved)
            .join(format!("{}.jar", resolved));
        let classpath = crate::launcher::utils::build_classpath_from_manifest_with_instance(
            &manifest,
            &libraries_path,
            &version_jar_path,
            Some(&context.installation.id),
        );

        // Run pre-launch Java/native compatibility check. This may return Err to abort launch with
        // an actionable message (e.g., 32-bit Java vs 64-bit natives). Use configured java path.
        let java_path = crate::launcher::java::find_java_executable(
            context.settings.general.java_path.as_ref(),
        )?;

        crate::launcher::utils::pre_launch_java_native_compat_check(
            &java_path,
            &manifest,
            Some(&context.installation.id),
        )?;

        // Inspect classpath for LWJGL version consistency and log warnings if needed.
        let _ = crate::launcher::utils::check_lwjgl_classpath_consistency(
            &classpath,
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
        let (mut jvm_args_vec, game_args_vec) =
            crate::launcher::utils::build_jvm_and_game_args_with_instance(
                &manifest,
                &variables,
                Some(&context.installation.id),
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
        let java_path = match &context.settings.general.java_path {
            Some(path) => path.clone(),
            None => crate::launcher::java::find_java_executable(None)?,
        };
        let main_class = manifest
            .get("mainClass")
            .and_then(|v| v.as_str())
            .unwrap_or("net.minecraft.client.main.Main");
        let mut cmd = Command::new(&java_path);
        cmd.args(&jvm_args_vec);
        cmd.arg("-cp");
        cmd.arg(&classpath);
        cmd.arg(main_class);
        cmd.args(&game_args_vec);
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
