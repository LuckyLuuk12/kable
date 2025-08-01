use std::fs;
use std::path::{Path, PathBuf};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Command;
use tauri::Emitter;
use crate::launchables::LaunchContext;
use tokio::io::{BufReader};
use serde::{Deserialize, Serialize};
use crate::logging::{Logger, LogLevel};


/// Loads a Minecraft version manifest, recursively merging inherited manifests if needed.
/// Returns the fully merged manifest as serde_json::Value.
pub fn load_and_merge_manifest_with_instance(minecraft_dir: &str, version_id: &str, instance_id: Option<&str>) -> Result<Value, String> {
    if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
        if let Some(app_handle) = handle_guard.as_ref() {
            Logger::log(app_handle, LogLevel::Info, &format!("Loading manifest for version_id: {}", version_id), instance_id);
        } else {
            Logger::debug_global(&format!("Loading manifest for version_id: {}", version_id), instance_id);
        }
    } else {
        Logger::debug_global(&format!("Loading manifest for version_id: {}", version_id), instance_id);
    }
    let manifest_path = PathBuf::from(minecraft_dir)
        .join("versions").join(version_id).join(format!("{}.json", version_id));
    let manifest_str = match fs::read_to_string(&manifest_path) {
        Ok(s) => s,
        Err(e) => {
            if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
                if let Some(app_handle) = handle_guard.as_ref() {
                    Logger::log(app_handle, LogLevel::Error, &format!("Failed to read manifest for {}: {}", version_id, e), instance_id);
                } else {
                    Logger::debug_global(&format!("Failed to read manifest for {}: {}", version_id, e), instance_id);
                }
            } else {
                Logger::debug_global(&format!("Failed to read manifest for {}: {}", version_id, e), instance_id);
            }
            return Err(format!("Failed to read manifest: {}", e));
        }
    };
    let mut manifest: Value = match serde_json::from_str(&manifest_str) {
        Ok(m) => m,
        Err(e) => {
            if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
                if let Some(app_handle) = handle_guard.as_ref() {
                    Logger::log(app_handle, LogLevel::Error, &format!("Failed to parse manifest for {}: {}", version_id, e), instance_id);
                } else {
                    Logger::debug_global(&format!("Failed to parse manifest for {}: {}", version_id, e), instance_id);
                }
            } else {
                Logger::debug_global(&format!("Failed to parse manifest for {}: {}", version_id, e), instance_id);
            }
            return Err(format!("Failed to parse manifest: {}", e));
        }
    };
    // If inheritsFrom, recursively merge
    if let Some(parent_id) = manifest.get("inheritsFrom").and_then(|v| v.as_str()) {
        if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
            if let Some(app_handle) = handle_guard.as_ref() {
                Logger::log(app_handle, LogLevel::Info, &format!("Manifest {} inherits from {}. Recursively merging...", version_id, parent_id), instance_id);
            } else {
                Logger::debug_global(&format!("Manifest {} inherits from {}. Recursively merging...", version_id, parent_id), instance_id);
            }
        } else {
            Logger::debug_global(&format!("Manifest {} inherits from {}. Recursively merging...", version_id, parent_id), instance_id);
        }
        let parent = load_and_merge_manifest_with_instance(minecraft_dir, parent_id, instance_id)?;
        manifest = merge_manifests_with_instance(parent, manifest, instance_id);
    }
    if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
        if let Some(app_handle) = handle_guard.as_ref() {
            Logger::log(app_handle, LogLevel::Info, &format!("Loaded and merged manifest for version_id: {}", version_id), instance_id);
        } else {
            Logger::debug_global(&format!("Loaded and merged manifest for version_id: {}", version_id), instance_id);
        }
    } else {
        Logger::debug_global(&format!("Loaded and merged manifest for version_id: {}", version_id), instance_id);
    }
    Ok(manifest)
}

// Backwards compatible version for callers that don't have instance_id
pub fn load_and_merge_manifest(minecraft_dir: &str, version_id: &str) -> Result<Value, String> {
    load_and_merge_manifest_with_instance(minecraft_dir, version_id, None)
}

/// Merges two manifests (parent, child), with child values taking precedence.
pub fn merge_manifests_with_instance(parent: Value, child: Value, instance_id: Option<&str>) -> Value {
    if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
        if let Some(app_handle) = handle_guard.as_ref() {
            Logger::log(app_handle, LogLevel::Debug, "Merging manifests (child overrides parent)", instance_id);
        } else {
            Logger::debug_global("Merging manifests (child overrides parent)", instance_id);
        }
    } else {
        Logger::debug_global("Merging manifests (child overrides parent)", instance_id);
    }
    match (parent, child) {
        (Value::Object(mut p), Value::Object(c)) => {
            for (k, v) in c {
                let key = k.clone();
                let old_value = p.remove(&key).unwrap_or(Value::Null);
                p.insert(k, merge_manifests_with_instance(old_value, v, instance_id));
            }
            Value::Object(p)
        }
        (_, v) => v,
    }
}

