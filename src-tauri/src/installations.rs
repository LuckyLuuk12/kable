use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use tauri::{AppHandle, Emitter};
use crate::AppError;
use crate::auth::{MicrosoftAccount, check_auth_status, get_access_token, get_active_launcher_account};
use crate::logging::{Logger, LogLevel};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinecraftInstallation {
    pub id: String,
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub is_valid: bool,
    #[serde(rename = "type")]
    pub installation_type: String, // 'vanilla' | 'fabric' | 'forge' | 'quilt' | 'neoforge'
    pub loader_version: Option<String>,
    pub last_played: Option<String>, // ISO date string
    pub created: Option<String>,
    pub game_dir: PathBuf,
    pub java_path: Option<String>,
    pub jvm_args: Option<Vec<String>>,
    pub memory: Option<u32>, // Memory allocation in MB for this installation
    pub resolution: Option<Resolution>,
    pub use_global_mods: bool, // Whether this installation uses global mod folder
    pub custom_mods_path: Option<String>, // Custom mods directory path for mod loaders
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LauncherProfiles {
    pub profiles: std::collections::HashMap<String, ProfileData>,
    pub settings: Option<serde_json::Value>,
    pub version: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileData {
    pub name: String,
    #[serde(rename = "lastVersionId")]
    pub last_version_id: String,
    #[serde(rename = "gameDir")]
    pub game_dir: Option<String>,
    #[serde(rename = "javaArgs")]
    pub java_args: Option<String>,
    #[serde(rename = "lastUsed")]
    pub last_used: Option<String>,
    #[serde(rename = "created")]
    pub created: Option<String>,
    pub icon: Option<String>,
    #[serde(rename = "javaDir")]
    pub java_dir: Option<String>,
    pub resolution: Option<Resolution>,
}

/// Get all Minecraft installations from the .minecraft directory
#[tauri::command]
pub async fn get_minecraft_installations(minecraft_path: Option<String>) -> Result<Vec<MinecraftInstallation>, String> {
    let minecraft_dir = if let Some(path) = minecraft_path {
        PathBuf::from(path)
    } else {
        get_default_minecraft_directory()?
    };

    if !minecraft_dir.exists() {
        return Ok(Vec::new());
    }

    let mut installations = Vec::new();

    // Read launcher profiles for installation metadata
    let profiles = read_launcher_profiles(&minecraft_dir)?;
    
    // Get versions from the versions directory
    let versions_dir = minecraft_dir.join("versions");
    if !versions_dir.exists() {
        return Ok(installations);
    }

    // Read available versions
    if let Ok(entries) = fs::read_dir(&versions_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let version_id = entry.file_name().to_string_lossy().to_string();
                
                if let Ok(installation) = create_installation_from_version(&minecraft_dir, &version_id, &profiles) {
                    installations.push(installation);
                }
            }
        }
    }

    // Sort by last played (most recent first)
    installations.sort_by(|a, b| {
        let default_time = "1970-01-01T00:00:00Z".to_string();
        let a_time = a.last_played.as_ref().unwrap_or(&default_time);
        let b_time = b.last_played.as_ref().unwrap_or(&default_time);
        b_time.cmp(a_time)
    });

    Ok(installations)
}

/// Create a Minecraft installation from a version directory
fn create_installation_from_version(
    minecraft_dir: &Path,
    version_id: &str,
    profiles: &LauncherProfiles,
) -> Result<MinecraftInstallation, AppError> {
    let version_dir = minecraft_dir.join("versions").join(version_id);
    let jar_path = version_dir.join(format!("{}.jar", version_id));
    let json_path = version_dir.join(format!("{}.json", version_id));

    let is_valid = jar_path.exists() && json_path.exists();

    // Try to find a profile that uses this version
    // First try exact match, then try partial matches for mod loader versions
    let profile_entry = profiles.profiles.iter()
        .find(|(_, p)| p.last_version_id == version_id)
        .or_else(|| {
            // If no exact match, try to find a profile that might correspond to this version
            // This helps with mod loader installations where the version ID might be different
            profiles.profiles.iter()
                .find(|(_, p)| {
                    // Check if the profile's last_version_id contains key parts of our version_id
                    version_id.contains(&p.last_version_id) || 
                    p.last_version_id.contains(version_id) ||
                    // Check for common patterns in modded installations
                    (version_id.contains("fabric") && p.name.to_lowercase().contains("fabric")) ||
                    (version_id.contains("forge") && p.name.to_lowercase().contains("forge")) ||
                    (version_id.contains("iris") && p.name.to_lowercase().contains("iris"))
                })
        });
    let profile = profile_entry.map(|(_, p)| p);
    let profile_key = profile_entry.map(|(key, _)| key.clone());

    // Determine installation type and loader version
    let (installation_type, loader_version) = detect_installation_type(&version_dir, version_id)?;

    // Use profile name if available, otherwise generate one
    let name = profile
        .map(|p| p.name.clone())
        .unwrap_or_else(|| {
            if installation_type == "vanilla" {
                format!("Minecraft {}", version_id)
            } else {
                format!("{} {}", installation_type, version_id)
            }
        });

    Ok(MinecraftInstallation {
        id: profile_key.clone().unwrap_or_else(|| version_id.to_string()),
        name,
        version: version_id.to_string(),
        path: version_dir,
        is_valid,
        installation_type,
        loader_version,
        last_played: profile.and_then(|p| p.last_used.clone()),
        created: profile.and_then(|p| p.created.clone()),
        game_dir: profile
            .and_then(|p| p.game_dir.as_ref())
            .map(PathBuf::from)
            .unwrap_or_else(|| minecraft_dir.to_path_buf()),
        java_path: profile.and_then(|p| p.java_dir.clone()),
        jvm_args: profile.and_then(|p| {
            p.java_args.as_ref().map(|args| {
                args.split_whitespace()
                    .map(|s| s.to_string())
                    .collect()
            })
        }),
        memory: profile.and_then(|p| extract_memory_from_jvm_args(p.java_args.as_ref())),
        resolution: profile.and_then(|p| p.resolution.clone()),
        use_global_mods: true, // Default to global mods
        custom_mods_path: None, // Will be set up when needed
    })
}

/// Detect the installation type (vanilla, fabric, forge, etc.) and loader version
fn detect_installation_type(version_dir: &Path, version_id: &str) -> Result<(String, Option<String>), AppError> {
    // Method 1: Check version ID patterns first (most reliable for common patterns)
    if let Some((loader_type, version)) = detect_from_version_id(version_id) {
        return Ok((loader_type, version));
    }
    
    // Method 2: Check JSON file for library dependencies
    let json_path = version_dir.join(format!("{}.json", version_id));
    if json_path.exists() {
        if let Ok((loader_type, version)) = detect_from_json_file(&json_path, version_id) {
            if loader_type != "vanilla" {
                return Ok((loader_type, version));
            }
        }
    }
    
    // Method 3: Check for additional files that indicate mod loaders
    if let Some((loader_type, version)) = detect_from_directory_contents(version_dir) {
        return Ok((loader_type, version));
    }
    
    // Method 4: Check JAR file name patterns
    if let Some((loader_type, version)) = detect_from_jar_name(version_dir, version_id) {
        return Ok((loader_type, version));
    }
    
    // Default to vanilla if no mod loader detected
    Ok(("vanilla".to_string(), None))
}

