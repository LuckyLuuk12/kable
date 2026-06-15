use crate::constants::{LIBRARIES_DIR, VERSIONS_DIR};
use crate::features::launcher::utils::{
    build_classpath_from_manifest_with_instance, build_jvm_and_game_args_with_instance, build_variable_map,
    check_lwjgl_classpath_consistency, ensure_assets_for_manifest, ensure_libraries, ensure_version_manifest_and_jar, extract_natives,
    load_and_merge_manifest_with_instance, pre_launch_java_native_compat_check, spawn_and_log_process, AssetMode, Library,
};
use crate::features::launcher::{LaunchContext, LaunchResult, Launchable};
use crate::logging::Logger;
use crate::system::java::find_java_executable;
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;

#[derive(Default)]
pub struct VanillaLaunchable;

#[async_trait]
impl Launchable for VanillaLaunchable {
    async fn prepare(&self, context: &LaunchContext) -> Result<(), String> {
        let version_id = &context.installation.version_id;
        let minecraft_dir = &context.minecraft_dir;

        let resolved = ensure_version_manifest_and_jar(version_id, minecraft_dir).await?;

        let manifest = load_and_merge_manifest_with_instance(minecraft_dir, &resolved, Some(&context.installation.id))?;

        let libraries_path = PathBuf::from(minecraft_dir).join(LIBRARIES_DIR);
        ensure_libraries(&manifest, &libraries_path).await?;

        ensure_assets_for_manifest(minecraft_dir, &manifest, AssetMode::MinimalWithSounds, Some(&context.installation.id)).await?;

        Ok(())
    }

    async fn launch(&self, context: &LaunchContext) -> Result<LaunchResult, String> {
        let version_id = &context.installation.version_id;

        let resolved = ensure_version_manifest_and_jar(version_id, &context.minecraft_dir).await?;

        let manifest = load_and_merge_manifest_with_instance(&context.minecraft_dir, &resolved, Some(&context.installation.id))?;

        let libraries_path = PathBuf::from(&context.minecraft_dir).join(LIBRARIES_DIR);
        let version_jar_path = PathBuf::from(&context.minecraft_dir).join(VERSIONS_DIR).join(&resolved).join(format!("{}.jar", resolved));

        let classpath =
            build_classpath_from_manifest_with_instance(&manifest, &libraries_path, &version_jar_path, Some(&context.installation.id));

        let java_path = find_java_executable(context.settings.general.java_path.as_ref())?;

        pre_launch_java_native_compat_check(&java_path, &manifest, Some(&context.installation.id))?;

        let _ = check_lwjgl_classpath_consistency(&classpath, Some(&context.installation.id));

        let natives_dir = PathBuf::from(&context.minecraft_dir).join("natives");

        if let Some(libs_array) = manifest.get("libraries").and_then(|v| v.as_array()) {
            let libraries: Vec<Library> = libs_array.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect();
            extract_natives(&libraries, &libraries_path, &natives_dir, Some(&context.installation.id))?;
        }

        let variables = build_variable_map(context, Some(&manifest), &classpath, Some(&context.installation.parameters_map));

        let (mut jvm_args_vec, game_args_vec) =
            build_jvm_and_game_args_with_instance(&manifest, &variables, Some(&context.installation.id));

        for (k, v) in &context.installation.parameters_map {
            if k.starts_with("--") {
                jvm_args_vec.push(k.clone());
                if !v.is_empty() {
                    jvm_args_vec.push(v.clone());
                }
            }
        }

        let main_class = manifest.get("mainClass").and_then(|v| v.as_str()).unwrap_or("net.minecraft.client.main.Main");

        let mut cmd = Command::new(&java_path);
        cmd.args(&jvm_args_vec);
        cmd.arg("-cp");
        cmd.arg(&classpath);
        cmd.arg(main_class);
        cmd.args(&game_args_vec);
        cmd.current_dir(&context.minecraft_dir);

        spawn_and_log_process(cmd, &context.minecraft_dir, &context.installation.id, &context.installation, &context.settings).await
    }
}
