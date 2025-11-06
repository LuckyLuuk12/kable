use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use strum::{EnumIter, IntoEnumIterator};
use tauri::Emitter;
use tokio::fs as async_fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Serialize, Deserialize)]
pub enum LoaderKind {
    Vanilla,
    Fabric,
    IrisFabric,
    Forge,
    NeoForge,
    Quilt,
}

impl LoaderKind {
    pub fn cache_filename(&self) -> &'static str {
        match self {
            LoaderKind::Vanilla => "vanilla.json",
            LoaderKind::Fabric => "fabric.json",
            LoaderKind::IrisFabric => "iris-fabric.json",
            LoaderKind::Forge => "forge.json",
            LoaderKind::NeoForge => "neoforge.json",
            LoaderKind::Quilt => "quilt.json",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedManifest {
    timestamp: u64,
    data: serde_json::Value,
}

const CACHE_TTL_SECONDS: u64 = 24 * 60 * 60; // 24 hours

/// Get the manifests cache directory
async fn get_manifests_cache_dir() -> Result<PathBuf, String> {
    let minecraft_dir = crate::get_default_minecraft_dir()
        .map_err(|e| format!("Failed to get Minecraft dir: {}", e))?;
    let cache_dir = minecraft_dir.join("kable").join("manifests");
    crate::ensure_folder(&cache_dir).await?;
    Ok(cache_dir)
}

/// Check if cached manifest is still valid (less than 24 hours old)
async fn is_cache_valid(cache_path: &Path) -> bool {
    if !cache_path.exists() {
        return false;
    }
    
    match async_fs::read_to_string(cache_path).await {
        Ok(content) => {
            if let Ok(cached) = serde_json::from_str::<CachedManifest>(&content) {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                return now - cached.timestamp < CACHE_TTL_SECONDS;
            }
            false
        }
        Err(_) => false,
    }
}

/// Get cached manifest data if valid
async fn get_cached_manifest(cache_path: &Path) -> Option<serde_json::Value> {
    if !is_cache_valid(cache_path).await {
        return None;
    }
    
    match async_fs::read_to_string(cache_path).await {
        Ok(content) => {
            if let Ok(cached) = serde_json::from_str::<CachedManifest>(&content) {
                return Some(cached.data);
            }
            None
        }
        Err(_) => None,
    }
}

/// Save manifest data to cache
async fn save_to_cache(cache_path: &Path, data: &serde_json::Value) -> Result<(), String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let cached = CachedManifest {
        timestamp,
        data: data.clone(),
    };
    
    let json = serde_json::to_string_pretty(&cached)
        .map_err(|e| format!("Failed to serialize cache: {}", e))?;
    
    crate::write_file_atomic_async(cache_path, json.as_bytes()).await?;
    Ok(())
}

/// Fetch manifest from URL with caching support
async fn fetch_with_cache(
    url: &str,
    cache_filename: &str,
    force_refresh: bool,
) -> Result<serde_json::Value> {
    let cache_dir = get_manifests_cache_dir()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get cache dir: {}", e))?;
    let cache_path = cache_dir.join(cache_filename);
    
    // Try to use cache if not forcing refresh
    if !force_refresh {
        if let Some(cached_data) = get_cached_manifest(&cache_path).await {
            crate::logging::Logger::debug_global(
                &format!("Using cached manifest for {}", cache_filename),
                None,
            );
            return Ok(cached_data);
        }
    }
    
    // Fetch from network
    crate::logging::Logger::debug_global(
        &format!("Fetching manifest from network: {}", url),
        None,
    );
    
    let data = reqwest::get(url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch {}: {}", url, e))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON from {}: {}", url, e))?;
    
    // Save to cache
    if let Err(e) = save_to_cache(&cache_path, &data).await {
        crate::logging::Logger::warn_global(
            &format!("Failed to cache manifest {}: {}", cache_filename, e),
            None,
        );
    }
    
