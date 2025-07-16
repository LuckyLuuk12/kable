use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use crate::auth::MicrosoftAccount;
use crate::settings::LauncherSettings;
use crate::AppError;
use crate::logging::{Logger, LogLevel};

#[derive(Debug, Clone)]
pub struct LaunchContext {
    pub account: MicrosoftAccount,
    pub settings: LauncherSettings,
    pub installation_path: PathBuf,
    pub assets_path: PathBuf,
    pub natives_path: PathBuf,
    pub libraries_path: PathBuf,
    pub version_manifest: VersionManifest,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionManifest {
    pub id: String,
    pub r#type: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(default = "default_assets")]
    pub assets: String,
    #[serde(default)]
    pub arguments: Arguments,
    #[serde(default)]
    pub libraries: Vec<Library>,
    #[serde(default)]
    pub downloads: Option<Downloads>,
    #[serde(rename = "javaVersion", default)]
    pub java_version: Option<JavaVersion>,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub time: String,
    #[serde(rename = "minimumLauncherVersion", default)]
    pub minimum_launcher_version: u32,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: Option<u32>,
    pub logging: Option<serde_json::Value>,
    #[serde(rename = "assetIndex", default)]
    pub asset_index: Option<AssetIndex>,
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: Option<String>,
}

fn default_assets() -> String {
    "legacy".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<serde_json::Value>,
    #[serde(default)]
    pub jvm: Vec<serde_json::Value>,
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Downloads {
    pub client: Option<ClientDownload>,
    pub server: Option<ClientDownload>,
    pub client_mappings: Option<ClientDownload>,
    pub server_mappings: Option<ClientDownload>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientDownload {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    pub url: String,
}

/// Load version manifest from .minecraft/versions/{version}/{version}.json
pub fn load_version_manifest(version_id: &str, minecraft_dir: &Path) -> Result<VersionManifest, AppError> {
    load_version_manifest_recursive(version_id, minecraft_dir, &mut Vec::new())
}

/// Load version manifest recursively, handling inheritance
fn load_version_manifest_recursive(
    version_id: &str, 
    minecraft_dir: &Path, 
    visited: &mut Vec<String>
) -> Result<VersionManifest, AppError> {
    // Prevent infinite recursion
    if visited.contains(&version_id.to_string()) {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Circular inheritance detected for version: {}", version_id)
        )));
    }
    visited.push(version_id.to_string());
    
    let version_dir = minecraft_dir.join("versions").join(version_id);
    let manifest_path = version_dir.join(format!("{}.json", version_id));
    
    if !manifest_path.exists() {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Version manifest not found: {}", manifest_path.display())
        )));
    }
    
    let content = fs::read_to_string(&manifest_path)?;
    
    // Try to parse the version manifest with error handling
    let mut manifest: VersionManifest = match serde_json::from_str(&content) {
        Ok(manifest) => manifest,
        Err(err) => {
            Logger::console_log(LogLevel::Error, &format!("Failed to parse version manifest for {}: {}", version_id, err), Some(version_id));
            Logger::console_log(LogLevel::Error, &format!("Manifest path: {}", manifest_path.display()), Some(version_id));
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse version manifest for {}: {}", version_id, err)
            )));
        }
    };
    
    // If this version inherits from another, load and merge the parent
    if let Some(parent_id) = &manifest.inherits_from {
        Logger::console_log(LogLevel::Debug, &format!("Loading parent version: {} for {}", parent_id, version_id), Some(version_id));
        let parent_manifest = load_version_manifest_recursive(parent_id, minecraft_dir, visited)?;
        
        // Merge parent libraries with current libraries
        let parent_lib_count = parent_manifest.libraries.len();
        let current_lib_count = manifest.libraries.len();
        
        let mut all_libraries = parent_manifest.libraries;
        all_libraries.extend(manifest.libraries);
        
        // Deduplicate libraries - newer versions override older ones
        manifest.libraries = deduplicate_libraries(all_libraries);
        
        // Inherit missing fields from parent
        if manifest.assets == "legacy" && parent_manifest.assets != "legacy" {
            manifest.assets = parent_manifest.assets;
        }
        
        if manifest.arguments.game.is_empty() && !parent_manifest.arguments.game.is_empty() {
            manifest.arguments.game = parent_manifest.arguments.game;
        }
        
        if manifest.arguments.jvm.is_empty() && !parent_manifest.arguments.jvm.is_empty() {
            manifest.arguments.jvm = parent_manifest.arguments.jvm;
        }
        
        if manifest.downloads.is_none() && parent_manifest.downloads.is_some() {
            manifest.downloads = parent_manifest.downloads;
        }
        
        if manifest.asset_index.is_none() && parent_manifest.asset_index.is_some() {
            manifest.asset_index = parent_manifest.asset_index;
        }
        
        Logger::console_log(LogLevel::Debug, &format!("Merged {} parent + {} current = {} total libraries (after deduplication)", 
                parent_lib_count, current_lib_count, manifest.libraries.len()), Some(version_id));
    }
    
    Ok(manifest)
}

