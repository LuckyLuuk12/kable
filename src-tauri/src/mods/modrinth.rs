use crate::{kable_profiles::KableInstallation, mods::cache::ModCache, mods::manager::*};
use kable_macros::log_result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FilterFacets {
    pub query: Option<String>,                     // User search string
    pub categories: Option<Vec<(String, String)>>, // (operation, value) - OR'd together
    pub client_side: Option<(String, String)>,     // (operation, value)
    pub server_side: Option<(String, String)>,     // (operation, value)
    pub open_source: Option<bool>,                 // Open source flag
    pub license: Option<(String, String)>,         // (operation, value)
    pub downloads: Option<(String, u64)>,          // (operation, value)
                                                   // Note: loader and game_versions come from installation, not user filters
}

impl FilterFacets {
    /// Build facets array for Modrinth API from FilterFacets + installation info
    /// Returns Vec<Vec<String>> where:
    /// - Each inner Vec is OR'd together
    /// - Outer Vec items are AND'd together
    ///   Each user filter (category, environment, etc.) is AND'd as separate array
    pub fn to_modrinth_facets(
        &self,
        loader: Option<&str>,
        mc_version: Option<&str>,
    ) -> Vec<Vec<String>> {
        let mut facets: Vec<Vec<String>> = Vec::new();

        // Categories - each filter is AND'd (separate array per filter)
        if let Some(ref cats) = self.categories {
            for (op, val) in cats {
                facets.push(vec![format!("categories{}{}", op, val)]);
            }
        }

        // Loader from installation - AND (separate array)
        if let Some(loader) = loader {
            facets.push(vec![format!("categories:{}", loader)]);
        }

        // MC Version from installation - AND (separate array)
        if let Some(mc_version) = mc_version {
            facets.push(vec![format!("versions:{}", mc_version)]);
        }

        // Client side - AND (separate array)
        if let Some((op, val)) = &self.client_side {
            facets.push(vec![format!("client_side{}{}", op, val)]);
        }

        // Server side - AND (separate array)
        if let Some((op, val)) = &self.server_side {
            facets.push(vec![format!("server_side{}{}", op, val)]);
        }

        // License - AND (separate array)
        if let Some((op, val)) = &self.license {
            facets.push(vec![format!("license{}{}", op, val)]);
        }

        // Downloads - AND (separate array)
        if let Some((op, val)) = &self.downloads {
            facets.push(vec![format!("downloads{}{}", op, val)]);
        }

        // Project type - always OR mod and modpack (single array = OR)
        facets.push(vec![
            "project_type:mod".to_string(),
            "project_type:modpack".to_string(),
        ]);

        facets
    }
}

/// Modrinth project info (see https://docs.modrinth.com/api/operations/getproject/)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModrinthInfo {
    #[serde(rename = "project_id")]
    pub project_id: String,
    pub project_type: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub categories: Vec<String>,
    #[serde(default)]
    pub display_categories: Vec<String>,
    #[serde(default)]
    pub versions: Vec<String>,
    pub downloads: u64,
    #[serde(rename = "follows")]
    pub followers: Option<u64>,
    #[serde(rename = "icon_url")]
    pub icon_url: Option<String>,
    #[serde(rename = "date_created")]
    pub date_created: Option<String>,
    #[serde(rename = "date_modified")]
    pub date_modified: Option<String>,
    #[serde(rename = "latest_version")]
    pub latest_version: Option<String>,
    pub license: Option<String>,
    pub client_side: Option<String>,
    pub server_side: Option<String>,
    pub gallery: Option<Vec<String>>,
    #[serde(rename = "featured_gallery")]
    pub featured_gallery: Option<String>,
    pub color: Option<u32>,
    // The following fields are not present in the search API, but may be present in the project details API
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issues_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wiki_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discord_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub donation_urls: Option<Vec<DonationUrl>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_obj: Option<ModrinthLicense>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub versions_obj: Option<Vec<ModrinthVersion>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_versions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loaders: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featured: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation_message: Option<ModerationMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    pub loader: Option<String>,             // From installation
    pub mc_version: Option<String>,         // From installation
    pub user_filters: Option<FilterFacets>, // User-defined filters
    pub cache: ModCache<Vec<ModrinthInfo>>,
    pub cache_path: PathBuf,
    pub index: Option<String>, // For sorting
}

impl Default for ModrinthProvider {
    fn default() -> Self {
        let cache_path = match crate::get_minecraft_kable_dir() {
            Ok(dir) => dir.join("modrinth_cache.json"),
            Err(_) => PathBuf::from("modrinth_cache.json"),
        };
        let cache = ModCache::load_from_disk(&cache_path).unwrap_or_else(|_| ModCache::new(3600));
        Self {
            limit: 20,
            loader: None,
            mc_version: None,
            user_filters: None,
            cache,
            cache_path,
            index: None,
        }
    }
}

