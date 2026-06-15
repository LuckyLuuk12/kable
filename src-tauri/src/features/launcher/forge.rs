use crate::constants::{LIBRARIES_DIR, VERSIONS_DIR};
use crate::features::launcher::utils::{
    build_classpath_from_manifest_with_instance, build_jvm_and_game_args_with_instance, build_variable_map,
    check_lwjgl_classpath_consistency, ensure_assets_for_manifest, ensure_libraries, ensure_version_manifest_and_jar, extract_natives,
    load_and_merge_manifest_with_instance, merge_manifests_with_instance, pre_launch_java_native_compat_check, spawn_and_log_process,
    AssetMode, Library,
};
use crate::features::launcher::{LaunchContext, LaunchResult, Launchable};
use crate::system::java::find_java_executable;
use async_trait::async_trait;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Default)]
pub struct ForgeLaunchable;

fn load_forge_manifest(minecraft_dir: &str, version_id: &str) -> Result<Value, String> {
    let manifest_path = PathBuf::from(minecraft_dir).join(VERSIONS_DIR).join(version_id).join(format!("{}.json", version_id));
    let contents = fs::read_to_string(&manifest_path).map_err(|e| format!("Failed to read forge manifest: {}", e))?;
    serde_json::from_str(&contents).map_err(|e| format!("Failed to parse forge manifest JSON: {}", e))
}

#[async_trait]
impl Launchable for ForgeLaunchable {
    async fn prepare(&self, _context: &LaunchContext) -> Result<(), String> {
        // Forge preparation logic is usually complex and depends on installer.
        // For now we assume files are already there or handled by caller.
        Ok(())
    }

    async fn launch(&self, context: &LaunchContext) -> Result<LaunchResult, String> {
        let version_id = &context.installation.version_id;
        let forge_manifest = load_forge_manifest(&context.minecraft_dir, version_id)?;

        let inherited_id = forge_manifest
            .get("inheritsFrom")
            .and_then(|v| v.as_str())
            .ok_or("Forge manifest must inherit from a vanilla version")?;

        ensure_version_manifest_and_jar(inherited_id, &context.minecraft_dir).await?;

        let vanilla_manifest = load_and_merge_manifest_with_instance(&context.minecraft_dir, inherited_id, Some(&context.installation.id))?;

        let merged_manifest = merge_manifests_with_instance(vanilla_manifest, forge_manifest, Some(&context.installation.id));

        let libraries_path = PathBuf::from(&context.minecraft_dir).join(LIBRARIES_DIR);
        let version_jar_path =
            PathBuf::from(&context.minecraft_dir).join(VERSIONS_DIR).join(inherited_id).join(format!("{}.jar", inherited_id));

        let classpath = build_classpath_from_manifest_with_instance(
            &merged_manifest,
            &libraries_path,
            &version_jar_path,
            Some(&context.installation.id),
        );

        let java_path = find_java_executable(context.settings.general.java_path.as_ref())?;

        pre_launch_java_native_compat_check(&java_path, &merged_manifest, Some(&context.installation.id))?;

        let _ = check_lwjgl_classpath_consistency(&classpath, Some(&context.installation.id));

        let natives_dir = PathBuf::from(&context.minecraft_dir).join("natives");
        if let Some(libs_array) = merged_manifest.get("libraries").and_then(|v| v.as_array()) {
            let libraries: Vec<Library> = libs_array.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect();
            extract_natives(&libraries, &libraries_path, &natives_dir, Some(&context.installation.id))?;
        }

        let variables = build_variable_map(context, Some(&merged_manifest), &classpath, Some(&context.installation.parameters_map));

        let (mut jvm_args_vec, game_args_vec) =
            build_jvm_and_game_args_with_instance(&merged_manifest, &variables, Some(&context.installation.id));

        for (k, v) in &context.installation.parameters_map {
            if k.starts_with("--") {
                jvm_args_vec.push(k.clone());
                if !v.is_empty() {
                    jvm_args_vec.push(v.clone());
                }
            }
        }

        let main_class = merged_manifest.get("mainClass").and_then(|v| v.as_str()).unwrap_or("net.minecraft.client.main.Main");

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
