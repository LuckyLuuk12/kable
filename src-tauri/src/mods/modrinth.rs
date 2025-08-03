use crate::{kable_profiles::KableInstallation, mods::cache::ModCache, mods::manager::*};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FilterFacets {
    pub query: Option<String>,                     // User search string
    pub categories: Option<Vec<(String, String)>>, // (operation, value)
    pub client_side: Option<(String, String)>,     // (operation, value)
    pub server_side: Option<(String, String)>,     // (operation, value)
    pub index: Option<String>,                     // sort order
    pub open_source: Option<bool>,                 // Open source flag
    pub license: Option<(String, String)>,         // (operation, value)
    pub downloads: Option<(String, u64)>,          // (operation, value)
                                                   // ...other fields that are NOT available in KableInstallation
}

impl fmt::Display for FilterFacets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut facets: Vec<Vec<String>> = Vec::new();
        if let Some(ref cats) = self.categories {
            let arr: Vec<String> = cats
                .iter()
                .map(|(op, val)| format!("categories{}{}", op, val))
                .collect();
            if !arr.is_empty() {
                facets.push(arr);
            }
        }
        if let Some((op, val)) = &self.client_side {
            facets.push(vec![format!("client_side{}{}", op, val)]);
        }
        if let Some((op, val)) = &self.server_side {
            facets.push(vec![format!("server_side{}{}", op, val)]);
        }
        if let Some((op, val)) = &self.license {
            facets.push(vec![format!("license{}{}", op, val)]);
        }
        if let Some((op, val)) = &self.downloads {
            facets.push(vec![format!("downloads{}{}", op, val)]);
        }
        // You can add more fields here as needed
        // Example: always AND mod and modpack types
        facets.push(vec![
            "project_type:mod".to_string(),
            "project_type:modpack".to_string(),
        ]);
        write!(f, "{}", serde_json::to_string(&facets).unwrap_or_default())
    }
}