#[async_trait::async_trait]
impl ModProvider for ModrinthProvider {
    fn set_limit(&mut self, limit: usize) {
        self.limit = limit;
    }
    #[log_result(log_values = true, max_length = 150)]
    async fn get(&mut self, offset: usize) -> Result<Vec<ModInfoKind>, String> {
        // Generate cache key from user filters + installation context
        let cache_key = format!(
            "offset:{}:index:{}:filters:{:?}:loader:{}:mc_version:{}",
            offset,
            self.index.as_deref().unwrap_or(""),
            self.user_filters,
            self.loader.as_deref().unwrap_or(""),
            self.mc_version.as_deref().unwrap_or("")
        );
        println!("[ModrinthProvider] Using cache key: {}", cache_key);

        if let Some(entry) = self.cache.get(&cache_key) {
            if !self.cache.is_stale(&cache_key) {
                println!(
                    "[ModrinthProvider] Returning cached results for key: {}",
                    cache_key
                );
                return Ok(entry
                    .value
                    .clone()
                    .into_iter()
                    .map(ModInfoKind::Modrinth)
                    .collect());
            } else {
                println!(
                    "[ModrinthProvider] Cache entry is stale for key: {}",
                    cache_key
                );
            }
        } else {
            println!(
                "[ModrinthProvider] No cache entry found for key: {}",
                cache_key
            );
        }

        // Build facets using the new method
        let mods = if let Some(ref user_filters) = self.user_filters {
            println!("[ModrinthProvider] Making filtered API call with user filters");
            let facets =
                user_filters.to_modrinth_facets(self.loader.as_deref(), self.mc_version.as_deref());
            get_mods_with_facets(
                &facets,
                offset,
                self.limit,
                self.index.as_deref(),
                user_filters.query.as_deref(),
            )
            .await?
        } else if self.loader.is_some() || self.mc_version.is_some() {
            println!("[ModrinthProvider] Making filtered API call (installation only)");
            // Even without user filters, build facets from installation
            let empty_filters = FilterFacets::default();
            let facets = empty_filters
                .to_modrinth_facets(self.loader.as_deref(), self.mc_version.as_deref());
            get_mods_with_facets(&facets, offset, self.limit, self.index.as_deref(), None).await?
        } else {
            println!("[ModrinthProvider] Making unfiltered API call");
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
        println!(
            "[ModrinthProvider] Filtering called with installation: {:?}, filter: {:?}",
            installation.map(|i| &i.name),
            filter
        );

        // Store user-defined filters
        if let Some(crate::mods::manager::ModFilter::Modrinth(facets)) = filter {
            self.user_filters = Some(facets);
            println!(
                "[ModrinthProvider] Set user filters: {:?}",
                self.user_filters
            );
        }

        // Extract installation-specific context
        if let Some(installation) = installation {
            // Extract loader from version_id if present
            if let Some(loader) = extract_loader_from_version_id(&installation.version_id) {
                self.loader = Some(loader.clone());
                println!(
                    "[ModrinthProvider] Set loader from installation: {}",
                    loader
                );
            }

            // Extract Minecraft version from version_id
            if let Some(mc_version) = extract_minecraft_version(&installation.version_id) {
                self.mc_version = Some(mc_version.clone());
                println!(
                    "[ModrinthProvider] Set mc_version from installation: {}",
                    mc_version
                );
            } else {
                // Fallback: use the version_id as-is if we can't extract a proper version
                self.mc_version = Some(installation.version_id.clone());
                println!(
                    "[ModrinthProvider] Set mc_version (fallback) from installation: {}",
                    installation.version_id
                );
            }
        }

        println!(
            "[ModrinthProvider] Current state - loader: {:?}, mc_version: {:?}, user_filters: {:?}",
            self.loader, self.mc_version, self.user_filters
        );
    }

    #[log_result]
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

        // Ensure the mods directory exists (async)
        crate::ensure_parent_dir_exists_async(&mods_dir)
            .await
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
#[log_result(log_values = true, max_length = 100, debug_only = false)]
pub async fn get_all_mods_with_index(
    offset: usize,
    limit: usize,
    index: Option<&str>,
) -> Result<Vec<ModrinthInfo>, String> {
    let client = Client::new();
    let mut url = format!(
        "https://api.modrinth.com/v2/search?limit={}&offset={}",
        limit, offset
    );
    if let Some(index) = index {
        if !index.is_empty() {
            url.push_str(&format!("&index={}", index));
        }
    }
    println!("[ModrinthAPI] Calling URL: {}", url);
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
    println!("[ModrinthAPI] Received {} mods from API", mods.len());
    Ok(mods)
}

/// Fetch mods with properly structured facets array
#[log_result(log_values = true, max_length = 100, debug_only = false)]
pub async fn get_mods_with_facets(
    facets: &[Vec<String>],
    offset: usize,
    limit: usize,
    index: Option<&str>,
    query: Option<&str>,
) -> Result<Vec<ModrinthInfo>, String> {
    let client = Client::new();
    let mut url = format!(
        "https://api.modrinth.com/v2/search?limit={}&offset={}",
        limit, offset
    );

    // Add query if present
    if let Some(q) = query {
        if !q.is_empty() {
            // Simple URL encoding for query
            let encoded = q.replace(' ', "%20").replace('&', "%26");
            url.push_str(&format!("&query={}", encoded));
        }
    }

    // Build facets parameter
    if !facets.is_empty() {
        let facets_json: Vec<String> = facets
            .iter()
            .map(|inner_array| {
                let items: Vec<String> = inner_array
                    .iter()
                    .map(|item| format!("\"{}\"", item))
                    .collect();
                format!("[{}]", items.join(","))
            })
            .collect();
        url.push_str(&format!("&facets=[{}]", facets_json.join(",")));
    }

    // Add index if present
    if let Some(index) = index {
        if !index.is_empty() {
            url.push_str(&format!("&index={}", index));
        }
    }

    println!("[ModrinthAPI] Calling URL with facets: {}", url);

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Modrinth get mods with facets failed: {e}"))?;
    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Modrinth get mods with facets parse failed: {e}"))?;
    let hits = json
        .get("hits")
        .and_then(|v| v.as_array())
        .ok_or("No hits in Modrinth response")?;
    let mods: Vec<ModrinthInfo> = hits
        .iter()
        .filter_map(|hit| serde_json::from_value(hit.clone()).ok())
        .collect();
    println!(
        "[ModrinthAPI] Received {} mods with facets from API",
        mods.len()
    );
    Ok(mods)
}

/// Fetch mods by category, loader, and/or Minecraft version (with pagination and optional index)
#[log_result(log_values = true, max_length = 100, debug_only = false)]
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
        "https://api.modrinth.com/v2/search?limit={}&offset={}",
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
    println!("[ModrinthAPI] Calling filtered URL: {}", url);
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
    println!(
        "[ModrinthAPI] Received {} filtered mods from API",
        mods.len()
    );
    Ok(mods)
}

/// Search mods on Modrinth, optionally filtered by loader and Minecraft version
#[log_result(log_values = true, max_length = 80)]
pub async fn search_mods(
    query: &str,
    loader: Option<&str>,
    mc_version: Option<&str>,
) -> Result<Vec<ModrinthInfo>, String> {
    let client = Client::new();
    let mut url = format!("https://api.modrinth.com/v2/search?query={}", query);
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
#[log_result]
pub async fn get_mod_versions(mod_id: &str) -> Result<Vec<ModrinthVersion>, String> {
    let client = Client::new();
    let url = format!("https://api.modrinth.com/v2/project/{}/version", mod_id);
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
#[log_result]
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
        crate::ensure_parent_dir_exists_async(parent).await?;
    }
    // Atomically write downloaded bytes to save_path
    crate::write_file_atomic_async(save_path, &bytes).await?;
    Ok(())
}

/// Extract Minecraft version from a version_id string
/// Examples:
/// - "iris-fabric-loader-0.16.10-1.21.4" -> Some("1.21.4")
/// - "1.20.1" -> Some("1.20.1")
/// - "forge-1.19.2-43.2.0" -> Some("1.19.2")
/// - "neoforge-21.0.167-beta" -> None (fallback to original)
fn extract_minecraft_version(version_id: &str) -> Option<String> {
    // Common Minecraft version pattern: X.Y.Z where X, Y, Z are numbers
    if let Ok(mc_version_regex) = regex::Regex::new(r"\b(\d+\.\d+(?:\.\d+)?)\b") {
        // Try to find Minecraft version patterns in the version_id
        for cap in mc_version_regex.captures_iter(version_id) {
            if let Some(version) = cap.get(1) {
                let version_str = version.as_str();
                // Validate it looks like a Minecraft version (starts with 1.)
                if version_str.starts_with("1.") {
                    println!(
                        "[ModrinthProvider] Extracted MC version '{}' from version_id '{}'",
                        version_str, version_id
                    );
                    return Some(version_str.to_string());
                }
            }
        }
    }

    println!(
        "[ModrinthProvider] Could not extract MC version from version_id '{}'",
        version_id
    );
    None
}

/// Extract loader type from version_id string
/// Examples:
/// - "iris-fabric-loader-0.16.10-1.21.4" -> Some("fabric")
/// - "forge-1.19.2-43.2.0" -> Some("forge")
/// - "neoforge-21.0.167-beta" -> Some("neoforge")
/// - "quilt-loader-0.29.1-1.21.8" -> Some("quilt")
fn extract_loader_from_version_id(version_id: &str) -> Option<String> {
    let version_lower = version_id.to_lowercase();

    let loader = if version_lower.contains("fabric") {
        Some("fabric".to_string())
    } else if version_lower.contains("neoforge") {
        Some("neoforge".to_string())
    } else if version_lower.contains("forge") {
        Some("forge".to_string())
    } else if version_lower.contains("quilt") {
        Some("quilt".to_string())
    } else {
        None
    };

    if let Some(ref loader_name) = loader {
        println!(
            "[ModrinthProvider] Extracted loader '{}' from version_id '{}'",
            loader_name, version_id
        );
    } else {
        println!(
            "[ModrinthProvider] Could not extract loader from version_id '{}'",
            version_id
        );
    }

    loader
}