/// Detect mod loader from version ID patterns
fn detect_from_version_id(version_id: &str) -> Option<(String, Option<String>)> {
    let version_id_lower = version_id.to_lowercase();
    
    // Fabric patterns (broadest first)
    if version_id_lower.contains("fabric") {
        // Try to extract fabric loader version
        if let Ok(re) = regex::Regex::new(r"fabric-loader-([0-9]+\.[0-9]+\.[0-9]+)") {
            if let Some(captures) = re.captures(version_id) {
                if let Some(version_match) = captures.get(1) {
                    let version = version_match.as_str().to_string();
                    return Some(("fabric".to_string(), Some(version)));
                }
            }
        }
        
        // Broader fabric pattern for any version number
        if let Ok(re) = regex::Regex::new(r"([0-9]+\.[0-9]+\.[0-9]+)") {
            if let Some(captures) = re.captures(version_id) {
                if let Some(version_match) = captures.get(1) {
                    let version = version_match.as_str().to_string();
                    return Some(("fabric".to_string(), Some(version)));
                }
            }
        }
        
        return Some(("fabric".to_string(), None));
    }
    
    // Iris + Fabric patterns
    if version_id_lower.contains("iris") {
        return Some(("fabric".to_string(), None));
    }
    
    // Forge patterns
    if version_id_lower.contains("forge") {
        // Try to extract forge version
        if let Ok(re) = regex::Regex::new(r"([0-9]+\.[0-9]+\.[0-9]+)") {
            if let Some(captures) = re.captures(version_id) {
                if let Some(version_match) = captures.get(1) {
                    let version = version_match.as_str().to_string();
                    return Some(("forge".to_string(), Some(version)));
                }
            }
        }
        
        return Some(("forge".to_string(), None));
    }
    
    // NeoForge patterns
    if version_id_lower.contains("neoforge") {
        if let Ok(re) = regex::Regex::new(r"([0-9]+\.[0-9]+\.[0-9]+)") {
            if let Some(captures) = re.captures(version_id) {
                if let Some(version_match) = captures.get(1) {
                    let version = version_match.as_str().to_string();
                    return Some(("neoforge".to_string(), Some(version)));
                }
            }
        }
        
        return Some(("neoforge".to_string(), None));
    }
    
    // Quilt patterns
    if version_id_lower.contains("quilt") {
        if let Ok(re) = regex::Regex::new(r"([0-9]+\.[0-9]+\.[0-9]+)") {
            if let Some(captures) = re.captures(version_id) {
                if let Some(version_match) = captures.get(1) {
                    let version = version_match.as_str().to_string();
                    return Some(("quilt".to_string(), Some(version)));
                }
            }
        }
        
        return Some(("quilt".to_string(), None));
    }
    
    // OptiFine patterns
    if version_id_lower.contains("optifine") {
        return Some(("optifine".to_string(), None));
    }
    
    None
}

/// Detect mod loader from JSON file analysis
fn detect_from_json_file(json_path: &Path, _version_id: &str) -> Result<(String, Option<String>), AppError> {
    let json_content = fs::read_to_string(json_path)?;
    let version_json: serde_json::Value = match serde_json::from_str(&json_content) {
        Ok(json) => json,
        Err(_) => {
            // If JSON parsing fails, try other detection methods
            return Ok(("vanilla".to_string(), None));
        }
    };

    // Check libraries array for mod loader dependencies
    if let Some(libraries) = version_json["libraries"].as_array() {
        for lib in libraries {
            if let Some(name) = lib["name"].as_str() {
                // Fabric detection
                if name.contains("net.fabricmc:fabric-loader") {
                    let loader_version = extract_version_from_library_name(name, "fabric-loader");
                    return Ok(("fabric".to_string(), loader_version));
                }
                
                // Forge detection (multiple patterns)
                if name.contains("net.minecraftforge:forge") || 
                   name.contains("net.minecraftforge:minecraftforge") ||
                   name.contains("cpw.mods:modlauncher") {
                    let loader_version = extract_version_from_library_name(name, "forge")
                        .or_else(|| extract_version_from_library_name(name, "minecraftforge"));
                    return Ok(("forge".to_string(), loader_version));
                }
                
                // NeoForge detection
                if name.contains("net.neoforged:forge") || name.contains("net.neoforged:neoforge") {
                    let loader_version = extract_version_from_library_name(name, "neoforge")
                        .or_else(|| extract_version_from_library_name(name, "forge"));
                    return Ok(("neoforge".to_string(), loader_version));
                }
                
                // Quilt detection
                if name.contains("org.quiltmc:quilt-loader") {
                    let loader_version = extract_version_from_library_name(name, "quilt-loader");
                    return Ok(("quilt".to_string(), loader_version));
                }
                
                // OptiFine detection
                if name.contains("optifine") || name.contains("OptiFine") {
                    return Ok(("optifine".to_string(), None));
                }
            }
        }
    }
    
    // Check mainClass field for mod loader indicators
    if let Some(main_class) = version_json["mainClass"].as_str() {
        if main_class.contains("fabric") {
            return Ok(("fabric".to_string(), None));
        }
        if main_class.contains("forge") || main_class.contains("minecraftforge") {
            return Ok(("forge".to_string(), None));
        }
        if main_class.contains("neoforge") {
            return Ok(("neoforge".to_string(), None));
        }
        if main_class.contains("quilt") {
            return Ok(("quilt".to_string(), None));
        }
    }
    
    Ok(("vanilla".to_string(), None))
}

/// Detect mod loader from directory contents (additional files)
fn detect_from_directory_contents(version_dir: &Path) -> Option<(String, Option<String>)> {
    if let Ok(entries) = fs::read_dir(version_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_lowercase();
            
            // Look for mod loader specific files
            if file_name.contains("fabric") && (file_name.ends_with(".jar") || file_name.ends_with(".json")) {
                return Some(("fabric".to_string(), None));
            }
            if file_name.contains("forge") && (file_name.ends_with(".jar") || file_name.ends_with(".json")) {
                return Some(("forge".to_string(), None));
            }
            if file_name.contains("neoforge") && (file_name.ends_with(".jar") || file_name.ends_with(".json")) {
                return Some(("neoforge".to_string(), None));
            }
            if file_name.contains("quilt") && (file_name.ends_with(".jar") || file_name.ends_with(".json")) {
                return Some(("quilt".to_string(), None));
            }
            if file_name.contains("optifine") && file_name.ends_with(".jar") {
                return Some(("optifine".to_string(), None));
            }
        }
    }
    None
}

/// Detect mod loader from JAR file name patterns
fn detect_from_jar_name(version_dir: &Path, version_id: &str) -> Option<(String, Option<String>)> {
    let jar_path = version_dir.join(format!("{}.jar", version_id));
    if jar_path.exists() {
        let jar_name = jar_path.file_name()?.to_string_lossy().to_lowercase();
        
        // Check JAR name for mod loader patterns
        if jar_name.contains("fabric") {
            return Some(("fabric".to_string(), None));
        }
        if jar_name.contains("forge") {
            return Some(("forge".to_string(), None));
        }
        if jar_name.contains("neoforge") {
            return Some(("neoforge".to_string(), None));
        }
        if jar_name.contains("quilt") {
            return Some(("quilt".to_string(), None));
        }
        if jar_name.contains("optifine") {
            return Some(("optifine".to_string(), None));
        }
        if jar_name.contains("iris") {
            return Some(("fabric".to_string(), None)); // Iris typically uses Fabric
        }
    }
    None
}

