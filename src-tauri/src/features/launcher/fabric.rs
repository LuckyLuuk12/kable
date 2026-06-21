use crate::constants::{fabric_profile_jar_url, fabric_profile_url, LIBRARIES_DIR, VERSIONS_DIR};
use crate::features::launcher::utils::{
    build_classpath_from_manifest_with_instance, build_jvm_and_game_args_with_instance, build_variable_map,
    check_lwjgl_classpath_consistency, ensure_assets_for_manifest, ensure_libraries, ensure_version_manifest_and_jar, extract_natives,
    load_and_merge_manifest_with_instance, merge_manifests_with_instance, pre_launch_java_native_compat_check, spawn_and_log_process,
    AssetMode, Library,
};
use crate::features::launcher::{LaunchContext, LaunchResult, Launchable};
use crate::system::java::find_java_executable;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

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

fn load_fabric_manifest(minecraft_dir: &str, version_id: &str) -> Result<FabricManifest, String> {
    let manifest_path = PathBuf::from(minecraft_dir).join(VERSIONS_DIR).join(version_id).join(format!("{}.json", version_id));
    let mut file = File::open(&manifest_path).map_err(|e| format!("Failed to open manifest {}: {}", manifest_path.display(), e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(|e| format!("Failed to read manifest {}: {}", manifest_path.display(), e))?;
    serde_json::from_str(&contents).map_err(|e| format!("Failed to parse manifest JSON: {}", e))
}

fn load_and_merge_fabric_manifest_struct(minecraft_dir: &str, version_id: &str) -> Result<FabricManifest, String> {
    let mut manifest = load_fabric_manifest(minecraft_dir, version_id)?;
    if let Some(parent_id) = &manifest.inherits_from {
        let parent = load_and_merge_fabric_manifest_struct(minecraft_dir, parent_id)?;
        let mut merged_libraries = parent.libraries.clone();
        merged_libraries.extend(manifest.libraries.clone());
        manifest.libraries = merged_libraries;
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
        if manifest.main_class.is_none() {
            manifest.main_class = parent.main_class;
        }
    }
    Ok(manifest)
}

impl From<FabricManifest> for Value {
    fn from(f: FabricManifest) -> Self {
        let mut map = serde_json::Map::new();
        if let Some(inherits) = f.inherits_from {
            map.insert("inheritsFrom".to_string(), json!(inherits));
        }
        if let Some(main) = f.main_class {
            map.insert("mainClass".to_string(), json!(main));
        }
        map.insert(
            "libraries".to_string(),
            json!(f
                .libraries
                .into_iter()
                .map(|l| {
                    let mut l_map: serde_json::Map<String, Value> = l.extra.into_iter().collect();

                    if let Some(name) = l.name {
                        l_map.insert("name".to_string(), json!(name));
                    }
                    Value::Object(l_map)
                })
                .collect::<Vec<Value>>()),
        );
        if let Some(args) = f.arguments {
            map.insert(
                "arguments".to_string(),
                json!({
                    "jvm": args.jvm,
                    "game": args.game
                }),
            );
        }
        Value::Object(map)
    }
}

#[derive(Default)]
pub struct FabricLaunchable;

#[async_trait]
impl Launchable for FabricLaunchable {
    async fn prepare(&self, context: &LaunchContext) -> Result<(), String> {
        let version = &context.installation.version;
        let versions_dir = PathBuf::from(&context.minecraft_dir).join(VERSIONS_DIR);
        let fabric_json = versions_dir.join(version.id).join(format!("{}.json", version.id));
        let fabric_jar = versions_dir.join(version.id).join(format!("{}.jar", version.id));

        if !fabric_json.exists() || !fabric_jar.exists() {
            let mc_version = version.minecraft_version.as_ref().ok_or("Missing minecraft_version in version")?;
            let fabric_version = version.loader_version.as_ref().ok_or("Missing loader_version in version")?;

            crate::system::fs::create_dir(&versions_dir.join(version.id))
                .await
                .map_err(|e| format!("Failed to create version dir: {e}"))?;

            let profile_url = fabric_profile_url(mc_version, fabric_version);

            let client = Client::new();
            let profile_json_str = client
                .get(&profile_url)
                .send()
                .await
                .map_err(|e| format!("Failed to download Fabric profile: {e}"))?
                .text()
                .await
                .map_err(|e| format!("Failed to read Fabric profile: {e}"))?;

            std::fs::write(&fabric_json, profile_json_str).map_err(|e| format!("Failed to save Fabric manifest: {e}"))?;

            let jar_url = fabric_profile_jar_url(mc_version, fabric_version);
            let jar_bytes = client
                .get(&jar_url)
                .send()
                .await
                .map_err(|e| format!("Failed to download Fabric JAR: {e}"))?
                .bytes()
                .await
                .map_err(|e| format!("Failed to read Fabric JAR: {e}"))?;
            std::fs::write(&fabric_jar, jar_bytes).map_err(|e| format!("Failed to save Fabric JAR: {e}"))?;
        }

        let manifest_struct = load_and_merge_fabric_manifest_struct(&context.minecraft_dir, version.id.as_ref())?;
        let manifest: Value = manifest_struct.into();
        let libraries_path = PathBuf::from(&context.minecraft_dir).join(LIBRARIES_DIR);
        ensure_libraries(&manifest, &libraries_path).await?;

        let inherited_id =
            manifest.get("inheritsFrom").and_then(|v| v.as_str()).ok_or("Fabric manifest must inherit from a vanilla version")?;
        ensure_version_manifest_and_jar(inherited_id, &context.minecraft_dir).await?;

        let vanilla_manifest = load_and_merge_manifest_with_instance(&context.minecraft_dir, inherited_id, Some(&context.installation.id))?;
        ensure_libraries(&vanilla_manifest, &libraries_path).await?;
        ensure_assets_for_manifest(&context.minecraft_dir, &vanilla_manifest, AssetMode::MinimalWithSounds, Some(&context.installation.id))
            .await?;

        Ok(())
    }

    async fn launch(&self, context: &LaunchContext) -> Result<LaunchResult, String> {
        let version_id = &context.installation.version.id;
        let manifest_struct = load_and_merge_fabric_manifest_struct(&context.minecraft_dir, version_id)?;
        let inherited_id = manifest_struct.inherits_from.as_ref().ok_or("Missing inheritsFrom")?;

        let vanilla_manifest = load_and_merge_manifest_with_instance(&context.minecraft_dir, inherited_id, Some(&context.installation.id))?;
        let fabric_manifest_value: Value = manifest_struct.into();
        let merged_manifest = merge_manifests_with_instance(vanilla_manifest, fabric_manifest_value, Some(&context.installation.id));

        let libraries_path = PathBuf::from(&context.minecraft_dir).join(LIBRARIES_DIR);
        let version_jar_path =
            PathBuf::from(&context.minecraft_dir).join(VERSIONS_DIR).join(version_id).join(format!("{}.jar", version_id));

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
            extract_natives(&libraries, &libraries_path, &natives_dir, Some(&context.installation.id)).await?;
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
