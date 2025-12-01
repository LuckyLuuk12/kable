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

        // Track if we need to download/install Fabric files
        let need_fabric_files = !fabric_json.exists() || !fabric_jar.exists();

        // Variables needed for library checking and Iris mods (if applicable)
        let profile_json: String;
        let mc_version: String;
        let client = Client::new();

        if need_fabric_files {
            // 2. Get version metadata from version.extra
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

            // Extract Minecraft version and Fabric loader version
            mc_version = extra
                .get("minecraft_version")
                .and_then(|v| v.as_str())
                .ok_or("No 'minecraft_version' in version.extra for Fabric")?
                .to_string();
            let fabric_version = extra
                .get("version")
                .and_then(|v| v.as_str())
                .ok_or("No 'version' in version.extra for Fabric loader")?;

            crate::logging::Logger::debug_global(
                &format!(
                    "Installing Fabric: MC {} with loader {}",
                    mc_version, fabric_version
                ),
                Some(&context.installation.id),
            );

            // 3. Ensure version subdirectory exists
            let version_subdir = versions_dir.join(version_id);
            if !version_subdir.exists() {
                crate::logging::Logger::debug_global(
                    &format!("Creating version subdir: {}", version_subdir.display()),
                    Some(&context.installation.id),
                );
                crate::ensure_folder_sync(&version_subdir)
                    .map_err(|e| format!("Failed to create version dir: {e}"))?;
            }

            // 4. Download Fabric profile JSON directly from Meta API
            // This is the same method used by Iris Installer - simpler than running installer JAR
            let profile_url = format!(
                "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
                mc_version, fabric_version
            );

            crate::logging::Logger::debug_global(
                &format!("Downloading Fabric profile from: {}", profile_url),
                Some(&context.installation.id),
            );

            let downloaded_profile_json = client
                .get(&profile_url)
                .send()
                .await
                .map_err(|e| format!("Failed to download Fabric profile: {e}"))?
                .text()
                .await
                .map_err(|e| format!("Failed to read Fabric profile: {e}"))?;

            // 5. For IrisFabric, modify the JSON to add JVM arguments
            profile_json = if version_id.contains("iris-fabric") {
                // Parse JSON to add Iris-specific JVM arguments
                match serde_json::from_str::<serde_json::Value>(&downloaded_profile_json) {
                    Ok(mut json_obj) => {
                        // Log original ID
                        if let Some(original_id) = json_obj.get("id").and_then(|i| i.as_str()) {
                            crate::logging::Logger::debug_global(
                                &format!("Original JSON id: {}", original_id),
                                Some(&context.installation.id),
                            );
                        }

                        // Get or create arguments -> jvm array
                        let jvm_args = json_obj
                            .get_mut("arguments")
                            .and_then(|a| a.as_object_mut())
                            .and_then(|args| args.get_mut("jvm"))
                            .and_then(|jvm| jvm.as_array_mut());

                        if let Some(jvm_args) = jvm_args {
                            // Add Iris-specific JVM arguments
                            jvm_args.push(serde_json::Value::String(
                                "-Diris.installer=true".to_string(),
                            ));

                            // Add custom mods folder path
                            let minecraft_dir = std::path::PathBuf::from(&context.minecraft_dir);
                            let mods_folder = if version_id.contains("beta") {
                                minecraft_dir.join("iris-beta-reserved").join(&mc_version)
                            } else {
                                minecraft_dir.join("iris-reserved").join(&mc_version)
                            };

                            crate::logging::Logger::debug_global(
                                &format!(
                                    "Setting Fabric mods folder to: {}",
                                    mods_folder.display()
                                ),
                                Some(&context.installation.id),
                            );

                            jvm_args.push(serde_json::Value::String(format!(
                                "-Dfabric.modsFolder={}",
                                mods_folder.display()
                            )));
                        }

                        // Update version ID to match our folder/file naming
                        // The downloaded JSON has id like "fabric-loader-0.17.3-1.21.10"
                        // We need to change it to "iris-fabric-loader-0.17.3-1.21.10"
                        if let Some(id_str) = json_obj.get("id").and_then(|i| i.as_str()) {
                            let new_id = format!("iris-{}", id_str);
                            json_obj["id"] = serde_json::Value::String(new_id.clone());

                            crate::logging::Logger::debug_global(
                                &format!("Modified JSON id to: {}", new_id),
                                Some(&context.installation.id),
                            );
                        }

                        serde_json::to_string_pretty(&json_obj).map_err(|e| {
                            format!("Failed to serialize modified profile JSON: {e}")
                        })?
                    }
                    Err(e) => {
                        crate::logging::Logger::debug_global(
                            &format!("Warning: Could not parse profile JSON for modification: {e}"),
                            Some(&context.installation.id),
                        );
                        downloaded_profile_json // Use original if parsing fails
                    }
                }
            } else {
                downloaded_profile_json
            };

            // 6. Write profile JSON
            crate::write_file_atomic_async(&fabric_json, profile_json.as_bytes())
                .await
                .map_err(|e| format!("Failed to write Fabric profile JSON: {e}"))?;

            crate::logging::Logger::debug_global(
                &format!("Created Fabric profile: {}", fabric_json.display()),
                Some(&context.installation.id),
            );

            // 7. Ensure the parent vanilla Minecraft version JAR exists and copy it
            // The Fabric version JAR should be a copy of the vanilla Minecraft client JAR
            // This is referenced by the "inheritsFrom" field in the profile JSON

            // Parse the JSON to get the inheritsFrom version
            let profile_parsed: serde_json::Value = serde_json::from_str(&profile_json)
                .map_err(|e| format!("Failed to parse profile JSON to get inheritsFrom: {e}"))?;

            let parent_version = profile_parsed
                .get("inheritsFrom")
                .and_then(|v| v.as_str())
                .ok_or("No 'inheritsFrom' in Fabric profile JSON")?;

            crate::logging::Logger::debug_global(
                &format!("Fabric inherits from vanilla version: {}", parent_version),
                Some(&context.installation.id),
            );

            // Ensure the parent vanilla version exists (downloads if missing)
            let resolved_parent = crate::launcher::utils::ensure_version_manifest_and_jar(
                parent_version,
                &context.minecraft_dir,
            )
            .await
            .map_err(|e| format!("Failed to ensure parent Minecraft version: {e}"))?;

            // Copy the vanilla JAR to the Fabric version folder
            let parent_jar = versions_dir
                .join(&resolved_parent)
                .join(format!("{}.jar", resolved_parent));

            if !parent_jar.exists() {
                return Err(format!(
                    "Parent Minecraft JAR not found: {}",
                    parent_jar.display()
                ));
            }

            crate::logging::Logger::debug_global(
                &format!("Copying vanilla JAR from: {}", parent_jar.display()),
                Some(&context.installation.id),
            );

            tokio::fs::copy(&parent_jar, &fabric_jar)
                .await
                .map_err(|e| format!("Failed to copy vanilla JAR to Fabric version: {e}"))?;

            crate::logging::Logger::debug_global(
                &format!(
                    "Created Fabric version JAR (copy of vanilla): {}",
                    fabric_jar.display()
                ),
                Some(&context.installation.id),
            );
        } else {
            // Fabric files already exist, just read the existing JSON for library checking
            crate::logging::Logger::debug_global(
                "Fabric manifest and JAR already exist, reading existing JSON",
                Some(&context.installation.id),
            );

            profile_json = tokio::fs::read_to_string(&fabric_json)
                .await
                .map_err(|e| format!("Failed to read existing Fabric profile JSON: {e}"))?;

            // Extract mc_version from the existing installation metadata
            use crate::installations::get_version;
            let version_data = get_version(version_id.clone()).await;
            if let Some(version_data) = version_data {
                if let Some(mc_ver) = version_data
                    .extra
                    .get("minecraft_version")
                    .and_then(|v| v.as_str())
                {
                    mc_version = mc_ver.to_string();
                } else {
                    return Err("No 'minecraft_version' in version.extra for Fabric".to_string());
                }
            } else {
                return Err("Could not find version data for installation's version_id".to_string());
            }
        }

        // 8. Ensure all libraries (including Fabric Loader) are downloaded
        // This is CRITICAL - without this, the Fabric Loader JAR won't be downloaded from Maven!
        crate::logging::Logger::debug_global(
            "Ensuring Fabric libraries are downloaded...",
            Some(&context.installation.id),
        );

        // Re-parse the JSON to get the manifest for library checking
        let manifest_for_libs: serde_json::Value = serde_json::from_str(&profile_json)
            .map_err(|e| format!("Failed to parse profile JSON for library check: {e}"))?;

        let libraries_path = PathBuf::from(&context.minecraft_dir).join("libraries");
        crate::launcher::utils::ensure_libraries(&manifest_for_libs, &libraries_path)
            .await
            .map_err(|e| format!("Failed to ensure Fabric libraries: {e}"))?;

        crate::logging::Logger::debug_global(
            "All Fabric libraries downloaded successfully",
            Some(&context.installation.id),
        );

        // 9. For IrisFabric, download and install Iris+Sodium mods
        if version_id.contains("iris-fabric") {
            crate::logging::Logger::debug_global(
                "Downloading Iris+Sodium mods...",
                Some(&context.installation.id),
            );

            let is_beta = version_id.contains("beta");
            let zip_name = if is_beta {
                format!("Iris-Sodium-Beta-{}.zip", &mc_version)
            } else {
                format!("Iris-Sodium-{}.zip", &mc_version)
            };

            let download_url = format!(
                "https://github.com/IrisShaders/Iris-Installer-Files/releases/latest/download/{}",
                zip_name
            );

            // Create mods directory
            let minecraft_dir = std::path::PathBuf::from(&context.minecraft_dir);
            let mods_folder = if is_beta {
                minecraft_dir.join("iris-beta-reserved").join(&mc_version)
            } else {
                minecraft_dir.join("iris-reserved").join(&mc_version)
            };

            crate::ensure_folder_sync(&mods_folder)
                .map_err(|e| format!("Failed to create Iris mods folder: {e}"))?;

            // Download ZIP file
            crate::logging::Logger::debug_global(
                &format!("Downloading from: {}", download_url),
                Some(&context.installation.id),
            );

            let zip_response = client
                .get(&download_url)
                .send()
                .await
                .map_err(|e| format!("Failed to download Iris mods ZIP: {e}"))?;

            let zip_bytes = zip_response
                .bytes()
                .await
                .map_err(|e| format!("Failed to read Iris mods ZIP: {e}"))?;

            // Extract ZIP to mods folder
            let cursor = std::io::Cursor::new(zip_bytes);
            let mut archive = zip::ZipArchive::new(cursor)
                .map_err(|e| format!("Failed to open Iris mods ZIP: {e}"))?;

            for i in 0..archive.len() {
                let mut file = archive
                    .by_index(i)
                    .map_err(|e| format!("Failed to read ZIP entry: {e}"))?;

                let file_name = file.name().to_string();

                // Only extract files from the "mods/" directory
                if file_name.starts_with("mods/") && !file.is_dir() {
                    let mod_name = file_name.strip_prefix("mods/").unwrap();
                    let mod_path = mods_folder.join(mod_name);

                    crate::logging::Logger::debug_global(
                        &format!("Extracting: {}", mod_name),
                        Some(&context.installation.id),
                    );

                    let mut out_file = std::fs::File::create(&mod_path)
                        .map_err(|e| format!("Failed to create mod file {}: {e}", mod_name))?;

                    std::io::copy(&mut file, &mut out_file)
                        .map_err(|e| format!("Failed to extract mod {}: {e}", mod_name))?;
                }
            }

            crate::logging::Logger::debug_global(
                &format!("Installed Iris+Sodium mods to: {}", mods_folder.display()),
                Some(&context.installation.id),
            );
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

        // 5. Remove any -Dfabric.modsFolder from JVM args (we'll set it correctly below)
        cleaned_jvm_args.retain(|arg| !arg.starts_with("-Dfabric.modsFolder="));

        // 6. Prepend installation-specific JVM args (Vec<String>) if present
        if !context.installation.java_args.is_empty() {
            cleaned_jvm_args.splice(0..0, context.installation.java_args.clone());
        }

        // 7. Add/overwrite with parameters_map (for --key style)
        for (k, v) in &context.installation.parameters_map {
            if k.starts_with("--") {
                cleaned_jvm_args.push(k.clone());
                if !v.is_empty() {
                    cleaned_jvm_args.push(v.clone());
                }
            }
        }

        // 8. Set the correct Fabric mods folder
        // Priority: dedicated_mods_folder > default Fabric mods folder
        let mods_path = if let Some(mods_folder) = &context.installation.dedicated_mods_folder {
            // Use the installation's dedicated mods folder
            let p = PathBuf::from(mods_folder);
            if p.is_absolute() {
                p
            } else {
                // Relative paths are relative to .minecraft/kable/
                // They already include the folder type prefix (e.g., "mods/{id}")
                PathBuf::from(&context.minecraft_dir).join("kable").join(p)
            }
        } else {
            // Default: use standard Fabric mods folder
            PathBuf::from(&context.minecraft_dir).join("mods")
        };

        // Set the mods folder via JVM property (this is what Fabric Loader reads)
        cleaned_jvm_args.push(format!(
            "-Dfabric.modsFolder={}",
            mods_path.to_string_lossy()
        ));

        crate::logging::Logger::debug_global(
            &format!("Using Fabric mods folder: {}", mods_path.display()),
            Some(&context.installation.id),
        );

        // 9. Game args (no mods folder override needed here, JVM property takes precedence)
        let final_game_args_vec = game_args_vec.clone();

        // 10. Build command: exactly like vanilla (single -cp, correct order)
        let java_path = crate::launcher::java::find_java_executable(context.settings.general.java_path.as_ref())?;

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