/// Extract version from library name (e.g., "net.fabricmc:fabric-loader:0.15.0" -> Some("0.15.0"))
fn extract_version_from_library_name(library_name: &str, loader_type: &str) -> Option<String> {
    if library_name.contains(loader_type) {
        let parts: Vec<&str> = library_name.split(':').collect();
        if parts.len() >= 3 {
            return Some(parts[2].to_string());
        }
    }
    
    // Try alternative patterns for different mod loaders
    match loader_type {
        "forge" | "minecraftforge" => {
            // Handle forge patterns like "net.minecraftforge:forge:1.20.1-47.2.0"
            if let Some(captures) = regex::Regex::new(r"([0-9]+\.[0-9]+\.[0-9]+(?:-[0-9]+\.[0-9]+\.[0-9]+)?)")
                .ok()?
                .captures(library_name) {
                return Some(captures.get(1)?.as_str().to_string());
            }
        }
        "fabric-loader" => {
            // Handle fabric loader patterns
            if let Some(captures) = regex::Regex::new(r"fabric-loader:([0-9]+\.[0-9]+\.[0-9]+)")
                .ok()?
                .captures(library_name) {
                return Some(captures.get(1)?.as_str().to_string());
            }
        }
        _ => {}
    }
    
    None
}

/// Extract memory allocation from JVM arguments (e.g., "-Xmx2G" -> Some(2048))
fn extract_memory_from_jvm_args(java_args: Option<&String>) -> Option<u32> {
    if let Some(args) = java_args {
        for arg in args.split_whitespace() {
            if let Some(memory_part) = arg.strip_prefix("-Xmx") {
                
                // Parse different memory formats: 2G, 2048M, 2048
                if memory_part.ends_with('G') || memory_part.ends_with('g') {
                    if let Ok(gb) = memory_part[..memory_part.len()-1].parse::<u32>() {
                        return Some(gb * 1024); // Convert GB to MB
                    }
                } else if memory_part.ends_with('M') || memory_part.ends_with('m') {
                    if let Ok(mb) = memory_part[..memory_part.len()-1].parse::<u32>() {
                        return Some(mb);
                    }
                } else if let Ok(mb) = memory_part.parse::<u32>() {
                    return Some(mb); // Assume MB if no unit specified
                }
            }
        }
    }
    None
}

/// Read launcher profiles from launcher_profiles.json
fn read_launcher_profiles(minecraft_dir: &Path) -> Result<LauncherProfiles, AppError> {
    let profiles_path = minecraft_dir.join("launcher_profiles.json");
    
    if !profiles_path.exists() {
        // Return empty profiles if file doesn't exist
        return Ok(LauncherProfiles {
            profiles: std::collections::HashMap::new(),
            settings: None,
            version: None,
        });
    }

    let content = fs::read_to_string(&profiles_path)?;
    let profiles: LauncherProfiles = serde_json::from_str(&content)
        .map_err(AppError::Json)?;

    Ok(profiles)
}

/// Get the default Minecraft directory for the current platform
fn get_default_minecraft_directory() -> Result<PathBuf, String> {
    let minecraft_dir = if cfg!(target_os = "windows") {
        dirs::data_dir()
            .ok_or("Could not find AppData directory")?
            .join(".minecraft")
    } else if cfg!(target_os = "macos") {
        dirs::home_dir()
            .ok_or("Could not find home directory")?
            .join("Library")
            .join("Application Support")
            .join("minecraft")
    } else {
        // Linux and other Unix-like systems
        dirs::home_dir()
            .ok_or("Could not find home directory")?
            .join(".minecraft")
    };

    Ok(minecraft_dir)
}

/// Refresh installation data for a specific installation
#[tauri::command]
pub async fn refresh_installation(installation_id: String, minecraft_path: Option<String>) -> Result<Option<MinecraftInstallation>, String> {
    let installations = get_minecraft_installations(minecraft_path).await?;
    Ok(installations.into_iter().find(|i| i.id == installation_id))
}

/// Update the last played time for an installation
#[tauri::command]
pub async fn update_installation_last_played(installation_id: String, minecraft_path: Option<String>) -> Result<(), String> {
    let minecraft_dir = if let Some(path) = minecraft_path {
        PathBuf::from(path)
    } else {
        get_default_minecraft_directory().map_err(|e| e.to_string())?
    };

    let profiles_path = minecraft_dir.join("launcher_profiles.json");
    
    if !profiles_path.exists() {
        return Err("launcher_profiles.json not found".to_string());
    }

    let mut profiles = read_launcher_profiles(&minecraft_dir)
        .map_err(|e| format!("Failed to read profiles: {}", e))?;

    // Find and update the profile that uses this version
    let now = chrono::Utc::now().to_rfc3339();
    
    for profile in profiles.profiles.values_mut() {
        if profile.last_version_id == installation_id {
            profile.last_used = Some(now.clone());
            break;
        }
    }

    // Write back to file
    let content = serde_json::to_string_pretty(&profiles)
        .map_err(|e| format!("Failed to serialize profiles: {}", e))?;
    
    fs::write(&profiles_path, content)
        .map_err(|e| format!("Failed to write profiles: {}", e))?;

    Ok(())
}

/// Extended installation management for Kable launcher
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInstallationRequest {
    pub name: String,
    pub version: String,
    pub mod_loader: String,
    pub game_directory: Option<String>,
    pub java_path: Option<String>,
    pub jvm_args: Option<String>,
    pub memory: Option<u32>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KableInstallation {
    pub id: String,
    pub name: String,
    pub version: String,
    pub mod_loader: String, // vanilla, fabric, forge, quilt, neoforge
    pub loader_version: Option<String>,
    pub description: Option<String>,
    pub game_directory: Option<String>,
    pub java_path: Option<String>,
    pub jvm_args: Option<String>,
    pub memory: Option<u32>, // Memory allocation in MB for this installation
    pub last_played: Option<String>,
    pub created: String,
    pub path: String,
    pub is_valid: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KableProfiles {
    pub profiles: HashMap<String, KableInstallation>,
    pub version: String,
    pub last_used: Option<String>,
}

impl Default for KableProfiles {
    fn default() -> Self {
        Self {
            profiles: HashMap::new(),
            version: "1.0.0".to_string(),
            last_used: None,
        }
    }
}

pub fn get_kable_directory() -> Result<PathBuf, AppError> {
    let minecraft_dir = get_default_minecraft_directory()
        .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, e)))?;
    Ok(minecraft_dir.join("kable"))
}

pub fn get_kable_profiles_path() -> Result<PathBuf, AppError> {
    let kable_dir = get_kable_directory()?;
    Ok(kable_dir.join("profiles.json"))
}

pub fn ensure_kable_directory() -> Result<PathBuf, AppError> {
    let kable_dir = get_kable_directory()?;
    
    if !kable_dir.exists() {
        fs::create_dir_all(&kable_dir)?;
        Logger::info_global(&format!("Created Kable directory: {:?}", kable_dir), None);
    }
    
    // Create only essential subdirectories - much simpler!
    let mods_dir = kable_dir.join("mods");
    if !mods_dir.exists() {
        fs::create_dir_all(&mods_dir)?;
        Logger::info_global(&format!("Created mods directory: {:?}", mods_dir), None);
    }
    
    Ok(kable_dir)
}

