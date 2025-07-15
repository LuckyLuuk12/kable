use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use crate::AppError;
use crate::auth::{MicrosoftAccount, check_auth_status, get_access_token};

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
    pub created: Option<String>,     // ISO date string
    pub game_dir: PathBuf,
    pub java_path: Option<String>,
    pub jvm_args: Option<Vec<String>>,
    pub resolution: Option<Resolution>,
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
    println!("=== Creating installation for version: {} ===", version_id);
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
    
    if let Some(key) = &profile_key {
        println!("Found matching profile key: {}", key);
    } else {
        println!("No matching profile found, using version_id as ID");
    }

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
        resolution: profile.and_then(|p| p.resolution.clone()),
    })
}

/// Detect the installation type (vanilla, fabric, forge, etc.) and loader version
fn detect_installation_type(version_dir: &Path, version_id: &str) -> Result<(String, Option<String>), AppError> {
    let json_path = version_dir.join(format!("{}.json", version_id));
    
    if !json_path.exists() {
        return Ok(("vanilla".to_string(), None));
    }

    let json_content = fs::read_to_string(&json_path)?;
    let version_json: serde_json::Value = match serde_json::from_str(&json_content) {
        Ok(json) => json,
        Err(e) => {
            println!("Warning: Failed to parse JSON for {}: {}", version_id, e);
            println!("JSON file path: {:?}", json_path);
            // Return vanilla as fallback when JSON parsing fails
            return Ok(("vanilla".to_string(), None));
        }
    };

    // Check for Fabric
    if let Some(libraries) = version_json["libraries"].as_array() {
        for lib in libraries {
            if let Some(name) = lib["name"].as_str() {
                if name.contains("fabric-loader") {
                    let loader_version = extract_version_from_library_name(name, "fabric-loader");
                    return Ok(("fabric".to_string(), loader_version));
                }
                if name.contains("forge") || name.contains("minecraftforge") {
                    let loader_version = extract_version_from_library_name(name, "forge");
                    return Ok(("forge".to_string(), loader_version));
                }
                if name.contains("quilt-loader") {
                    let loader_version = extract_version_from_library_name(name, "quilt-loader");
                    return Ok(("quilt".to_string(), loader_version));
                }
                if name.contains("neoforge") {
                    let loader_version = extract_version_from_library_name(name, "neoforge");
                    return Ok(("neoforge".to_string(), loader_version));
                }
            }
        }
    }

    // Check version ID for loader info
    if version_id.contains("fabric") {
        return Ok(("fabric".to_string(), None));
    }
    if version_id.contains("forge") {
        return Ok(("forge".to_string(), None));
    }
    if version_id.contains("quilt") {
        return Ok(("quilt".to_string(), None));
    }

    Ok(("vanilla".to_string(), None))
}