/// Modrinth project info (see https://docs.modrinth.com/api/operations/getproject/)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModrinthInfo {
    pub id: String,
    pub slug: String,
    pub project_type: String,
    pub title: String,
    pub description: String,
    pub body: Option<String>,
    pub additional_categories: Option<Vec<String>>,
    pub categories: Vec<String>,
    pub client_side: Option<String>,
    pub server_side: Option<String>,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    pub donation_urls: Option<Vec<DonationUrl>>,
    pub published: Option<String>,
    pub updated: Option<String>,
    pub approved: Option<String>,
    pub followers: Option<u64>,
    pub owner: String,
    pub team: Option<String>,
    pub host: Option<String>,
    pub license: Option<ModrinthLicense>,
    pub gallery: Option<Vec<String>>,
    pub versions: Option<Vec<ModrinthVersion>>,
    pub game_versions: Option<Vec<String>>,
    pub loaders: Option<Vec<String>>,
    pub featured: Option<bool>,
    pub project_id: Option<String>,
    pub published_by: Option<String>,
    pub approved_by: Option<String>,
    pub moderation_message: Option<ModerationMessage>,
    pub moderation_message_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DonationUrl {
    pub id: String,
    pub platform: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModrinthLicense {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModerationMessage {
    pub message: String,
    pub body: Option<String>,
}

/// Modrinth mod version info
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModrinthVersion {
    pub id: String,
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub files: Vec<ModrinthFile>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
}

/// Modrinth mod file info
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModrinthFile {
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub hashes: std::collections::HashMap<String, String>,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModrinthProvider {
    pub limit: usize,
    pub category: Option<String>,
    pub loader: Option<String>,
    pub mc_version: Option<String>,
    pub cache: ModCache<Vec<ModrinthInfo>>,
    pub cache_path: PathBuf,
    pub index: Option<String>, // For sorting/filtering
}

impl Default for ModrinthProvider {
    fn default() -> Self {
        let cache_path = PathBuf::from("modrinth_cache.json");
        let cache = ModCache::load_from_disk(&cache_path).unwrap_or_else(|_| ModCache::new(3600));
        Self {
            limit: 20,
            category: None,
            loader: None,
            mc_version: None,
            cache,
            cache_path,
            index: None, // Default to no sorting
        }
    }
}

#[async_trait::async_trait]
impl ModProvider for ModrinthProvider {
    fn set_limit(&mut self, limit: usize) {
        self.limit = limit;
    }

    async fn get(&mut self, offset: usize) -> Result<Vec<ModInfoKind>, String> {
        let cache_key = format!(
            "offset:{}:index:{}",
            offset,
            self.index.as_deref().unwrap_or("")
        );
        if let Some(entry) = self.cache.get(&cache_key) {
            if !self.cache.is_stale(&cache_key) {
                return Ok(entry
                    .value
                    .clone()
                    .into_iter()
                    .map(ModInfoKind::Modrinth)
                    .collect());
            }
        }
        let mods = if self.category.is_some() || self.loader.is_some() || self.mc_version.is_some()
        {
            get_mods_filtered_with_index(
                self.category.as_deref(),
                self.loader.as_deref(),
                self.mc_version.as_deref(),
                offset,
                self.limit,
                self.index.as_deref(),
            )
            .await?
        } else {
            get_all_mods_with_index(offset, self.limit, self.index.as_deref()).await?
        };
        self.cache.insert(cache_key.clone(), mods.clone());
        let _ = self.cache.save_to_disk(&self.cache_path);
        Ok(mods.into_iter().map(ModInfoKind::Modrinth).collect())
    }

    fn filter(
        &mut self,
        installation: Option<&KableInstallation>,
        filter: Option<crate::mods::manager::ModFilter>,
    ) {
        if let Some(crate::mods::manager::ModFilter::Modrinth(facets)) = filter {
            // Example: extract loader, mc_version, category from FilterFacets if present
            if let Some(ref cats) = facets.categories {
                // Just use the first category for now
                if let Some((_, val)) = cats.first() {
                    self.category = Some(val.clone());
                }
            }
            // Loader and mc_version could be encoded in categories or other fields, adapt as needed
            // For demonstration, not extracting loader/mc_version from facets
        }
        if let Some(installation) = installation {
            // Optionally set loader/mc_version from installation
            if self.loader.is_none() {
                // Example: set loader from installation
                // self.loader = Some("fabric".to_string());
            }
            if self.mc_version.is_none() {
                self.mc_version = Some(installation.version_id.clone());
            }
        }
    }

    async fn download(
        &self,
        mod_id: &str,
        version_id: Option<&str>,
        installation: &KableInstallation,
    ) -> Result<(), String> {
        // Determine the mods directory for the installation
        use std::path::PathBuf;
        let mods_dir: PathBuf = if let Some(ref custom_mods) = installation.dedicated_mods_folder {
            let custom_path = PathBuf::from(custom_mods);
            if custom_path.is_absolute() {
                custom_path
            } else {
                let mc_dir = crate::get_minecraft_kable_dir()?;
                mc_dir.join(custom_mods)
            }
        } else {
            let mc_dir = crate::get_minecraft_kable_dir()?;
            mc_dir.join("mods").join(&installation.version_id)
        };

        // Ensure the mods directory exists
        std::fs::create_dir_all(&mods_dir)
            .map_err(|e| format!("Failed to create mods directory: {}", e))?;

        let versions = get_mod_versions(mod_id).await?;
        let version = if let Some(version_id) = version_id {
            versions.into_iter().find(|v| v.id == version_id)
        } else {
            versions.into_iter().next()
        };
        let version = version.ok_or("Mod version not found")?;
        let mut files_iter = version.files.into_iter();
        let file = files_iter
            .clone()
            .find(|f| f.primary)
            .or_else(|| files_iter.next())
            .ok_or("No mod file found")?;
        download_mod_file(&file.url, &mods_dir.join(&file.filename)).await
    }

    fn set_index(&mut self, index: Option<String>) {
        self.index = index;
    }
    fn get_index(&self) -> Option<&String> {
        self.index.as_ref()
    }
}

/// Fetch all mods from Modrinth (paginated, with optional index)
pub async fn get_all_mods_with_index(
    offset: usize,
    limit: usize,
    index: Option<&str>,
) -> Result<Vec<ModrinthInfo>, String> {
    let client = Client::new();
    let mut url = format!(
        "https://staging-api.modrinth.com/v2/search?limit={}&offset={}",
        limit, offset
    );
    if let Some(index) = index {
        if !index.is_empty() {
            url.push_str(&format!("&index={}", index));
        }
    }
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Modrinth get all mods failed: {e}"))?;
    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Modrinth get all mods parse failed: {e}"))?;
    let hits = json
        .get("hits")
        .and_then(|v| v.as_array())
        .ok_or("No hits in Modrinth response")?;
    let mods: Vec<ModrinthInfo> = hits
        .iter()
        .filter_map(|hit| serde_json::from_value(hit.clone()).ok())
        .collect();
    Ok(mods)
}

/// Fetch mods by category, loader, and/or Minecraft version (with pagination and optional index)
pub async fn get_mods_filtered_with_index(
    category: Option<&str>,
    loader: Option<&str>,
    mc_version: Option<&str>,
    offset: usize,
    limit: usize,
    index: Option<&str>,
) -> Result<Vec<ModrinthInfo>, String> {
    let client = Client::new();
    let mut url = format!(
        "https://staging-api.modrinth.com/v2/search?limit={}&offset={}",
        limit, offset
    );
    let mut facets = Vec::new();
    if let Some(category) = category {
        facets.push(format!("[\"categories:{}\"]", category));
    }
    if let Some(loader) = loader {
        facets.push(format!("[\"categories:{}\"]", loader));
    }
    if let Some(mc_version) = mc_version {
        facets.push(format!("[\"versions:{}\"]", mc_version));
    }
    if !facets.is_empty() {
        url.push_str("&facets=[");
        url.push_str(&facets.join(","));
        url.push(']');
    }
    if let Some(index) = index {
        if !index.is_empty() {
            url.push_str(&format!("&index={}", index));
        }
    }
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Modrinth get mods filtered failed: {e}"))?;
    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Modrinth get mods filtered parse failed: {e}"))?;
    let hits = json
        .get("hits")
        .and_then(|v| v.as_array())
        .ok_or("No hits in Modrinth response")?;
    let mods: Vec<ModrinthInfo> = hits
        .iter()
        .filter_map(|hit| serde_json::from_value(hit.clone()).ok())
        .collect();
    Ok(mods)
}

/// Search mods on Modrinth, optionally filtered by loader and Minecraft version
pub async fn search_mods(
    query: &str,
    loader: Option<&str>,
    mc_version: Option<&str>,
) -> Result<Vec<ModrinthInfo>, String> {
    let client = Client::new();
    let mut url = format!("https://staging-api.modrinth.com/v2/search?query={}", query);
    if let Some(loader) = loader {
        url.push_str(&format!("&facets=[[\"categories:{}\"]]", loader));
    }
    if let Some(mc_version) = mc_version {
        url.push_str(&format!("&facets=[[\"versions:{}\"]]", mc_version));
    }
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Modrinth search failed: {e}"))?;
    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Modrinth search parse failed: {e}"))?;
    let hits = json
        .get("hits")
        .and_then(|v| v.as_array())
        .ok_or("No hits in Modrinth response")?;
    let mods: Vec<ModrinthInfo> = hits
        .iter()
        .filter_map(|hit| serde_json::from_value(hit.clone()).ok())
        .collect();
    Ok(mods)
}

/// Get all versions for a given Modrinth mod ID
pub async fn get_mod_versions(mod_id: &str) -> Result<Vec<ModrinthVersion>, String> {
    let client = Client::new();
    let url = format!(
        "https://staging-api.modrinth.com/v2/project/{}/version",
        mod_id
    );
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Modrinth get versions failed: {e}"))?;
    let versions: Vec<ModrinthVersion> = resp
        .json()
        .await
        .map_err(|e| format!("Modrinth get versions parse failed: {e}"))?;
    Ok(versions)
}

/// Download a mod file from Modrinth and save to the given path
pub async fn download_mod_file(url: &str, save_path: &std::path::Path) -> Result<(), String> {
    let client = Client::new();
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Modrinth download failed: {e}"))?;
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Modrinth download bytes failed: {e}"))?;
    if let Some(parent) = save_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create mod dir: {e}"))?;
    }
    std::fs::write(save_path, &bytes).map_err(|e| format!("Failed to write mod file: {e}"))?;
    Ok(())
}