pub fn load_kable_profiles() -> Result<KableProfiles, AppError> {
    let profiles_path = get_kable_profiles_path()?;
    
    if !profiles_path.exists() {
        // Create default profiles file
        let default_profiles = KableProfiles::default();
        save_kable_profiles(&default_profiles)?;
        return Ok(default_profiles);
    }
    
    let content = fs::read_to_string(profiles_path)?;
    let profiles: KableProfiles = serde_json::from_str(&content)
        .unwrap_or_else(|_| KableProfiles::default());
    
    Ok(profiles)
}

pub fn save_kable_profiles(profiles: &KableProfiles) -> Result<(), AppError> {
    ensure_kable_directory()?;
    let profiles_path = get_kable_profiles_path()?;
    let content = serde_json::to_string_pretty(profiles)?;
    fs::write(profiles_path, content)?;
    Ok(())
}

pub fn get_installation_mods_directory(installation_id: &str) -> Result<PathBuf, AppError> {
    let kable_dir = get_kable_directory()?;
    Ok(kable_dir.join("mods").join(installation_id))
}

pub fn create_installation_mods_directory(installation_id: &str) -> Result<PathBuf, AppError> {
    let mods_dir = get_installation_mods_directory(installation_id)?;
    
    if !mods_dir.exists() {
        fs::create_dir_all(&mods_dir)?;
        Logger::info_global(&format!("Created mods directory for installation {}: {:?}", installation_id, mods_dir), Some(installation_id));
    }
    
    Ok(mods_dir)
}

#[allow(dead_code)]
pub fn get_installation_directory(installation_id: &str) -> Result<PathBuf, String> {
    let kable_dir = get_kable_directory()
        .map_err(|e| format!("Failed to get kable directory: {}", e))?;
    let installation_dir = kable_dir.join("instances").join(installation_id);
    
    // Create directory if it doesn't exist
    if !installation_dir.exists() {
        fs::create_dir_all(&installation_dir)
            .map_err(|e| format!("Failed to create installation directory: {}", e))?;
    }
    
    Ok(installation_dir)
}

#[tauri::command]
pub async fn get_installations() -> Result<Vec<KableInstallation>, String> {
    let mut all_installations = Vec::new();
    let mut stats = std::collections::HashMap::new();
    
    // 1. Load Kable-managed profiles
    match load_kable_profiles() {
        Ok(profiles) => {
            let kable_installations: Vec<KableInstallation> = profiles.profiles.into_values().collect();
            
            // Count by mod loader for stats
            for install in &kable_installations {
                *stats.entry(install.mod_loader.clone()).or_insert(0) += 1;
            }
            
            all_installations.extend(kable_installations);
        }
        Err(e) => {
            Logger::warn_global(&format!("Failed to load Kable profiles: {}", e), None);
        }
    }
    
    // 2. Scan for existing Minecraft installations and convert them to KableInstallation format
    match get_minecraft_installations(None).await {
        Ok(minecraft_installations) => {
            let mut added_count = 0;
            let mut skipped_count = 0;
            
            for mc_install in minecraft_installations {
                // Check if we already have this in our Kable profiles
                let already_exists = all_installations.iter().any(|kable_install| {
                    kable_install.id == mc_install.id
                });
                
                if !already_exists {
                    let kable_installation = KableInstallation {
                        id: mc_install.id.clone(),
                        name: mc_install.name.clone(),
                        version: mc_install.version.clone(),
                        mod_loader: mc_install.installation_type.clone(), // Use the detected mod loader instead of hardcoded "vanilla"
                        loader_version: mc_install.loader_version.clone(),
                        description: Some("Existing Minecraft installation".to_string()),
                        game_directory: Some(mc_install.game_dir.to_string_lossy().to_string()),
                        java_path: mc_install.java_path.clone(),
                        jvm_args: mc_install.jvm_args.as_ref().map(|args| args.join(" ")),
                        memory: mc_install.memory,
                        last_played: mc_install.last_played.clone(),
                        created: mc_install.created.clone().unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
                        path: mc_install.path.to_string_lossy().to_string(),
                        is_valid: mc_install.is_valid,
                    };
                    
                    // Count by mod loader for stats
                    *stats.entry(kable_installation.mod_loader.clone()).or_insert(0) += 1;
                    
                    all_installations.push(kable_installation);
                    added_count += 1;
                } else {
                    skipped_count += 1;
                }
            }
            
            if added_count > 0 || skipped_count > 0 {
                Logger::info_global(&format!("Processed {} existing installations: {} added, {} skipped", 
                    added_count + skipped_count, added_count, skipped_count), None);
            }
        }
        Err(e) => {
            Logger::warn_global(&format!("Failed to scan existing Minecraft installations: {}", e), None);
        }
    }
    
    // Print installation statistics
    let stats_parts: Vec<String> = stats.iter()
        .map(|(mod_loader, count)| format!("{}: {}", mod_loader, count))
        .collect();
    let stats_summary = if stats_parts.is_empty() {
        "No installations found".to_string()
    } else {
        format!("Installation Statistics - {} (Total: {})", stats_parts.join(", "), all_installations.len())
    };
    Logger::info_global(&stats_summary, None);
    
    Ok(all_installations)
}

#[tauri::command]
pub async fn create_installation(
    request: CreateInstallationRequest,
) -> Result<KableInstallation, String> {
    let installation_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    
    // Use the default Minecraft directory structure - no custom installation paths
    let minecraft_dir = get_default_minecraft_directory()
        .map_err(|e| format!("Failed to get Minecraft directory: {}", e))?;
    
    let installation = KableInstallation {
        id: installation_id.clone(),
        name: request.name,
        version: request.version,
        mod_loader: request.mod_loader,
        loader_version: None, // TODO: Detect mod loader version
        description: request.description,
        game_directory: request.game_directory,
        java_path: request.java_path,
        jvm_args: request.jvm_args,
        memory: request.memory, // Use the provided memory allocation
        last_played: None,
        created: now,
        path: minecraft_dir.to_string_lossy().to_string(),
        is_valid: true,
    };
    
    // Create mods directory for this installation
    create_installation_mods_directory(&installation_id)
        .map_err(|e| format!("Failed to create mods directory: {}", e))?;
    
    // Save to kable profiles
    let mut profiles = load_kable_profiles()
        .map_err(|e| format!("Failed to load kable profiles: {}", e))?;
    
    profiles.profiles.insert(installation_id, installation.clone());
    
    save_kable_profiles(&profiles)
        .map_err(|e| format!("Failed to save kable profiles: {}", e))?;
    
    Logger::info_global(&format!("Created installation: {} ({})", installation.name, installation.id), Some(&installation.id));
    Ok(installation)
}

#[tauri::command]
pub async fn delete_installation(installation_id: String) -> Result<(), String> {
    let mut profiles = load_kable_profiles()
        .map_err(|e| format!("Failed to load kable profiles: {}", e))?;
    
    if let Some(installation) = profiles.profiles.remove(&installation_id) {
        // Remove the mods directory for this installation
        let mods_dir = get_installation_mods_directory(&installation_id)
            .map_err(|e| format!("Failed to get mods directory: {}", e))?;
        
        if mods_dir.exists() {
            fs::remove_dir_all(&mods_dir)
                .map_err(|e| format!("Failed to remove mods directory: {}", e))?;
            Logger::info_global(&format!("Removed mods directory: {:?}", mods_dir), Some(&installation_id));
        }
        
        save_kable_profiles(&profiles)
            .map_err(|e| format!("Failed to save kable profiles: {}", e))?;
        
        Logger::info_global(&format!("Deleted installation: {} ({})", installation.name, installation.id), Some(&installation.id));
        Ok(())
    } else {
        Err("Installation not found".to_string())
    }
}

