use crate::constants::{LATEST_RELEASE, LATEST_SNAPSHOT, MINECRAFT_VERSION_MANIFEST_URL, VERSIONS_DIR};
use crate::features::launcher::LaunchResult;
pub use crate::integrations::minecraft::manifest::{
    compare_versions, ensure_assets_for_manifest, load_and_merge_manifest_sync as load_and_merge_manifest_with_instance, merge_manifests,
    merge_manifests_with_instance, Artifact, AssetMode, Extract, Library, LibraryDownloads, OsRule, Rule,
};
use crate::Logger;
use api_types::profiles::KableProfile;
use api_types::settings::CategorizedLauncherSettings;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::Emitter;
use tokio::fs as async_fs;
use tokio::io::BufReader;

pub fn load_and_merge_manifest_sync(minecraft_dir: &str, version_id: &str, instance_id: Option<&str>) -> Result<Value, String> {
    load_and_merge_manifest_with_instance(minecraft_dir, version_id, instance_id)
}

pub fn try_find_library_manually(library_name: &str, libraries_path: &Path) -> Option<PathBuf> {
    let parts: Vec<&str> = library_name.split(':').collect();
    if parts.len() < 3 {
        return None;
    }
    let group = parts[0].replace('.', "/");
    let name = parts[1];
    let version = parts[2];

    let path = libraries_path.join(group).join(name).join(version).join(format!("{}-{}.jar", name, version));

    if path.exists() {
        Some(path)
    } else {
        None
    }
}