    Ok(data)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VersionData {
    pub version_id: String,
    pub loader: LoaderKind,
    pub display_name: String,
    pub is_stable: bool,
    pub extra: serde_json::Value,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Versions(pub Vec<VersionData>);

impl Versions {
    pub fn get_version(&self, version_id: &str) -> Option<&VersionData> {
        self.0.iter().find(|v| v.version_id == version_id)
    }
    pub fn extend<I: IntoIterator<Item = VersionData>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
    pub fn iter(&self) -> impl Iterator<Item = &VersionData> {
        self.0.iter()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl IntoIterator for Versions {
    type Item = VersionData;
    type IntoIter = std::vec::IntoIter<VersionData>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl<'a> IntoIterator for &'a Versions {
    type Item = &'a VersionData;
    type IntoIter = std::slice::Iter<'a, VersionData>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
impl<'a> IntoIterator for &'a mut Versions {
    type Item = &'a mut VersionData;
    type IntoIter = std::slice::IterMut<'a, VersionData>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}
impl std::iter::FromIterator<VersionData> for Versions {
    fn from_iter<I: IntoIterator<Item = VersionData>>(iter: I) -> Self {
        Versions(iter.into_iter().collect())
    }
}

pub trait Loader: Send + Sync {
    fn kind(&self) -> LoaderKind;
    /// If the loader needs vanilla versions, they are provided as Some(&[VersionData]), otherwise None.
    /// If force_refresh is true, ignore cache and fetch fresh data.
    fn get_versions<'a>(
        &'a self,
        vanilla_versions: Option<&'a [VersionData]>,
        force_refresh: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VersionData>>> + Send + 'a>>;
    fn download<'a>(
        &'a self,
        version_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
}

// Loader Implementations 

// Vanilla Loader
pub struct VanillaLoader;
impl Loader for VanillaLoader {
    fn kind(&self) -> LoaderKind {
        LoaderKind::Vanilla
    }
    fn get_versions<'a>(
        &'a self,
        _vanilla_versions: Option<&'a [VersionData]>,
        force_refresh: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VersionData>>> + Send + 'a>>
    {
        Box::pin(async move {
            let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
            
            // Try to fetch with cache/network
            let resp = match fetch_with_cache(url, self.kind().cache_filename(), force_refresh).await {
                Ok(data) => data,
                Err(e) => {
                    // Fallback: Try to read from local .minecraft/versions/version_manifest_v2.json
                    crate::logging::Logger::warn_global(
                        &format!("Failed to fetch vanilla manifest: {}. Trying local fallback...", e),
                        None,
                    );
                    
                    match crate::get_default_minecraft_dir() {
                        Ok(minecraft_dir) => {
                            let local_manifest = minecraft_dir.join("versions").join("version_manifest_v2.json");
                            
                            if local_manifest.exists() {
                                match async_fs::read_to_string(&local_manifest).await {
                                    Ok(content) => {
                                        match serde_json::from_str::<serde_json::Value>(&content) {
                                            Ok(data) => {
                                                crate::logging::Logger::info_global(
                                                    "Successfully loaded vanilla versions from local .minecraft/versions/version_manifest_v2.json",
                                                    None,
                                                );
                                                data
                                            }
                                            Err(parse_err) => {
                                                return Err(anyhow::anyhow!(
                                                    "Failed to parse local manifest: {}. Original error: {}",
                                                    parse_err,
                                                    e
                                                ));
                                            }
                                        }
                                    }
                                    Err(read_err) => {
                                        return Err(anyhow::anyhow!(
                                            "Failed to read local manifest: {}. Original error: {}",
                                            read_err,
                                            e
                                        ));
                                    }
                                }
                            } else {
                                return Err(anyhow::anyhow!(
                                    "Local manifest not found at {:?}. Original error: {}",
                                    local_manifest,
                                    e
                                ));
                            }
                        }
                        Err(dir_err) => {
                            return Err(anyhow::anyhow!(
                                "Failed to get Minecraft directory: {}. Original error: {}",
                                dir_err,
                                e
                            ));
                        }
                    }
                }
            };
            
            let versions_val = &resp["versions"];
            let versions: &[serde_json::Value] = match versions_val.as_array() {
                Some(arr) => arr,
                None => &[],
            };
            let out = versions
                .iter()
                .map(|v| {
                    let id_val = &v["id"];
                    let id = id_val.as_str().unwrap_or("");
                    let typ_val = &v["type"];
                    let typ = typ_val.as_str().unwrap_or("");
                    let is_stable = typ == "release";
                    VersionData {
                        version_id: id.to_string(),
                        loader: LoaderKind::Vanilla,
                        display_name: id.to_string(),
                        is_stable,
                        extra: v.clone(),
                    }
                })
                .collect();
            Ok(out)
        })
    }
    fn download<'a>(
        &'a self,
        _version_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }
}

// Fabric Loader
pub struct FabricLoader;
impl Loader for FabricLoader {
    fn kind(&self) -> LoaderKind {
        LoaderKind::Fabric
    }
    fn get_versions<'a>(
        &'a self,
        vanilla_versions: Option<&'a [VersionData]>,
        force_refresh: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VersionData>>> + Send + 'a>>
    {
        Box::pin(async move {
            let url = "https://meta.fabricmc.net/v2/versions/loader";
            let resp = fetch_with_cache(url, self.kind().cache_filename(), force_refresh).await?;
            let arr_val = &resp;
            let arr: &[serde_json::Value] = match arr_val.as_array() {
                Some(a) => a,
                None => &[],
            };
            let mut out = Vec::new();
            if let Some(vanilla) = vanilla_versions {
                for v in arr {
                    let fabricv = v.get("version").and_then(|x| x.as_str()).unwrap_or("");
                    let stable = v.get("stable").and_then(|x| x.as_bool()).unwrap_or(false);
                    
                    // Extract installer maven coordinates from the version object directly
                    let installer_maven = v
                        .get("maven")
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string());
                    
                    for mcv in vanilla {
                        // Create extra data with minecraft_version and installer_maven
                        let mut extra = v.clone();
                        if let Some(extra_obj) = extra.as_object_mut() {
                            extra_obj.insert(
                                "minecraft_version".to_string(),
                                serde_json::json!(mcv.version_id.clone()),
                            );
                            if let Some(ref maven) = installer_maven {
                                extra_obj.insert(
                                    "installer_maven".to_string(),
                                    serde_json::json!(maven),
                                );
                            }
                        }
                        
                        out.push(VersionData {
                            version_id: format!("fabric-loader-{}-{}", fabricv, mcv.version_id),
                            loader: LoaderKind::Fabric,
                            display_name: format!("Fabric {} for MC {}", fabricv, mcv.version_id),
                            is_stable: stable,
                            extra,
                        });
                    }
                }
            }
            Ok(out)
        })
    }
    fn download<'a>(
        &'a self,
        _version_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }
}

// IrisFabric Loader (uses Fabric manifest)
pub struct IrisFabricLoader;
impl Loader for IrisFabricLoader {
    fn kind(&self) -> LoaderKind {
        LoaderKind::IrisFabric
    }
    fn get_versions<'a>(
        &'a self,
        vanilla_versions: Option<&'a [VersionData]>,
        force_refresh: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VersionData>>> + Send + 'a>>
    {
        Box::pin(async move {
            let url = "https://meta.fabricmc.net/v2/versions/loader";
            let resp = fetch_with_cache(url, self.kind().cache_filename(), force_refresh).await?;
            let arr_val = &resp;
            let arr: &[serde_json::Value] = match arr_val.as_array() {
                Some(a) => a,
                None => &[],
            };
            let mut out = Vec::new();
            if let Some(vanilla) = vanilla_versions {
                for v in arr {
                    let fabricv = v.get("version").and_then(|x| x.as_str()).unwrap_or("");
                    let stable = v.get("stable").and_then(|x| x.as_bool()).unwrap_or(false);
                    
                    // Extract installer maven coordinates from the version object directly
                    let installer_maven = v
                        .get("maven")
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string());
                    
                    for mcv in vanilla {
                        // Create extra data with minecraft_version and installer_maven
                        let mut extra = v.clone();
                        if let Some(extra_obj) = extra.as_object_mut() {
                            extra_obj.insert(
                                "minecraft_version".to_string(),
                                serde_json::json!(mcv.version_id.clone()),
                            );
                            if let Some(ref maven) = installer_maven {
                                extra_obj.insert(
                                    "installer_maven".to_string(),
                                    serde_json::json!(maven),
                                );
                            }
                        }
                        
                        out.push(VersionData {
                            version_id: format!(
                                "iris-fabric-loader-{}-{}",
                                fabricv, mcv.version_id
                            ),
                            loader: LoaderKind::IrisFabric,
                            display_name: format!(
                                "Iris Fabric {} for MC {}",
                                fabricv, mcv.version_id
                            ),
                            is_stable: stable,
                            extra,
                        });
                    }
                }
            }
            Ok(out)
        })
    }
    fn download<'a>(
        &'a self,
        _version_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }
}

// Forge Loader
pub struct ForgeLoader;
impl Loader for ForgeLoader {
    fn kind(&self) -> LoaderKind {
        LoaderKind::Forge
    }
    fn get_versions<'a>(
        &'a self,
        _vanilla_versions: Option<&'a [VersionData]>,
        force_refresh: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VersionData>>> + Send + 'a>>
    {
        Box::pin(async move {
            let url =
                "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";
            let resp = fetch_with_cache(url, self.kind().cache_filename(), force_refresh).await?;
            let binding = serde_json::Map::new();
            let obj = resp.as_object().unwrap_or(&binding);
            let mut out = Vec::new();
            for (mc_version, forge_versions) in obj {
                if let Some(forge_arr) = forge_versions.as_array() {
                    for forge_version in forge_arr {
                        let forge_version_str = forge_version.as_str().unwrap_or("");
                        // Extract the part after the first dash for display purposes
                        let forge_version_display = match forge_version_str.find('-') {
                            Some(idx) if idx + 1 < forge_version_str.len() => {
                                &forge_version_str[idx + 1..]
                            }
                            _ => forge_version_str,
                        };
                        // Forge version_id format: forge-{mc_version}-{forge_version}
                        let version_id = format!("{}-forge-{}", mc_version, forge_version_display);
                        let display_name =
                            format!("Forge {} for MC {}", forge_version_display, mc_version);
                        out.push(VersionData {
                            version_id,
                            loader: LoaderKind::Forge,
                            display_name,
                            is_stable: true,
                            extra: json!({ "forge_version": forge_version_display, "minecraft_version": mc_version }),
                        });
                    }
                }
            }
            Ok(out)
        })
    }
    fn download<'a>(
        &'a self,
        _version_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }
}

// NeoForge Loader
pub struct NeoForgeLoader;
impl Loader for NeoForgeLoader {
    fn kind(&self) -> LoaderKind {
        LoaderKind::NeoForge
    }
    fn get_versions<'a>(
        &'a self,
        _vanilla_versions: Option<&'a [VersionData]>,
        force_refresh: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VersionData>>> + Send + 'a>>
    {
        Box::pin(async move {
            let url = "https://maven.neoforged.net/api/maven/versions/releases/net%2Fneoforged%2Fneoforge";
            let resp = fetch_with_cache(url, self.kind().cache_filename(), force_refresh).await?;
            let arr_val = &resp["versions"];
            let arr: &[serde_json::Value] = match arr_val.as_array() {
                Some(a) => a,
                None => &[],
            };
            let is_snapshot = resp["is_snapshot"].as_bool().unwrap_or(false);
            let mut out = Vec::new();
            for v in arr {
                let neoforge_ver = v.as_str().unwrap_or("");
                // NeoForge versions are in format like "21.4.14" where first part is MC version
                // Extract MC version from NeoForge version (e.g., "21.4.14" -> "1.21.4")
                let mc_version = if let Some(_first_part) = neoforge_ver.split('.').next() {
                    // NeoForge 20.x = MC 1.20.x, NeoForge 21.x = MC 1.21.x, etc.
                    format!("1.{}", neoforge_ver.split('.').take(2).collect::<Vec<_>>().join("."))
                } else {
                    neoforge_ver.to_string()
                };
                
                out.push(VersionData {
                    version_id: format!("neoforge-{}", neoforge_ver),
                    loader: LoaderKind::NeoForge,
                    display_name: format!("NeoForge {} for MC {}", neoforge_ver, mc_version),
                    is_stable: !is_snapshot,
                    extra: json!({ 
                        "neoforge_version": neoforge_ver, 
                        "minecraft_version": mc_version,
                        "is_snapshot": is_snapshot 
                    }),
                });
            }
            Ok(out)
        })
    }
    fn download<'a>(
        &'a self,
        _version_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }
}

