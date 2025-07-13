use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::AppError;

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
    let profile = profiles.profiles.values()
        .find(|p| p.last_version_id == version_id);

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
        id: version_id.to_string(),
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
    let version_json: serde_json::Value = serde_json::from_str(&json_content)?;

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
        .map_err(|e| AppError::Json(e))?;

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
