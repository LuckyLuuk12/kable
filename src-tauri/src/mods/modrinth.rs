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

/// Response from /project/{id}/dependencies endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectDependencies {
    pub projects: Vec<ModrinthInfo>,
    pub versions: Vec<ModrinthVersion>,
}

/// Dependency type from version dependencies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionDependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: String, // "required", "optional", "incompatible", "embedded"
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

        // Store user-defined filters (or clear them if None)
        match filter {
            Some(crate::mods::manager::ModFilter::Modrinth(facets)) => {
                self.user_filters = Some(facets);
                println!(
                    "[ModrinthProvider] Set user filters: {:?}",
                    self.user_filters
                );
            }
            None => {
                self.user_filters = None;
                println!("[ModrinthProvider] Cleared user filters");
            }
            _ => {
                // Other providers - don't change user_filters
                println!("[ModrinthProvider] Ignoring non-Modrinth filter");
            }
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
        } else {
            // Clear installation-specific filters when no installation provided
            self.loader = None;
            self.mc_version = None;
            println!(
                "[ModrinthProvider] Cleared installation-specific filters (loader, mc_version)"
            );
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

        // Disable any existing versions of this mod (same project_id)
        // and check if the old version was in the disabled folder
        let was_disabled = disable_old_mod_versions(&mods_dir, mod_id).await?;

        // Determine target directory based on whether old mod was disabled
        let download_dir = if was_disabled {
            let disabled_dir = mods_dir.join("disabled");
            crate::ensure_parent_dir_exists_async(&disabled_dir)
                .await
                .map_err(|e| format!("Failed to create disabled directory: {}", e))?;
            println!("[ModrinthProvider] Old mod was disabled, downloading new version to disabled folder");
            disabled_dir
        } else {
            mods_dir.clone()
        };

        // If specific version_id provided, download that version
        if let Some(version_id) = version_id {
            let versions = get_mod_versions(mod_id).await?;
            let version = versions
                .into_iter()
                .find(|v| v.id == version_id)
                .ok_or("Specified mod version not found")?;

            let mut files_iter = version.files.into_iter();
            let file = files_iter
                .clone()
                .find(|f| f.primary)
                .or_else(|| files_iter.next())
                .ok_or("No mod file found")?;

            // Download the file to appropriate directory (active or disabled)
            download_mod_file(&file.url, &download_dir.join(&file.filename)).await?;

            // Save metadata in the same directory as the mod file
            save_mod_metadata(
                &download_dir,
                &file.filename,
                mod_id,
                &version.version_number,
            )
            .await?;

            // Resolve and install dependencies
            println!("[ModrinthProvider] Resolving dependencies for {}", mod_id);
            if let Err(e) = resolve_and_install_dependencies(mod_id, installation, 0).await {
                println!(
                    "[ModrinthProvider] Warning: Failed to resolve dependencies: {}",
                    e
                );
            }

            return Ok(());
        }

        // Otherwise, intelligently find best matching version
        let loader = extract_loader_from_version_id(&installation.version_id);
        let mc_version = extract_minecraft_version(&installation.version_id);

        println!(
            "[ModrinthProvider] Finding best version for installation: loader={:?}, mc_version={:?}",
            loader, mc_version
        );

        // Fetch filtered versions for better performance
        let versions = if loader.is_some() || mc_version.is_some() {
            get_project_versions_filtered(
                mod_id,
                loader.clone().map(|l| vec![l]),
                mc_version.clone().map(|v| vec![v]),
            )
            .await?
        } else {
            get_mod_versions(mod_id).await?
        };

        if versions.is_empty() {
            return Err("No compatible mod versions found for this installation".to_string());
        }

        // Find the best version using our smart selection logic
        let version = find_best_version(&versions, loader.as_deref(), mc_version.as_deref())
            .ok_or("No compatible mod version found for this installation")?;

        println!(
            "[ModrinthProvider] Selected version: {} ({})",
            version.version_number, version.id
        );

        let mut files_iter = version.files.into_iter();
        let file = files_iter
            .clone()
            .find(|f| f.primary)
            .or_else(|| files_iter.next())
            .ok_or("No mod file found")?;

        // Download the file to appropriate directory (active or disabled)
        download_mod_file(&file.url, &download_dir.join(&file.filename)).await?;

        // Save metadata in the same directory as the mod file
        save_mod_metadata(
            &download_dir,
            &file.filename,
            mod_id,
            &version.version_number,
        )
        .await?;

        // Resolve and install dependencies
        println!("[ModrinthProvider] Resolving dependencies for {}", mod_id);
        if let Err(e) = resolve_and_install_dependencies(mod_id, installation, 0).await {
            println!(
                "[ModrinthProvider] Warning: Failed to resolve dependencies: {}",
                e
            );
            // Don't fail the main download if dependencies fail
        }

        Ok(())
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

    let response_text = resp
        .text()
        .await
        .map_err(|e| format!("Modrinth get versions response read failed: {e}"))?;

    let versions: Vec<ModrinthVersion> = serde_json::from_str(&response_text).map_err(|e| {
        let preview = if response_text.len() > 500 {
            format!("{}...", &response_text[..500])
        } else {
            response_text.clone()
        };
        format!("Modrinth get versions parse failed: {e}\nJSON preview: {preview}")
    })?;
    Ok(versions)
}

