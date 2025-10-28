use crate::logging::Logger;
use crate::profiles::LauncherProfile;
use kable_macros::log_result;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fs::File;
use std::path::PathBuf;
use std::{
    fs,
    io::{Read, Write},
};
use tokio::fs as async_fs;
use tokio::task;
use toml::Value as TomlValue;
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KableInstallation {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub version_id: String,
    pub created: String,
    pub last_used: String,
    pub java_args: Vec<String>,
    // optional folders to temporarily use assets from
    pub dedicated_mods_folder: Option<String>,
    pub dedicated_resource_pack_folder: Option<String>,
    pub dedicated_shaders_folder: Option<String>,
    pub favorite: bool,
    pub total_time_played_ms: u64,
    /// User-overridable parameters for advanced use cases
    pub parameters_map: std::collections::HashMap<String, String>,
    /// Optional user-provided description or notes
    pub description: Option<String>,
    /// Number of times this installation has been launched
    pub times_launched: u32,
}

impl Default for KableInstallation {
    fn default() -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        KableInstallation {
            id: id.clone(),
            name: String::new(),
            icon: None,
            version_id: String::new(),
            created: chrono::Utc::now().to_rfc3339(),
            last_used: chrono::Utc::now().to_rfc3339(),
            java_args: vec![
                "-Xmx10G".to_string(),
                "-XX:+UnlockExperimentalVMOptions".to_string(),
                "-XX:+UseG1GC".to_string(),
                "-XX:G1NewSizePercent=20".to_string(),
                "-XX:G1ReservePercent=20".to_string(),
                "-XX:MaxGCPauseMillis=50".to_string(),
                "-XX:G1HeapRegionSize=32M".to_string(),
            ],
            dedicated_mods_folder: None,
            dedicated_resource_pack_folder: Some(format!("resourcepacks/{}", id.clone())),
            dedicated_shaders_folder: Some(format!("shaderpacks/{}", id.clone())),
            favorite: false,
            total_time_played_ms: 0,
            parameters_map: std::collections::HashMap::new(),
            description: None,
            times_launched: 0,
        }
    }
}

impl From<LauncherProfile> for KableInstallation {
    fn from(profile: LauncherProfile) -> Self {
        let installation_id = uuid::Uuid::new_v4().to_string();

        // Create a temporary installation to check for mod loader-specific folder
        let temp_installation = KableInstallation {
            id: installation_id.clone(),
            name: profile.name.clone(),
            icon: profile.icon.clone(),
            version_id: profile.last_version_id.clone(),
            created: profile
                .created
                .clone()
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            last_used: profile
                .last_used
                .clone()
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            java_args: vec![],           // Temporary, will be set below
            dedicated_mods_folder: None, // Will be determined below
            dedicated_resource_pack_folder: Some(format!("resourcepacks/{}", installation_id.clone())),
            dedicated_shaders_folder: Some(format!("shaderpacks/{}", installation_id.clone())),
            favorite: false,
            total_time_played_ms: 0,
            parameters_map: std::collections::HashMap::new(),
            description: None,
            times_launched: 0,
        };

        // Check version manifest for custom mods folder first, fallback to default
        let dedicated_mods_folder = temp_installation
            .get_mods_folder_from_version_manifest()
            .map(|path| {
                // Store the full absolute path as-is if specified in manifest
                path.to_string_lossy().to_string()
            })
            .or_else(|| Some(format!("mods/{}", installation_id))); // Fallback to default

        KableInstallation {
            id: installation_id.clone(),
            name: profile.name,
            icon: profile.icon,
            version_id: profile.last_version_id.clone(),
            created: profile
                .created
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            last_used: profile
                .last_used
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            java_args: match profile.java_args {
                Some(ref args) if !args.trim().is_empty() => {
                    args.split_whitespace().map(String::from).collect()
                }
                _ => vec![
                    "-Xmx2048M".to_string(),
                    "-XX:+UnlockExperimentalVMOptions".to_string(),
                    "-XX:+UseG1GC".to_string(),
                    "-XX:G1NewSizePercent=20".to_string(),
                    "-XX:G1ReservePercent=20".to_string(),
                    "-XX:MaxGCPauseMillis=50".to_string(),
                    "-XX:G1HeapRegionSize=32M".to_string(),
                ],
            },
            dedicated_mods_folder,
            dedicated_resource_pack_folder: Some(format!("resourcepacks/{}", installation_id.clone())),
            dedicated_shaders_folder: Some(format!("shaderpacks/{}", installation_id.clone())),
            favorite: false,
            total_time_played_ms: 0,
            parameters_map: std::collections::HashMap::new(),
            description: None,
            times_launched: 0,
        }
    }
}

#[log_result]
pub fn read_kable_profiles() -> Result<Vec<KableInstallation>, String> {
    // Synchronous version for compatibility
    let kable_dir = crate::get_minecraft_kable_dir()?;
    let path = kable_dir.join("kable_profiles.json");
    // Ensure the kable_profiles.json file exists. If missing, create it with an empty array.
    if !path.exists() {
        let default_profiles: Vec<KableInstallation> = Vec::new();
        let json = serde_json::to_string_pretty(&default_profiles)
            .map_err(|e| format!("Failed to serialize default kable profiles: {}", e))?;
        if let Some(parent) = path.parent() {
            crate::ensure_folder_sync(parent)
                .map_err(|e| format!("Failed to create kable directory: {}", e))?;
        }
        crate::write_file_atomic_sync(&path, json.as_bytes())
            .map_err(|e| format!("Failed to write kable_profiles.json: {}", e))?;
    }

    let data = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read kable_profiles.json: {}", e))?;
    serde_json::from_str::<Vec<KableInstallation>>(&data)
        .map_err(|e| format!("Failed to parse kable_profiles.json: {}", e))
}

