// use crate::constants::{CONFIG_DIR, KABLE_PROFILES_FILE, VERSIONS_DIR};
// use crate::logging::Logger;
// use api_types::profiles::KableProfile;
// use kable_macros::log_result;
// use once_cell::sync::Lazy;
// use std::collections::HashMap;
// use std::fs;
// use std::path::PathBuf;
// use std::sync::Mutex;
// use tokio::fs as async_fs;
// use tokio::task;

pub mod advanced;
pub mod create;
pub mod kable_profile;
pub mod management;

// #[derive(Clone)]
// struct ProfileCache {
//     profiles: Vec<KableProfile>,
//     last_modified: Option<std::time::SystemTime>,
// }

// static PROFILE_CACHE: Lazy<Mutex<Option<ProfileCache>>> = Lazy::new(|| Mutex::new(None));
// static VERSION_MANIFEST_CACHE: Lazy<Mutex<HashMap<String, Option<PathBuf>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// #[log_result]
// pub async fn read_kable_profiles() -> Result<Vec<KableProfile>, String> {
//     let kable_dir = crate::get_minecraft_kable_dir()?;
//     let path = kable_dir.join(PROFILES_FILE);

//     if let Ok(cache) = PROFILE_CACHE.lock() {
//         if let Some(cached) = cache.as_ref() {
//             if let Ok(metadata) = std::fs::metadata(&path) {
//                 if let Ok(modified) = metadata.modified() {
//                     if Some(modified) == cached.last_modified {
//                         Logger::debug_global("📦 Using cached profiles", None);
//                         return Ok(cached.profiles.clone());
//                     }
//                 }
//             }
//         }
//     }

//     Logger::debug_global("💾 Reading profiles from disk", None);
//     if !path.exists() {
//         crate::ensure_parent_dir_exists_async(&path).await?;
//         crate::write_file_atomic_async(&path, b"[]").await?;
//     }

//     let data = async_fs::read_to_string(&path).await.map_err(|e| format!("Failed to read {}: {}", PROFILES_FILE, e))?;
//     let mut installations = task::spawn_blocking(move || {
//         serde_json::from_str::<Vec<KableProfile>>(&data).map_err(|e| format!("Failed to parse {}: {}", PROFILES_FILE, e))
//     })
//     .await
//     .unwrap()?;

//     let mut needs_update = false;
//     for installation in installations.iter_mut() {
//         if installation.dedicated_config_folder.is_none() {
//             installation.dedicated_config_folder = Some(format!("{}/{}", CONFIG_DIR, installation.id));
//             needs_update = true;
//         }
//     }
//     installations.sort_by(|a, b| b.last_used.cmp(&a.last_used));

//     if needs_update {
//         write_kable_profiles(&installations).await?;
//     }

//     if let Ok(mut cache) = PROFILE_CACHE.lock() {
//         if let Ok(metadata) = std::fs::metadata(&path) {
//             if let Ok(modified) = metadata.modified() {
//                 *cache = Some(ProfileCache { profiles: installations.clone(), last_modified: Some(modified) });
//             }
//         }
//     }

//     Ok(installations)
// }

// pub async fn write_kable_profiles(profiles: &[KableProfile]) -> Result<(), String> {
//     let kable_dir = crate::get_minecraft_kable_dir()?;
//     let path = kable_dir.join(PROFILES_FILE);
//     let json = serde_json::to_string_pretty(profiles).map_err(|e| format!("Failed to serialize kable profiles: {}", e))?;
//     crate::write_file_atomic_async(&path, json.as_bytes())
//         .await
//         .map_err(|e| format!("Failed to write {}: {}", PROFILES_FILE, e))
// }

// pub async fn get_installation(id: &str) -> Result<Option<KableProfile>, String> {
//     let profiles = read_kable_profiles().await?;
//     Ok(profiles.into_iter().find(|i| i.id == id))
// }

// pub async fn get_version(version_id: String) -> Option<api_types::profiles::VersionData> {
//     let kable_dir = match crate::get_minecraft_kable_dir() {
//         Ok(d) => d,
//         Err(_) => return None,
//     };
//     let path = kable_dir.join(VERSIONS_DIR).join(format!("{}.json", version_id));
//     if !path.exists() {
//         return None;
//     }
//     let data = fs::read_to_string(path).ok()?;
//     serde_json::from_str(&data).ok()
// }