#[tauri::command]
pub async fn get_minecraft_versions() -> Result<Vec<VersionInfo>, String> {
    // Fetch from Mojang's version manifest API
    let client = reqwest::Client::new();
    
    match client
        .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .send()
        .await
    {
        Ok(response) => {
            match response.json::<VersionManifest>().await {
                Ok(manifest) => {
                    // Return only release versions for now, limit to recent versions
                    let mut versions: Vec<VersionInfo> = manifest
                        .versions
                        .into_iter()
                        .filter(|v| v.version_type == "release")
                        .take(20) // Limit to most recent 20 releases
                        .collect();
                    
                    // Sort by release time (newest first)
                    versions.sort_by(|a, b| b.release_time.cmp(&a.release_time));
                    
                    Ok(versions)
                }
                Err(e) => {
                    Logger::error_global(&format!("Failed to parse version manifest: {}", e), None);
                    // Return fallback versions
                    Ok(get_fallback_versions())
                }
            }
        }
        Err(e) => {
            Logger::error_global(&format!("Failed to fetch version manifest: {}", e), None);
            // Return fallback versions
            Ok(get_fallback_versions())
        }
    }
}

fn get_fallback_versions() -> Vec<VersionInfo> {
    vec![
        VersionInfo {
            id: "1.21.3".to_string(),
            version_type: "release".to_string(),
            url: "".to_string(),
            time: "2024-10-23T12:00:00Z".to_string(),
            release_time: "2024-10-23T12:00:00Z".to_string(),
        },
        VersionInfo {
            id: "1.21.2".to_string(),
            version_type: "release".to_string(),
            url: "".to_string(),
            time: "2024-10-22T12:00:00Z".to_string(),
            release_time: "2024-10-22T12:00:00Z".to_string(),
        },
        VersionInfo {
            id: "1.21.1".to_string(),
            version_type: "release".to_string(),
            url: "".to_string(),
            time: "2024-08-08T12:00:00Z".to_string(),
            release_time: "2024-08-08T12:00:00Z".to_string(),
        },
        VersionInfo {
            id: "1.21".to_string(),
            version_type: "release".to_string(),
            url: "".to_string(),
            time: "2024-06-13T12:00:00Z".to_string(),
            release_time: "2024-06-13T12:00:00Z".to_string(),
        },
        VersionInfo {
            id: "1.20.6".to_string(),
            version_type: "release".to_string(),
            url: "".to_string(),
            time: "2024-04-29T12:00:00Z".to_string(),
            release_time: "2024-04-29T12:00:00Z".to_string(),
        },
    ]
}

#[tauri::command]
pub async fn open_installation_folder(installation_id: String) -> Result<(), String> {
    let profiles = load_kable_profiles()
        .map_err(|e| format!("Failed to load kable profiles: {}", e))?;
    
    if let Some(_installation) = profiles.profiles.get(&installation_id) {
        // Open the mods directory for this installation
        let mods_dir = get_installation_mods_directory(&installation_id)
            .map_err(|e| format!("Failed to get mods directory: {}", e))?;
        
        if mods_dir.exists() {
            #[cfg(target_os = "windows")]
            {
                std::process::Command::new("explorer")
                    .arg(&mods_dir)
                    .spawn()
                    .map_err(|e| format!("Failed to open folder: {}", e))?;
            }
            
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .arg(&mods_dir)
                    .spawn()
                    .map_err(|e| format!("Failed to open folder: {}", e))?;
            }
            
            #[cfg(target_os = "linux")]
            {
                std::process::Command::new("xdg-open")
                    .arg(&mods_dir)
                    .spawn()
                    .map_err(|e| format!("Failed to open folder: {}", e))?;
            }
            
            Ok(())
        } else {
            Err("Mods directory does not exist".to_string())
        }
    } else {
        Err("Installation not found".to_string())
    }
}