pub async fn read_kable_profiles_async() -> Result<Vec<KableInstallation>, String> {
    let kable_dir = crate::get_minecraft_kable_dir()?;
    let path = kable_dir.join("kable_profiles.json");
    // Ensure the kable_profiles.json file exists. If missing, create it with an empty array.
    let default_profiles: Vec<KableInstallation> = Vec::new();
    let json = serde_json::to_string_pretty(&default_profiles)
        .map_err(|e| format!("Failed to serialize default kable profiles: {}", e))?;
    // Atomically create default file if missing
    if !path.exists() {
        crate::ensure_parent_dir_exists_async(&path).await?;
        crate::write_file_atomic_async(&path, json.as_bytes()).await?;
    }
    let data = async_fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read kable_profiles.json: {}", e))?;
    task::spawn_blocking(move || {
        serde_json::from_str::<Vec<KableInstallation>>(&data)
            .map_err(|e| format!("Failed to parse kable_profiles.json: {}", e))
    })
    .await
    .unwrap()
}

pub fn write_kable_profiles(profiles: &[KableInstallation]) -> Result<(), String> {
    // Synchronous version for compatibility
    let kable_dir = crate::get_minecraft_kable_dir()?;
    let path = kable_dir.join("kable_profiles.json");
    let json = serde_json::to_string_pretty(profiles)
        .map_err(|e| format!("Failed to serialize kable profiles: {}", e))?;
    // Use atomic sync writer to avoid partial writes in synchronous calls
    crate::write_file_atomic_sync(&path, json.as_bytes())
        .map_err(|e| format!("Failed to write kable_profiles.json: {}", e))
}

pub async fn write_kable_profiles_async(profiles: &[KableInstallation]) -> Result<(), String> {
    let kable_dir = crate::get_minecraft_kable_dir()?;
    let path = kable_dir.join("kable_profiles.json");
    let profiles_owned = profiles.to_vec();
    let json = task::spawn_blocking(move || {
        serde_json::to_string_pretty(&profiles_owned)
            .map_err(|e| format!("Failed to serialize kable profiles: {}", e))
    })
    .await
    .unwrap()?;
    // Ensure parent directories exist and write the profiles asynchronously
    crate::ensure_parent_dir_exists_async(&path).await?;
    crate::write_file_atomic_async(&path, json.as_bytes())
        .await
        .map_err(|e| format!("Failed to write kable_profiles.json: {}", e))
}

