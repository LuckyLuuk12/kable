/**
 * This file read the .minecraft/launcher_profiles.json file and extracts the active profiles (aka installations).
 * Then it converts those to KableInstallation objects if they weren't already.
 * Meaning that the KableInstallations should have been read before this from .minecraft/kable_profiles.json.
 * Furthermore, it provides some getters/setters to access and manipulate the installations.
 * for actual launching of an installation, @see `src-tauri/src/launch.rs`.
 */

use std::{collections::HashMap, fmt::Display, path::{ PathBuf}};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::settings::{load_settings};
use crate::logging::{Logger, LogLevel};

lazy_static::lazy_static! {
    pub static ref LOADER_MANIFEST_URLS: HashMap<Loader, &'static str> = {
        let mut m = HashMap::new();
        m.insert(Loader::Vanilla, "https://launchermeta.mojang.com/mc/game/version_manifest.json");
        m.insert(Loader::Fabric, "https://meta.fabricmc.net/v2/versions/loader");
        m.insert(Loader::Forge, "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json");
        m.insert(Loader::Quilt, "https://meta.quiltmc.org/v3/versions/loader");
        m.insert(Loader::NeoForge, "https://maven.neoforged.net/api/maven/versions/releases/net%2Fneoforged%2Fneoforge");
        m.insert(Loader::IrisFabric, "https://meta.fabricmc.net/v2/versions/loader"); // Iris uses Fabric manifest
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
    NeoForge,
    /// Iris has its own installer.jar but no version manifest, so we treat it as a separate loader but implement fabric manifest
    IrisFabric, 
}
impl Display for Loader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Loader::Vanilla => "Vanilla",
            Loader::Fabric => "Fabric",
            Loader::Forge => "Forge",
            Loader::Quilt => "Quilt",
            Loader::NeoForge => "NeoForge",
            Loader::IrisFabric => "Iris Fabric",
        })
    }
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
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")] // The JSON keys are in camelCase in the .minecraft/launcher_profiles.json
pub struct LauncherProfile {
    pub created: String,
    pub icon: String,
    pub java_args: Option<String>,
    pub last_used: String,
    /// Matches a version folder name in .minecraft/versions/
    pub last_version_id: String,
    pub name: String,
    // "latest-release", "latest-snapshot", ""
    #[serde(rename = "type")]
    pub profile_type: String,
}

fn get_launcher_profiles_path() -> Result<PathBuf, String> {
    let minecraft_dir = crate::get_default_minecraft_dir()?;
    let path = minecraft_dir.join("launcher_profiles.json");
    // ensure the path exists by creating it if it doesn't (just empty file)
    if !path.exists() {
        match std::fs::write(&path, "{}") {
            Ok(_) => {
                Logger::console_log(LogLevel::Info, &format!("Created empty launcher_profiles.json at {:?}", path), None);
                return Ok(path)
            }
            Err(e) => {
                Logger::console_log(LogLevel::Error, &format!("Failed to create launcher_profiles.json: {}", e), None);
                return Err(format!("Failed to create launcher_profiles.json: {}", e))
            }
        }
    }
    Ok(path)
}
fn get_kable_profiles_path() -> Result<PathBuf, String> {
    let minecraft_kable_dir = crate::get_minecraft_kable_dir()?;
    let path = minecraft_kable_dir.join("kable_profiles.json");
    // ensure the path exists by creating it if it doesn't (just empty file)
    if !path.exists() {
        match std::fs::write(&path, "[]") {
            Ok(_) => {
                Logger::console_log(LogLevel::Info, &format!("Created empty kable_profiles.json at {:?}", path), None);
                return Ok(path)
            }
            Err(e) => {
                Logger::console_log(LogLevel::Error, &format!("Failed to create kable_profiles.json: {}", e), None);
                return Err(format!("Failed to create kable_profiles.json: {}", e))
            }
        }
    } 
    Ok(path)
}

// Read the launcher_profiles.json file and load them into LauncherProfile objects
#[derive(Deserialize)]
struct LauncherProfilesFile {
    #[serde(default)]
    profiles: std::collections::HashMap<String, LauncherProfile>,
    #[serde(default)]
    #[allow(dead_code)]
    settings: Option<serde_json::Value>,
    #[serde(default)]
    #[allow(dead_code)]
    version: Option<u32>,
}