/// Get paths for minecraft directories
pub fn get_minecraft_paths(minecraft_dir: &Path) -> Result<(PathBuf, PathBuf, PathBuf, PathBuf), String> {
    let assets_path = minecraft_dir.join("assets");
    let libraries_path = minecraft_dir.join("libraries");
    let versions_path = minecraft_dir.join("versions");
    let natives_path = minecraft_dir.join("natives"); // We'll extract natives here
    
    // Ensure directories exist
    for dir in [&assets_path, &libraries_path, &versions_path, &natives_path] {
        if !dir.exists() {
            fs::create_dir_all(dir)
                .map_err(|e| format!("Failed to create directory {}: {}", dir.display(), e))?;
        }
    }
    
    Ok((assets_path, libraries_path, versions_path, natives_path))
}

/// Build variable substitution map for launch arguments
pub fn build_variable_map(
    context: &LaunchContext,
    version_id: &str,
    classpath: &str,
) -> Result<HashMap<String, String>, String> {
    let mut variables = HashMap::new();
    
    // Authentication variables
    variables.insert("auth_player_name".to_string(), context.account.username.clone());
    variables.insert("auth_uuid".to_string(), context.account.uuid.clone());
    variables.insert("auth_access_token".to_string(), 
        context.account.minecraft_access_token.clone().unwrap_or_else(|| "offline".to_string()));
    variables.insert("auth_xuid".to_string(), context.account.xbox_user_hash.clone());
    variables.insert("user_type".to_string(), 
        if context.account.minecraft_access_token.is_some() { "microsoft" } else { "offline" }.to_string());
    variables.insert("clientid".to_string(), uuid::Uuid::new_v4().to_string());
    
    // Version and launcher info
    variables.insert("version_name".to_string(), version_id.to_string());
    variables.insert("version_type".to_string(), context.version_manifest.r#type.clone());
    variables.insert("launcher_name".to_string(), "Kable".to_string());
    variables.insert("launcher_version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    
    // Paths
    variables.insert("game_directory".to_string(), context.installation_path.to_string_lossy().to_string());
    variables.insert("assets_root".to_string(), context.assets_path.to_string_lossy().to_string());
    variables.insert("assets_index_name".to_string(), context.version_manifest.assets.clone());
    variables.insert("natives_directory".to_string(), context.natives_path.to_string_lossy().to_string());
    variables.insert("classpath".to_string(), classpath.to_string());
    
    // Resolution (from settings)
    variables.insert("resolution_width".to_string(), context.settings.window_width.to_string());
    variables.insert("resolution_height".to_string(), context.settings.window_height.to_string());
    
    Ok(variables)
}

/// Process arguments with rules and variable substitution
pub fn process_arguments(args: &[serde_json::Value], variables: &HashMap<String, String>) -> Result<Vec<String>, String> {
    let mut processed = Vec::new();
    
    for arg in args {
        match arg {
            serde_json::Value::String(s) => {
                // Simple string argument - substitute variables
                let substituted = substitute_variables(s, variables);
                // Only add non-empty arguments
                if !substituted.trim().is_empty() {
                    processed.push(substituted);
                }
            },
            serde_json::Value::Object(obj) => {
                // Complex argument with rules
                if let Some(rules) = obj.get("rules") {
                    if evaluate_rules(rules)? {
                        if let Some(value) = obj.get("value") {
                            match value {
                                serde_json::Value::String(s) => {
                                    let substituted = substitute_variables(s, variables);
                                    // Filter out problematic arguments and empty ones
                                    if !substituted.trim().is_empty() && !is_problematic_argument(&substituted) {
                                        processed.push(substituted);
                                    }
                                },
                                serde_json::Value::Array(arr) => {
                                    for item in arr {
                                        if let serde_json::Value::String(s) = item {
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
                    }
                }
            },
            _ => {}
        }
    }
    
    Ok(processed)
}

/// Check if an argument is problematic for the current platform
fn is_problematic_argument(arg: &str) -> bool {
    // Filter out macOS-specific arguments on Windows/Linux
    if cfg!(not(target_os = "macos")) && arg == "-XstartOnFirstThread" {
        return true;
    }
    
    // Filter out demo mode argument - we want normal gameplay
    if arg == "--demo" {
        return true;
    }
    
    // Filter out quick play arguments until we implement proper quick play functionality
    if arg.starts_with("--quickPlay") || arg.starts_with("--quick-play") {
        return true;
    }
    
    false
}

/// Substitute variables in template string
pub fn substitute_variables(template: &str, variables: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    
    for (key, value) in variables {
        let placeholder = format!("${{{}}}", key);
        result = result.replace(&placeholder, value);
    }
    
    // Handle any remaining unresolved variables by removing them
    // This handles cases like ${quickPlayPath} that aren't in our variable map
    let re = regex::Regex::new(r"\$\{[^}]+\}").unwrap();
    result = re.replace_all(&result, "").to_string();
    
    result
}

/// Evaluate rules for conditional arguments
pub fn evaluate_rules(rules: &serde_json::Value) -> Result<bool, String> {
    if let serde_json::Value::Array(rules_array) = rules {
        for rule in rules_array {
            if let serde_json::Value::Object(rule_obj) = rule {
                let action = rule_obj.get("action")
                    .and_then(|v| v.as_str())
                    .unwrap_or("disallow");
                
                let mut condition_met = true;
                
                // Check OS condition
                if let Some(os_condition) = rule_obj.get("os") {
                    condition_met &= evaluate_os_condition(os_condition)?;
                }
                
                // Check features condition (for now, assume false for all features)
                if let Some(_features) = rule_obj.get("features") {
                    condition_met = false; // Disable feature-dependent arguments for now
                }
                
                if condition_met {
                    return Ok(action == "allow");
                }
            }
        }
    }
    
    // Default to allow if no rules match
    Ok(true)
}

/// Evaluate OS condition
pub fn evaluate_os_condition(os_condition: &serde_json::Value) -> Result<bool, String> {
    if let serde_json::Value::Object(os_obj) = os_condition {
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

/// Build classpath from libraries
pub fn build_classpath(
    libraries: &[Library], 
    libraries_path: &Path,
    version_jar_path: &Path
) -> Result<String, String> {
    let mut classpath_entries = Vec::new();
    let mut fabric_libraries = Vec::new();
    let mut mod_dependencies = Vec::new();
    
    Logger::console_log(LogLevel::Debug, "=== CLASSPATH DEBUG ===", None);
    Logger::console_log(LogLevel::Debug, &format!("Libraries path: {}", libraries_path.display()), None);
    Logger::console_log(LogLevel::Debug, &format!("Version JAR path: {}", version_jar_path.display()), None);
    Logger::console_log(LogLevel::Debug, &format!("Total libraries to process: {}", libraries.len()), None);
    
    // Debug: Count different library types
    let lwjgl_count = libraries.iter().filter(|lib| lib.name.contains("lwjgl")).count();
    let fabric_count = libraries.iter().filter(|lib| lib.name.contains("fabric")).count();
    let jetbrains_count = libraries.iter().filter(|lib| lib.name.contains("jetbrains")).count();
    
    Logger::console_log(LogLevel::Debug, &format!("LWJGL libraries found: {}", lwjgl_count), None);
    Logger::console_log(LogLevel::Debug, &format!("Fabric libraries found: {}", fabric_count), None);
    Logger::console_log(LogLevel::Debug, &format!("JetBrains libraries found: {}", jetbrains_count), None);
    
    // Add all library JARs
    for library in libraries {
        // Check if library applies to current OS
        if let Some(rules) = &library.rules {
            let rules_value = serde_json::to_value(rules)
                .map_err(|e| format!("Failed to serialize rules: {}", e))?;
            if !evaluate_rules(&rules_value)? {
                Logger::console_log(LogLevel::Debug, &format!("  Skipping library {} (rule failed)", library.name), None);
                continue; // Skip this library
            }
        }
        
        // Categorize libraries for better handling
        let is_fabric_lib = library.name.contains("fabric") || library.name.contains("quilt");
        let is_mod_dependency = library.name.contains("jetbrains") || 
                               library.name.contains("sodium") ||
                               library.name.contains("annotations") ||
                               library.name.contains("mixin");
        
        if let Some(downloads) = &library.downloads {
            if let Some(artifact) = &downloads.artifact {
                let lib_path = libraries_path.join(&artifact.path);
                if lib_path.exists() {
                    if library.name.contains("lwjgl") {
                        Logger::console_log(LogLevel::Debug, &format!("  Found LWJGL library: {} -> {}", library.name, artifact.path), None);
                    } else if is_fabric_lib {
                        Logger::console_log(LogLevel::Debug, &format!("  Found Fabric library: {} -> {}", library.name, artifact.path), None);
                        fabric_libraries.push(lib_path.to_string_lossy().to_string());
                    } else if is_mod_dependency {
                        Logger::console_log(LogLevel::Debug, &format!("  Found Mod dependency: {} -> {}", library.name, artifact.path), None);
                        mod_dependencies.push(lib_path.to_string_lossy().to_string());
                    } else {
                        Logger::console_log(LogLevel::Debug, &format!("  Found library: {}", artifact.path), None);
                    }
                    classpath_entries.push(lib_path.to_string_lossy().to_string());
                } else if library.name.contains("lwjgl") {
                    Logger::console_log(LogLevel::Debug, &format!("  Missing LWJGL library: {} (path: {})", library.name, lib_path.display()), None);
                } else if is_fabric_lib {
                    Logger::console_log(LogLevel::Debug, &format!("  Missing Fabric library: {} (path: {})", library.name, lib_path.display()), None);
                } else if is_mod_dependency {
                    Logger::console_log(LogLevel::Debug, &format!("  Missing Mod dependency: {} (path: {})", library.name, lib_path.display()), None);
                } else {
                    Logger::console_log(LogLevel::Debug, &format!("  Missing library: {} (path: {})", library.name, lib_path.display()), None);
                }
            } else {
                Logger::console_log(LogLevel::Debug, &format!("  No artifact for library: {}", library.name), None);
            }
        } else {
            // For libraries without downloads (like Fabric), try to find them manually
            // This is a fallback for mod loader libraries that might be embedded
            if library.name.contains("lwjgl") {
                Logger::console_log(LogLevel::Debug, &format!("  No downloads for LWJGL library: {}", library.name), None);
            } else if is_fabric_lib {
                Logger::console_log(LogLevel::Debug, &format!("  No downloads for Fabric library: {}", library.name), None);
            } else if is_mod_dependency {
                Logger::console_log(LogLevel::Debug, &format!("  No downloads for Mod dependency: {}", library.name), None);
            } else {
                Logger::console_log(LogLevel::Debug, &format!("  No downloads for library: {}", library.name), None);
            }
            
            // Try to construct the path manually for common libraries
            if let Some(jar_path) = try_find_library_manually(&library.name, libraries_path) {
                if jar_path.exists() {
                    if library.name.contains("lwjgl") {
                        Logger::console_log(LogLevel::Debug, &format!("  Found LWJGL library manually: {}", jar_path.display()), None);
                    } else if is_fabric_lib {
                        Logger::console_log(LogLevel::Debug, &format!("  Found Fabric library manually: {}", jar_path.display()), None);
                        fabric_libraries.push(jar_path.to_string_lossy().to_string());
                    } else if is_mod_dependency {
                        Logger::console_log(LogLevel::Debug, &format!("  Found Mod dependency manually: {}", jar_path.display()), None);
                        mod_dependencies.push(jar_path.to_string_lossy().to_string());
                    } else {
                        Logger::console_log(LogLevel::Debug, &format!("  Found library manually: {}", jar_path.display()), None);
                    }
                    classpath_entries.push(jar_path.to_string_lossy().to_string());
                }
            }
        }
    }
    
    // Add the main version JAR
    if version_jar_path.exists() {
        Logger::console_log(LogLevel::Debug, &format!("  Found version JAR: {}", version_jar_path.display()), None);
        classpath_entries.push(version_jar_path.to_string_lossy().to_string());
    } else {
        Logger::console_log(LogLevel::Debug, &format!("  Missing version JAR: {}", version_jar_path.display()), None);
        
        // For Fabric installations, the JAR might be embedded in the version JAR
        // Try using just the version JAR as the full classpath
        classpath_entries.push(version_jar_path.to_string_lossy().to_string());
    }
    
    // Check for mod JARs in the mods directory and add common mod dependencies
    let version_dir = version_jar_path.parent().unwrap();
    let minecraft_dir = version_dir.parent().unwrap().parent().unwrap();
    let mods_dir = minecraft_dir.join("mods");
    
    if mods_dir.exists() {
        Logger::console_log(LogLevel::Debug, &format!("Scanning mods directory: {}", mods_dir.display()), None);
        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                    let mod_name = path.file_name().unwrap().to_string_lossy();
                    if mod_name.contains("sodium") || mod_name.contains("fabric-api") || 
                       mod_name.contains("jetbrains") || mod_name.contains("annotations") {
                        Logger::console_log(LogLevel::Debug, &format!("  Adding mod dependency: {}", mod_name), None);
                        classpath_entries.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    
    Logger::console_log(LogLevel::Debug, &format!("Total classpath entries: {}", classpath_entries.len()), None);
    Logger::console_log(LogLevel::Debug, &format!("Fabric libraries: {}", fabric_libraries.len()), None);
    Logger::console_log(LogLevel::Debug, &format!("Mod dependencies: {}", mod_dependencies.len()), None);
    Logger::console_log(LogLevel::Debug, "=======================", None);
    
    // Join with platform-specific separator
    let separator = if cfg!(windows) { ";" } else { ":" };
    Ok(classpath_entries.join(separator))
}

/// Try to find library manually using common Maven patterns
fn try_find_library_manually(library_name: &str, libraries_path: &Path) -> Option<PathBuf> {
    // Parse library name like "net.fabricmc:fabric-loader:0.16.10"
    let parts: Vec<&str> = library_name.split(':').collect();
    if parts.len() >= 3 {
        let group = parts[0].replace('.', "/");
        let artifact = parts[1];
        let version = parts[2];
        
        // Construct Maven-style path: group/artifact/version/artifact-version.jar
        let jar_name = format!("{}-{}.jar", artifact, version);
        let jar_path = libraries_path.join(&group).join(artifact).join(version).join(jar_name);
        
        return Some(jar_path);
    }
    None
}

/// Find Java executable
pub fn find_java_executable(java_path: Option<&String>) -> Result<String, String> {
    if let Some(path) = java_path {
        if PathBuf::from(path).exists() {
            return Ok(path.clone());
        }
    }
    
    // Try common Java locations
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

/// Tauri command to get Java path for frontend validation
#[tauri::command]
pub fn get_java_path(java_path: Option<String>) -> Result<String, String> {
    find_java_executable(java_path.as_ref())
}

/// Build memory arguments
pub fn build_memory_arguments(settings: &LauncherSettings) -> Vec<String> {
    build_memory_arguments_with_override(settings, None)
}

/// Build memory arguments with optional memory override for per-installation settings
pub fn build_memory_arguments_with_override(settings: &LauncherSettings, memory_override: Option<u32>) -> Vec<String> {
    let mut args = Vec::new();
    
    // Use installation-specific memory if provided, otherwise use global default
    let memory = memory_override.unwrap_or(settings.default_memory);
    args.push(format!("-Xmx{}M", memory));
    
    // Add other common JVM arguments for better performance
    args.extend_from_slice(&[
        "-XX:+UnlockExperimentalVMOptions".to_string(),
        "-XX:+UseG1GC".to_string(),
        "-XX:G1NewSizePercent=20".to_string(),
        "-XX:G1ReservePercent=20".to_string(),
        "-XX:MaxGCPauseMillis=50".to_string(),
        "-XX:G1HeapRegionSize=32M".to_string(),
    ]);
    
    args
}

/// Build JVM arguments including natives path
pub fn build_jvm_arguments(settings: &LauncherSettings, natives_path: &Path) -> Vec<String> {
    build_jvm_arguments_with_memory(settings, natives_path, None)
}

/// Build JVM arguments including natives path with optional memory override
pub fn build_jvm_arguments_with_memory(settings: &LauncherSettings, natives_path: &Path, memory_override: Option<u32>) -> Vec<String> {
    let mut args = build_memory_arguments_with_override(settings, memory_override);
    
    // Add the crucial java.library.path for native libraries (LWJGL)
    args.push(format!("-Djava.library.path={}", natives_path.display()));
    
    args
}

/// Extract native libraries for the current platform
pub fn extract_natives(
    libraries: &[Library],
    libraries_path: &Path,
    natives_path: &PathBuf,
) -> Result<(), String> {
    // Clear existing natives
    if natives_path.exists() {
        fs::remove_dir_all(natives_path)
            .map_err(|e| format!("Failed to clear natives directory: {}", e))?;
    }
    fs::create_dir_all(natives_path)
        .map_err(|e| format!("Failed to create natives directory: {}", e))?;
    
    let current_os = std::env::consts::OS;
    let natives_classifier = match current_os {
        "windows" => "natives-windows",
        "macos" => "natives-macos",
        "linux" => "natives-linux",
        _ => return Err(format!("Unsupported OS: {}", current_os)),
    };
    
    for library in libraries {
        // Check if library applies to current OS
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
                        // Extract the native JAR
                        extract_jar(&native_path, natives_path)?;
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Extract a JAR file to a directory
fn extract_jar(jar_path: &PathBuf, extract_to: &Path) -> Result<(), String> {
    let file = fs::File::open(jar_path)
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
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }
            }
            let mut outfile = fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file: {}", e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {}", e))?;
        }
    }
    
    Ok(())
}

/// Deduplicate libraries by name, keeping the last (newer) version
/// Special handling for native libraries - keep both core and native versions
fn deduplicate_libraries(libraries: Vec<Library>) -> Vec<Library> {
    let mut library_map: HashMap<String, Library> = HashMap::new();
    let original_count = libraries.len();
    
    for library in libraries {
        // For native libraries, use the full name as key to avoid conflicts
        // e.g., "org.lwjgl:lwjgl:3.3.3" vs "org.lwjgl:lwjgl:3.3.3:natives-windows"
        let dedup_key = if library.name.contains(":natives-") {
            // For native libraries, use full name to preserve both core and native versions
            library.name.clone()
        } else {
            // For regular libraries, extract base name for version deduplication
            // e.g., "org.ow2.asm:asm:9.6" -> "org.ow2.asm:asm"
            let parts: Vec<&str> = library.name.split(':').collect();
            if parts.len() >= 2 {
                format!("{}:{}", parts[0], parts[1])
            } else {
                library.name.clone()
            }
        };
        
        // Always keep the latest version (last one wins)
        if let Some(existing) = library_map.get(&dedup_key) {
            Logger::console_log(LogLevel::Debug, &format!("  Deduplicating: {} (keeping) vs {} (removing)", library.name, existing.name), None);
        }
        library_map.insert(dedup_key, library);
    }
    
    let deduplicated: Vec<Library> = library_map.into_values().collect();
    Logger::console_log(LogLevel::Debug, &format!("Deduplicated libraries: {} -> {}", original_count, deduplicated.len()), None);
    
    deduplicated
}