// Quilt Loader
pub struct QuiltLoader;
impl Loader for QuiltLoader {
    fn kind(&self) -> LoaderKind {
        LoaderKind::Quilt
    }
    fn get_versions<'a>(
        &'a self,
        vanilla_versions: Option<&'a [VersionData]>,
        force_refresh: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VersionData>>> + Send + 'a>>
    {
        Box::pin(async move {
            let url = "https://meta.quiltmc.org/v3/versions/loader";
            let resp = fetch_with_cache(url, self.kind().cache_filename(), force_refresh).await?;
            let arr_val = &resp;
            let arr: &[serde_json::Value] = match arr_val.as_array() {
                Some(a) => a,
                None => &[],
            };
            let mut out = Vec::new();
            if let Some(vanilla) = vanilla_versions {
                for v in arr {
                    let quilt_ver = v["version"].as_str().unwrap_or("");
                    let stable = v["stable"].as_bool().unwrap_or(false);
                    for mcv in vanilla {
                        out.push(VersionData {
                            version_id: format!("quilt-loader-{}-{}", quilt_ver, mcv.version_id),
                            loader: LoaderKind::Quilt,
                            display_name: format!("Quilt {} for MC {}", quilt_ver, mcv.version_id),
                            is_stable: stable,
                            extra: json!({ "quilt_version": quilt_ver, "minecraft_version": mcv.version_id, "stable": stable }),
                        });
                    }
                }
            }
            Ok(out)
        })
    }
    fn download<'a>(
        &'a self,
        _version_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }
}

