/**
 * This file read the .minecraft/launcher_profiles.json file and extracts the active profiles (aka installations).
 * Then it converts those to KableInstallation objects if they weren't already.
 * Meaning that the KableInstallations should have been read before this from .minecraft/kable_profiles.json.
 * Furthermore, it provides some getters/setters to access and manipulate the installations.
 * for actual launching of an installation, @see `src-tauri/src/launch.rs`.
 */

use std::{collections::HashMap, path::PathBuf};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::settings::{load_settings};

lazy_static::lazy_static! {
    pub static ref LOADER_MANIFEST_URLS: HashMap<Loader, &'static str> = {
        let mut m = HashMap::new();
        m.insert(Loader::Vanilla, "https://launchermeta.mojang.com/mc/game/version_manifest.json");
        m.insert(Loader::Fabric, "https://meta.fabricmc.net/v2/versions/loader");
        m.insert(Loader::Forge, "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json");
        m.insert(Loader::Quilt, "https://meta.quiltmc.org/v3/versions/loader");
        m.insert(Loader::NeoForge, "https://maven.neoforged.net/api/maven/versions/releases/net%2Fneoforged%2Fneoforge");
        m
    };
}
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Loader {
    /// https://launchermeta.mojang.com/mc/game/version_manifest.json
    Vanilla,
    /// https://meta.fabricmc.net/v2/versions/loader
    Fabric,
    /// https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json
    Forge,
    /// https://meta.quiltmc.org/v3/versions/loader
    Quilt,
    /// https://maven.neoforged.net/api/maven/versions/releases/net%2Fneoforged%2Fneoforge
    NeoForge
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct VersionData {
    pub id: String,
    pub loader: Loader,
    pub stable: bool,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct KableInstallation {
    pub id: String, // Unique ID for the installation
    pub name: String,
    pub icon: String,
    pub version: VersionData,
    pub created: String,
    pub last_used: String,
    pub java_args: Vec<String>,
    pub dedicated_resource_pack_folder: Option<String>,
    pub dedicated_shaders_folder: Option<String>,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")] // The JSON keys are in camelCase in the .minecraft/launcher_profiles.json
pub struct LauncherProfile {
    pub created: String,
    pub icon: String,
    pub java_args: String,
    pub last_used: String,
    /// Matches a version folder name in .minecraft/versions/
    pub last_version_id: String,
    pub name: String,
    // "latest-release", "latest-snapshot", ""
    #[serde(rename = "type")]
    pub profile_type: String,
}

pub fn get_launcher_profiles_path() -> Result<PathBuf, String> {
    let minecraft_dir = crate::get_default_minecraft_dir()?;
    Ok(minecraft_dir.join("launcher_profiles.json"))
}
pub fn get_kable_profiles_path() -> Result<PathBuf, String> {
    let kable_launcher_dir = crate::get_kable_launcher_dir()?;
    Ok(kable_launcher_dir.join("kable_profiles.json"))
}

 // Read the launcher_profiles.json file and load them into LauncherProfile objects
pub fn get_launcher_profiles() -> Result<Vec<LauncherProfile>, String> {
    let profiles_path = get_launcher_profiles_path()?;
    let profiles_data = std::fs::read_to_string(profiles_path)
        .map_err(|e| format!("Failed to read launcher profiles: {}", e))?;
    
    serde_json::from_str(&profiles_data)
        .map_err(|e| format!("Failed to parse launcher profiles JSON: {}", e))
}

// Use the Loader -> manifest url map to get all VersionData objects
pub fn get_all_versions() -> Result<Vec<VersionData>, String> {
    let mut all_versions = Vec::new();
    for (loader, url) in LOADER_MANIFEST_URLS.iter() {
        let response = reqwest::blocking::get(*url)
            .map_err(|e| format!("Failed to fetch version data from {}: {}", url, e))?;
        let json: serde_json::Value = response.json()
            .map_err(|e| format!("Failed to parse JSON from {}: {}", url, e))?;

        let versions = match loader {
            Loader::Vanilla => {
                let versions = json["versions"].as_array()
                    .ok_or("Expected 'versions' to be an array")?;
                versions.iter().map(|v| VersionData {
                    id: v["id"].as_str().unwrap_or_default().to_string(),
                    loader: Loader::Vanilla,
                    stable: v["type"].as_str() == Some("release"),
                }).collect()
            },
            Loader::Fabric => {
                let versions = json.as_array()
                    .ok_or("Expected JSON to be an array for Fabric")?;
                versions.iter().map(|v| {
                    let loader_version = v["version"].as_str().unwrap_or_default();
                    let mc_version = v["gameVersion"].as_str().unwrap_or_default();
                    VersionData {
                        id: format!("fabric-loader-{}-{}", loader_version, mc_version),
                        loader: Loader::Fabric,
                        stable: v["stable"].as_bool().unwrap_or(false),
                    }
                }).collect()
            },
            Loader::Forge => {
                let mut result = Vec::new();
                if let Some(obj) = json.as_object() {
                    for (mc_version, forge_versions) in obj.iter() {
                        if let Some(arr) = forge_versions.as_array() {
                            for forge_version in arr {
                                let forge_version_str = forge_version.as_str().unwrap_or_default();
                                result.push(VersionData {
                                    id: format!("{}-forge-{}", mc_version, forge_version_str),
                                    loader: Loader::Forge,
                                    stable: true,
                                });
                            }
                        }
                    }
                }
                result
            },
            Loader::Quilt => {
                let versions = json.as_array()
                    .ok_or("Expected JSON to be an array for Quilt")?;
                versions.iter().map(|v| {
                    let loader_version = v["version"].as_str().unwrap_or_default();
                    let mc_version = v["gameVersion"].as_str().unwrap_or_default();
                    VersionData {
                        id: format!("quilt-loader-{}-{}", loader_version, mc_version),
                        loader: Loader::Quilt,
                        stable: v["stable"].as_bool().unwrap_or(false),
                    }
                }).collect()
            },
            Loader::NeoForge => {
                let versions = json["versions"].as_array()
                    .ok_or("Expected 'versions' to be an array for NeoForge")?;
                versions.iter().map(|v| {
                    let version_id = v["version"].as_str().unwrap_or_default();
                    VersionData {
                        id: version_id.to_string(),
                        loader: Loader::NeoForge,
                        stable: v["stable"].as_bool().unwrap_or(false),
                    }
                }).collect()
            },
        };
        all_versions.extend(versions);
    }
    Ok(all_versions)
}

/// Given a LauncherProfile, get the corresponding VersionData
#[tauri::command]
pub fn get_version_data_for_profile(profile: &LauncherProfile) -> Result<VersionData, String> {
    if profile.last_version_id.is_empty() {
        return Err("Profile last_version_id is empty".to_string());
    }
    // We use the get_all_versions function to get a list of all possible supported version data
    let all_versions = get_all_versions()?;
    // Then we find the version data that matches the profile's last_version_id
    all_versions.iter()
    .find(|v| v.id == profile.last_version_id)
    .cloned()
    .ok_or_else(|| format!("No version data found for last_version_id: {}", profile.last_version_id))
}

// Loads the KableInstallations from the .minecraft/kable/kable_profiles.json file, does not ensure that launcher profiles have been converted
#[tauri::command]
pub fn get_kable_installations() -> Result<Vec<KableInstallation>, String> {
    let kable_profiles_path = get_kable_profiles_path()?;
    let profiles_data = std::fs::read_to_string(kable_profiles_path)
        .map_err(|e| format!("Failed to read Kable installations: {}", e))?;
    
    serde_json::from_str(&profiles_data)
        .map_err(|e| format!("Failed to parse Kable installations JSON: {}", e))
}

/// Takes the launcher profiles and converts them to KableInstallations by mapping relevant fields and extracting the version data
#[tauri::command]
pub fn convert_launcher_profiles_to_kable_installations() -> Result<(), String> {
    let mut installations = get_kable_installations()?;
    let to_be_added = 
        get_launcher_profiles()?
        .into_iter()
        .filter(|profile| {
            // If the name and created date already exist in the Kable installations, skip it
            !installations.iter().any(|i| i.name == profile.name && i.created == profile.created)
        })
        .map(|profile| {
            let version_data = get_version_data_for_profile(&profile)?;
            Ok(KableInstallation {
                // Generate a unique ID with uuid library
                id: uuid::Uuid::new_v4().to_string(),
                name: profile.name,
                icon: profile.icon,
                version: version_data,
                created: profile.created,
                last_used: profile.last_used,
                java_args: profile.java_args.split_whitespace().map(String::from).collect(),
                dedicated_resource_pack_folder: None, // Placeholder, can be set later
                dedicated_shaders_folder: None, // Placeholder, can be set later
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    // Save the installations to a file in .minecraft/kable/kable_profiles.json

    let kable_profiles_path = get_kable_profiles_path()?;
    installations.extend(to_be_added);
    let json_content = serde_json::to_string_pretty(&installations)
        .map_err(|e| format!("Failed to serialize Kable installations: {}", e))?;
    std::fs::write(kable_profiles_path, json_content)
        .map_err(|e| format!("Failed to write Kable installations: {}", e))
}

#[tauri::command]
pub fn convert_kable_installations_to_launcher_profiles() -> Result<(), String> {
    let installations = get_kable_installations()?;
    let launcher_profiles: Vec<LauncherProfile> = installations.into_iter()
        .map(|installation| LauncherProfile {
            created: installation.created,
            icon: installation.icon,
            java_args: installation.java_args.join(" "),
            last_used: installation.last_used,
            last_version_id: installation.version.id,
            name: installation.name,
            profile_type: "".to_string(), // TODO: maybe add profile_type to KableInstallation
        })
        .collect();

    let launcher_profiles_path = get_launcher_profiles_path()?;
    let json_content = serde_json::to_string_pretty(&launcher_profiles)
        .map_err(|e| format!("Failed to serialize launcher profiles: {}", e))?;
    std::fs::write(launcher_profiles_path, json_content)
        .map_err(|e| format!("Failed to write launcher profiles: {}", e))
}

/// Get ALL installations, both kable-added ones as old-official launcher profile ones
#[tauri::command]
pub fn get_installations() -> Result<Vec<KableInstallation>, String> {
    convert_launcher_profiles_to_kable_installations()?;
    get_kable_installations()
}

/// Gets an individual KableInstallation by ID
#[tauri::command]
pub fn get_installation(id: &str) -> Result<Option<KableInstallation>, String> {
    let installations = get_kable_installations()?;
    Ok(installations.into_iter().find(|i| i.id == id))
}

/// Given an ID and new installation data, modify the existing KableInstallation (also updates the file)
#[tauri::command]
pub fn modify_kable_installation(id: &str, new_installation: KableInstallation) -> Result<(), String> {
    let mut installations = get_kable_installations()?;
    let index = installations.iter().position(|i| i.id == id);
    if let Some(index) = index {
        installations[index] = new_installation;
        let kable_profiles_path = get_kable_profiles_path()?;
        let json_content = serde_json::to_string_pretty(&installations)
            .map_err(|e| format!("Failed to serialize Kable installations: {}", e))?;
        std::fs::write(kable_profiles_path, json_content)
            .map_err(|e| format!("Failed to write Kable installations: {}", e))?;
        Ok(())
    } else {
        Err(format!("No Kable installation found with id: {}", id))
    }
}

#[tauri::command]
pub fn get_last_played_installation() -> Result<KableInstallation, String> {
    let installations = get_installations()?;
    // Find the installation with the most recent last_used date
    installations.into_iter()
    .max_by_key(|i| i.last_used.clone())
    .ok_or("No installations found".to_string())
}

/// Gets and updates the last played installation to the current time
#[tauri::command]
pub fn modify_last_played_installation() -> Result<(), String> {
    let mut installation = get_last_played_installation()?;
    installation.last_used = chrono::Utc::now().to_rfc3339();
    modify_kable_installation(&installation.id, installation.clone())?;
    Ok(())
}

#[tauri::command]
pub fn modify_all_installations(new_installations: Vec<KableInstallation>) -> Result<(), String> {
    let kable_profiles_path = get_kable_profiles_path()?;
    let json_content = serde_json::to_string_pretty(&new_installations)
        .map_err(|e| format!("Failed to serialize Kable installations: {}", e))?;
    std::fs::write(kable_profiles_path, json_content)
        .map_err(|e| format!("Failed to write Kable installations: {}", e))?;

    // Now for all installations that also exist in the launcher_profiles.json, we need to update them
    let launcher_profiles = get_launcher_profiles()?;
    // Build a map for the profiles object
    use serde_json::Map;
    let mut profiles_map = Map::new();
    for installation in &new_installations {
        if let Some(profile) = launcher_profiles.iter().find(|p| p.name == installation.name && p.created == installation.created) {
            // Use installation.id as key if available, else fallback to name+created
            let key = if !installation.id.is_empty() {
                installation.id.clone()
            } else {
                format!("{}-{}", installation.name, installation.created)
            };
            let value = serde_json::to_value(LauncherProfile {
                created: profile.created.clone(),
                icon: profile.icon.clone(),
                java_args: profile.java_args.clone(),
                last_used: installation.last_used.clone(),
                last_version_id: installation.version.id.clone(),
                name: installation.name.clone(),
                profile_type: profile.profile_type.clone(),
            }).unwrap();
            profiles_map.insert(key, value);
        }
    }

    // put the json with all profiles in { "profiles": <object>, ... } while preserving the rest of the file
    let launcher_profiles_path = get_launcher_profiles_path()?;
    let launcher_profiles_data = std::fs::read_to_string(&launcher_profiles_path)
        .map_err(|e| format!("Failed to read launcher profiles: {}", e))?;
    let mut launcher_profiles_json: serde_json::Value = serde_json::from_str(&launcher_profiles_data)
        .map_err(|e| format!("Failed to parse launcher profiles JSON: {}", e))?;
    launcher_profiles_json["profiles"] = serde_json::Value::Object(profiles_map);

    // Write back the updated launcher_profiles.json
    let updated_content = serde_json::to_string_pretty(&launcher_profiles_json)
        .map_err(|e| format!("Failed to serialize updated launcher profiles JSON: {}", e))?;
    std::fs::write(launcher_profiles_path, updated_content)
        .map_err(|e| format!("Failed to write updated launcher profiles: {}", e))?;
    Ok(())
}

/// Deletes a KableInstallation by ID from both the .minecraft/kable/kable_profiles.json file and .minecraft/launcher_profiles.json
#[tauri::command]
pub fn delete_installation(id: &str) -> Result<(), String> {
    let installation = get_installation(id)?;
    if installation.is_some() {
        // Make a new vector without the installation to be deleted and then use modify_all_installations
        let mut installations = get_kable_installations()?;
        installations.retain(|i| i.id != id);
        modify_all_installations(installations)?;
    }
    Ok(())
}



#[tauri::command]
pub async fn create_installation(version_id: &str) -> Result<(), String> {
    let mut installations = get_kable_installations()?;
    let default_memory = load_settings().await?.advanced.default_memory;
    let new_installation = KableInstallation {
            id: Uuid::new_v4().to_string(),
            name: format!("Kable [{}]", version_id),
            icon: String::new(), // Placeholder, can be set later
            version: VersionData {
                id: version_id.to_string(),
                loader: Loader::Vanilla, // Default to Vanilla, can be set later
                stable: true, // Default to stable, can be set later
            },
            created: chrono::Utc::now().to_rfc3339(),
            last_used: chrono::Utc::now().to_rfc3339(),
            // A vector of load_settings().advanced.default_memory and:  -XX:+UnlockExperimentalVMOptions -XX:+UseG1GC -XX:G1NewSizePercent=20 -XX:G1ReservePercent=20 -XX:MaxGCPauseMillis=50 -XX:G1HeapRegionSize=32M
            java_args: vec![
                format!("-Xmx{}M", default_memory),
                "-XX:+UnlockExperimentalVMOptions".into(),
                "-XX:+UseG1GC".into(),
                "-XX:G1NewSizePercent=20".into(),
                "-XX:G1ReservePercent=20".into(),
                "-XX:MaxGCPauseMillis=50".into(),
                "-XX:G1HeapRegionSize=32M".into(),
            ],
            dedicated_resource_pack_folder: None,
            dedicated_shaders_folder: None,
        };
    installations.push(new_installation);
    modify_all_installations(installations)?;
    Ok(())
}




/*// TODO: Move struct-specific commands to respective impl's and use tauri_struct_commands! macro to generate commands
impl KableInstallation {
    pub fn new(&self, version_id: &str) -> Self {
        KableInstallation {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("Kable [{}]", version_id),
            icon: String::new(), // Placeholder, can be set later
            version: VersionData {
                id: version_id.to_string(),
                loader: Loader::Vanilla, // Default to Vanilla, can be set later
                stable: true, // Default to stable, can be set later
            },
            created: chrono::Utc::now().to_rfc3339(),
            last_used: chrono::Utc::now().to_rfc3339(),
            java_args: Vec::new(), // TODO: Use settings to get default Java args
            dedicated_resource_pack_folder: None,
            dedicated_shaders_folder: None,
        }
    }
}


/// Macro to generate Tauri commands for a struct with methods (works even if multiple structs have the same method names)
#[macro_export]
macro_rules! tauri_struct_commands {
    (
        $struct_name:ident {
            $($fn_name:ident),* $(,)?
        }
    ) => {
        $(
            #[tauri::command(name = concat!(stringify!($struct_name), "::", stringify!($fn_name)))]
            pub fn $fn_name(command_arg: $struct_name) -> impl serde::Serialize {
                command_arg.$fn_name()
            }
        )*
    };
}
*/

