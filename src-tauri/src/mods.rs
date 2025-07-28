use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// use crate::installations::MinecraftInstallation;

// Mod management structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModProject {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub client_side: Compatibility,
    pub server_side: Compatibility,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub source: ModSource,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Compatibility {
    Required,
    Optional,
    Unsupported,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModSource {
    Modrinth,
    CurseForge,
    Local,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModVersion {
    pub id: String,
    pub version_number: String,
    pub version_type: VersionType,
    pub minecraft_versions: Vec<String>,
    pub mod_loaders: Vec<String>,
    pub date_published: String,
    pub downloads: u64,
    pub changelog: Option<String>,
    pub files: Vec<ModFile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum VersionType {
    Release,
    Beta,
    Alpha,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModFile {
    pub url: String,
    pub filename: String,
    pub size: u64,
    pub sha1: String,
    pub primary: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledMod {
    pub id: String,
    pub name: String,
    pub version: String,
    pub source: ModSource,
    pub source_id: String,
    pub file_path: String,
    pub minecraft_version: String,
    pub mod_loader: ModLoader,
    pub enabled: bool,
    pub dependencies: Vec<String>,
    pub auto_update: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModLoader {
    Fabric,
    Forge,
    Quilt,
    NeoForge,
}

// TODO: Implement mod management functions
// - Search Modrinth and CurseForge APIs
// - Download and install mods
// - Manage mod versions and updates
// - Handle mod dependencies
// - Create and manage modpacks
// - Import/export modpack configurations

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModInstallationConfig {
    pub id: String,
    pub name: String,
    pub installation_type: String,
    pub use_global_mods: bool,
    pub mods_folder_path: String,
}

/// Get all modded installations (excludes vanilla)
// #[tauri::command]
// pub async fn get_modded_installations(minecraft_path: String) -> Result<Vec<ModInstallationConfig>, String> {
//     let installations = crate::installations::get_installations()
//         .map_err(|e| format!("Failed to get installations: {}", e))?;
    
//     let modded_installations: Vec<ModInstallationConfig> = installations
//         .into_iter()
//         .filter(|installation| installation.version.loader != crate::Loader::Vanilla)
//         .map(|installation| {
//             let mods_folder_path = get_mods_folder_path(&minecraft_path, &installation);
//             ModInstallationConfig {
//                 id: installation.id,
//                 name: installation.name,
//                 installation_type: installation.version.loader.to_string(),
//                 use_global_mods: false,
//                 mods_folder_path,
//             }
//         })
//         .collect();
    
//     Ok(modded_installations)
// }

/// Set up mod folder for an installation
#[tauri::command]
pub async fn setup_installation_mods(
    minecraft_path: String,
    installation_id: String,
    use_global: bool,
) -> Result<String, String> {
    let minecraft_dir = PathBuf::from(&minecraft_path);
    let kable_dir = minecraft_dir.join("kable");
    let mods_base_dir = kable_dir.join("mods");
    
    // Create kable/mods directory if it doesn't exist
    if !mods_base_dir.exists() {
        fs::create_dir_all(&mods_base_dir)
            .map_err(|e| format!("Failed to create mods directory: {}", e))?;
    }
    
    let mods_folder_path = if use_global {
        mods_base_dir.join("kable-global")
    } else {
        mods_base_dir.join(&installation_id)
    };
    
    // Create the specific mods folder
    if !mods_folder_path.exists() {
        fs::create_dir_all(&mods_folder_path)
            .map_err(|e| format!("Failed to create installation mods directory: {}", e))?;
    }
    
    Ok(mods_folder_path.to_string_lossy().to_string())
}

// Get the mods folder path for an installation
// fn get_mods_folder_path(minecraft_path: &str, installation: &KableInstallation) -> String {
//     let kable_dir = get_minecraft_kable_dir();
//     if let Ok(kable_dir) = kable_dir {
//         return kable_dir.join("mods").join(&installation.id).to_string_lossy().to_string();
//     }
//     dirs::data_dir().map(|p| p.join(".minecraft").join("kable").join("mods").join(&installation.id).to_string_lossy().to_string())
//         .unwrap_or_else(|| format!("{}/.minecraft/kable/mods/{}", minecraft_path, installation.id))
// }

/// Get installed mods for a specific installation
// #[tauri::command]
// pub async fn get_installed_mods(
//     minecraft_path: String,
//     installation_id: String,
// ) -> Result<Vec<InstalledMod>, String> {
//     let installations = crate::installations_new::get_installations()
//         .map_err(|e| format!("Failed to get installations: {}", e))?;
    
//     let installation = installations
//         .iter()
//         .find(|i| i.id == installation_id)
//         .ok_or("Installation not found")?;
    
//     let mods_folder_path = get_mods_folder_path(&minecraft_path, installation);
//     let mods_dir = PathBuf::from(&mods_folder_path);
    
//     if !mods_dir.exists() {
//         return Ok(Vec::new());
//     }
    
//     let mut installed_mods = Vec::new();
    
//     // Read all .jar files in the mods directory
//     if let Ok(entries) = fs::read_dir(&mods_dir) {
//         for entry in entries.flatten() {
//             if let Some(file_name) = entry.file_name().to_str() {
//                 if file_name.ends_with(".jar") {
//                     // Create a basic installed mod entry
//                     // TODO: Parse mod metadata from jar files
//                     let installed_mod = InstalledMod {
//                         id: file_name.replace(".jar", ""),
//                         name: file_name.replace(".jar", "").replace("-", " "),
//                         version: "Unknown".to_string(),
//                         source: ModSource::Local,
//                         source_id: "".to_string(),
//                         file_path: entry.path().to_string_lossy().to_string(),
//                         minecraft_version: "Unknown".to_string(),
//                         mod_loader: match installation.version.loader {
//                             crate::Loader::Fabric => ModLoader::Fabric,
//                             crate::Loader::Forge => ModLoader::Forge,
//                             crate::Loader::Quilt => ModLoader::Quilt,
//                             crate::Loader::NeoForge => ModLoader::NeoForge,
//                             _ => ModLoader::Fabric, // Default fallback
//                         },
//                         enabled: !file_name.ends_with(".disabled"),
//                         dependencies: Vec::new(),
//                         auto_update: false,
//                     };
//                     installed_mods.push(installed_mod);
//                 }
//             }
//         }
//     }
    
//     Ok(installed_mods)
// }

/// Toggle mod enabled/disabled state
#[tauri::command]
pub async fn toggle_mod_enabled(
    mod_file_path: String,
    enabled: bool,
) -> Result<(), String> {
    let current_path = PathBuf::from(&mod_file_path);
    
    if !current_path.exists() {
        return Err("Mod file not found".to_string());
    }
    
    let new_path = if enabled {
        // Remove .disabled extension if present
        if mod_file_path.ends_with(".disabled") {
            PathBuf::from(mod_file_path.replace(".disabled", ""))
        } else {
            return Ok(()); // Already enabled
        }
    } else {
        // Add .disabled extension
        if !mod_file_path.ends_with(".disabled") {
            PathBuf::from(format!("{}.disabled", mod_file_path))
        } else {
            return Ok(()); // Already disabled
        }
    };
    
    fs::rename(&current_path, &new_path)
        .map_err(|e| format!("Failed to toggle mod state: {}", e))?;
    
    Ok(())
}

/// Update installation mod configuration
#[tauri::command]
pub async fn update_installation_mod_config(
    minecraft_path: String,
    installation_id: String,
    use_global_mods: bool,
    _custom_mods_path: Option<String>,
) -> Result<(), String> {
    // TODO: Update the installation config in launcher_profiles.json
    // For now, we'll just ensure the mod folder exists
    setup_installation_mods(minecraft_path, installation_id, use_global_mods).await?;
    Ok(())
}