pub fn build_classpath_from_manifest_with_instance(
    manifest: &Value,
    libraries_path: &Path,
    version_jar_path: &Path,
    instance_id: Option<&str>,
) -> String {
    Logger::debug_global("Building classpath from manifest", instance_id);
    let mut dedup_map: HashMap<String, (String, String)> = HashMap::new();
    if let Some(libs) = manifest.get("libraries").and_then(|v| v.as_array()) {
        for lib in libs {
            if let Some(obj) = lib.as_object() {
                let mut jar_path_opt = None;
                let mut lib_name: Option<String> = None;

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
                            lib_name = Some(name.to_string());
                            if let Some(jar_path) = try_find_library_manually(name, libraries_path) {
                                if jar_path.exists() {
                                    jar_path_opt = Some(jar_path.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                } else if let Some(name_val) = obj.get("name") {
                    if let Some(name) = name_val.as_str() {
                        lib_name = Some(name.to_string());
                    }
                }

                if let Some(jar_path) = jar_path_opt {
                    if let Some(full_name) = lib_name {
                        let (dedup_key, version) = if full_name.contains(":natives-") {
                            (full_name.clone(), String::new())
                        } else {
                            let parts: Vec<&str> = full_name.split(':').collect();
                            if parts.len() >= 3 {
                                let key = format!("{}:{}", parts[0], parts[1]);
                                let ver = parts[2].to_string();
                                (key, ver)
                            } else if parts.len() >= 2 {
                                (format!("{}:{}", parts[0], parts[1]), String::new())
                            } else {
                                (full_name.clone(), String::new())
                            }
                        };

                        if let Some((_, existing_version)) = dedup_map.get(&dedup_key) {
                            if !version.is_empty() && !existing_version.is_empty() {
                                if compare_versions(&version, existing_version) > 0 {
                                    dedup_map.insert(dedup_key, (jar_path, version));
                                }
                            } else {
                                dedup_map.insert(dedup_key, (jar_path, version));
                            }
                        } else {
                            dedup_map.insert(dedup_key, (jar_path, version));
                        }
                    } else {
                        let key = jar_path.clone();
                        dedup_map.insert(key.clone(), (jar_path, String::new()));
                    }
                }
            }
        }
    }
    let mut entries: Vec<String> = dedup_map.into_values().map(|(path, _)| path).collect();
    entries.push(version_jar_path.to_string_lossy().to_string());
    let sep = if cfg!(windows) { ";" } else { ":" };
    entries.join(sep)
}

pub fn build_classpath_from_manifest(manifest: &Value, libraries_path: &Path, version_jar_path: &Path) -> String {
    build_classpath_from_manifest_with_instance(manifest, libraries_path, version_jar_path, None)
}

pub fn build_jvm_and_game_args_with_instance(
    manifest: &Value,
    variables: &HashMap<String, String>,
    instance_id: Option<&str>,
) -> (Vec<String>, Vec<String>) {
    Logger::debug_global(&format!("Variables: {:?}", variables), instance_id);
    let arguments = manifest.get("arguments").and_then(|v| v.as_object()).expect("No arguments in manifest");
    let empty_vec = Vec::new();
    let jvm_args = arguments.get("jvm").and_then(|v| v.as_array()).unwrap_or(&empty_vec);
    let game_args = arguments.get("game").and_then(|v| v.as_array()).unwrap_or(&empty_vec);
    let jvm_args_vec = process_arguments(jvm_args, variables);
    let game_args_vec = process_arguments(game_args, variables);
    Logger::debug_global(&format!("JVM args: {:?}", jvm_args_vec), instance_id);
    Logger::debug_global(&format!("Game args: {:?}", game_args_vec), instance_id);
    (jvm_args_vec, game_args_vec)
}

pub fn build_jvm_and_game_args(manifest: &Value, variables: &HashMap<String, String>) -> (Vec<String>, Vec<String>) {
    build_jvm_and_game_args_with_instance(manifest, variables, None)
}

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

pub fn process_arguments(args: &[Value], variables: &HashMap<String, String>) -> Vec<String> {
    let mut result = Vec::new();
    for arg in args {
        match arg {
            Value::String(s) => {
                let substituted = substitute_variables(s, variables);
                if !substituted.is_empty() {
                    result.push(substituted);
                }
            }
            Value::Object(obj) => {
                if let Some(rules) = obj.get("rules") {
                    if evaluate_rules(rules).unwrap_or(false) {
                        if let Some(value) = obj.get("value") {
                            match value {
                                Value::String(s) => {
                                    let substituted = substitute_variables(s, variables);
                                    if !substituted.is_empty() {
                                        result.push(substituted);
                                    }
                                }
                                Value::Array(arr) => {
                                    for item in arr {
                                        if let Value::String(s) = item {
                                            let substituted = substitute_variables(s, variables);
                                            if !substituted.is_empty() {
                                                result.push(substituted);
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    result
}

pub fn evaluate_rules(rules: &Value) -> Result<bool, String> {
    let mut allow = false;
    if let Some(rules_array) = rules.as_array() {
        for rule in rules_array {
            let action = rule.get("action").and_then(|v| v.as_str()).ok_or("Missing action in rule")?;
            let mut matches = true;
            if let Some(os_condition) = rule.get("os") {
                matches = evaluate_os_condition(os_condition)?;
            }
            if let Some(_features) = rule.get("features") {
                // Feature rules not fully implemented, skip for now or assume false
                matches = false;
            }

            if action == "allow" {
                if matches {
                    allow = true;
                }
            } else if action == "disallow" {
                if matches {
                    allow = false;
                }
            }
        }
    }
    Ok(allow)
}

pub async fn ensure_version_manifest_and_jar(version_id: &str, minecraft_dir: &str) -> Result<String, String> {
    use reqwest::Client;
    use zip::ZipArchive;

    let mut resolved_version = version_id.to_string();
    let mut maybe_version_list: Option<Value> = None;
    if version_id == LATEST_RELEASE || version_id == LATEST_SNAPSHOT || version_id == "latest" {
        let version_list_url = MINECRAFT_VERSION_MANIFEST_URL;
        let client = Client::new();
        let resp = client.get(version_list_url).send().await.map_err(|e| format!("Failed to fetch version list: {e}"))?;
        let manifest: Value = resp.json().await.map_err(|e| format!("Failed to parse version list: {e}"))?;
        maybe_version_list = Some(manifest.clone());
        if let Some(latest) = manifest.get("latest") {
            if version_id == LATEST_SNAPSHOT {
                if let Some(snapshot) = latest.get("snapshot").and_then(|v| v.as_str()) {
                    resolved_version = snapshot.to_string();
                }
            } else if let Some(release) = latest.get("release").and_then(|v| v.as_str()) {
                resolved_version = release.to_string();
            }
        }
        Logger::debug_global(&format!("Resolved {} => {}", version_id, resolved_version), None);
    }

    let version_subdir = PathBuf::from(minecraft_dir).join(VERSIONS_DIR).join(&resolved_version);
    crate::system::fs::create_dir(&version_subdir).await.map_err(|e| format!("Failed to create versions dir: {}", e))?;
    let manifest_path = version_subdir.join(format!("{}.json", resolved_version));
    let jar_path = version_subdir.join(format!("{}.jar", resolved_version));

    fn validate_client_jar(jar_path: &Path) -> Result<(), String> {
        let file = std::fs::File::open(jar_path).map_err(|e| format!("Failed to open JAR for validation: {}", e))?;
        let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to read JAR archive: {}", e))?;
        for i in 0..archive.len() {
            if let Ok(entry) = archive.by_index(i) {
                let name = entry.name();
                if name == "net/minecraft/client/main/Main.class" {
                    return Ok(());
                }
            }
        }
        Err("Client JAR does not contain net.minecraft.client.main.Main".to_string())
    }

    if manifest_path.exists() && jar_path.exists() {
        match validate_client_jar(&jar_path) {
            Ok(_) => {
                Logger::debug_global(
                    &format!("Version folder already exists for {} ({}) - skipping download", version_id, resolved_version),
                    None,
                );
                return Ok(resolved_version.clone());
            }
            Err(e) => {
                Logger::debug_global(
                    &format!("Existing JAR failed validation for {} ({}): {}. Will re-download.", version_id, resolved_version, e),
                    None,
                );
                let _ = std::fs::remove_file(&jar_path);
            }
        }
    }

    if !manifest_path.exists() {
        let manifest_list = if let Some(v) = maybe_version_list {
            v
        } else {
            let version_list_url = MINECRAFT_VERSION_MANIFEST_URL;
            let client = Client::new();
            let resp = client.get(version_list_url).send().await.map_err(|e| format!("Failed to fetch version list: {e}"))?;
            resp.json::<Value>().await.map_err(|e| format!("Failed to parse version list: {e}"))?
        };

        if let Some(versions) = manifest_list.get("versions").and_then(|v| v.as_array()) {
            let version_info = versions.iter().find(|v| v.get("id").and_then(|id| id.as_str()) == Some(&resolved_version));
            if let Some(info) = version_info {
                if let Some(url) = info.get("url").and_then(|u| u.as_str()) {
                    let client = Client::new();
                    let resp = client.get(url).send().await.map_err(|e| format!("Failed to download manifest: {e}"))?;
                    let bytes = resp.bytes().await.map_err(|e| format!("Failed to read manifest bytes: {e}"))?;
                    std::fs::write(&manifest_path, bytes).map_err(|e| format!("Failed to save manifest: {e}"))?;
                }
            }
        }
    }

    if !jar_path.exists() {
        let manifest_str = std::fs::read_to_string(&manifest_path).map_err(|e| format!("Failed to read manifest: {e}"))?;
        let manifest: Value = serde_json::from_str(&manifest_str).map_err(|e| format!("Failed to parse manifest: {e}"))?;
        if let Some(downloads) = manifest.get("downloads") {
            if let Some(client) = downloads.get("client") {
                if let Some(url) = client.get("url").and_then(|u| u.as_str()) {
                    let client = Client::new();
                    let resp = client.get(url).send().await.map_err(|e| format!("Failed to download JAR: {e}"))?;
                    let bytes = resp.bytes().await.map_err(|e| format!("Failed to read JAR bytes: {e}"))?;
                    std::fs::write(&jar_path, bytes).map_err(|e| format!("Failed to save JAR: {e}"))?;
                }
            }
        }
    }

    Ok(resolved_version)
}

pub async fn extract_natives(
    libraries: &[Library],
    libraries_path: &Path,
    natives_path: &PathBuf,
    instance_id: Option<&str>,
) -> Result<(), String> {
    if natives_path.exists() {
        if let Err(e) = std::fs::remove_dir_all(natives_path) {
            Logger::debug_global(&format!("Failed to clear natives directory (will continue): {}", e), None);
        }
    }
    crate::system::fs::create_dir(natives_path).await.map_err(|e| format!("Failed to create natives directory: {}", e))?;
    let current_os = if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    };

    let arch_tag = if cfg!(target_arch = "x86") {
        "x86"
    } else if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        std::env::consts::ARCH
    };

    let os_tag = current_os;

    let mut dedup_map: HashMap<String, (PathBuf, String)> = HashMap::new();

    for library in libraries {
        if let Some(rules) = &library.rules {
            let rules_value = serde_json::to_value(rules).map_err(|e| format!("Failed to serialize rules: {}", e))?;
            if !evaluate_rules(&rules_value)? {
                continue;
            }
        }
        if let Some(downloads) = &library.downloads {
            if let Some(classifiers) = &downloads.classifiers {
                let mut chosen: Option<String> = None;
                let candidate1 = format!("natives-{}-{}", os_tag, arch_tag);
                if classifiers.contains_key(&candidate1) {
                    chosen = Some(candidate1);
                }
                if chosen.is_none() {
                    let candidate2 = format!("natives-{}", os_tag);
                    if classifiers.contains_key(&candidate2) {
                        chosen = Some(candidate2);
                    }
                }
                if chosen.is_none() {
                    for k in classifiers.keys() {
                        if k.starts_with(&format!("natives-{}", os_tag)) {
                            chosen = Some(k.clone());
                            break;
                        }
                    }
                }
                if let Some(key) = chosen {
                    if let Some(native_artifact) = classifiers.get(&key) {
                        let native_path = libraries_path.join(&native_artifact.path);
                        if native_path.exists() {
                            let (dedup_key, version) = {
                                let parts: Vec<&str> = library.name.split(':').collect();
                                if parts.len() >= 3 {
                                    let key = format!("{}:{}", parts[0], parts[1]);
                                    let ver = parts[2].to_string();
                                    (key, ver)
                                } else {
                                    (library.name.clone(), String::new())
                                }
                            };

                            if let Some((_, existing_version)) = dedup_map.get(&dedup_key) {
                                if !version.is_empty() && !existing_version.is_empty() {
                                    if compare_versions(&version, existing_version) > 0 {
                                        dedup_map.insert(dedup_key, (native_path, version));
                                    }
                                } else {
                                    dedup_map.insert(dedup_key, (native_path, version));
                                }
                            } else {
                                dedup_map.insert(dedup_key, (native_path, version));
                            }
                        }
                    }
                }
            }
        }
    }

    for (native_path, _) in dedup_map.into_values() {
        extract_jar(&native_path, natives_path)?;
    }

    Ok(())
}

pub fn pre_launch_java_native_compat_check(java_path: &str, manifest: &Value, instance_id: Option<&str>) -> Result<(), String> {
    use std::collections::HashSet;

    let trimmed_path = java_path.trim();
    if trimmed_path.is_empty() {
        Logger::warn_global("Java path is empty or whitespace. Cannot perform pre-launch compatibility check.", instance_id);
        return Ok(());
    }

    let output = Command::new(trimmed_path).arg("-version").output();
    let mut java_info = String::new();
    match output {
        Ok(o) => {
            java_info.push_str(&String::from_utf8_lossy(&o.stdout));
            java_info.push_str(&String::from_utf8_lossy(&o.stderr));
        }
        Err(e) => {
            Logger::info_global(&format!("Failed to execute '{}' to probe java version: {}", trimmed_path, e), instance_id);
            return Ok(());
        }
    }

    let java_arch = if java_info.contains("64-Bit") || java_info.contains("x86_64") || java_info.contains("amd64") {
        "x86_64"
    } else if java_info.to_lowercase().contains("arm") || java_info.contains("aarch64") || java_info.contains("arm64") {
        "arm64"
    } else if java_info.contains("32-Bit") || java_info.contains("x86") {
        "x86"
    } else {
        std::env::consts::ARCH
    };

    let mut required_archs: HashSet<String> = HashSet::new();
    if let Some(libs) = manifest.get("libraries").and_then(|v| v.as_array()) {
        for lib in libs {
            if let Some(obj) = lib.as_object() {
                if let Some(downloads) = obj.get("downloads") {
                    if let Some(classifiers) = downloads.get("classifiers") {
                        if let Some(map) = classifiers.as_object() {
                            for key in map.keys() {
                                if key.starts_with("natives-") {
                                    let parts: Vec<&str> = key.split('-').collect();
                                    if parts.len() >= 3 {
                                        let maybe_arch = parts[parts.len() - 1];
                                        let arch_tag = match maybe_arch {
                                            "x86" | "i386" => "x86",
                                            "x86_64" | "x64" | "amd64" => "x86_64",
                                            "arm64" | "aarch64" => "arm64",
                                            _ => continue,
                                        };
                                        required_archs.insert(arch_tag.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if !required_archs.is_empty() && !required_archs.contains(java_arch) {
        Logger::warn_global(
            &format!("Potential arch mismatch: Java arch is {}, but manifest requires {:?}", java_arch, required_archs),
            instance_id,
        );
    }

    Ok(())
}

pub fn build_variable_map(
    context: &crate::features::launcher::LaunchContext,
    manifest: Option<&Value>,
    classpath: &str,
    parameters_map: Option<&HashMap<String, String>>,
) -> HashMap<String, String> {
    let mut variables = HashMap::new();

    variables.insert("auth_player_name".to_string(), context.account.minecraft_profile.name.clone());
    variables.insert("version_name".to_string(), context.installation.version.id.clone());
    variables.insert("game_directory".to_string(), context.minecraft_dir.clone());
    variables.insert("assets_root".to_string(), PathBuf::from(&context.minecraft_dir).join("assets").to_string_lossy().to_string());

    let assets_index_name = manifest.and_then(|m| m.get("assets").and_then(|v| v.as_str())).unwrap_or("legacy");
    variables.insert("assets_index_name".to_string(), assets_index_name.to_string());

    variables.insert("auth_uuid".to_string(), context.account.minecraft_profile.id.clone());
    variables.insert("auth_access_token".to_string(), context.account.access_token.clone());
    variables.insert("user_type".to_string(), "mojang".to_string());
    variables.insert("version_type".to_string(), "release".to_string());
    variables.insert("natives_directory".to_string(), PathBuf::from(&context.minecraft_dir).join("natives").to_string_lossy().to_string());
    variables.insert("launcher_name".to_string(), "kable".to_string());
    variables.insert("launcher_version".to_string(), "2.0.0".to_string());
    variables.insert("classpath".to_string(), classpath.to_string());

    if let Some(params) = parameters_map {
        for (k, v) in params {
            if k.starts_with("--") {
                continue;
            }
            variables.insert(k.clone(), v.clone());
        }
    }
    variables
}

pub async fn spawn_and_log_process(
    cmd: Command,
    working_dir: &str,
    instance_id: &str,
    installation: &KableProfile,
    _settings: &api_types::settings::CategorizedLauncherSettings,
) -> Result<LaunchResult, String> {
    use std::process::Stdio;
    use tokio::io::AsyncBufReadExt;
    use tokio::process::Command as TokioCommand;

    let mut tokio_cmd = TokioCommand::new(cmd.get_program());
    tokio_cmd.args(cmd.get_args());
    tokio_cmd.current_dir(working_dir);

    #[cfg(target_os = "windows")]
    {
        tokio_cmd.creation_flags(0x08000000);
    }
    tokio_cmd.stdout(Stdio::piped());
    tokio_cmd.stderr(Stdio::piped());

    let mut child = tokio_cmd.spawn().map_err(|e| format!("Failed to launch: {e}"))?;
    let pid = child.id().unwrap_or(0);

    crate::system::processes::track_process(pid);

    let app = crate::app_handle();

    let _ = app.emit(
        "game-launched",
        serde_json::json!({
            "instanceId": instance_id,
            "profile": { "name": installation.name },
            "installation": installation
        }),
    );

    let _ = app.emit(
        "game-process-event",
        serde_json::json!({
            "instanceId": instance_id,
            "type": "started",
            "data": { "pid": pid }
        }),
    );
    Logger::info("=== MINECRAFT PROCESS SPAWNED ===", Some(instance_id));

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let instance_id_clone = instance_id.to_string();

    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = crate::app_handle().emit(
                "game-process-event",
                serde_json::json!({
                    "instanceId": instance_id_clone,
                    "type": "output",
                    "data": line
                }),
            );
            Logger::info(&line, Some(&instance_id_clone));
        }
    });

    let instance_id_clone2 = instance_id.to_string();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = crate::app_handle().emit(
                "game-process-event",
                serde_json::json!({
                    "instanceId": instance_id_clone2,
                    "type": "error",
                    "data": line
                }),
            );
            Logger::error(&line, Some(&instance_id_clone2));
        }
    });

    let instance_id_clone3 = instance_id.to_string();
    tokio::spawn(async move {
        let status = child.wait().await;

        let exit_code = status.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);
        let _ = crate::app_handle().emit(
            "game-process-event",
            serde_json::json!({
                "instanceId": instance_id_clone3,
                "type": "exit",
                "data": { "exitCode": exit_code }
            }),
        );
        Logger::info(&format!("=== MINECRAFT PROCESS EXITED WITH CODE {} ===", exit_code), Some(&instance_id_clone3));
    });

    Ok(LaunchResult { pid, command: format!("{:?}", cmd) })
}