// Loader factory
pub fn loader_for_kind(kind: LoaderKind) -> Box<dyn Loader> {
    match kind {
        LoaderKind::Vanilla => Box::new(VanillaLoader),
        LoaderKind::Fabric => Box::new(FabricLoader),
        LoaderKind::IrisFabric => Box::new(IrisFabricLoader),
        LoaderKind::Forge => Box::new(ForgeLoader),
        LoaderKind::NeoForge => Box::new(NeoForgeLoader),
        LoaderKind::Quilt => Box::new(QuiltLoader),
    }
}

// Build all versions for all loaders (async) - emits events as each loader completes
pub async fn build_versions(force_refresh: bool) -> Versions {
    use crate::logging::Logger;
    let mut versions = Versions::default();
    
    // First, get vanilla versions (needed for some loaders)
    let vanilla_loader = VanillaLoader;
    let vanilla_versions = vanilla_loader.get_versions(None, force_refresh).await.unwrap_or_default();
    Logger::debug_global(&format!("Loaded {} vanilla versions", vanilla_versions.len()), None);
    versions.extend(vanilla_versions.clone());
    
    // Emit vanilla versions immediately
    if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
        if let Some(app_handle) = handle_guard.as_ref() {
            let _ = app_handle.emit("versions-chunk-loaded", VersionsChunk {
                loader: LoaderKind::Vanilla,
                versions: vanilla_versions.to_vec(),
                is_complete: false,
            });
        }
    }
    
    for kind in LoaderKind::iter() {
        if kind == LoaderKind::Vanilla {
            continue; // Already loaded above
        }
        
        let loader = loader_for_kind(kind);
        let vers = loader.get_versions(Some(&vanilla_versions), force_refresh).await;
        
        match vers {
            Ok(vers) => {
                Logger::debug_global(&format!("Loaded {} versions for {:?}", vers.len(), kind), None);
                versions.extend(vers.clone());
                
                // Emit event for this loader's versions
                if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
                    if let Some(app_handle) = handle_guard.as_ref() {
                        let _ = app_handle.emit("versions-chunk-loaded", VersionsChunk {
                            loader: kind,
                            versions: vers,
                            is_complete: false,
                        });
                    }
                }
            }
            Err(e) => {
                Logger::warn_global(&format!("Failed to load versions for {:?}: {}", kind, e), None);
            }
        }
    }
    
    // Emit completion event
    if let Ok(handle_guard) = crate::logging::GLOBAL_APP_HANDLE.lock() {
        if let Some(app_handle) = handle_guard.as_ref() {
            let _ = app_handle.emit("versions-loading-complete", VersionsComplete {
                total_count: versions.len(),
            });
        }
    }
    
    Logger::debug_global(&format!("Total versions loaded: {}", versions.len()), None);
    versions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionsChunk {
    pub loader: LoaderKind,
    pub versions: Vec<VersionData>,
    pub is_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionsComplete {
    pub total_count: usize,
}