/// Get multiple projects by their IDs
/// See: https://docs.modrinth.com/api/operations/getprojects/
#[log_result]
pub async fn get_projects(project_ids: Vec<String>) -> Result<Vec<ModrinthInfo>, String> {
    if project_ids.is_empty() {
        return Ok(Vec::new());
    }

    let client = Client::new();
    let ids_param = serde_json::to_string(&project_ids)
        .map_err(|e| format!("Failed to serialize project IDs: {e}"))?;
    let url = format!(
        "https://api.modrinth.com/v2/projects?ids={}",
        urlencoding::encode(&ids_param)
    );

    println!("[ModrinthAPI] Fetching {} projects", project_ids.len());

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Modrinth get projects failed: {e}"))?;

    let response_text = resp
        .text()
        .await
        .map_err(|e| format!("Modrinth get projects response read failed: {e}"))?;

    let projects: Vec<ModrinthInfo> = serde_json::from_str(&response_text).map_err(|e| {
        let preview = if response_text.len() > 500 {
            format!("{}...", &response_text[..500])
        } else {
            response_text.clone()
        };
        format!("Modrinth get projects parse failed: {e}\nJSON preview: {preview}")
    })?;

    println!("[ModrinthAPI] Received {} projects", projects.len());
    Ok(projects)
}

/// Get versions for a Modrinth project filtered by loaders and game versions
/// See: https://docs.modrinth.com/api/operations/getprojectversions/
#[log_result]
pub async fn get_project_versions_filtered(
    project_id: &str,
    loaders: Option<Vec<String>>,
    game_versions: Option<Vec<String>>,
) -> Result<Vec<ModrinthVersion>, String> {
    let client = Client::new();
    let mut url = format!("https://api.modrinth.com/v2/project/{}/version", project_id);

    let mut params = Vec::new();

    // Add loaders parameter (e.g., ["fabric", "forge"])
    if let Some(loaders) = loaders {
        if !loaders.is_empty() {
            let loaders_json = serde_json::to_string(&loaders)
                .map_err(|e| format!("Failed to serialize loaders: {e}"))?;
            params.push(format!("loaders={}", urlencoding::encode(&loaders_json)));
        }
    }

    // Add game_versions parameter (e.g., ["1.20.1", "1.20.2"])
    if let Some(game_versions) = game_versions {
        if !game_versions.is_empty() {
            let game_versions_json = serde_json::to_string(&game_versions)
                .map_err(|e| format!("Failed to serialize game_versions: {e}"))?;
            params.push(format!(
                "game_versions={}",
                urlencoding::encode(&game_versions_json)
            ));
        }
    }

    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    println!("[ModrinthAPI] Fetching filtered versions from: {}", url);

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Modrinth get filtered versions failed: {e}"))?;

    // Get the response text first to debug parse errors
    let response_text = resp
        .text()
        .await
        .map_err(|e| format!("Modrinth get filtered versions response read failed: {e}"))?;

    // Check if Modrinth returned an error response
    if let Ok(error_check) = serde_json::from_str::<serde_json::Value>(&response_text) {
        if let Some(error) = error_check.get("error") {
            let error_type = error.as_str().unwrap_or("unknown");
            let description = error_check
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("No description provided");
            return Err(format!(
                "Modrinth API error ({}): {}",
                error_type, description
            ));
        }
    }

    let versions: Vec<ModrinthVersion> = serde_json::from_str(&response_text).map_err(|e| {
        // Show first 500 chars of JSON to help debug
        let preview = if response_text.len() > 500 {
            format!("{}...", &response_text[..500])
        } else {
            response_text.clone()
        };
        format!("Modrinth get filtered versions parse failed: {e}\nJSON preview: {preview}")
    })?;

    println!(
        "[ModrinthAPI] Received {} filtered versions",
        versions.len()
    );
    Ok(versions)
}

