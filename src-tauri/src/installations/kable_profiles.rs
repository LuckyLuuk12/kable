use crate::profiles::LauncherProfile;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
};
use tokio::fs as async_fs;
use tokio::task;

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
        KableInstallation {
            id: uuid::Uuid::new_v4().to_string(),
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
            dedicated_resource_pack_folder: None,
            dedicated_shaders_folder: None,
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
        KableInstallation {
            id: uuid::Uuid::new_v4().to_string(),
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
            dedicated_mods_folder: Some(profile.last_version_id),
            dedicated_resource_pack_folder: None,
            dedicated_shaders_folder: None,
            favorite: false,
            total_time_played_ms: 0,
            parameters_map: std::collections::HashMap::new(),
            description: None,
            times_launched: 0,
        }
    }
}

pub fn read_kable_profiles() -> Result<Vec<KableInstallation>, String> {
    // Synchronous version for compatibility
    let kable_dir = crate::get_minecraft_kable_dir()?;
    let path = kable_dir.join("kable_profiles.json");
    let data = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read kable_profiles.json: {}", e))?;
    serde_json::from_str::<Vec<KableInstallation>>(&data)
        .map_err(|e| format!("Failed to parse kable_profiles.json: {}", e))
}

pub async fn read_kable_profiles_async() -> Result<Vec<KableInstallation>, String> {
    let kable_dir = crate::get_minecraft_kable_dir()?;
    let path = kable_dir.join("kable_profiles.json");
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
    fs::write(&path, json).map_err(|e| format!("Failed to write kable_profiles.json: {}", e))
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
    async_fs::write(&path, json)
        .await
        .map_err(|e| format!("Failed to write kable_profiles.json: {}", e))
}

impl KableInstallation {
    /// Exports this KableInstallation as a bundled zip file containing a kable_export.json with the data
    /// and if applicable, the resource pack and shaders folder.
    /// Returns the path to the exported file.
    pub fn export(&self) -> Result<String, String> {
        let kable_dir = crate::get_minecraft_kable_dir()?;
        let path = kable_dir.join("exports");
        // Now we make a zip file inside the exports folder
        fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create exports directory: {}", e))?;
        let export_path = path.join(format!("{}_export.zip", self.id));
        let file = fs::File::create(&export_path)
            .map_err(|e| format!("Failed to create export file: {}", e))?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o755);
        // Write the kable_export.json file
        zip.start_file("kable_export.json", options)
            .map_err(|e| format!("Failed to write kable_export.json: {}", e))?;
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize KableInstallation: {}", e))?;
        zip.write_all(json.as_bytes())
            .map_err(|e| format!("Failed to write KableInstallation data: {}", e))?;
        // If there is a dedicated resource pack folder, add it to the zip
        if let Some(ref resource_pack) = self.dedicated_resource_pack_folder {
            let resource_pack_path = kable_dir.join(resource_pack);
            if resource_pack_path.exists() {
                zip.start_file("resource_packs.zip", options)
                    .map_err(|e| format!("Failed to write resource pack: {}", e))?;
                let mut resource_pack_file = fs::File::open(&resource_pack_path)
                    .map_err(|e| format!("Failed to open resource pack: {}", e))?;
                std::io::copy(&mut resource_pack_file, &mut zip)
                    .map_err(|e| format!("Failed to copy resource pack: {}", e))?;
            }
        }
        // If there is a dedicated shaders folder, add it to the zip
        if let Some(ref shaders_folder) = self.dedicated_shaders_folder {
            let shaders_path = kable_dir.join(shaders_folder);
            if shaders_path.exists() {
                zip.start_file("shaders.zip", options)
                    .map_err(|e| format!("Failed to write shaders folder: {}", e))?;
                let mut shaders_file = fs::File::open(&shaders_path)
                    .map_err(|e| format!("Failed to open shaders folder: {}", e))?;
                std::io::copy(&mut shaders_file, &mut zip)
                    .map_err(|e| format!("Failed to copy shaders folder: {}", e))?;
            }
        }
        zip.finish()
            .map_err(|e| format!("Failed to finish zip file: {}", e))?;
        Ok(export_path.to_string_lossy().to_string())
    }

    /// This import does the opposite of export by extracting the KableInstallation data from a zip file and putting it in the right places.
    pub fn import(path: &str) -> Result<KableInstallation, String> {
        let kable_dir = crate::get_minecraft_kable_dir()?;
        let file =
            fs::File::open(path).map_err(|e| format!("Failed to open import file: {}", e))?;
        let mut zip =
            zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip file: {}", e))?;
        let mut installation = KableInstallation::default();
        // Extract the kable_export.json file
        if let Ok(mut file) = zip.by_name("kable_export.json") {
            let mut json = String::new();
            file.read_to_string(&mut json)
                .map_err(|e| format!("Failed to read kable_export.json: {}", e))?;
            installation = serde_json::from_str(&json)
                .map_err(|e| format!("Failed to parse kable_export.json: {}", e))?;
        }
        // Extract the resource pack if it exists
        if let Ok(mut file) = zip.by_name("resource_packs.zip") {
            let resource_pack_rel = installation
                .dedicated_resource_pack_folder
                .as_deref()
                .unwrap_or(&installation.id);
            let resource_pack_path = kable_dir.join("resource_packs").join(resource_pack_rel);
            fs::create_dir_all(&resource_pack_path)
                .map_err(|e| format!("Failed to create resource pack directory: {}", e))?;
            let mut resource_pack_file =
                fs::File::create(resource_pack_path.join("resource_packs.zip"))
                    .map_err(|e| format!("Failed to create resource pack file: {}", e))?;
            std::io::copy(&mut file, &mut resource_pack_file)
                .map_err(|e| format!("Failed to copy resource pack: {}", e))?;
        }
        // Extract the shaders folder if it exists
        if let Ok(mut file) = zip.by_name("shaders.zip") {
            let shaders_rel = installation
                .dedicated_shaders_folder
                .as_deref()
                .unwrap_or(&installation.id);
            let shaders_path = kable_dir.join("shaders").join(shaders_rel);
            fs::create_dir_all(&shaders_path)
                .map_err(|e| format!("Failed to create shaders directory: {}", e))?;
            let mut shaders_file = fs::File::create(shaders_path.join("shaders.zip"))
                .map_err(|e| format!("Failed to create shaders file: {}", e))?;
            std::io::copy(&mut file, &mut shaders_file)
                .map_err(|e| format!("Failed to copy shaders folder: {}", e))?;
        }
        Ok(installation)
    }
}