/// Extract version from library name (e.g., "net.fabricmc:fabric-loader:0.15.0" -> Some("0.15.0"))
fn extract_version_from_library_name(library_name: &str, loader_type: &str) -> Option<String> {
    if library_name.contains(loader_type) {
        let parts: Vec<&str> = library_name.split(':').collect();
        if parts.len() >= 3 {
            return Some(parts[2].to_string());
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
        println!("Created Kable directory: {:?}", kable_dir);
    }
    
    // Create only essential subdirectories - much simpler!
    let mods_dir = kable_dir.join("mods");
    if !mods_dir.exists() {
        fs::create_dir_all(&mods_dir)?;
        println!("Created mods directory: {:?}", mods_dir);
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
        println!("Created mods directory for installation {}: {:?}", installation_id, mods_dir);
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
    
    println!("=== Loading installations ===");
    
    // 1. Load Kable-managed profiles
    match load_kable_profiles() {
        Ok(profiles) => {
            let kable_installations: Vec<KableInstallation> = profiles.profiles.into_values().collect();
            println!("Found {} Kable-managed installations", kable_installations.len());
            for install in &kable_installations {
                println!("  - {} ({})", install.name, install.version);
            }
            all_installations.extend(kable_installations);
        }
        Err(e) => {
            println!("Warning: Failed to load Kable profiles: {}", e);
            // Continue anyway - we can still show existing Minecraft installations
        }
    }
    
    // 2. Scan for existing Minecraft installations and convert them to KableInstallation format
    match get_minecraft_installations(None).await {
        Ok(minecraft_installations) => {
            println!("Found {} existing Minecraft installations", minecraft_installations.len());
            for mc_install in minecraft_installations {
                println!("  - Processing: {} ({})", mc_install.name, mc_install.version);
                
                // Convert MinecraftInstallation to KableInstallation format
                // Use the MinecraftInstallation's ID which should be the profile key
                
                // Check if we already have this in our Kable profiles
                let already_exists = all_installations.iter().any(|kable_install| {
                    kable_install.id == mc_install.id
                });
                
                if !already_exists {
                    let kable_installation = KableInstallation {
                        id: mc_install.id,
                        name: mc_install.name,
                        version: mc_install.version,
                        mod_loader: "vanilla".to_string(), // Existing installations are vanilla unless detected otherwise
                        loader_version: None,
                        description: Some("Existing Minecraft installation".to_string()),
                        game_directory: Some(mc_install.game_dir.to_string_lossy().to_string()),
                        java_path: mc_install.java_path,
                        jvm_args: mc_install.jvm_args.map(|args| args.join(" ")),
                        last_played: mc_install.last_played,
                        created: mc_install.created.unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
                        path: mc_install.path.to_string_lossy().to_string(),
                        is_valid: mc_install.is_valid,
                    };
                    
                    println!("    + Added as: {}", kable_installation.name);
                    all_installations.push(kable_installation);
                } else {
                    println!("    - Skipped (already exists in Kable profiles)");
                }
            }
        }
        Err(e) => {
            println!("Warning: Failed to scan existing Minecraft installations: {}", e);
            // Continue anyway - we can still show Kable-managed installations
        }
    }
    
    println!("=== Total installations found: {} ===", all_installations.len());
    Ok(all_installations)
}

#[tauri::command]
pub async fn create_installation(
    name: String,
    version: String,
    mod_loader: String,
    game_directory: Option<String>,
    java_path: Option<String>,
    jvm_args: Option<String>,
    description: Option<String>,
) -> Result<KableInstallation, String> {
    let installation_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    
    // Use the default Minecraft directory structure - no custom installation paths
    let minecraft_dir = get_default_minecraft_directory()
        .map_err(|e| format!("Failed to get Minecraft directory: {}", e))?;
    
    let installation = KableInstallation {
        id: installation_id.clone(),
        name,
        version,
        mod_loader,
        loader_version: None, // TODO: Detect mod loader version
        description,
        game_directory,
        java_path,
        jvm_args,
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
    
    println!("Created installation: {} ({})", installation.name, installation.id);
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
            println!("Removed mods directory: {:?}", mods_dir);
        }
        
        save_kable_profiles(&profiles)
            .map_err(|e| format!("Failed to save kable profiles: {}", e))?;
        
        println!("Deleted installation: {} ({})", installation.name, installation.id);
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
                    println!("Failed to parse version manifest: {}", e);
                    // Return fallback versions
                    Ok(get_fallback_versions())
                }
            }
        }
        Err(e) => {
            println!("Failed to fetch version manifest: {}", e);
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
pub async fn launch_minecraft_installation(installation_id: String) -> Result<(), String> {
    use crate::launcher::*;
    use crate::settings::load_settings;
    use std::process::Command;
    
    println!("=== Launching installation: {} ===", installation_id);
    
    // Load required data
    let settings = load_settings().await
        .map_err(|e| format!("Failed to load settings: {}", e))?;
    
    // Get minecraft directory
    let minecraft_dir = PathBuf::from(settings.minecraft_path.as_ref()
        .ok_or("Minecraft path not set in settings")?);
    
    // Load launcher profiles  
    let launcher_profiles = read_launcher_profiles(&minecraft_dir)
        .map_err(|e| format!("Failed to load launcher profiles: {}", e))?;
    
    // Debug: Print available profile keys
    println!("Available profile keys:");
    for key in launcher_profiles.profiles.keys() {
        println!("  - {}", key);
    }
    
    // Get current account using mock auth system
    let account = get_mock_auth_account().await?;
    
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
    command_args.extend(build_jvm_arguments(&settings, &natives_path));
    
    // Process JVM arguments from manifest
    let jvm_args = process_arguments(&version_manifest.arguments.jvm, &variables)?;
    command_args.extend(jvm_args);
    
    // Add classpath
    command_args.push("-cp".to_string());
    command_args.push(classpath);
    
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
    println!("=== LAUNCH DEBUG INFO ===");
    println!("Java executable: {}", java_executable);
    println!("Working directory: {}", game_dir.display());
    println!("Command args count: {}", command_args.len());
    println!("Full command:");
    println!("  {} {}", java_executable, command_args.join(" "));
    println!("=========================");
    
    let mut command = Command::new(&java_executable);
    command.args(&command_args);
    command.current_dir(&game_dir);
    
    // Set environment variables
    command.env("JAVA_HOME", java_executable.clone());
    
    // For better debugging, let's run the command and capture output immediately
    match command.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            println!("=== PROCESS OUTPUT ===");
            println!("Exit status: {}", output.status);
            
            if !stdout.trim().is_empty() {
                println!("STDOUT:");
                println!("{}", stdout);
            }
            
            if !stderr.trim().is_empty() {
                println!("STDERR:");
                println!("{}", stderr);
            }
            println!("=====================");
            
            if output.status.success() {
                println!("Minecraft launched successfully!");
                Ok(())
            } else {
                Err(format!("Minecraft process failed to start. Exit code: {}", output.status))
            }
        },
        Err(e) => {
            Err(format!("Failed to spawn Minecraft process: {}", e))
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
    
    // Get current account using mock auth system
    let account = get_mock_auth_account().await?;
    
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
    
    // Add classpath
    command_args.push("-cp".to_string());
    command_args.push(classpath);
    
    // Add main class
    command_args.push(version_manifest.main_class.clone());
    
    // Process game arguments from manifest
    let game_args = process_arguments(&version_manifest.arguments.game, &variables)?;
    command_args.extend(game_args);
    
    // Use default minecraft directory as game directory for quick launch
    let game_dir = &minecraft_dir;
    
    // Launch Minecraft
    println!("Quick launching Minecraft {} with command: {} {}", version_name, java_executable, command_args.join(" "));
    
    let mut command = Command::new(&java_executable);
    command.args(&command_args);
    command.current_dir(game_dir);
    
    // Set environment variables
    command.env("JAVA_HOME", java_executable.clone());
    
    let child = command.spawn()
        .map_err(|e| format!("Failed to launch Minecraft: {}", e))?;
    
    println!("Minecraft launched with PID: {}", child.id());
    
    Ok(())
}

/// Helper function to get mock auth account for testing
async fn get_mock_auth_account() -> Result<MicrosoftAccount, String> {
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
        minecraft_access_token: Some("mock_minecraft_token".to_string()),
        minecraft_expires_at: Some(chrono::Utc::now().timestamp() + 3600),
        xbox_user_hash: "mock_xbox_hash".to_string(),
    })
}

#[tauri::command]
pub async fn launch_most_recent_installation() -> Result<(), String> {
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
        .values()
        .filter(|profile| profile.last_used.is_some())
        .max_by(|a, b| {
            let a_time = a.last_used.as_ref().unwrap();
            let b_time = b.last_used.as_ref().unwrap();
            a_time.cmp(b_time)
        });
    
    let profile = most_recent_profile
        .ok_or("No recently used installations found. Please select an installation to play.")?;
    
    // Launch using the installation's version
    launch_minecraft_installation(profile.last_version_id.clone()).await
}