/// Find the best matching version from a list of versions
/// Returns the version with the highest version_number that matches the criteria
pub fn find_best_version(
    versions: &[ModrinthVersion],
    preferred_loader: Option<&str>,
    preferred_game_version: Option<&str>,
) -> Option<ModrinthVersion> {
    let mut candidates: Vec<&ModrinthVersion> = versions.iter().collect();

    // Filter by loader if specified
    if let Some(loader) = preferred_loader {
        candidates.retain(|v| v.loaders.iter().any(|l| l.eq_ignore_ascii_case(loader)));
    }

    // Filter by game version if specified
    if let Some(game_version) = preferred_game_version {
        candidates.retain(|v| v.game_versions.contains(&game_version.to_string()));
    }

    if candidates.is_empty() {
        return None;
    }

    // Sort by version number (reverse semver-like comparison)
    candidates.sort_by(|a, b| compare_version_strings(&b.version_number, &a.version_number));

    candidates.first().map(|v| (*v).clone())
}

/// Compare two version strings in a semver-like manner
/// Returns Ordering: Greater if a > b, Less if a < b, Equal if a == b
fn compare_version_strings(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let parse_version_parts = |s: &str| -> Vec<u32> {
        s.split(['.', '-', '+'])
            .filter_map(|part| part.parse::<u32>().ok())
            .collect()
    };

    let a_parts = parse_version_parts(a);
    let b_parts = parse_version_parts(b);

    for (a_part, b_part) in a_parts.iter().zip(b_parts.iter()) {
        match a_part.cmp(b_part) {
            Ordering::Equal => continue,
            other => return other,
        }
    }

    // If all parts are equal, compare by length (more parts = more specific)
    a_parts.len().cmp(&b_parts.len())
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

/// Remove old versions of a mod before downloading a new version
/// Checks for mods with the same project_id and removes them completely
/// Returns true if the old mod was in the disabled folder
async fn disable_old_mod_versions(
    mods_dir: &std::path::Path,
    project_id: &str,
) -> Result<bool, String> {
    use tokio::fs;

    let mut was_disabled = false;

    // Check both active mods directory and disabled subdirectory
    let dirs_to_check = vec![
        (mods_dir.to_path_buf(), false),
        (mods_dir.join("disabled"), true),
    ];

    for (dir, is_disabled_dir) in dirs_to_check {
        if !dir.exists() {
            continue;
        }

        // Read all files in the directory
        let mut entries = fs::read_dir(&dir)
            .await
            .map_err(|e| format!("Failed to read mods directory: {}", e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("Failed to read directory entry: {}", e))?
        {
            let path = entry.path();

            // Check for .kable_metadata.json files
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.ends_with(".kable_metadata.json") {
                    // Read the metadata file
                    if let Ok(metadata_content) = fs::read_to_string(&path).await {
                        if let Ok(metadata) =
                            serde_json::from_str::<serde_json::Value>(&metadata_content)
                        {
                            // Check if this metadata belongs to the same project
                            if let Some(stored_project_id) =
                                metadata.get("project_id").and_then(|v| v.as_str())
                            {
                                if stored_project_id == project_id {
                                    // Get the associated jar file name
                                    if let Some(jar_file) =
                                        metadata.get("file_name").and_then(|v| v.as_str())
                                    {
                                        let jar_path = dir.join(jar_file);

                                        // Delete the old jar file completely
                                        if jar_path.exists() {
                                            fs::remove_file(&jar_path).await.map_err(|e| {
                                                format!("Failed to remove old mod version: {}", e)
                                            })?;

                                            println!(
                                                "[ModrinthProvider] Removed old version: {} from {} (project: {})",
                                                jar_file,
                                                if is_disabled_dir { "disabled" } else { "active" },
                                                project_id
                                            );

                                            // Track if we removed from disabled folder
                                            if is_disabled_dir {
                                                was_disabled = true;
                                            }
                                        }

                                        // Also remove the metadata file
                                        let _ = fs::remove_file(&path).await;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(was_disabled)
}

/// Save metadata for a downloaded mod
pub async fn save_mod_metadata(
    mods_dir: &std::path::Path,
    file_name: &str,
    project_id: &str,
    version_number: &str,
) -> Result<(), String> {
    use tokio::fs;

    let metadata = serde_json::json!({
        "project_id": project_id,
        "file_name": file_name,
        "version_number": version_number,
        "download_time": chrono::Utc::now().to_rfc3339(),
    });

    let metadata_path = mods_dir.join(format!("{}.kable_metadata.json", file_name));
    let metadata_content = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

    fs::write(&metadata_path, metadata_content)
        .await
        .map_err(|e| format!("Failed to write metadata file: {}", e))?;

    Ok(())
}

/// Get all dependencies for a project
#[log_result]
pub async fn get_project_dependencies(project_id: &str) -> Result<ProjectDependencies, String> {
    let client = Client::new();
    let url = format!(
        "https://api.modrinth.com/v2/project/{}/dependencies",
        project_id
    );

    println!(
        "[ModrinthAPI] Fetching dependencies for project: {}",
        project_id
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Modrinth get dependencies failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Modrinth API error: {}", resp.status()));
    }

    let deps: ProjectDependencies = resp
        .json()
        .await
        .map_err(|e| format!("Modrinth parse dependencies failed: {}", e))?;

    println!(
        "[ModrinthAPI] Found {} dependency projects for {}",
        deps.projects.len(),
        project_id
    );
    Ok(deps)
}

/// Resolve and install all required dependencies for a project
#[log_result]
pub async fn resolve_and_install_dependencies(
    project_id: &str,
    installation: &KableInstallation,
    depth: usize,
) -> Result<(), String> {
    // Prevent infinite recursion
    if depth > 10 {
        println!("[DependencyResolver] Max recursion depth reached");
        return Ok(());
    }

    // Get dependencies from API
    let deps = match get_project_dependencies(project_id).await {
        Ok(d) => d,
        Err(e) => {
            println!(
                "[DependencyResolver] Failed to get dependencies for {}: {}",
                project_id, e
            );
            return Ok(()); // Don't fail the main download if dependencies fail
        }
    };

    // Get mods directory to check what's already installed
    let mods_dir: std::path::PathBuf =
        if let Some(ref custom_mods) = installation.dedicated_mods_folder {
            let custom_path = std::path::PathBuf::from(custom_mods);
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

    // Load installed mods metadata to check by project_id
    let mut installed_project_ids = std::collections::HashSet::new();
    if mods_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false)
                    && path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.ends_with(".kable_metadata.json"))
                        .unwrap_or(false)
                {
                    // Read metadata file
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(metadata) =
                            serde_json::from_str::<crate::mods::ModMetadata>(&content)
                        {
                            installed_project_ids.insert(metadata.project_id);
                        }
                    }
                }
            }
        }
    }

    // Process each dependency project
    for dep_project in deps.projects {
        let dep_project_id = &dep_project.project_id;

        // Skip if already installed
        if installed_project_ids.contains(dep_project_id) {
            println!(
                "[DependencyResolver] Dependency {} already installed, skipping",
                dep_project_id
            );
            continue;
        }

        println!(
            "[DependencyResolver] Installing dependency: {} ({})",
            dep_project.title, dep_project_id
        );

        // Download the dependency (without version_id to get best match)
        // Use ModrinthProvider's download logic
        let provider = ModrinthProvider::default();
        match provider.download(dep_project_id, None, installation).await {
            Ok(_) => {
                println!(
                    "[DependencyResolver] Successfully installed dependency: {}",
                    dep_project_id
                );
                // Recursively resolve dependencies of this dependency (boxed for async recursion)
                let dep_id = dep_project_id.to_string();
                let inst = installation.clone();
                let resolve_future = Box::pin(async move {
                    resolve_and_install_dependencies(&dep_id, &inst, depth + 1).await
                });
                if let Err(e) = resolve_future.await {
                    println!(
                        "[DependencyResolver] Failed to resolve nested dependencies for {}: {}",
                        dep_project_id, e
                    );
                }
            }
            Err(e) => {
                println!(
                    "[DependencyResolver] Failed to install dependency {}: {}",
                    dep_project_id, e
                );
                // Continue with other dependencies even if one fails
            }
        }
    }

    Ok(())
}