#[tauri::command]
pub fn get_launcher_profiles() -> Result<Vec<LauncherProfile>, String> {
    let profiles_path = get_launcher_profiles_path()?;
    Logger::console_log(LogLevel::Info, &format!("Reading launcher profiles from: {:?}", profiles_path), None);
    let profiles_data = match std::fs::read_to_string(&profiles_path) {
        Ok(data) => data,
        Err(e) => {
            Logger::console_log(LogLevel::Error, &format!("Failed to read launcher profiles: {}", e), None);
            return Err(format!("Failed to read launcher profiles: {}", e));
        }
    };
    match serde_json::from_str::<LauncherProfilesFile>(&profiles_data) {
        Ok(profiles_file) => Ok(profiles_file.profiles.into_values().collect()),
        Err(e) => {
            Logger::console_log(LogLevel::Error, &format!("Failed to parse launcher profiles JSON: {}", e), None);
            Err(format!("Failed to parse launcher profiles JSON: {}", e))
        }
    }
}

// Use the Loader -> manifest url map to get all VersionData objects
#[tauri::command]
pub fn get_all_versions() -> Result<Vec<VersionData>, String> {
    let mut all_versions = Vec::new();
    for (loader, url) in LOADER_MANIFEST_URLS.iter() {
        Logger::console_log(LogLevel::Info, &format!("Fetching version data for loader {:?} from {}", loader, url), None);
        let loader_versions = (|| {
            let response = match reqwest::blocking::get(*url) {
                Ok(resp) => resp,
                Err(e) => {
                    Logger::console_log(LogLevel::Error, &format!("Failed to fetch version data from {}: {}", url, e), None);
                    return Ok::<Vec<VersionData>, String>(Vec::new());
                }
            };
            let json: serde_json::Value = match response.json() {
                Ok(j) => j,
                Err(e) => {
                    Logger::console_log(LogLevel::Error, &format!("Failed to parse JSON from {}: {}", url, e), None);
                    return Ok::<Vec<VersionData>, String>(Vec::new());
                }
            };
            let versions = match loader {
                Loader::Vanilla => {
                    let versions = match json["versions"].as_array() {
                        Some(arr) => arr,
                        None => {
                            Logger::console_log(LogLevel::Error, "Expected 'versions' to be an array for Vanilla", None);
                            return Ok::<Vec<VersionData>, String>(Vec::new());
                        }
                    };
                    versions.iter().map(|v| VersionData {
                        id: v["id"].as_str().unwrap_or_default().to_string(),
                        loader: Loader::Vanilla,
                        stable: v["type"].as_str() == Some("release"),
                    }).collect()
                },
                Loader::Fabric => {
                    let versions = match json.as_array() {
                        Some(arr) => arr,
                        None => {
                            Logger::console_log(LogLevel::Error, "Expected JSON to be an array for Fabric", None);
                            return Ok::<Vec<VersionData>, String>(Vec::new());
                        }
                    };
                    versions.iter().filter_map(|v| {
                        let loader_version = v["loader"].as_str().unwrap_or_else(|| v["version"].as_str().unwrap_or(""));
                        let mc_version = v["gameVersion"].as_str().unwrap_or("");
                        if loader_version.is_empty() || mc_version.is_empty() {
                            // Silently skip if missing loader or gameVersion
                            return None;
                        }
                        Some(VersionData {
                            id: format!("fabric-loader-{}-{}", loader_version, mc_version),
                            loader: Loader::Fabric,
                            stable: v["stable"].as_bool().unwrap_or(false),
                        })
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
                    let versions = match json.as_array() {
                        Some(arr) => arr,
                        None => {
                            Logger::console_log(LogLevel::Error, "Expected JSON to be an array for Quilt", None);
                            return Ok::<Vec<VersionData>, String>(Vec::new());
                        }
                    };
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
                    let versions = match json["versions"].as_array() {
                        Some(arr) => arr,
                        None => {
                            Logger::console_log(LogLevel::Error, "Expected 'versions' to be an array for NeoForge", None);
                            return Ok::<Vec<VersionData>, String>(Vec::new());
                        }
                    };
                    versions.iter().map(|v| {
                        let version_id = v["version"].as_str().unwrap_or_default();
                        VersionData {
                            id: version_id.to_string(),
                            loader: Loader::NeoForge,
                            stable: v["stable"].as_bool().unwrap_or(false),
                        }
                    }).collect()
                },
                Loader::IrisFabric => {
                    let versions = match json.as_array() {
                        Some(arr) => arr,
                        None => {
                            Logger::console_log(LogLevel::Error, "Expected JSON to be an array for IrisFabric (using Fabric manifest)", None);
                            return Ok::<Vec<VersionData>, String>(Vec::new());
                        }
                    };
                    versions.iter().map(|v| {
                        let loader_version = v["version"].as_str().unwrap_or_default();
                        let mc_version = v["gameVersion"].as_str().unwrap_or_default();
                        VersionData {
                            id: format!("iris-fabric-loader-{}-{}", loader_version, mc_version),
                            loader: Loader::IrisFabric,
                            stable: v["stable"].as_bool().unwrap_or(false),
                        }
                    }).collect()
                }
            };
            Ok::<Vec<VersionData>, String>(versions)
        })();
        match loader_versions {
            Ok(versions) => {
                Logger::console_log(LogLevel::Debug, &format!("Found the following versions for loader {:?}: {:?}", loader, &versions[..5.min(versions.len())]), None);
                all_versions.extend(versions);
            }
            Err(e) => {
                Logger::console_log(LogLevel::Error, &format!("Error retrieving versions for loader {:?}: {}", loader, e), None);
            }
        }
    }
    Ok(all_versions)
}

/// Given a LauncherProfile, get the corresponding VersionData
#[tauri::command]
pub fn get_version_data_for_profile(profile: LauncherProfile) -> Result<VersionData, String> {
    Logger::console_log(LogLevel::Info, &format!("Getting version data for profile: {}", profile.name), None);
    if profile.last_version_id.is_empty() {
        Logger::console_log(LogLevel::Error, "Profile last_version_id is empty", None);
        return Err("Profile last_version_id is empty".to_string());
    }
    let all_versions = get_all_versions()?;
    match all_versions.iter().find(|v| v.id == profile.last_version_id).cloned() {
        Some(vd) => Ok(vd),
        None => {
            Logger::console_log(LogLevel::Error, &format!("No version data found for last_version_id: {}", profile.last_version_id), None);
            Err(format!("No version data found for last_version_id: {}", profile.last_version_id))
        }
    }
}

// Loads the KableInstallations from the .minecraft/kable/kable_profiles.json file, does not ensure that launcher profiles have been converted
#[tauri::command]
pub fn get_kable_installations() -> Result<Vec<KableInstallation>, String> {
    let kable_profiles_path = get_kable_profiles_path()?;
    Logger::console_log(LogLevel::Info, &format!("Reading Kable installations from: {:?}", kable_profiles_path), None);
    let profiles_data = match std::fs::read_to_string(&kable_profiles_path) {
        Ok(data) => data,
        Err(e) => {
            Logger::console_log(LogLevel::Error, &format!("Failed to read Kable installations: {}", e), None);
            return Err(format!("Failed to read Kable installations: {}", e));
        }
    };
    match serde_json::from_str(&profiles_data) {
        Ok(profiles) => Ok(profiles),
        Err(e) => {
            Logger::console_log(LogLevel::Error, &format!("Failed to parse Kable installations JSON: {}", e), None);
            Err(format!("Failed to parse Kable installations JSON: {}", e))
        }
    }
}

/// Takes the launcher profiles and converts them to KableInstallations by mapping relevant fields and extracting the version data
#[tauri::command]
pub fn convert_launcher_profiles_to_kable_installations() -> Result<(), String> {
    Logger::console_log(LogLevel::Info, "Converting launcher profiles to Kable installations", None);
    let mut installations = get_kable_installations()?;
    // Fetch all version data once
    let all_versions = match get_all_versions() {
        Ok(v) => v,
        Err(e) => {
            Logger::console_log(LogLevel::Error, &format!("Failed to fetch all version data: {}", e), None);
            return Err(e);
        }
    };
    let to_be_added = match get_launcher_profiles() {
        Ok(profiles) => {
            // Find latest release/snapshot from Vanilla manifest
            let vanilla_versions: Vec<&VersionData> = all_versions.iter().filter(|v| v.loader == Loader::Vanilla).collect();
            let latest_release = vanilla_versions.iter().find(|v| v.stable).map(|v| &v.id);
            let latest_snapshot = vanilla_versions.iter().find(|v| !v.stable).map(|v| &v.id);

            profiles.into_iter()
                .filter(|profile| {
                    let mut version_id = profile.last_version_id.clone();
                    // Translate 'latest-release' and 'latest-snapshot' to actual version ids
                    if version_id == "latest-release" {
                        if let Some(latest) = latest_release {
                            version_id = latest.clone();
                        }
                    } else if version_id == "latest-snapshot" {
                        if let Some(latest) = latest_snapshot {
                            version_id = latest.clone();
                        }
                    }
                    let name = if profile.name.trim().is_empty() {
                        version_id.clone()
                    } else {
                        profile.name.clone()
                    };
                    // Prevent duplicates: check both name and version id
                    !installations.iter().any(|i| i.name == name && i.version.id == version_id)
                })
                .filter_map(|profile| {
                    let mut version_id = profile.last_version_id.clone();
                    // Translate 'latest-release' and 'latest-snapshot' to actual version ids
                    if version_id == "latest-release" {
                        if let Some(latest) = latest_release {
                            version_id = latest.clone();
                        } else {
                            Logger::console_log(LogLevel::Error, &format!("Could not resolve latest-release for profile {} (skipping)", profile.name), None);
                            return None;
                        }
                    } else if version_id == "latest-snapshot" {
                        if let Some(latest) = latest_snapshot {
                            version_id = latest.clone();
                        } else {
                            Logger::console_log(LogLevel::Error, &format!("Could not resolve latest-snapshot for profile {} (skipping)", profile.name), None);
                            return None;
                        }
                    }
                    if version_id.is_empty() {
                        Logger::console_log(LogLevel::Error, &format!("Profile last_version_id is empty for {} (skipping)", profile.name), None);
                        return None;
                    }
                    // TODO: OptiFine support: currently not supported, skip
                    if version_id.contains("OptiFine") {
                        Logger::console_log(LogLevel::Error, &format!("OptiFine version detected (not supported): {} (skipping profile {})", version_id, profile.name), None);
                        return None;
                    }
                    // TODO: IrisFabric support: check formatting and manifest
                    // If you want to improve Iris detection, add logic here
                    match all_versions.iter().find(|v| v.id == version_id).cloned() {
                        Some(version_data) => {
                            let name = if profile.name.trim().is_empty() {
                                version_id.clone()
                            } else {
                                profile.name.clone()
                            };
                            // Use default Java args if none found
                            let java_args = {
                                let args_str = profile.java_args.unwrap_or_default();
                                let args_vec: Vec<String> = args_str.split_whitespace().map(String::from).collect();
                                if args_vec.is_empty() {
                                    vec![
                                        "-Xmx2048M".to_string(),
                                        "-XX:+UnlockExperimentalVMOptions".to_string(),
                                        "-XX:+UseG1GC".to_string(),
                                        "-XX:G1NewSizePercent=20".to_string(),
                                        "-XX:G1ReservePercent=20".to_string(),
                                        "-XX:MaxGCPauseMillis=50".to_string(),
                                        "-XX:G1HeapRegionSize=32M".to_string(),
                                    ]
                                } else {
                                    args_vec
                                }
                            };
                            Some(KableInstallation {
                                id: uuid::Uuid::new_v4().to_string(),
                                name,
                                icon: profile.icon,
                                version: version_data,
                                created: profile.created,
                                last_used: profile.last_used,
                                java_args,
                                dedicated_resource_pack_folder: None,
                                dedicated_shaders_folder: None,
                            })
                        },
                        None => {
                            Logger::console_log(LogLevel::Error, &format!("No version data found for last_version_id: {} (skipping profile {})", version_id, profile.name), None);
                            None
                        }
                    }
                })
                .collect::<Vec<_>>()
        }
        Err(e) => {
            Logger::console_log(LogLevel::Error, &format!("Failed to get launcher profiles: {}", e), None);
            return Err(e);
        }
    };
    let kable_profiles_path = get_kable_profiles_path()?;
    installations.extend(to_be_added);
    let json_content = match serde_json::to_string_pretty(&installations) {
        Ok(content) => content,
        Err(e) => {
            Logger::console_log(LogLevel::Error, &format!("Failed to serialize Kable installations: {}", e), None);
            return Err(format!("Failed to serialize Kable installations: {}", e));
        }
    };
    match std::fs::write(&kable_profiles_path, &json_content) {
        Ok(_) => {
            Logger::console_log(LogLevel::Info, "Successfully converted and saved Kable installations", None);
            Ok(())
        },
        Err(e) => {
            Logger::console_log(LogLevel::Error, &format!("Failed to write Kable installations: {}", e), None);
            Err(format!("Failed to write Kable installations: {}", e))
        }
    }
}

#[tauri::command]
pub fn convert_kable_installations_to_launcher_profiles() -> Result<(), String> {
    let installations = get_kable_installations()?;
    let launcher_profiles: Vec<LauncherProfile> = installations.into_iter()
        .map(|installation| LauncherProfile {
            created: installation.created,
            icon: installation.icon,
            java_args: Some(installation.java_args.join(" "))    ,
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

/// Get ALL installations, both kable-added ones as old-official launcher profile ones, as long as they have a version directory
#[tauri::command]
pub fn get_installations() -> Result<Vec<KableInstallation>, String> {
    Logger::console_log(LogLevel::Info, "Getting all installations (Kable + launcher)", None);
    convert_launcher_profiles_to_kable_installations()?;
    let installations = get_kable_installations()?;
    let filtered: Vec<KableInstallation> = installations.clone().into_iter()
        .filter(|i| version_dir_exists(&i.version.id))
        .collect();
    let removed_installations = installations.into_iter()
        .filter(|i| !version_dir_exists(&i.version.id))
        .collect::<Vec<_>>();
    Logger::console_log(LogLevel::Info, &format!("Found {} installations with valid version directories", filtered.len()), None);
    Logger::console_log(LogLevel::Debug, &format!("Removed the following installations without valid version directories: {:?}", removed_installations), None);
    Ok(filtered)
}

/// Gets an individual KableInstallation by ID
#[tauri::command]
pub fn get_installation(id: &str) -> Result<Option<KableInstallation>, String> {
    Logger::console_log(LogLevel::Info, &format!("Getting installation by id: {}", id), None);
    let installations = get_kable_installations()?;
    let found = installations.into_iter().find(|i| i.id == id);
    if found.is_some() {
        Logger::console_log(LogLevel::Info, &format!("Found installation for id: {}", id), None);
    } else {
        Logger::console_log(LogLevel::Warning, &format!("No installation found for id: {}", id), None);
    }
    Ok(found)
}

/// Given an ID and new installation data, modify the existing KableInstallation (also updates the file)
#[tauri::command]
pub fn modify_kable_installation(id: &str, new_installation: KableInstallation) -> Result<(), String> {
    Logger::console_log(LogLevel::Info, &format!("Modifying Kable installation with id: {}", id), None);
    let mut installations = get_kable_installations()?;
    let index = installations.iter().position(|i| i.id == id);
    if let Some(index) = index {
        installations[index] = new_installation;
        let kable_profiles_path = get_kable_profiles_path()?;
        let json_content = match serde_json::to_string_pretty(&installations) {
            Ok(content) => content,
            Err(e) => {
                Logger::console_log(LogLevel::Error, &format!("Failed to serialize Kable installations: {}", e), None);
                return Err(format!("Failed to serialize Kable installations: {}", e));
            }
        };
        match std::fs::write(&kable_profiles_path, &json_content) {
            Ok(_) => {
                Logger::console_log(LogLevel::Info, &format!("Successfully modified installation with id: {}", id), None);
                Ok(())
            },
            Err(e) => {
                Logger::console_log(LogLevel::Error, &format!("Failed to write Kable installations: {}", e), None);
                Err(format!("Failed to write Kable installations: {}", e))
            }
        }
    } else {
        Logger::console_log(LogLevel::Error, &format!("No Kable installation found with id: {}", id), None);
        Err(format!("No Kable installation found with id: {}", id))
    }
}

#[tauri::command]
pub fn get_last_played_installation() -> Result<KableInstallation, String> {
    Logger::console_log(LogLevel::Info, "Getting last played installation", None);
    let installations = get_installations()?;
    let result = installations.into_iter()
        .max_by_key(|i| i.last_used.clone());
    match result {
        Some(inst) => {
            Logger::console_log(LogLevel::Info, &format!("Last played installation: {}", inst.name), None);
            Ok(inst)
        },
        None => {
            Logger::console_log(LogLevel::Error, "No installations found", None);
            Err("No installations found".to_string())
        }
    }
}

/// Gets and updates the last played installation to the current time
#[tauri::command]
pub fn modify_last_played_installation() -> Result<(), String> {
    Logger::console_log(LogLevel::Info, "Modifying last played installation timestamp", None);
    let mut installation = get_last_played_installation()?;
    installation.last_used = chrono::Utc::now().to_rfc3339();
    modify_kable_installation(&installation.id, installation.clone())?;
    Logger::console_log(LogLevel::Info, &format!("Updated last played for installation: {}", installation.name), None);
    Ok(())
}

#[tauri::command]
pub fn modify_all_installations(new_installations: Vec<KableInstallation>) -> Result<(), String> {
    Logger::console_log(LogLevel::Info, "Modifying all installations", None);
    let kable_profiles_path = get_kable_profiles_path()?;
    let json_content = match serde_json::to_string_pretty(&new_installations) {
        Ok(content) => content,
        Err(e) => {
            Logger::console_log(LogLevel::Error, &format!("Failed to serialize Kable installations: {}", e), None);
            return Err(format!("Failed to serialize Kable installations: {}", e));
        }
    };
    match std::fs::write(&kable_profiles_path, &json_content) {
        Ok(_) => Logger::console_log(LogLevel::Info, "Successfully wrote Kable installations", None),
        Err(e) => {
            Logger::console_log(LogLevel::Error, &format!("Failed to write Kable installations: {}", e), None);
            return Err(format!("Failed to write Kable installations: {}", e));
        }
    }

    let launcher_profiles = get_launcher_profiles()?;
    use serde_json::Map;
    let mut profiles_map = Map::new();
    for installation in &new_installations {
        if let Some(profile) = launcher_profiles.iter().find(|p| p.name == installation.name && p.created == installation.created) {
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

    let launcher_profiles_path = get_launcher_profiles_path()?;
    let launcher_profiles_data = match std::fs::read_to_string(&launcher_profiles_path) {
        Ok(data) => data,
        Err(e) => {
            Logger::console_log(LogLevel::Error, &format!("Failed to read launcher profiles: {}", e), None);
            return Err(format!("Failed to read launcher profiles: {}", e));
        }
    };
    let mut launcher_profiles_json: serde_json::Value = match serde_json::from_str(&launcher_profiles_data) {
        Ok(json) => json,
        Err(e) => {
            Logger::console_log(LogLevel::Error, &format!("Failed to parse launcher profiles JSON: {}", e), None);
            return Err(format!("Failed to parse launcher profiles JSON: {}", e));
        }
    };
    launcher_profiles_json["profiles"] = serde_json::Value::Object(profiles_map);

    let updated_content = match serde_json::to_string_pretty(&launcher_profiles_json) {
        Ok(content) => content,
        Err(e) => {
            Logger::console_log(LogLevel::Error, &format!("Failed to serialize updated launcher profiles JSON: {}", e), None);
            return Err(format!("Failed to serialize updated launcher profiles JSON: {}", e));
        }
    };
    match std::fs::write(&launcher_profiles_path, &updated_content) {
        Ok(_) => {
            Logger::console_log(LogLevel::Info, "Successfully updated launcher profiles", None);
            Ok(())
        },
        Err(e) => {
            Logger::console_log(LogLevel::Error, &format!("Failed to write updated launcher profiles: {}", e), None);
            Err(format!("Failed to write updated launcher profiles: {}", e))
        }
    }
}

/// Deletes a KableInstallation by ID from both the .minecraft/kable/kable_profiles.json file and .minecraft/launcher_profiles.json
#[tauri::command]
pub fn delete_installation(id: &str) -> Result<(), String> {
    Logger::console_log(LogLevel::Info, &format!("Deleting installation with id: {}", id), None);
    let installation = get_installation(id)?;
    if installation.is_some() {
        let mut installations = get_kable_installations()?;
        installations.retain(|i| i.id != id);
        modify_all_installations(installations)?;
        Logger::console_log(LogLevel::Info, &format!("Successfully deleted installation with id: {}", id), None);
    } else {
        Logger::console_log(LogLevel::Warning, &format!("No installation found to delete with id: {}", id), None);
    }
    Ok(())
}

/// Creates a new KableInstallation with the given version_id, using default settings for other fields (also writes to file)
#[tauri::command]
pub async fn create_installation(version_id: &str) -> Result<KableInstallation, String> {
    Logger::console_log(LogLevel::Info, &format!("Creating new installation for version_id: {}", version_id), None);
    let mut installations = get_kable_installations()?;
    let default_memory = load_settings().await?.advanced.default_memory;
    let new_installation = KableInstallation {
        id: Uuid::new_v4().to_string(),
        name: format!("Kable [{}]", version_id),
        icon: String::new(),
        version: VersionData {
            id: version_id.to_string(),
            loader: Loader::Vanilla,
            stable: true,
        },
        created: chrono::Utc::now().to_rfc3339(),
        last_used: chrono::Utc::now().to_rfc3339(),
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
    installations.push(new_installation.clone());
    modify_all_installations(installations)?;
    Logger::console_log(LogLevel::Info, &format!("Successfully created installation: {}", new_installation.name), None);
    Ok(new_installation)
}

// TODO: for now we assumed that the versions already exist in the .minecraft/versions folder but with code WIP below we should
// TODO: ensure that the right files are downloaded and created for the version such that launcher.rs can use those for launching 
// !NOTE: launching should not use these functions but the installation creation/modification process should ensure that
// !NOTE: the mod folder per-installation exists 

/// Checks if the version directory exists
fn version_dir_exists(version_id: &str) -> bool {
    let minecraft_dir = crate::get_default_minecraft_dir().unwrap();
    minecraft_dir.join("versions").join(version_id).exists()
}
/*
/// Ensures the mods folder exists for the installation
fn ensure_mods_folder(installation_id: &str) -> Result<PathBuf, String> {
    let minecraft_dir = crate::get_default_minecraft_dir().unwrap();
    let mods_dir = minecraft_dir.join("kable").join("mods").join(installation_id);
    if !mods_dir.exists() {
        fs::create_dir_all(&mods_dir).map_err(|e| format!("Failed to create mods folder: {}", e))?;
    }
    Ok(mods_dir)
}
/// Downloads a file from a URL to a given path
async fn download_file(url: &str, dest: &Path) -> Result<(), String> {
    let response = get(url).await
        .map_err(|e| format!("Failed to download file from {}: {}", url, e))?;
    let bytes = response.bytes().await.map_err(|e| format!("Failed to read bytes: {}", e))?;
    let mut file = fs::File::create(dest).map_err(|e| format!("Failed to create file: {}", e))?;
    file.write_all(&bytes).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}
/// Writes the version JSON to the version directory
fn write_version_json(minecraft_dir: &Path, version_id: &str, json: &Value) -> Result<(), String> {
    let version_dir = minecraft_dir.join("versions").join(version_id);
    if !version_dir.exists() {
        fs::create_dir_all(&version_dir).map_err(|e| format!("Failed to create version dir: {}", e))?;
    }
    let json_path = version_dir.join(format!("{}.json", version_id));
    let mut file = fs::File::create(&json_path).map_err(|e| format!("Failed to create version json: {}", e))?;
    let content = serde_json::to_string_pretty(json).map_err(|e| format!("Failed to serialize json: {}", e))?;
    file.write_all(content.as_bytes()).map_err(|e| format!("Failed to write version json: {}", e))?;
    Ok(())
}
/// Main helper to ensure version folder, jar, and json exist for a new installation
async fn ensure_version_files(
    minecraft_dir: &Path,
    version_id: &str,
    client_jar_url: Option<&str>,
    version_json: &Value,
) -> Result<(), String> {
    let version_dir = minecraft_dir.join("versions").join(version_id);
    if !version_dir.exists() {
        fs::create_dir_all(&version_dir).map_err(|e| format!("Failed to create version dir: {}", e))?;
    }
    // Download jar if URL is provided and jar doesn't exist
    let jar_path = version_dir.join(format!("{}.jar", version_id));
    if let Some(url) = client_jar_url {
        if !jar_path.exists() {
            download_file(url, &jar_path).await?;
        }
    }
    // Write JSON
    write_version_json(minecraft_dir, version_id, version_json)?;
    Ok(())
}
//  Uses above helpers to check in the versions folder and create/download the necessary files for a version
// pub async fn ensure_version_files(
//     version_id: &str,
// ) -> Result<(), String> {
//     let minecraft_dir = crate::get_default_minecraft_dir()?;

// }


// TODO: Move struct-specific commands to respective impl's and use tauri_struct_commands! macro to generate commands
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