pub fn merge_manifests(parent: Value, child: Value) -> Value {
    merge_manifests_with_instance(parent, child, None)
}

/// Builds the Java classpath string from a merged manifest, libraries path, and version jar path.
/// This is the single source of classpath logic for all loaders (vanilla, fabric, etc).
/// It includes all libraries (with rules applied) and the version JAR as the last entry.
/// No loader JAR filtering or prepending is needed; for Fabric, the version JAR is the loader JAR.
pub fn build_classpath_from_manifest_with_instance(manifest: &Value, libraries_path: &Path, version_jar_path: &Path, instance_id: Option<&str>) -> String {
    if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
        if let Some(app_handle) = handle_guard.as_ref() {
            Logger::log(app_handle, LogLevel::Info, "Building classpath from manifest", instance_id);
        } else {
            Logger::debug_global("Building classpath from manifest", instance_id);
        }
    } else {
        Logger::debug_global("Building classpath from manifest", instance_id);
    }
    // Deduplicate by group:artifact (or full name for natives), keeping only the last occurrence (child overrides parent)
    let mut dedup_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    if let Some(libs) = manifest.get("libraries").and_then(|v| v.as_array()) {
        for lib in libs {
            if let Some(obj) = lib.as_object() {
                let mut jar_path_opt = None;
                if let Some(downloads) = obj.get("downloads") {
                    if let Some(artifact) = downloads.get("artifact") {
                        if let Some(path) = artifact.get("path").and_then(|v| v.as_str()) {
                            let jar_path = libraries_path.join(path);
                            jar_path_opt = Some(jar_path.to_string_lossy().to_string());
                        }
                    }
                }
                if jar_path_opt.is_none() {
                    if let Some(name_val) = obj.get("name") {
                        if let Some(name) = name_val.as_str() {
                            if let Some(jar_path) = crate::launcher::utils::try_find_library_manually(name, libraries_path) {
                                if jar_path.exists() {
                                    jar_path_opt = Some(jar_path.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                }
                if let Some(jar_path) = jar_path_opt {
                    // Deduplication key: group:artifact for normal, full name for natives
                    let dedup_key = if let Some(name_val) = obj.get("name") {
                        if let Some(name) = name_val.as_str() {
                            if name.contains(":natives-") {
                                name.to_string()
                            } else {
                                let parts: Vec<&str> = name.split(':').collect();
                                if parts.len() >= 2 {
                                    format!("{}:{}", parts[0], parts[1])
                                } else {
                                    name.to_string()
                                }
                            }
                        } else { jar_path.clone() }
                    } else { jar_path.clone() };
                    dedup_map.insert(dedup_key, jar_path);
                }
            }
        }
    }
    let mut entries: Vec<String> = dedup_map.into_values().collect();
    entries.push(version_jar_path.to_string_lossy().to_string());
    let sep = if cfg!(windows) { ";" } else { ":" };
    let classpath = entries.join(sep);
    if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
        if let Some(app_handle) = handle_guard.as_ref() {
            Logger::log(app_handle, LogLevel::Info, &format!("Classpath built: {} entries", entries.len()), instance_id);
        } else {
            Logger::debug_global(&format!("Classpath built: {} entries", entries.len()), instance_id);
        }
    } else {
        Logger::debug_global(&format!("Classpath built: {} entries", entries.len()), instance_id);
    }
    classpath
}

pub fn build_classpath_from_manifest(manifest: &Value, libraries_path: &Path, version_jar_path: &Path) -> String {
    build_classpath_from_manifest_with_instance(manifest, libraries_path, version_jar_path, None)
}

/// Builds JVM and game arguments from a merged manifest and variable map.
pub fn build_jvm_and_game_args_with_instance(manifest: &Value, variables: &std::collections::HashMap<String, String>, instance_id: Option<&str>) -> (Vec<String>, Vec<String>) {
    if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
        if let Some(app_handle) = handle_guard.as_ref() {
            Logger::log(app_handle, LogLevel::Info, "Building JVM and game arguments from manifest", instance_id);
        } else {
            Logger::debug_global("Building JVM and game arguments from manifest", instance_id);
        }
    } else {
        Logger::debug_global("Building JVM and game arguments from manifest", instance_id);
    }
    let arguments = manifest.get("arguments").and_then(|v| v.as_object()).expect("No arguments in manifest");
    let empty_vec = Vec::new();
    let jvm_args = arguments.get("jvm").and_then(|v| v.as_array()).unwrap_or(&empty_vec);
    let game_args = arguments.get("game").and_then(|v| v.as_array()).unwrap_or(&empty_vec);
    let jvm_args_vec = process_arguments(jvm_args, variables);
    let game_args_vec = process_arguments(game_args, variables);
    if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
        if let Some(app_handle) = handle_guard.as_ref() {
            Logger::log(app_handle, LogLevel::Info, &format!("JVM args: {:?}", jvm_args_vec), instance_id);
            Logger::log(app_handle, LogLevel::Info, &format!("Game args: {:?}", game_args_vec), instance_id);
        } else {
            Logger::debug_global(&format!("JVM args: {:?}", jvm_args_vec), instance_id);
            Logger::debug_global(&format!("Game args: {:?}", game_args_vec), instance_id);
        }
    } else {
        Logger::debug_global(&format!("JVM args: {:?}", jvm_args_vec), instance_id);
        Logger::debug_global(&format!("Game args: {:?}", game_args_vec), instance_id);
    }
    (jvm_args_vec, game_args_vec)
}

pub fn build_jvm_and_game_args(manifest: &Value, variables: &std::collections::HashMap<String, String>) -> (Vec<String>, Vec<String>) {
    build_jvm_and_game_args_with_instance(manifest, variables, None)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Library {
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
    pub rules: Option<Vec<Rule>>,
    pub natives: Option<HashMap<String, String>>,
    pub extract: Option<Extract>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artifact {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
    pub action: String,
    pub os: Option<OsRule>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OsRule {
    pub name: Option<String>,
    pub arch: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Extract {
    pub exclude: Option<Vec<String>>,
}

// --- Variable substitution and argument processing ---
/// Substitute variables in a template string with values from the provided map.
///
/// Variables are in the form `${key}` and are replaced with the corresponding value from `variables`.
/// Any unreplaced placeholders are removed.
/// Used for argument and path processing in launcher modules.
///
/// # Arguments
/// * `template` - The string containing placeholders.
/// * `variables` - Map of variable names to values.
///
/// # Returns
/// The string with all placeholders replaced or removed.
pub fn substitute_variables(template: &str, variables: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in variables {
        let placeholder = format!("${{{}}}", key);
        result = result.replace(&placeholder, value);
    }
    let re = regex::Regex::new(r"\$\{[^}]+\}").unwrap();
    result = re.replace_all(&result, "").to_string();
    result
}

/// Processes a list of Minecraft-style argument definitions (from version JSONs),
/// performing variable substitution and rule evaluation.
///
/// Handles both string and object argument forms, including OS/feature rules and arrays.
/// Used by all loader modules to build the JVM and game argument lists.
///
/// # Arguments
/// * `args` - Slice of serde_json::Value representing arguments (from JSON files).
/// * `variables` - Map of variable names to values for substitution.
///
/// # Returns
/// Vector of processed argument strings, ready for command-line use.
pub fn process_arguments(args: &[Value], variables: &HashMap<String, String>) -> Vec<String> {
    let mut processed = Vec::new();
    for arg in args {
        match arg {
            Value::String(s) => {
                let substituted = substitute_variables(s, variables);
                // Fix -Dkey = value or -Dkey= value or -Dkey =value to -Dkey=value (no spaces)
                if substituted.starts_with("-D") {
                    // Try to fix only the first '=' occurrence
                    let mut parts = substituted.splitn(2, '=');
                    if let (Some(left), Some(right)) = (parts.next(), parts.next()) {
                        let left = left.trim_end();
                        let right = right.trim_start();
                        processed.push(format!("{}={}", left, right));
                    } else {
                        processed.push(substituted);
                    }
                } else {
                    processed.push(substituted);
                }
            },
            Value::Object(obj) => {
                if let Some(rules) = obj.get("rules") {
                    if !evaluate_rules(rules).unwrap_or(true) { continue; }
                }
                if let Some(val) = obj.get("value") {
                    match val {
                        Value::String(s) => {
                            let substituted = substitute_variables(s, variables);
                            if !substituted.trim().is_empty() && !is_problematic_argument(&substituted) {
                                processed.push(substituted);
                            }
                        },
                        Value::Array(arr) => {
                            for v in arr {
                                if let Some(s) = v.as_str() {
                                    let substituted = substitute_variables(s, variables);
                                    if !substituted.trim().is_empty() && !is_problematic_argument(&substituted) {
                                        processed.push(substituted);
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                }
            },
            _ => {}
        }
    }
    processed
}

/// Evaluates a set of Minecraft-style rules (from version JSONs) to determine if an argument or library should be included.
///
/// Supports OS-based rules. Feature-based rules are currently disabled (always false).
/// Used by argument and library processing in all loader modules.
///
/// # Arguments
/// * `rules` - serde_json::Value representing the rules array.
///
/// # Returns
/// Ok(true) if allowed, Ok(false) if disallowed, Err if evaluation fails.
pub fn evaluate_rules(rules: &Value) -> Result<bool, String> {
    if let Value::Array(rules_array) = rules {
        for rule in rules_array {
            if let Value::Object(rule_obj) = rule {
                let action = rule_obj.get("action")
                    .and_then(|v| v.as_str())
                    .unwrap_or("disallow");
                let mut condition_met = true;
                if let Some(os_condition) = rule_obj.get("os") {
                    condition_met &= evaluate_os_condition(os_condition)?;
                }
                if let Some(_features) = rule_obj.get("features") {
                    condition_met = false; // Disable feature-dependent arguments for now
                }
                if condition_met {
                    return Ok(action == "allow");
                }
            }
        }
    }
    Ok(true)
}

/// Checks if the current OS matches the given OS rule from a Minecraft version JSON.
///
/// Used by `evaluate_rules` for OS-based argument/library inclusion.
///
/// # Arguments
/// * `os_condition` - serde_json::Value representing the OS rule object.
///
/// # Returns
/// Ok(true) if the OS matches, Ok(false) otherwise.
pub fn evaluate_os_condition(os_condition: &Value) -> Result<bool, String> {
    if let Value::Object(os_obj) = os_condition {
        if let Some(name) = os_obj.get("name").and_then(|v| v.as_str()) {
            let current_os = std::env::consts::OS;
            let matches = match name {
                "windows" => current_os == "windows",
                "osx" => current_os == "macos",
                "linux" => current_os == "linux",
                _ => false,
            };
            return Ok(matches);
        }
    }
    Ok(false)
}

/// Returns true if the argument is known to cause issues or is not supported by this launcher.
///
/// Used to filter out problematic JVM/game arguments (e.g., --demo, --quickPlay).
///
/// # Arguments
/// * `arg` - Argument string to check.
///
/// # Returns
/// true if the argument should be excluded, false otherwise.
pub fn is_problematic_argument(arg: &str) -> bool {
    if cfg!(not(target_os = "macos")) && arg == "-XstartOnFirstThread" {
        return true;
    }
    if arg == "--demo" {
        return true;
    }
    if arg.starts_with("--quickPlay") || arg.starts_with("--quick-play") {
        return true;
    }
    false
}

// --- Classpath and library utilities ---
/// Builds the Java classpath string from a list of Minecraft libraries and the main version JAR.
///
/// Used by all loader modules to construct the classpath for launching Minecraft.
///
/// # Arguments
/// * `libraries` - List of Library structs (from version JSONs).
/// * `libraries_path` - Path to the root of the libraries directory.
/// * `version_jar_path` - Path to the main Minecraft version JAR.
///
/// # Returns
/// Ok(classpath string) or Err if any error occurs.
pub fn build_classpath(
    libraries: &[Library],
    libraries_path: &Path,
    version_jar_path: &Path
) -> Result<String, String> {
    let mut classpath_entries = Vec::new();
    for library in libraries {
        if let Some(downloads) = &library.downloads {
            if let Some(artifact) = &downloads.artifact {
                let lib_path = libraries_path.join(&artifact.path);
                if lib_path.exists() {
                    classpath_entries.push(lib_path.to_string_lossy().to_string());
                }
            }
        }
    }
    classpath_entries.push(version_jar_path.to_string_lossy().to_string());
    let separator = if cfg!(windows) { ";" } else { ":" };
    Ok(classpath_entries.join(separator))
}

/// Attempts to construct the path to a library JAR manually from its Maven-style name.
///
/// Used as a fallback if the library is not found in the downloads/artifact field.
///
/// # Arguments
/// * `library_name` - Maven-style library name (e.g., group:artifact:version).
/// * `libraries_path` - Path to the root of the libraries directory.
///
/// # Returns
/// Some(PathBuf) if the path can be constructed, None otherwise.
pub fn try_find_library_manually(library_name: &str, libraries_path: &Path) -> Option<PathBuf> {
    let parts: Vec<&str> = library_name.split(':').collect();
    if parts.len() >= 3 {
        let group = parts[0].replace('.', "/");
        let artifact = parts[1];
        let version = parts[2];
        let jar_name = format!("{}-{}.jar", artifact, version);
        let jar_path = libraries_path.join(&group).join(artifact).join(version).join(jar_name);
        return Some(jar_path);
    }
    None
}

/// Deduplicates a list of Minecraft libraries, keeping only one entry per group:artifact (or full name for natives).
///
/// Used to avoid duplicate classpath entries and native conflicts.
///
/// # Arguments
/// * `libraries` - Vector of Library structs.
///
/// # Returns
/// Deduplicated vector of Library structs.
pub fn deduplicate_libraries(libraries: Vec<Library>) -> Vec<Library> {
    let mut library_map: HashMap<String, Library> = HashMap::new();
    let mut fabric_loader_key: Option<String> = None;
    let mut fabric_loader_lib: Option<Library> = None;
    for library in libraries.into_iter() {
        let dedup_key = if library.name.contains(":natives-") {
            library.name.clone()
        } else {
            let parts: Vec<&str> = library.name.split(':').collect();
            if parts.len() >= 2 {
                format!("{}:{}", parts[0], parts[1])
            } else {
                library.name.clone()
            }
        };
        // Special case: always keep the net.fabricmc:fabric-loader entry from libraries
        if dedup_key == "net.fabricmc:fabric-loader" {
            // If we already found a fabric-loader, prefer the one from libraries (with downloads.artifact)
            let is_real_loader = library.downloads.as_ref().and_then(|d| d.artifact.as_ref()).is_some();
            if is_real_loader {
                fabric_loader_key = Some(dedup_key.clone());
                fabric_loader_lib = Some(library);
                continue;
            }
        }
        library_map.insert(dedup_key, library);
    }
    // Insert the real fabric-loader from libraries if found
    if let (Some(key), Some(lib)) = (fabric_loader_key, fabric_loader_lib) {
        library_map.insert(key, lib);
    }
    library_map.into_values().collect()
}

// --- Java and JVM utilities ---
/// Ensures the version manifest JSON and JAR exist for the given version_id in minecraft_dir.
/// Downloads them from Mojang if missing.
pub async fn ensure_version_manifest_and_jar(version_id: &str, minecraft_dir: &str) -> Result<(), String> {
    use reqwest::Client;
    use std::fs;
    use std::path::PathBuf;
    let versions_dir = PathBuf::from(minecraft_dir).join("versions").join(version_id);
    fs::create_dir_all(&versions_dir).map_err(|e| format!("Failed to create versions dir: {e}"))?;
    let manifest_path = versions_dir.join(format!("{}.json", version_id));
    let jar_path = versions_dir.join(format!("{}.jar", version_id));
    // Download manifest JSON if missing
    if !manifest_path.exists() {
        let manifest_url = format!("https://launchermeta.mojang.com/v1/packages/{}/{}.json", version_id, version_id);
        // Fallback: Use version list to get URL
        let version_list_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
        let client = Client::new();
        let resp = client.get(version_list_url).send().await.map_err(|e| format!("Failed to fetch version list: {e}"))?;
        let manifest: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse version list: {e}"))?;
        let versions = manifest.get("versions").and_then(|v| v.as_array()).ok_or("No versions array")?;
        let version_obj = versions.iter().find(|v| v.get("id").and_then(|id| id.as_str()) == Some(version_id)).ok_or("Version not found")?;
        let url = version_obj.get("url").and_then(|v| v.as_str()).ok_or("No url for version")?;
        let resp = client.get(url).send().await.map_err(|e| format!("Failed to fetch manifest: {e}"))?;
        let manifest_str = resp.text().await.map_err(|e| format!("Failed to get manifest text: {e}"))?;
        fs::write(&manifest_path, manifest_str).map_err(|e| format!("Failed to write manifest: {e}"))?;
    }
    // Download JAR if missing
    if !jar_path.exists() {
        let manifest_str = fs::read_to_string(&manifest_path).map_err(|e| format!("Failed to read manifest: {e}"))?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_str).map_err(|e| format!("Failed to parse manifest: {e}"))?;
        let downloads = manifest.get("downloads").and_then(|v| v.as_object()).ok_or("No downloads object")?;
        let client = Client::new();
        if let Some(client_obj) = downloads.get("client").and_then(|v| v.as_object()) {
            let url = client_obj.get("url").and_then(|v| v.as_str()).ok_or("No client jar url")?;
            let resp = client.get(url).send().await.map_err(|e| format!("Failed to fetch jar: {e}"))?;
            let bytes = resp.bytes().await.map_err(|e| format!("Failed to get jar bytes: {e}"))?;
            fs::write(&jar_path, &bytes).map_err(|e| format!("Failed to write jar: {e}"))?;
        } else {
            return Err("No client jar info in manifest".to_string());
        }
    }
    Ok(())
}

/// Ensures all libraries listed in the manifest exist in libraries_path. Downloads any missing ones.
pub async fn ensure_libraries(manifest: &serde_json::Value, libraries_path: &std::path::Path) -> Result<(), String> {
    use reqwest::Client;
    use std::fs;
    let client = Client::new();
    if let Some(libs) = manifest.get("libraries").and_then(|v| v.as_array()) {
        for lib in libs {
            if let Some(obj) = lib.as_object() {
                let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let downloads = obj.get("downloads").and_then(|v| v.as_object());
                if let Some(downloads) = downloads {
                    if let Some(artifact) = downloads.get("artifact").and_then(|v| v.as_object()) {
                        let path = artifact.get("path").and_then(|v| v.as_str()).unwrap_or("");
                        let url = artifact.get("url").and_then(|v| v.as_str()).unwrap_or("");
                        let jar_path = libraries_path.join(path);
                        if !jar_path.exists() {
                            if !jar_path.parent().unwrap().exists() {
                                fs::create_dir_all(jar_path.parent().unwrap()).map_err(|e| format!("Failed to create lib dir: {e}"))?;
                            }
                            let resp = client.get(url).send().await.map_err(|e| format!("Failed to fetch lib: {e}"))?;
                            let bytes = resp.bytes().await.map_err(|e| format!("Failed to get lib bytes: {e}"))?;
                            fs::write(&jar_path, &bytes).map_err(|e| format!("Failed to write lib: {e}"))?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
/// Attempts to find a working Java executable, either from the provided path or common install locations.
///
/// Used by all loader modules to locate Java for launching Minecraft.
///
/// # Arguments
/// * `java_path` - Optional user-specified Java path.
///
/// # Returns
/// Ok(path to Java executable) or Err if not found.
pub fn find_java_executable(java_path: Option<&String>) -> Result<String, String> {
    if let Some(path) = java_path {
        if PathBuf::from(path).exists() {
            return Ok(path.clone());
        }
    }
    let java_candidates = if cfg!(windows) {
        vec![
            "java".to_string(),
            "C:\\Program Files\\Java\\jre1.8.0_301\\bin\\java.exe".to_string(),
            "C:\\Program Files\\Eclipse Adoptium\\jdk-17.0.2.8-hotspot\\bin\\java.exe".to_string(),
            "C:\\Program Files\\Eclipse Adoptium\\jdk-21.0.1.12-hotspot\\bin\\java.exe".to_string(),
        ]
    } else {
        vec![
            "java".to_string(),
            "/usr/bin/java".to_string(),
            "/usr/lib/jvm/default-java/bin/java".to_string(),
        ]
    };
    for candidate in java_candidates {
        if let Ok(output) = Command::new(&candidate).arg("-version").output() {
            if output.status.success() {
                return Ok(candidate);
            }
        }
    }
    Err("Java not found. Please install Java or specify the Java path in settings.".to_string())
}

/// Tauri command: Returns the path to a working Java executable, using the provided path or searching common locations.
///
/// Used by the frontend to validate or auto-detect Java installations.
///
/// # Arguments
/// * `java_path` - Optional user-specified Java path.
///
/// # Returns
/// Ok(path to Java executable) or Err if not found.
#[tauri::command]
pub fn get_java_path(java_path: Option<String>) -> Result<String, String> {
    find_java_executable(java_path.as_ref())
}

// --- Native extraction ---
/// Extracts native libraries from Minecraft library JARs into the given natives directory.
///
/// Used by all loader modules to prepare the environment for launching Minecraft.
///
/// # Arguments
/// * `libraries` - List of Library structs (from version JSONs).
/// * `libraries_path` - Path to the root of the libraries directory.
/// * `natives_path` - Path to the directory where natives should be extracted.
///
/// # Returns
/// Ok(()) if successful, Err if extraction fails.
pub fn extract_natives(
    libraries: &[Library],
    libraries_path: &Path,
    natives_path: &PathBuf,
) -> Result<(), String> {
    if natives_path.exists() {
        std::fs::remove_dir_all(natives_path)
            .map_err(|e| format!("Failed to clear natives directory: {}", e))?;
    }
    std::fs::create_dir_all(natives_path)
        .map_err(|e| format!("Failed to create natives directory: {}", e))?;
    let current_os = std::env::consts::OS;
    let natives_classifier = match current_os {
        "windows" => "natives-windows",
        "macos" => "natives-macos",
        "linux" => "natives-linux",
        _ => return Err(format!("Unsupported OS: {}", current_os)),
    };
    for library in libraries {
        if let Some(rules) = &library.rules {
            let rules_value = serde_json::to_value(rules)
                .map_err(|e| format!("Failed to serialize rules: {}", e))?;
            if !evaluate_rules(&rules_value)? {
                continue;
            }
        }
        if let Some(downloads) = &library.downloads {
            if let Some(classifiers) = &downloads.classifiers {
                if let Some(native_artifact) = classifiers.get(natives_classifier) {
                    let native_path = libraries_path.join(&native_artifact.path);
                    if native_path.exists() {
                        extract_jar(&native_path, natives_path)?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Extracts all files from a JAR (ZIP) archive to the specified directory.
///
/// Used internally by `extract_natives` for native library extraction.
///
/// # Arguments
/// * `jar_path` - Path to the JAR file.
/// * `extract_to` - Directory to extract files into.
///
/// # Returns
/// Ok(()) if successful, Err if extraction fails.
fn extract_jar(jar_path: &PathBuf, extract_to: &Path) -> Result<(), String> {
    let file = std::fs::File::open(jar_path)
        .map_err(|e| format!("Failed to open JAR: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry: {}", e))?;
        let outpath = match file.enclosed_name() {
            Some(path) => extract_to.join(path),
            None => continue,
        };
        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file: {}", e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {}", e))?;
        }
    }
    Ok(())
}

// --- Variable map builder ---
/// Builds a map of variable substitutions for Minecraft argument templates, based on the launch context, manifest, and version info.
///
/// Used by all loader modules to provide variables for argument processing.
///
/// # Arguments
/// * `context` - LaunchContext struct containing user, account, and directory info.
/// * `manifest` - Optional serde_json::Value representing the version/loader manifest (for manifest-specific fields).
/// * `classpath` - Computed classpath string.
/// * `parameters_map` - Optional map of extra/override variables (e.g., from installation parameters).
///
/// # Returns
/// HashMap of variable names to values for use in argument templates.
pub fn build_variable_map(
    context: &LaunchContext,
    manifest: Option<&serde_json::Value>,
    classpath: &str,
    parameters_map: Option<&std::collections::HashMap<String, String>>,
) -> HashMap<String, String> {
    let mut variables = HashMap::new();
    // Authentication
    variables.insert("auth_player_name".to_string(), context.account.username.clone());
    variables.insert("auth_uuid".to_string(), context.account.minecraft_profile.id.clone());
    let has_valid_token = !context.account.access_token.is_empty();
    variables.insert("auth_access_token".to_string(), if has_valid_token { context.account.access_token.clone() } else { "offline".to_string() });
    variables.insert("auth_xuid".to_string(), context.account.minecraft_profile.id.clone());
    variables.insert("user_type".to_string(), if has_valid_token { "microsoft" } else { "offline" }.to_string());
    variables.insert("clientid".to_string(), uuid::Uuid::new_v4().to_string());
    // Version info
    if let Some(manifest) = manifest {
        // version_name
        if let Some(version_name) = manifest.get("id").and_then(|v| v.as_str()) {
            variables.insert("version_name".to_string(), version_name.to_string());
        } else if !context.installation.version_id.is_empty() {
            variables.insert("version_name".to_string(), context.installation.version_id.clone());
        } else {
            variables.insert("version_name".to_string(), "".to_string());
        }
        // version_type
        if let Some(version_type) = manifest.get("type").and_then(|v| v.as_str()) {
            variables.insert("version_type".to_string(), version_type.to_string());
        }
        // assets_index_name
        if let Some(assets_index_name) = manifest.get("assets").and_then(|v| v.as_str()) {
            variables.insert("assets_index_name".to_string(), assets_index_name.to_string());
        } else {
            variables.insert("assets_index_name".to_string(), "".to_string());
        }
    } else {
        variables.insert(
            "version_name".to_string(),
            context.installation.version_id.clone(),
        );
        variables.insert("assets_index_name".to_string(), "".to_string());
    }
    variables.insert("launcher_name".to_string(), "Kable".to_string());
    variables.insert("launcher_version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    variables.insert("classpath".to_string(), classpath.to_string());
    // Paths
    variables.insert("game_directory".to_string(), context.minecraft_dir.clone());
    variables.insert("assets_root".to_string(), PathBuf::from(&context.minecraft_dir).join("assets").to_string_lossy().to_string());
    variables.insert("natives_directory".to_string(), PathBuf::from(&context.minecraft_dir).join("natives").to_string_lossy().to_string());
    // Resolution
    variables.insert("resolution_width".to_string(), "1024".to_string());
    variables.insert("resolution_height".to_string(), "768".to_string());
    // Merge/overwrite with parameters_map if provided
    if let Some(params) = parameters_map {
        for (k, v) in params {
            if k.starts_with("--") {
                // Will be handled as extra args, not as variable
                continue;
            }
            variables.insert(k.clone(), v.clone());
        }
    }
    variables
}


/// Spawns a process, streams stdout/stderr, and logs each line to the logger with the given instance_id.
/// Returns the process PID and command string.
pub async fn spawn_and_log_process(
    cmd: Command,
    working_dir: &str,
    instance_id: &str,
    profile: &serde_json::Value,
    installation: &serde_json::Value,
) -> Result<crate::launcher::LaunchResult, String> {
    use serde_json::json;
    use crate::logging::LogLevel;
    use crate::logging::Logger;
    use tokio::io::AsyncBufReadExt;
    use tokio::process::Command as TokioCommand;
    use tokio::sync::mpsc::unbounded_channel;
    use tokio::task;
    use std::process::Stdio;

    // Helper to get AppHandle from global
    fn get_app_handle() -> Option<tauri::AppHandle> {
        if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
            handle_guard.as_ref().map(|global| (**global).clone())
        } else {
            None
        }
    }

    // --- Emit game-launched event FIRST (before any instance-aware utils are called) ---
    let app = get_app_handle();
    let mut patched_profile = profile.clone();
    if let Some(install_name) = installation.get("name") {
        patched_profile["name"] = install_name.clone();
    } else if let Some(id) = profile.get("id") {
        patched_profile["name"] = id.clone();
    }
    if let Some(ref app) = app {
        let _ = app.emit_to("main", "game-launched", json!({
            "instanceId": instance_id,
            "profile": patched_profile,
            "installation": installation
        }));
    }

    // Now spawn the process
    let mut tokio_cmd = TokioCommand::new(cmd.get_program());
    tokio_cmd.args(cmd.get_args());
    tokio_cmd.current_dir(working_dir);
    tokio_cmd.stdout(Stdio::piped());
    tokio_cmd.stderr(Stdio::piped());
    let mut child = tokio_cmd.spawn().map_err(|e| format!("Failed to launch: {e}"))?;
    let pid = child.id().unwrap_or(0);

    // Emit process started event
    if let Some(ref app) = app {
        let _ = app.emit_to("main", "game-process-event", json!({
            "instanceId": instance_id,
            "type": "started",
            "data": { "pid": pid }
        }));
    }
    if let Some(ref app) = app {
        Logger::log(app, LogLevel::Info, "=== MINECRAFT PROCESS SPAWNED ===", Some(instance_id));
        Logger::log(app, LogLevel::Info, &format!("Process ID: {}", pid), Some(instance_id));
        Logger::log(app, LogLevel::Info, "Minecraft launched successfully!", Some(instance_id));
        Logger::log(app, LogLevel::Info, "=================================", Some(instance_id));
    } else {
        Logger::debug_global("=== MINECRAFT PROCESS SPAWNED ===", Some(instance_id));
        Logger::debug_global(&format!("Process ID: {}", pid), Some(instance_id));
        Logger::debug_global("Minecraft launched successfully!", Some(instance_id));
        Logger::debug_global("=================================", Some(instance_id));
    }

    // Stream stdout and stderr, emit show-logs-page after first log line
    let (stdout_sender, mut stdout_receiver) = unbounded_channel::<String>();
    let (stderr_sender, mut stderr_receiver) = unbounded_channel::<String>();
    let instance_id_str = instance_id.to_string();

    // Stream stdout
    if let Some(stdout) = child.stdout.take() {
        let app = get_app_handle();
        let instance_id = instance_id_str.clone();
        let sender = stdout_sender.clone();
        task::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            let mut first = true;
            while let Ok(Some(line)) = lines.next_line().await {
                if first {
                    let _ = sender.send("__emit_show_logs_page__".to_string());
                    first = false;
                }
                let _ = sender.send(line.clone());
                if let Some(ref app) = app {
                    let _ = app.emit_to("main", "game-process-event", json!({
                        "instanceId": &instance_id,
                        "type": "output",
                        "data": { "line": &line }
                    }));
                }
            }
        });
    }

    // Stream stderr
    if let Some(stderr) = child.stderr.take() {
        let app = get_app_handle();
        let instance_id = instance_id_str.clone();
        let sender = stderr_sender.clone();
        task::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            let mut first = true;
            while let Ok(Some(line)) = lines.next_line().await {
                if first {
                    let _ = sender.send("__emit_show_logs_page__".to_string());
                    first = false;
                }
                let _ = sender.send(line.clone());
                if let Some(ref app) = app {
                    let _ = app.emit_to("main", "game-process-event", json!({
                        "instanceId": &instance_id,
                        "type": "error",
                        "data": { "line": &line }
                    }));
                }
            }
        });
    }

    // Wait for first log line from either stdout or stderr, then emit show-logs-page
    let instance_id_for_show = instance_id_str.clone();
    let app_for_show = get_app_handle();
    task::spawn(async move {
        let mut emitted = false;
        loop {
            tokio::select! {
                Some(line) = stdout_receiver.recv(), if !emitted => {
                    if line == "__emit_show_logs_page__" {
                        if let Some(app) = &app_for_show {
                            let _ = app.emit_to("main", "show-logs-page", serde_json::json!({
                                "instanceId": instance_id_for_show,
                                "installationId": instance_id_for_show,
                                "reason": "launch"
                            }));
                        }
                        emitted = true;
                    }
                },
                Some(line) = stderr_receiver.recv(), if !emitted => {
                    if line == "__emit_show_logs_page__" {
                        if let Some(app) = &app_for_show {
                            let _ = app.emit_to("main", "show-logs-page", serde_json::json!({
                                "instanceId": instance_id_for_show,
                                "installationId": instance_id_for_show,
                                "reason": "launch"
                            }));
                        }
                        emitted = true;
                    }
                },
                else => { break; }
            }
        }
    });

    // Wait for process to exit
    // eprintln!("Command:\n{:?}", cmd);
    let status = child.wait().await.map_err(|e| format!("Process wait failed: {e}"))?;
    let exit_code = status.code().unwrap_or(-1);
    Logger::info_global(&format!("Minecraft process exited with status: {}", exit_code), Some(&instance_id_str));
    if let Some(app) = get_app_handle() {
        let _ = app.emit_to("main", "game-process-event", json!({
            "instanceId": &instance_id_str,
            "type": "exit",
            "data": { "code": exit_code }
        }));
    }

    Ok(crate::launcher::LaunchResult {
        pid,
        command: format!("{:?}", cmd),
    })
}