impl KableInstallation {
    /// Exports this KableInstallation as a bundled zip file containing a kable_export.json with the data
    /// and if applicable, the resource pack and shaders folder.
    /// Returns the path to the exported file.
    pub async fn export(&self) -> Result<String, String> {
        let self_owned = self.clone();
        Logger::debug_global(
            &format!("Starting export for installation id={}", self_owned.id),
            None,
        );
        let res = task::spawn_blocking(move || {
            let kable_dir = crate::get_minecraft_kable_dir()?;
            let path = kable_dir.join("exports");
            // Now we make a zip file inside the exports folder
            crate::ensure_folder_sync(&path)
                .map_err(|e| format!("Failed to create exports directory: {}", e))?;
            let export_path = path.join(format!("{}_export.zip", self_owned.id));
            // Create a temporary file name in the same directory and write to it,
            // then atomically rename into place to avoid partial files.
            let mut tmp_path = path.join(format!("{}.tmp", self_owned.id));
            // Ensure tmp_path is unique
            if tmp_path.exists() {
                tmp_path = path.join(format!("{}_{}.tmp", self_owned.id, uuid::Uuid::new_v4()));
            }
            let tmp_file = fs::File::create(&tmp_path)
                .map_err(|e| format!("Failed to create temp export file: {}", e))?;
            let mut zip = zip::ZipWriter::new(tmp_file);
            let options= zip::write::FullFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored)
                .unix_permissions(0o755);
            // Prepare an export copy with reset metadata and placeholders for paths
            let mut export_install = self_owned.clone();
            export_install.created = chrono::Utc::now().to_rfc3339();
            export_install.last_used = chrono::Utc::now().to_rfc3339();
            export_install.times_launched = 0;
            export_install.total_time_played_ms = 0;
            
            // Convert all paths to use placeholders to ensure portability
            // Store mods folder as relative path with placeholder
            if let Some(ref mods) = export_install.dedicated_mods_folder {
                let mods_path = PathBuf::from(mods);
                if mods_path.is_absolute() {
                    // Convert absolute paths to relative with placeholder
                    export_install.dedicated_mods_folder = Some("mods/{{INSTALLATION_ID}}".to_string());
                } else if mods.contains(&self_owned.id) {
                    // Replace existing ID with placeholder
                    export_install.dedicated_mods_folder = Some(mods.replace(&self_owned.id, "{{INSTALLATION_ID}}"));
                }
            }
            
            if let Some(ref rp) = export_install.dedicated_resource_pack_folder {
                let rp_path = PathBuf::from(rp);
                if rp_path.is_absolute() {
                    export_install.dedicated_resource_pack_folder = Some("resourcepacks/{{INSTALLATION_ID}}".to_string());
                } else if rp.contains(&self_owned.id) {
                    export_install.dedicated_resource_pack_folder = Some(rp.replace(&self_owned.id, "{{INSTALLATION_ID}}"));
                }
            }
            
            if let Some(ref shaders) = export_install.dedicated_shaders_folder {
                let shaders_path = PathBuf::from(shaders);
                if shaders_path.is_absolute() {
                    export_install.dedicated_shaders_folder = Some("shaderpacks/{{INSTALLATION_ID}}".to_string());
                } else if shaders.contains(&self_owned.id) {
                    export_install.dedicated_shaders_folder = Some(shaders.replace(&self_owned.id, "{{INSTALLATION_ID}}"));
                }
            }
            
            // Use placeholder for ID in export
            export_install.id = "{{INSTALLATION_ID}}".to_string();
            
            // Write the kable_export.json file
            zip.start_file("kable_export.json", options.clone())
                .map_err(|e| format!("Failed to write kable_export.json: {}", e))?;
            let json = serde_json::to_string_pretty(&export_install)
                .map_err(|e| format!("Failed to serialize KableInstallation: {}", e))?;
            zip.write_all(json.as_bytes())
                .map_err(|e| format!("Failed to write KableInstallation data: {}", e))?;
            // If there is a dedicated resource pack folder or file, add it to the zip
            if let Some(ref resource_pack) = self_owned.dedicated_resource_pack_folder {
                // if resource_pack is absolute, use as-is; otherwise relative to kable_dir
                let rp_path = PathBuf::from(resource_pack);
                let resource_pack_path = if rp_path.is_absolute() {
                    rp_path
                } else {
                    kable_dir.join(resource_pack)
                };
                if resource_pack_path.exists() {
                    // If it's a file (zip), copy it directly
                        if resource_pack_path.is_file() {
                        zip.start_file("resource_packs.zip", options.clone())
                            .map_err(|e| format!("Failed to write resource pack: {}", e))?;
                        let mut resource_pack_file = fs::File::open(&resource_pack_path)
                            .map_err(|e| format!("Failed to open resource pack: {}", e))?;
                        std::io::copy(&mut resource_pack_file, &mut zip)
                            .map_err(|e| format!("Failed to copy resource pack: {}", e))?;
                    } else if resource_pack_path.is_dir() {
                        // Create a temporary zip of the directory and copy it into the main zip as resource_packs.zip
                        let mut tmp = std::env::temp_dir();
                        tmp.push(format!("resource_packs_{}.zip", export_install.id));
                        {
                            let tmp_file = fs::File::create(&tmp).map_err(|e| {
                                format!("Failed to create temp resource zip: {}", e)
                            })?;
                            let mut tmp_zip = zip::ZipWriter::new(tmp_file);
                            let walk = walkdir::WalkDir::new(&resource_pack_path);
                            for entry in walk.into_iter().filter_map(|e| e.ok()) {
                                let path = entry.path();
                                if path.is_file() {
                                    let rel = path.strip_prefix(&resource_pack_path).unwrap();
                                    let name = format!("{}", rel.to_string_lossy());
                                    tmp_zip.start_file(name, options.clone()).map_err(|e| {
                                        format!("Failed to add file to tmp resource zip: {}", e)
                                    })?;
                                    let mut f = fs::File::open(path).map_err(|e| {
                                        format!("Failed to open file for tmp zip: {}", e)
                                    })?;
                                    std::io::copy(&mut f, &mut tmp_zip).map_err(|e| {
                                        format!("Failed to copy to tmp resource zip: {}", e)
                                    })?;
                                }
                            }
                            tmp_zip
                                .finish()
                                .map_err(|e| format!("Failed to finish tmp resource zip: {}", e))?;
                        }
                        // Copy tmp into main zip
                        let mut tmp_file = fs::File::open(&tmp)
                            .map_err(|e| format!("Failed to open tmp resource zip: {}", e))?;
                        zip.start_file("resource_packs.zip", options.clone())
                            .map_err(|e| format!("Failed to write resource pack: {}", e))?;
                        std::io::copy(&mut tmp_file, &mut zip)
                            .map_err(|e| format!("Failed to copy resource pack: {}", e))?;
                        // Ignore failure to remove temp file
                        let _ = fs::remove_file(&tmp);
                    }
                }
            }
            // If there is a dedicated shaders folder, add it to the zip
            if let Some(ref shaders_folder) = self_owned.dedicated_shaders_folder {
                let sf_path = PathBuf::from(shaders_folder);
                let shaders_path = if sf_path.is_absolute() {
                    sf_path
                } else {
                    kable_dir.join(shaders_folder)
                };
                if shaders_path.exists() {
                        if shaders_path.is_file() {
                        zip.start_file("shaders.zip", options.clone())
                            .map_err(|e| format!("Failed to write shaders folder: {}", e))?;
                        let mut shaders_file = fs::File::open(&shaders_path)
                            .map_err(|e| format!("Failed to open shaders folder: {}", e))?;
                        std::io::copy(&mut shaders_file, &mut zip)
                            .map_err(|e| format!("Failed to copy shaders folder: {}", e))?;
                    } else if shaders_path.is_dir() {
                        // Zip dir to tmp and copy
                        let mut tmp = std::env::temp_dir();
                        tmp.push(format!("shaders_{}.zip", export_install.id));
                        {
                            let tmp_file = fs::File::create(&tmp)
                                .map_err(|e| format!("Failed to create tmp shaders zip: {}", e))?;
                            let mut tmp_zip = zip::ZipWriter::new(tmp_file);
                            let walk = walkdir::WalkDir::new(&shaders_path);
                            for entry in walk.into_iter().filter_map(|e| e.ok()) {
                                let path = entry.path();
                                if path.is_file() {
                                    let rel = path.strip_prefix(&shaders_path).unwrap();
                                    let name = format!("{}", rel.to_string_lossy());
                                    tmp_zip.start_file(name, options.clone()).map_err(|e| {
                                        format!("Failed to add file to tmp shaders zip: {}", e)
                                    })?;
                                    let mut f = fs::File::open(path).map_err(|e| {
                                        format!("Failed to open file for tmp shaders zip: {}", e)
                                    })?;
                                    std::io::copy(&mut f, &mut tmp_zip).map_err(|e| {
                                        format!("Failed to copy to tmp shaders zip: {}", e)
                                    })?;
                                }
                            }
                            tmp_zip
                                .finish()
                                .map_err(|e| format!("Failed to finish tmp shaders zip: {}", e))?;
                        }
                        let mut tmp_file = fs::File::open(&tmp)
                            .map_err(|e| format!("Failed to open tmp shaders zip: {}", e))?;
                        zip.start_file("shaders.zip", options.clone())
                            .map_err(|e| format!("Failed to write shaders folder: {}", e))?;
                        std::io::copy(&mut tmp_file, &mut zip)
                            .map_err(|e| format!("Failed to copy shaders folder: {}", e))?;
                        let _ = fs::remove_file(&tmp);
                    }
                }
            }

            // If there is a dedicated mods folder, add it to the zip
            if let Some(ref mods_folder) = self_owned.dedicated_mods_folder {
                let mf_path = PathBuf::from(mods_folder);
                let mods_path = if mf_path.is_absolute() {
                    mf_path
                } else {
                    kable_dir.join(mods_folder)
                };
                if mods_path.exists() {
                        if mods_path.is_file() {
                        zip.start_file("mods.zip", options.clone())
                            .map_err(|e| format!("Failed to write mods: {}", e))?;
                        let mut mods_file = fs::File::open(&mods_path)
                            .map_err(|e| format!("Failed to open mods file: {}", e))?;
                        std::io::copy(&mut mods_file, &mut zip)
                            .map_err(|e| format!("Failed to copy mods file: {}", e))?;
                    } else if mods_path.is_dir() {
                        // Zip dir to tmp and copy
                        let mut tmp = std::env::temp_dir();
                        tmp.push(format!("mods_{}.zip", export_install.id));
                        {
                            let tmp_file = fs::File::create(&tmp)
                                .map_err(|e| format!("Failed to create tmp mods zip: {}", e))?;
                            let mut tmp_zip = zip::ZipWriter::new(tmp_file);
                            let walk = walkdir::WalkDir::new(&mods_path);
                            for entry in walk.into_iter().filter_map(|e| e.ok()) {
                                let path = entry.path();
                                if path.is_file() {
                                    let rel = path.strip_prefix(&mods_path).unwrap();
                                    let name = format!("{}", rel.to_string_lossy());
                                    tmp_zip.start_file(name, options.clone()).map_err(|e| {
                                        format!("Failed to add file to tmp mods zip: {}", e)
                                    })?;
                                    let mut f = fs::File::open(path).map_err(|e| {
                                        format!("Failed to open file for tmp mods zip: {}", e)
                                    })?;
                                    std::io::copy(&mut f, &mut tmp_zip).map_err(|e| {
                                        format!("Failed to copy to tmp mods zip: {}", e)
                                    })?;
                                }
                            }
                            tmp_zip
                                .finish()
                                .map_err(|e| format!("Failed to finish tmp mods zip: {}", e))?;
                        }
                        let mut tmp_file = fs::File::open(&tmp)
                            .map_err(|e| format!("Failed to open tmp mods zip: {}", e))?;
                        zip.start_file("mods.zip", options.clone())
                            .map_err(|e| format!("Failed to write mods: {}", e))?;
                        std::io::copy(&mut tmp_file, &mut zip)
                            .map_err(|e| format!("Failed to copy mods: {}", e))?;
                        let _ = fs::remove_file(&tmp);
                    }
                }
            }
            zip.finish()
                .map_err(|e| format!("Failed to finish zip file: {}", e))?;
            // Atomically move tmp into final location
            std::fs::rename(&tmp_path, &export_path)
                .map_err(|e| format!("Failed to move export into place: {}", e))?;
            Ok::<String, String>(export_path.to_string_lossy().to_string())
        })
        .await
        .map_err(|e| format!("Export task join error: {}", e))?;
        if let Ok(ref p) = res {
            Logger::debug_global(&format!("Export completed: {}", p), None);
        }
        res
    }

    /// This import does the opposite of export by extracting the KableInstallation data from a zip file and putting it in the right places.
    pub async fn import(path: &str) -> Result<KableInstallation, String> {
        let path_owned = path.to_string();
        Logger::debug_global(&format!("Starting import from {}", path_owned), None);
        let res = task::spawn_blocking(move || {
            let kable_dir = crate::get_minecraft_kable_dir()?;
            let file = fs::File::open(&path_owned)
                .map_err(|e| format!("Failed to open import file: {}", e))?;
            let mut zip = zip::ZipArchive::new(file)
                .map_err(|e| format!("Failed to read zip file: {}", e))?;
            
            // Generate a new unique ID for this import
            let new_id = uuid::Uuid::new_v4().to_string();
            
            let mut installation = KableInstallation::default();
            // Extract the kable_export.json file
            if let Ok(mut file) = zip.by_name("kable_export.json") {
                let mut json = String::new();
                file.read_to_string(&mut json)
                    .map_err(|e| format!("Failed to read kable_export.json: {}", e))?;
                
                // Replace all placeholders with the new ID
                json = json.replace("{{INSTALLATION_ID}}", &new_id);
                
                installation = serde_json::from_str(&json)
                    .map_err(|e| format!("Failed to parse kable_export.json: {}", e))?;
                
                // Ensure the ID is set to the new one (in case replacement didn't work)
                installation.id = new_id.clone();
            }
            // Extract the resource pack if it exists (embedded zip) and unpack into destination
            if let Ok(mut file) = zip.by_name("resource_packs.zip") {
                // Use the new ID for the destination path
                let resource_pack_rel = installation
                    .dedicated_resource_pack_folder
                    .as_deref()
                    .unwrap_or(&new_id)
                    .to_string();
                let rp_path = PathBuf::from(&resource_pack_rel);
                // Always use relative path from kable_dir
                let dest_dir = if rp_path.is_absolute() {
                    rp_path
                } else {
                    kable_dir.join(&resource_pack_rel)
                };
                crate::ensure_folder_sync(&dest_dir)
                    .map_err(|e| format!("Failed to create resource pack directory: {}", e))?;

                // Copy the embedded zip to a temp file then extract its entries safely into dest_dir
                let mut tmp = std::env::temp_dir();
                tmp.push(format!("resource_packs_{}.zip", new_id));
                {
                    let mut tmp_file = fs::File::create(&tmp)
                        .map_err(|e| format!("Failed to create tmp resource zip: {}", e))?;
                    std::io::copy(&mut file, &mut tmp_file)
                        .map_err(|e| format!("Failed to copy resource pack to tmp: {}", e))?;
                }
                // Open tmp as a zip and extract
                let tmp_file = fs::File::open(&tmp)
                    .map_err(|e| format!("Failed to open tmp resource zip: {}", e))?;
                let mut inner_zip = zip::ZipArchive::new(tmp_file)
                    .map_err(|e| format!("Failed to read inner resource zip: {}", e))?;
                for i in 0..inner_zip.len() {
                    let mut entry = inner_zip
                        .by_index(i)
                        .map_err(|e| format!("Failed to access inner zip entry: {}", e))?;
                    let name = entry.name();
                    // Security: skip dangerous paths
                    if name.contains("..") || name.starts_with('/') {
                        continue;
                    }
                    let out_path = dest_dir.join(name);
                    if entry.is_dir() {
                        crate::ensure_folder_sync(&out_path)
                            .map_err(|e| format!("Failed to create dir during extract: {}", e))?;
                    } else {
                        if let Some(p) = out_path.parent() {
                            crate::ensure_folder_sync(p).map_err(|e| {
                                format!("Failed to create parent dir during extract: {}", e)
                            })?;
                        }
                        let mut outfile = fs::File::create(&out_path)
                            .map_err(|e| format!("Failed to create file during extract: {}", e))?;
                        std::io::copy(&mut entry, &mut outfile)
                            .map_err(|e| format!("Failed to write extracted file: {}", e))?;
                    }
                }
                let _ = fs::remove_file(&tmp);
            }
            // Extract the shaders folder if it exists (embedded zip) and unpack into destination
            if let Ok(mut file) = zip.by_name("shaders.zip") {
                let shaders_rel = installation
                    .dedicated_shaders_folder
                    .as_deref()
                    .unwrap_or(&new_id)
                    .to_string();
                let sf_path = PathBuf::from(&shaders_rel);
                // Always use relative path from kable_dir
                let dest_dir = if sf_path.is_absolute() {
                    sf_path
                } else {
                    kable_dir.join(&shaders_rel)
                };
                crate::ensure_folder_sync(&dest_dir)
                    .map_err(|e| format!("Failed to create shaders directory: {}", e))?;

                // Copy embedded zip to tmp and extract
                let mut tmp = std::env::temp_dir();
                tmp.push(format!("shaders_{}.zip", new_id));
                {
                    let mut tmp_file = fs::File::create(&tmp)
                        .map_err(|e| format!("Failed to create tmp shaders zip: {}", e))?;
                    std::io::copy(&mut file, &mut tmp_file)
                        .map_err(|e| format!("Failed to copy shaders to tmp: {}", e))?;
                }
                let tmp_file = fs::File::open(&tmp)
                    .map_err(|e| format!("Failed to open tmp shaders zip: {}", e))?;
                let mut inner_zip = zip::ZipArchive::new(tmp_file)
                    .map_err(|e| format!("Failed to read inner shaders zip: {}", e))?;
                for i in 0..inner_zip.len() {
                    let mut entry = inner_zip
                        .by_index(i)
                        .map_err(|e| format!("Failed to access inner zip entry: {}", e))?;
                    let name = entry.name();
                    if name.contains("..") || name.starts_with('/') {
                        continue;
                    }
                    let out_path = dest_dir.join(name);
                    if entry.is_dir() {
                        crate::ensure_folder_sync(&out_path)
                            .map_err(|e| format!("Failed to create dir during extract: {}", e))?;
                    } else {
                        if let Some(p) = out_path.parent() {
                            crate::ensure_folder_sync(p).map_err(|e| {
                                format!("Failed to create parent dir during extract: {}", e)
                            })?;
                        }
                        let mut outfile = fs::File::create(&out_path)
                            .map_err(|e| format!("Failed to create file during extract: {}", e))?;
                        std::io::copy(&mut entry, &mut outfile)
                            .map_err(|e| format!("Failed to write extracted file: {}", e))?;
                    }
                }
                let _ = fs::remove_file(&tmp);
            }
            // Extract mods.zip if present (embedded zip) and unpack into destination
            if let Ok(mut file) = zip.by_name("mods.zip") {
                let mods_rel = installation
                    .dedicated_mods_folder
                    .as_deref()
                    .unwrap_or(&new_id)
                    .to_string();
                let mf_path = PathBuf::from(&mods_rel);
                // Always use relative path from kable_dir
                let dest_dir = if mf_path.is_absolute() {
                    mf_path
                } else {
                    kable_dir.join(&mods_rel)
                };
                crate::ensure_folder_sync(&dest_dir)
                    .map_err(|e| format!("Failed to create mods directory: {}", e))?;

                // Copy embedded zip to tmp and extract
                let mut tmp = std::env::temp_dir();
                tmp.push(format!("mods_{}.zip", new_id));
                {
                    let mut tmp_file = fs::File::create(&tmp)
                        .map_err(|e| format!("Failed to create tmp mods zip: {}", e))?;
                    std::io::copy(&mut file, &mut tmp_file)
                        .map_err(|e| format!("Failed to copy mods to tmp: {}", e))?;
                }
                let tmp_file = fs::File::open(&tmp)
                    .map_err(|e| format!("Failed to open tmp mods zip: {}", e))?;
                let mut inner_zip = zip::ZipArchive::new(tmp_file)
                    .map_err(|e| format!("Failed to read inner mods zip: {}", e))?;
                for i in 0..inner_zip.len() {
                    let mut entry = inner_zip
                        .by_index(i)
                        .map_err(|e| format!("Failed to access inner zip entry: {}", e))?;
                    let name = entry.name();
                    if name.contains("..") || name.starts_with('/') {
                        continue;
                    }
                    let out_path = dest_dir.join(name);
                    if entry.is_dir() {
                        crate::ensure_folder_sync(&out_path)
                            .map_err(|e| format!("Failed to create dir during extract: {}", e))?;
                    } else {
                        if let Some(p) = out_path.parent() {
                            crate::ensure_folder_sync(p).map_err(|e| {
                                format!("Failed to create parent dir during extract: {}", e)
                            })?;
                        }
                        let mut outfile = fs::File::create(&out_path)
                            .map_err(|e| format!("Failed to create file during extract: {}", e))?;
                        std::io::copy(&mut entry, &mut outfile)
                            .map_err(|e| format!("Failed to write extracted file: {}", e))?;
                    }
                }
                let _ = fs::remove_file(&tmp);
            }

            Ok::<KableInstallation, String>(installation)
        })
        .await
        .map_err(|e| format!("Import task join error: {}", e))?;
        if let Ok(ref inst) = res {
            Logger::debug_global(
                &format!("Import completed: id={} name={}", inst.id, inst.name),
                None,
            );
        }
        res
    }

    /// Try to get the mods folder from the dedicated_mods_folder field.
    fn get_dedicated_mods_folder_path(&self) -> Option<PathBuf> {
        if let Some(ref custom_mods) = self.dedicated_mods_folder {
            let custom_path = PathBuf::from(custom_mods);
            if custom_path.is_absolute() {
                Some(custom_path)
            } else {
                crate::get_minecraft_kable_dir()
                    .ok()
                    .map(|dir| dir.join(custom_mods))
            }
        } else {
            None
        }
    }

    /// Try to get the mods folder from the version manifest (versions/<version_id>/<version_id>.json)
    fn get_mods_folder_from_version_manifest(&self) -> Option<PathBuf> {
        use crate::logging::Logger;
        let mc_dir = crate::get_default_minecraft_dir().ok()?;
        let version_json = mc_dir
            .join("versions")
            .join(&self.version_id)
            .join(format!("{}.json", &self.version_id));
        if !version_json.exists() {
            return None;
        }
        let json_str = std::fs::read_to_string(&version_json).ok()?;
        let json: serde_json::Value = serde_json::from_str(&json_str).ok()?;

        // 1. Look for -Dfabric.modsFolder=... or -DmodsFolder=... in arguments.jvm array (case-insensitive, robust)
        if let Some(arguments) = json.get("arguments") {
            Logger::debug_global(
                &format!(
                    "Checking JVM arguments for mods folder in {}",
                    version_json.display()
                ),
                None,
            );
            if let Some(jvm_args) = arguments.get("jvm") {
                Logger::debug_global(
                    &format!(
                        "Checking JVM arguments for mods folder in {}",
                        version_json.display()
                    ),
                    None,
                );
                if let Some(arr) = jvm_args.as_array() {
                    let re = regex::Regex::new(r"(?i)-d(fabric\.)?modsfolder=(.+)").unwrap();
                    for arg in arr {
                        if let Some(arg_str) = arg.as_str() {
                            let arg_trimmed = arg_str.trim();
                            if let Some(caps) = re.captures(arg_trimmed) {
                                if let Some(path_str) = caps.get(2) {
                                    let path = PathBuf::from(path_str.as_str().trim());
                                    if path.is_absolute() {
                                        Logger::debug_global(&format!("Found absolute mods folder path from JVM argument: {}", path.display()), None);
                                        return Some(path);
                                    } else {
                                        // Relative to .minecraft
                                        return Some(mc_dir.join(path_str.as_str().trim()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Fallback: check for common keys (legacy)
        let possible_keys = [
            "iris_installer_mods_folder",
            "custom_mods_folder",
            "mods_folder",
        ];
        for key in &possible_keys {
            if let Some(val) = json.get(key) {
                if let Some(path_str) = val.as_str() {
                    let path = PathBuf::from(path_str);
                    if path.is_absolute() {
                        return Some(path);
                    } else {
                        // Relative to .minecraft
                        return Some(mc_dir.join(path_str));
                    }
                }
            }
        }
        None
    }

    /// Try to locate the mods directory for this installation using the same
    /// candidate order as get_mod_info. Returns an Err(String) when no
    /// directory could be found.
    fn find_mods_dir(&self) -> Result<PathBuf, String> {
        use crate::logging::Logger;
        let mods_dirs = [
            self.get_dedicated_mods_folder_path(),
            self.get_mods_folder_from_version_manifest(),
            crate::get_minecraft_kable_dir()
                .ok()
                .map(|dir| dir.join("mods").join(&self.id)), // Use installation ID
        ];
        let mut found_dir = None;
        for (i, dir_opt) in mods_dirs.iter().enumerate() {
            if let Some(dir) = dir_opt {
                Logger::debug_global(
                    &format!("Checking mods dir candidate {}: {}", i, dir.display()),
                    None,
                );
                if dir.exists() {
                    Logger::debug_global(&format!("✅ Using mods dir: {}", dir.display()), None);
                    found_dir = Some(dir.clone());
                    break;
                }
            }
        }
        match found_dir {
            Some(d) => Ok(d),
            None => Err("No mods directory found for installation".to_string()),
        }
    }

    /// Move the given mod JAR into the installation's disabled/ subfolder.
    pub fn disable_mod(&self, file_name: &str) -> Result<(), String> {
        use crate::logging::Logger;
        let mods_dir = self.find_mods_dir()?;
        let disabled_dir = mods_dir.join("disabled");
        let src = mods_dir.join(file_name);
        let dst = disabled_dir.join(file_name);
        if !src.exists() {
            // Already disabled?
            if dst.exists() {
                return Ok(());
            }
            return Err(format!("Mod file not found: {}", file_name));
        }
        crate::ensure_folder_sync(&disabled_dir)
            .map_err(|e| format!("Failed to create disabled directory: {}", e))?;
        fs::rename(&src, &dst).map_err(|e| format!("Failed to move mod to disabled: {}", e))?;
        Logger::debug_global(
            &format!("Moved {} -> {}", src.display(), dst.display()),
            None,
        );
        Ok(())
    }

    /// Move the given mod JAR out of the installation's disabled/ subfolder back into the active mods folder.
    pub fn enable_mod(&self, file_name: &str) -> Result<(), String> {
        let mods_dir = self.find_mods_dir()?;
        let disabled_dir = mods_dir.join("disabled");
        let src = disabled_dir.join(file_name);
        let dst = mods_dir.join(file_name);
        if !src.exists() {
            // Already enabled?
            if dst.exists() {
                return Ok(());
            }
            return Err(format!("Disabled mod not found: {}", file_name));
        }
        fs::rename(&src, &dst).map_err(|e| format!("Failed to move mod to enabled: {}", e))?;
        Ok(())
    }

    /// Toggle the disabled state of a mod; returns the new disabled state (true = disabled).
    pub fn toggle_mod_disabled(&self, file_name: &str) -> Result<bool, String> {
        let mods_dir = self.find_mods_dir()?;
        let src_active = mods_dir.join(file_name);
        let src_disabled = mods_dir.join("disabled").join(file_name);
        if src_active.exists() {
            self.disable_mod(file_name)?;
            return Ok(true);
        }
        if src_disabled.exists() {
            self.enable_mod(file_name)?;
            return Ok(false);
        }
        Err(format!(
            "Mod file not found in either active or disabled folders: {}",
            file_name
        ))
    }

    /// Scans the mods folder for this installation and extracts mod info from each JAR
    pub fn get_mod_info(&self) -> Result<Vec<ModJarInfo>, String> {
        use crate::logging::Logger;
        Logger::debug_global(
            &format!(
                "🔍 get_mod_info for installation: {} (version_id: {})",
                self.name, self.version_id
            ),
            None,
        );
        let mods_dirs = [
            self.get_dedicated_mods_folder_path(),
            self.get_mods_folder_from_version_manifest(),
            crate::get_minecraft_kable_dir()
                .ok()
                .map(|dir| dir.join("mods").join(&self.id)), // Use installation ID, not version ID
        ];
        let mut found_dir = None;
        for (i, dir_opt) in mods_dirs.iter().enumerate() {
            if let Some(dir) = dir_opt {
                Logger::debug_global(
                    &format!("Checking mods dir candidate {}: {}", i, dir.display()),
                    None,
                );
                if dir.exists() {
                    Logger::debug_global(&format!("✅ Using mods dir: {}", dir.display()), None);
                    found_dir = Some(dir.clone());
                    break;
                }
            }
        }
        let mods_dir = match found_dir {
            Some(d) => d,
            None => {
                Logger::debug_global("No mods directory found for installation", None);
                return Ok(vec![]);
            }
        };
        let mut result = Vec::new();
        let read_dir = std::fs::read_dir(&mods_dir);
        if let Err(e) = &read_dir {
            Logger::debug_global(&format!("Failed to read mods dir: {}", e), None);
            return Err(format!("Failed to read mods dir: {}", e));
        }
        // First, read active mods in the mods_dir
        for entry in read_dir.unwrap() {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    Logger::debug_global(&format!("Failed to read entry: {}", e), None);
                    continue;
                }
            };
            let path = entry.path();
            if path.extension().map(|e| e == "jar").unwrap_or(false) {
                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let mut mod_name = None;
                let mut mod_version = None;
                let mut loader = None;
                if let Ok(file) = File::open(&path) {
                    match ZipArchive::new(file) {
                        Ok(mut zip) => {
                            // Try Fabric/Quilt/Forge loader files in order
                            let mut found = false;
                            if let Ok(mut f) = zip.by_name("fabric.mod.json") {
                                let mut buf = String::new();
                                if f.read_to_string(&mut buf).is_ok() {
                                    if let Ok(json) = serde_json::from_str::<JsonValue>(&buf) {
                                        mod_name = json
                                            .get("name")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string());
                                        mod_version = json
                                            .get("version")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string());
                                        loader = Some("fabric".to_string());
                                        found = true;
                                    }
                                }
                            }
                            if !found {
                                if let Ok(mut f) = zip.by_name("quilt.mod.json") {
                                    let mut buf = String::new();
                                    if f.read_to_string(&mut buf).is_ok() {
                                        if let Ok(json) = serde_json::from_str::<JsonValue>(&buf) {
                                            mod_name = json
                                                .get("name")
                                                .and_then(|v| v.as_str())
                                                .map(|s| s.to_string());
                                            mod_version = json
                                                .get("version")
                                                .and_then(|v| v.as_str())
                                                .map(|s| s.to_string());
                                            loader = Some("quilt".to_string());
                                            found = true;
                                        }
                                    }
                                }
                            }
                            if !found {
                                if let Ok(mut f) = zip.by_name("META-INF/mods.toml") {
                                    let mut buf = String::new();
                                    if f.read_to_string(&mut buf).is_ok() {
                                        if let Ok(toml) = toml::from_str::<TomlValue>(&buf) {
                                            if let Some(arr) =
                                                toml.get("mods").and_then(|v| v.as_array())
                                            {
                                                if let Some(first) = arr.first() {
                                                    mod_name = first
                                                        .get("displayName")
                                                        .and_then(|v| v.as_str())
                                                        .map(|s| s.to_string());
                                                    mod_version = first
                                                        .get("version")
                                                        .and_then(|v| v.as_str())
                                                        .map(|s| s.to_string());
                                                    loader = Some("forge".to_string());
                                                    // found = true; // not needed, last fallback
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            Logger::debug_global(
                                &format!("Failed to open zip for {:?}: {}", path, e),
                                None,
                            );
                            // Skip this JAR
                        }
                    }
                } else {
                    Logger::debug_global(&format!("Failed to open mod jar: {}", file_name), None);
                }
                result.push(ModJarInfo {
                    file_name,
                    mod_name,
                    mod_version,
                    loader,
                    disabled: false,
                });
            }
        }
        // Then, check for a disabled/ subfolder and include those jars with disabled = true
        let disabled_dir = mods_dir.join("disabled");
        if disabled_dir.exists() {
            if let Ok(dis_read) = std::fs::read_dir(&disabled_dir) {
                for dentry in dis_read {
                    match dentry {
                        Ok(e) => {
                            let path = e.path();
                            if path.extension().map(|ext| ext == "jar").unwrap_or(false) {
                                let file_name = path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                let mut mod_name = None;
                                let mut mod_version = None;
                                let mut loader = None;
                                if let Ok(file) = File::open(&path) {
                                    if let Ok(mut zip) = ZipArchive::new(file) {
                                        if let Ok(mut f) = zip.by_name("fabric.mod.json") {
                                            let mut buf = String::new();
                                            if f.read_to_string(&mut buf).is_ok() {
                                                if let Ok(json) =
                                                    serde_json::from_str::<JsonValue>(&buf)
                                                {
                                                    mod_name = json
                                                        .get("name")
                                                        .and_then(|v| v.as_str())
                                                        .map(|s| s.to_string());
                                                    mod_version = json
                                                        .get("version")
                                                        .and_then(|v| v.as_str())
                                                        .map(|s| s.to_string());
                                                    loader = Some("fabric".to_string());
                                                }
                                            }
                                        }
                                        if mod_name.is_none() {
                                            if let Ok(mut f) = zip.by_name("quilt.mod.json") {
                                                let mut buf = String::new();
                                                if f.read_to_string(&mut buf).is_ok() {
                                                    if let Ok(json) =
                                                        serde_json::from_str::<JsonValue>(&buf)
                                                    {
                                                        mod_name = json
                                                            .get("name")
                                                            .and_then(|v| v.as_str())
                                                            .map(|s| s.to_string());
                                                        mod_version = json
                                                            .get("version")
                                                            .and_then(|v| v.as_str())
                                                            .map(|s| s.to_string());
                                                        loader = Some("quilt".to_string());
                                                    }
                                                }
                                            }
                                        }
                                        if mod_name.is_none() {
                                            if let Ok(mut f) = zip.by_name("META-INF/mods.toml") {
                                                let mut buf = String::new();
                                                if f.read_to_string(&mut buf).is_ok() {
                                                    if let Ok(toml) =
                                                        toml::from_str::<TomlValue>(&buf)
                                                    {
                                                        if let Some(arr) = toml
                                                            .get("mods")
                                                            .and_then(|v| v.as_array())
                                                        {
                                                            if let Some(first) = arr.first() {
                                                                mod_name = first
                                                                    .get("displayName")
                                                                    .and_then(|v| v.as_str())
                                                                    .map(|s| s.to_string());
                                                                mod_version = first
                                                                    .get("version")
                                                                    .and_then(|v| v.as_str())
                                                                    .map(|s| s.to_string());
                                                                loader = Some("forge".to_string());
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                result.push(ModJarInfo {
                                    file_name,
                                    mod_name,
                                    mod_version,
                                    loader,
                                    disabled: true,
                                });
                            }
                        }
                        Err(e) => {
                            Logger::debug_global(
                                &format!("Failed to read disabled entry: {}", e),
                                None,
                            );
                        }
                    }
                }
            }
        }
        Logger::debug_global(
            &format!("Found {} mods in {}", result.len(), mods_dir.display()),
            None,
        );
        Ok(result)
    }

    /// See if the version_id contains a known mod loader identifier, and return it.
    pub fn get_loader_type(&self) -> Option<&str> {
        match self.version_id.as_str() {
            id if id.contains("fabric-loader") => Some("fabric"),
            id if id.contains("forge") => Some("forge"),
            id if id.contains("quilt-loader") => Some("quilt"),
            id if id.contains("neoforge") => Some("neoforge"),
            _ => Some("vanilla"),
        }
    }

    pub fn duplicate(&self) -> Result<Vec<KableInstallation>, String> {
        let mut new = self.clone();
        // generate new ID
        new.id = uuid::Uuid::new_v4().to_string();
        new.name = format!("{} (copy)", self.name);
        // Reset stats
        new.created = chrono::Utc::now().to_rfc3339();
        new.last_used = chrono::Utc::now().to_rfc3339();
        new.total_time_played_ms = 0;
        new.times_launched = 0;
        // Add to profiles
        if let Ok(mut profiles) = read_kable_profiles() {
            profiles.push(new);
            let _ = write_kable_profiles(&profiles);
            Ok(profiles)
        } else {
            Err("Failed to read Kable profiles".into())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModJarInfo {
    pub file_name: String,
    pub mod_name: Option<String>,
    pub mod_version: Option<String>,
    pub loader: Option<String>,
    /// true when the JAR was found in the installation's disabled/ subfolder
    pub disabled: bool,
}