#[tauri::command]
pub async fn launch_minecraft_installation(app: AppHandle, installation_id: String) -> Result<(), String> {
    use crate::launcher::*;
    use crate::settings::load_settings;
    use std::process::{Command, Stdio};
    use serde_json::json;
    use uuid::Uuid;
    
    let instance_id = Uuid::new_v4().to_string();
    
    Logger::info(&app, &format!("=== Launching installation: {} ===", installation_id), Some(&instance_id));
    
    // Emit game launch event  
    if let Err(e) = app.emit_to("main", "game-launched", json!({
        "instanceId": instance_id,
        "installationId": installation_id,
        "profile": { "name": installation_id.clone() },
        "installation": { "path": "loading..." }
    })) {
        Logger::error(&app, &format!("Failed to emit game-launched event: {}", e), Some(&instance_id));
    }
    
    // Load required data
    let settings = load_settings().await
        .map_err(|e| {
            Logger::error(&app, &format!("Failed to load settings: {}", e), Some(&instance_id));
            format!("Failed to load settings: {}", e)
        })?;
    
    // Get minecraft directory
    let minecraft_dir = PathBuf::from(settings.minecraft_path.as_ref()
        .ok_or("Minecraft path not set in settings")?);
    
    // Load launcher profiles  
    let launcher_profiles = read_launcher_profiles(&minecraft_dir)
        .map_err(|e| format!("Failed to load launcher profiles: {}", e))?;
    
    // Load Kable profiles to get installation-specific settings
    let kable_profiles = load_kable_profiles()
        .map_err(|e| format!("Failed to load Kable profiles: {}", e))?;
    
    let kable_installation = kable_profiles.profiles.get(&installation_id);
    
    // Debug: Print available profile keys
    Logger::debug(&app, "Available profile keys:", Some(&instance_id));
    for key in launcher_profiles.profiles.keys() {
        Logger::debug(&app, &format!("  - {}", key), Some(&instance_id));
    }
    
    // Get current account (try real auth first, fall back to mock)
    let account = get_launch_auth_account().await?;
    
    // Get installation
    let installation = launcher_profiles.profiles.get(&installation_id)
        .ok_or(format!("Installation not found. Looking for key: '{}'. Available keys: {:?}", 
            installation_id, 
            launcher_profiles.profiles.keys().collect::<Vec<_>>()))?;
    
    // Get minecraft directory and paths
    let minecraft_dir = PathBuf::from(settings.minecraft_path.as_ref()
        .ok_or("Minecraft path not set in settings")?);
    let (assets_path, libraries_path, versions_path, natives_path) = get_minecraft_paths(&minecraft_dir)?;
    
    // Load version manifest
    let version_manifest = load_version_manifest(&installation.last_version_id, &minecraft_dir)?;
    
    // Create launch context
    let launch_context = LaunchContext {
        account,
        settings: settings.clone(),
        installation_path: minecraft_dir.clone(),
        assets_path,
        natives_path: natives_path.clone(),
        libraries_path: libraries_path.clone(),
        version_manifest: version_manifest.clone(),
    };
    
    // Build classpath
    let version_jar_path = versions_path
        .join(&installation.last_version_id)
        .join(format!("{}.jar", installation.last_version_id));
    
    let classpath = build_classpath(&version_manifest.libraries, &libraries_path, &version_jar_path)?;
    
    // Extract native libraries
    extract_natives(&version_manifest.libraries, &libraries_path, &natives_path)?;
    
    // Build variable substitution map
    let variables = build_variable_map(&launch_context, &installation.last_version_id, &classpath)?;
    
    // Find Java executable
    let java_executable = find_java_executable(settings.java_path.as_ref())?;
    
    // Build command arguments
    let mut command_args = Vec::new();
    
    // Add memory and JVM arguments (including natives path)
    // Use installation-specific memory if available, otherwise fall back to global settings
    let installation_memory = kable_installation.and_then(|ki| ki.memory);
    command_args.extend(build_jvm_arguments_with_memory(&settings, &natives_path, installation_memory));
    
    // Process JVM arguments from manifest
    let jvm_args = process_arguments(&version_manifest.arguments.jvm, &variables)?;
    command_args.extend(jvm_args);
    
    // Add classpath only if not already present in JVM args
    if !command_args.contains(&"-cp".to_string()) && !command_args.contains(&"-classpath".to_string()) {
        command_args.push("-cp".to_string());
        command_args.push(classpath);
    }
    
    // Add main class
    command_args.push(version_manifest.main_class.clone());
    
    // Process game arguments from manifest
    let game_args = process_arguments(&version_manifest.arguments.game, &variables)?;
    command_args.extend(game_args);
    
    // Set up game directory (use installation-specific directory or default)
    let game_dir = if let Some(custom_game_dir) = &installation.game_dir {
        PathBuf::from(custom_game_dir)
    } else {
        minecraft_dir.clone()
    };
    
    // For Kable mod management, we need the mods directory
    let _mods_dir = get_installation_mods_directory(&installation_id)?;
    
    // Create necessary directories in the game directory
    for subdir in ["saves", "resourcepacks", "shaderpacks", "screenshots", "config"] {
        let dir_path = game_dir.join(subdir);
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)
                .map_err(|e| format!("Failed to create directory {}: {}", dir_path.display(), e))?;
        }
    }
    
    // TODO: Skip mods linking for now to get basic launching working
    // We'll implement proper mod management later
    /*
    // Link Kable-managed mods to the game directory
    let target_mods_dir = game_dir.join("mods");
    
    // Remove existing mods directory if it exists and is not a link to our mods
    if target_mods_dir.exists() && !target_mods_dir.is_symlink() {
        fs::remove_dir_all(&target_mods_dir)
            .map_err(|e| format!("Failed to remove existing mods directory: {}", e))?;
    }
    
    
    #[cfg(windows)]
    {
        // On Windows, create a junction link
        if !target_mods_dir.exists() {
            let output = Command::new("cmd")
                .args(["/C", "mklink", "/J", 
                       &target_mods_dir.to_string_lossy(), 
                       &mods_dir.to_string_lossy()])
                .output()
                .map_err(|e| format!("Failed to create mods junction: {}", e))?;
            
            if !output.status.success() {
                return Err(format!("Failed to create mods junction: {}", 
                    String::from_utf8_lossy(&output.stderr)));
            }
        }
    }
    
    #[cfg(not(windows))]
    {
        // On Unix-like systems, create a symlink
        if !target_mods_dir.exists() {
            std::os::unix::fs::symlink(&mods_dir, &target_mods_dir)
                .map_err(|e| format!("Failed to create mods symlink: {}", e))?;
        }
    }
    */
    
    // Launch Minecraft
    Logger::debug_global("=== LAUNCH DEBUG INFO ===", Some(&instance_id));
    Logger::debug_global(&format!("Java executable: {}", java_executable), Some(&instance_id));
    Logger::debug_global(&format!("Working directory: {}", game_dir.display()), Some(&instance_id));
    Logger::debug_global(&format!("Command args count: {}", command_args.len()), Some(&instance_id));
    Logger::debug_global("Full command:", Some(&instance_id));
    Logger::debug_global(&format!("  {} {}", java_executable, command_args.join(" ")), Some(&instance_id));
    Logger::debug_global("=========================", Some(&instance_id));
    
    let mut command = Command::new(&java_executable);
    command.args(&command_args);
    command.current_dir(&game_dir);
    
    // Set environment variables  
    command.env("JAVA_HOME", java_executable.clone());
    
    // Configure process for logging
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    
    // Emit launcher log for launch attempt
    if let Err(e) = app.emit_to("main", "launcher-log", json!({
        "level": "info",
        "message": format!("Starting {} with Java: {}", installation_id, java_executable),
        "instanceId": instance_id
    })) {
        Logger::error_global(&format!("Failed to emit launcher-log event: {}", e), Some(&instance_id));
    }
    
    // Spawn the Minecraft process
    match command.spawn() {
        Ok(mut child) => {
            let pid = child.id();
            Logger::log(&app, LogLevel::Info, "=== MINECRAFT PROCESS SPAWNED ===", Some(&instance_id));
            Logger::log(&app, LogLevel::Info, &format!("Process ID: {}", pid), Some(&instance_id));
            Logger::log(&app, LogLevel::Info, "Minecraft launched successfully!", Some(&instance_id));
            Logger::log(&app, LogLevel::Info, "================================", Some(&instance_id));

            // Emit process started event
            if let Err(e) = app.emit_to("main", "game-process-event", json!({
                "instanceId": instance_id,
                "type": "started",
                "data": { "pid": pid }
            })) {
                Logger::error_global(&format!("Failed to emit game-process-event: {}", e), Some(&instance_id));
            }
            
            // Update the lastUsed timestamp in launcher profiles
            if let Err(e) = update_profile_last_used(&installation_id, &minecraft_dir).await {
                Logger::log(&app, LogLevel::Warning, &format!("Failed to update lastUsed timestamp: {}", e), Some(&instance_id));
            }
            
            // Handle process output and completion
            let app_clone = app.clone();
            let instance_id_clone = instance_id.clone();
            std::thread::spawn(move || {
                // Handle stdout
                if let Some(stdout) = child.stdout.take() {
                    use std::io::{BufRead, BufReader};
                    let reader = BufReader::new(stdout);
                    let app_stdout = app_clone.clone();
                    let instance_stdout = instance_id_clone.clone();
                    
                    std::thread::spawn(move || {
                        for line_result in reader.lines() {
                            match line_result {
                                Ok(line) => {
                                    if let Err(e) = app_stdout.emit_to("main", "game-process-event", json!({
                                        "instanceId": instance_stdout,
                                        "type": "output",
                                        "data": { "line": line }
                                    })) {
                                        Logger::console_log(LogLevel::Error, &format!("Failed to emit stdout event: {}", e), Some(&instance_stdout));
                                    }
                                }
                                Err(e) => {
                                    Logger::console_log(LogLevel::Error, &format!("Error reading stdout line: {}", e), Some(&instance_stdout));
                                    break; // Exit on error to prevent infinite loop
                                }
                            }
                        }
                    });
                }
                
                // Handle stderr
                if let Some(stderr) = child.stderr.take() {
                    use std::io::{BufRead, BufReader};
                    let reader = BufReader::new(stderr);
                    let app_stderr = app_clone.clone();
                    let instance_stderr = instance_id_clone.clone();
                    
                    std::thread::spawn(move || {
                        for line_result in reader.lines() {
                            match line_result {
                                Ok(line) => {
                                    if let Err(e) = app_stderr.emit_to("main", "game-process-event", json!({
                                        "instanceId": instance_stderr,
                                        "type": "error", 
                                        "data": { "line": line }
                                    })) {
                                        Logger::console_log(LogLevel::Error, &format!("Failed to emit stderr event: {}", e), Some(&instance_stderr));
                                    }
                                }
                                Err(e) => {
                                    Logger::console_log(LogLevel::Error, &format!("Error reading stderr line: {}", e), Some(&instance_stderr));
                                    break; // Exit on error to prevent infinite loop
                                }
                            }
                        }
                    });
                }
                
                // Wait for process completion
                if let Ok(status) = child.wait() {
                    let exit_code = status.code().unwrap_or(-1);
                    Logger::info_global(&format!("Minecraft process exited with status: {}", exit_code), Some(&instance_id_clone));
                    
                    if let Err(e) = app_clone.emit_to("main", "game-process-event", json!({
                        "instanceId": instance_id_clone,
                        "type": "exit",
                        "data": { "code": exit_code }
                    })) {
                        Logger::error_global(&format!("Failed to emit exit event: {}", e), Some(&instance_id_clone));
                    }
                }
            });
            
            Ok(())
        },
        Err(e) => {
            let error_msg = format!("Failed to spawn Minecraft process: {}", e);
            if let Err(emit_err) = app.emit_to("main", "launcher-log", json!({
                "level": "error",
                "message": error_msg.clone(),
                "instanceId": instance_id
            })) {
                Logger::error_global(&format!("Failed to emit error event: {}", emit_err), Some(&instance_id));
            }
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn quick_launch_minecraft(version_name: String) -> Result<(), String> {
    use crate::launcher::*;
    use crate::settings::load_settings;
    use std::process::Command;
    
    // Load required data
    let settings = load_settings().await
        .map_err(|e| format!("Failed to load settings: {}", e))?;
    
    // Get current account (try real auth first, fall back to mock)
    let account = get_launch_auth_account().await?;
    
    // Get minecraft directory and paths
    let minecraft_dir = PathBuf::from(settings.minecraft_path.as_ref()
        .ok_or("Minecraft path not set in settings")?);
    let (assets_path, libraries_path, versions_path, natives_path) = get_minecraft_paths(&minecraft_dir)?;
    
    // Check if version exists
    let version_dir = versions_path.join(&version_name);
    if !version_dir.exists() {
        return Err(format!("Version '{}' not found in .minecraft/versions/", version_name));
    }
    
    // Load version manifest
    let version_manifest = load_version_manifest(&version_name, &minecraft_dir)?;
    
    // Create launch context
    let launch_context = LaunchContext {
        account,
        settings: settings.clone(),
        installation_path: minecraft_dir.clone(),
        assets_path,
        natives_path: natives_path.clone(),
        libraries_path: libraries_path.clone(),
        version_manifest: version_manifest.clone(),
    };
    
    // Build classpath
    let version_jar_path = version_dir.join(format!("{}.jar", version_name));
    let classpath = build_classpath(&version_manifest.libraries, &libraries_path, &version_jar_path)?;
    
    // Extract native libraries
    extract_natives(&version_manifest.libraries, &libraries_path, &natives_path)?;
    
    // Build variable substitution map
    let variables = build_variable_map(&launch_context, &version_name, &classpath)?;
    
    // Find Java executable
    let java_executable = find_java_executable(settings.java_path.as_ref())?;
    
    // Build command arguments
    let mut command_args = Vec::new();
    
    // Add memory and JVM arguments (including natives path)
    command_args.extend(build_jvm_arguments(&settings, &natives_path));
    
    // Process JVM arguments from manifest
    let jvm_args = process_arguments(&version_manifest.arguments.jvm, &variables)?;
    command_args.extend(jvm_args);
    
    // Add classpath only if not already present in JVM args
    if !command_args.contains(&"-cp".to_string()) && !command_args.contains(&"-classpath".to_string()) {
        command_args.push("-cp".to_string());
        command_args.push(classpath);
    }
    
    // Add main class
    command_args.push(version_manifest.main_class.clone());
    
    // Process game arguments from manifest
    let game_args = process_arguments(&version_manifest.arguments.game, &variables)?;
    command_args.extend(game_args);
    
    // Use default minecraft directory as game directory for quick launch
    let game_dir = &minecraft_dir;
    
    // Launch Minecraft
    Logger::info_global(&format!("Quick launching Minecraft {} with command: {} {}", version_name, java_executable, command_args.join(" ")), None);
    
    let mut command = Command::new(&java_executable);
    command.args(&command_args);
    command.current_dir(game_dir);
    
    // Set environment variables
    command.env("JAVA_HOME", java_executable.clone());
    
    let child = command.spawn()
        .map_err(|e| format!("Failed to launch Minecraft: {}", e))?;
    
    Logger::info_global(&format!("Minecraft launched with PID: {}", child.id()), None);
    
    // Try to find and update any profile that uses this version
    if let Err(e) = update_profile_last_used_by_version(&version_name, &minecraft_dir).await {
        Logger::warn_global(&format!("Failed to update lastUsed timestamp for version {}: {}", version_name, e), None);
        // Don't fail the launch for this, just log the warning
    }
    
    Ok(())
}

/// Helper function to get mock auth account for testing
async fn get_launch_auth_account() -> Result<MicrosoftAccount, String> {
    // First, try to get a real authenticated account from storage
    match get_active_launcher_account().await {
        Ok(Some(account)) => {
            Logger::info_global(&format!("Using real Microsoft account: {}", account.username), None);
            
            // Check if the Minecraft token is still valid
            if let Some(_minecraft_token) = &account.minecraft_access_token {
                if let Some(expires_at) = account.minecraft_expires_at {
                    if expires_at > chrono::Utc::now().timestamp() {
                        Logger::info_global("Minecraft token is valid, launching with real authentication", None);
                        return Ok(account);
                    } else {
                        Logger::warn_global("Minecraft token expired, need to refresh", None);
                    }
                } else {
                    Logger::info_global("No Minecraft token expiry info, assuming valid", None);
                    return Ok(account);
                }
            } else {
                Logger::warn_global("No Minecraft access token available", None);
            }
            
            // Use the real account data but fall back to offline mode for launch
            Logger::info_global("Using real account data in offline mode", None);
            return Ok(MicrosoftAccount {
                id: account.id,
                username: account.username, // Use real username
                uuid: account.uuid, // Use real UUID if available
                access_token: account.access_token,
                refresh_token: account.refresh_token,
                expires_at: account.expires_at,
                skin_url: account.skin_url,
                is_active: account.is_active,
                last_used: account.last_used,
                minecraft_access_token: None, // No valid Minecraft token, will use offline mode
                minecraft_expires_at: None,
                xbox_user_hash: account.xbox_user_hash,
            });
        }
        Ok(None) => {
            Logger::info_global("No active Microsoft account found", None);
        }
        Err(e) => {
            Logger::warn_global(&format!("Failed to load active account: {}", e), None);
        }
    }
    
    // Fall back to mock auth for testing
    Logger::info_global("No real Microsoft account found, using mock authentication", None);
    Logger::info_global("Note: This will launch Minecraft in offline mode", None);
    Logger::info_global("To get online authentication, please log in through the UI first", None);
    
    // Check if auth is enabled (this will always return true in our mock system)
    let auth_status = check_auth_status().await?;
    if !auth_status.authenticated {
        return Err("No authenticated Microsoft account found. Please log in first.".to_string());
    }
    
    // Get mock access token
    let access_token = get_access_token().await?;
    
    // Return mock account
    Ok(MicrosoftAccount {
        id: "test-id".to_string(),
        username: auth_status.username.unwrap_or("TestUser".to_string()),
        uuid: auth_status.uuid.unwrap_or("test-uuid-1234".to_string()),
        access_token,
        refresh_token: "mock_refresh_token".to_string(),
        expires_at: chrono::Utc::now().timestamp() + 3600,
        skin_url: None,
        is_active: true,
        last_used: chrono::Utc::now().timestamp(),
        minecraft_access_token: None, // Use offline mode instead of mock tokens
        minecraft_expires_at: None,
        xbox_user_hash: "mock_xbox_hash".to_string(),
    })
}

#[tauri::command]
pub async fn launch_most_recent_installation(app: AppHandle) -> Result<(), String> {
    use crate::settings::load_settings;
    
    // Load settings to get minecraft path
    let settings = load_settings().await
        .map_err(|e| format!("Failed to load settings: {}", e))?;
    
    let minecraft_dir = PathBuf::from(settings.minecraft_path.as_ref()
        .ok_or("Minecraft path not set in settings")?);
    
    // Read launcher profiles to find most recent installation
    let profiles = read_launcher_profiles(&minecraft_dir)?;
    
    // Find the most recently used profile
    let most_recent_profile = profiles.profiles
        .iter() // Use iter() to get both key and value
        .filter(|(_, profile)| profile.last_used.is_some())
        .max_by(|(_, a), (_, b)| {
            let a_time = a.last_used.as_ref().unwrap();
            let b_time = b.last_used.as_ref().unwrap();
            a_time.cmp(b_time)
        });
    
    let (profile_id, _profile) = most_recent_profile
        .ok_or("No recently used installations found. Please select an installation to play.")?;
    
    // Launch using the profile ID
    launch_minecraft_installation(app, profile_id.clone()).await
}

#[tauri::command]
pub async fn update_installation(
    installation_id: String,
    request: CreateInstallationRequest,
) -> Result<KableInstallation, String> {
    let mut profiles = load_kable_profiles()
        .map_err(|e| format!("Failed to load kable profiles: {}", e))?;
    
    if let Some(mut installation) = profiles.profiles.get(&installation_id).cloned() {
        // Update the installation with new values
        installation.name = request.name;
        installation.version = request.version;
        installation.mod_loader = request.mod_loader;
        installation.description = request.description;
        installation.game_directory = request.game_directory;
        installation.java_path = request.java_path;
        installation.jvm_args = request.jvm_args;
        installation.memory = request.memory;
        
        // Update the installation in the profiles
        profiles.profiles.insert(installation_id.clone(), installation.clone());
        
        save_kable_profiles(&profiles)
            .map_err(|e| format!("Failed to save kable profiles: {}", e))?;
        
        Logger::info_global(&format!("Updated installation: {} ({})", installation.name, installation.id), Some(&installation.id));
        Ok(installation)
    } else {
        Err("Installation not found".to_string())
    }
}

#[derive(Serialize)]
pub struct ModLoaderDetectionResult {
    pub mod_loader: String,
    pub loader_version: Option<String>,
}

#[tauri::command]
pub async fn detect_installation_mod_loader(installation_id: String) -> Result<ModLoaderDetectionResult, String> {
    // Try to find the installation in kable profiles first
    let kable_profiles = load_kable_profiles()
        .map_err(|e| format!("Failed to load kable profiles: {}", e))?;
    
    if let Some(installation) = kable_profiles.profiles.get(&installation_id) {
        // If it's a kable installation and has a configured mod loader, use that
        if installation.mod_loader != "vanilla" {
            return Ok(ModLoaderDetectionResult {
                mod_loader: installation.mod_loader.clone(),
                loader_version: installation.loader_version.clone(),
            });
        }
    }
    
    // If it's not a kable installation or is vanilla, scan the actual Minecraft installation
    let settings = crate::settings::load_settings().await
        .map_err(|e| format!("Failed to load settings: {}", e))?;
    
    let minecraft_installations = get_minecraft_installations(settings.minecraft_path)
        .await
        .map_err(|e| format!("Failed to get minecraft installations: {}", e))?;
    
    let installation = minecraft_installations
        .iter()
        .find(|i| i.id == installation_id)
        .ok_or("Installation not found")?;
    
    // Use the existing detection logic
    Ok(ModLoaderDetectionResult {
        mod_loader: installation.installation_type.clone(),
        loader_version: installation.loader_version.clone(),
    })
}

/// Update the lastUsed timestamp for a profile in launcher_profiles.json
async fn update_profile_last_used(installation_id: &str, minecraft_dir: &Path) -> Result<(), String> {
    // Load current launcher profiles
    let mut launcher_profiles = read_launcher_profiles(minecraft_dir)
        .map_err(|e| format!("Failed to read launcher profiles: {}", e))?;
    
    // Find and update the profile
    if let Some(profile) = launcher_profiles.profiles.get_mut(installation_id) {
        let now = chrono::Utc::now().to_rfc3339();
        profile.last_used = Some(now.clone());
        
        Logger::info_global(&format!("Updated lastUsed for profile '{}' to: {}", installation_id, now), Some(installation_id));
        
        // Write back to file
        let profiles_path = minecraft_dir.join("launcher_profiles.json");
        let json_content = serde_json::to_string_pretty(&launcher_profiles)
            .map_err(|e| format!("Failed to serialize launcher profiles: {}", e))?;
        
        fs::write(&profiles_path, json_content)
            .map_err(|e| format!("Failed to write launcher profiles: {}", e))?;
        
        Ok(())
    } else {
        Err(format!("Profile '{}' not found in launcher_profiles.json", installation_id))
    }
}

/// Update the lastUsed timestamp for profiles that use a specific version
async fn update_profile_last_used_by_version(version_name: &str, minecraft_dir: &Path) -> Result<(), String> {
    // Load current launcher profiles
    let mut launcher_profiles = read_launcher_profiles(minecraft_dir)
        .map_err(|e| format!("Failed to read launcher profiles: {}", e))?;
    
    let mut updated_count = 0;
    let now = chrono::Utc::now().to_rfc3339();
    
    // Find and update profiles that use this version
    for (profile_id, profile) in launcher_profiles.profiles.iter_mut() {
        if profile.last_version_id == version_name {
            profile.last_used = Some(now.clone());
            Logger::info_global(&format!("Updated lastUsed for profile '{}' (version: {}) to: {}", profile_id, version_name, now), None);
            updated_count += 1;
        }
    }
    
    if updated_count > 0 {
        // Write back to file
        let profiles_path = minecraft_dir.join("launcher_profiles.json");
        let json_content = serde_json::to_string_pretty(&launcher_profiles)
            .map_err(|e| format!("Failed to serialize launcher profiles: {}", e))?;
        
        fs::write(&profiles_path, json_content)
            .map_err(|e| format!("Failed to write launcher profiles: {}", e))?;
        
        Logger::info_global(&format!("Updated {} profile(s) for version '{}'", updated_count, version_name), None);
        Ok(())
    } else {
        // This is not necessarily an error - the version might be launched directly without a profile
        Logger::info_global(&format!("No profiles found for version '{}', this is normal for direct version launches", version_name), None);
        Ok(())
    }
}


