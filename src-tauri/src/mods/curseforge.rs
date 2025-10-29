use crate::{kable_profiles::KableInstallation, mods::cache::ModCache, mods::manager::*};
use kable_macros::log_result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

// Minecraft Game ID for CurseForge API
pub const MINECRAFT_GAME_ID: u32 = 432;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CurseForgeFilter {
    pub query: Option<String>,
    pub category_id: Option<u32>,
    pub game_version: Option<String>,
    pub mod_loader_type: Option<ModLoaderType>,
    pub sort_field: Option<ModsSearchSortField>,
    pub sort_order: Option<SortOrder>,
}

impl fmt::Display for CurseForgeFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CurseForgeFilter(query: {:?}, category: {:?}, version: {:?}, loader: {:?})",
            self.query, self.category_id, self.game_version, self.mod_loader_type
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u32)]
pub enum ModLoaderType {
    Any = 0,
    Forge = 1,
    Cauldron = 2,
    LiteLoader = 3,
    Fabric = 4,
    Quilt = 5,
    NeoForge = 6,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u32)]
pub enum ModsSearchSortField {
    Featured = 1,
    Popularity = 2,
    LastUpdated = 3,
    Name = 4,
    Author = 5,
    TotalDownloads = 6,
    Category = 7,
    GameVersion = 8,
    EarlyAccess = 9,
    FeaturedReleased = 10,
    ReleasedDate = 11,
    Rating = 12,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SortOrder {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

/// CurseForge mod info (see https://docs.curseforge.com/rest-api/)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurseForgeInfo {
    pub id: u32,
    #[serde(rename = "gameId")]
    pub game_id: u32,
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub links: ModLinks,
    pub summary: String,
    pub status: u32,
    #[serde(rename = "downloadCount")]
    pub download_count: u64,
    #[serde(rename = "isFeatured")]
    pub is_featured: bool,
    #[serde(rename = "primaryCategoryId")]
    pub primary_category_id: u32,
    #[serde(default)]
    pub categories: Vec<Category>,
    #[serde(rename = "classId")]
    pub class_id: Option<u32>,
    #[serde(default)]
    pub authors: Vec<ModAuthor>,
    pub logo: Option<ModAsset>,
    #[serde(default)]
    pub screenshots: Vec<ModAsset>,
    #[serde(rename = "mainFileId")]
    pub main_file_id: u32,
    #[serde(rename = "latestFiles", default)]
    pub latest_files: Vec<CurseForgeFile>,
    #[serde(rename = "latestFilesIndexes", default)]
    pub latest_files_indexes: Vec<FileIndex>,
    #[serde(rename = "dateCreated")]
    pub date_created: String,
    #[serde(rename = "dateModified")]
    pub date_modified: String,
    #[serde(rename = "dateReleased")]
    pub date_released: Option<String>,
    #[serde(rename = "allowModDistribution")]
    pub allow_mod_distribution: Option<bool>,
    #[serde(rename = "gamePopularityRank")]
    pub game_popularity_rank: u32,
    #[serde(rename = "isAvailable")]
    pub is_available: bool,
    #[serde(rename = "thumbsUpCount")]
    pub thumbs_up_count: u32,
    pub rating: Option<f64>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModLinks {
    #[serde(rename = "websiteUrl")]
    pub website_url: Option<String>,
    #[serde(rename = "wikiUrl")]
    pub wiki_url: Option<String>,
    #[serde(rename = "issuesUrl")]
    pub issues_url: Option<String>,
    #[serde(rename = "sourceUrl")]
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Category {
    pub id: u32,
    #[serde(rename = "gameId")]
    pub game_id: u32,
    pub name: String,
    pub slug: String,
    pub url: String,
    #[serde(rename = "iconUrl")]
    pub icon_url: String,
    #[serde(rename = "dateModified")]
    pub date_modified: String,
    #[serde(rename = "isClass")]
    pub is_class: bool,
    #[serde(rename = "classId")]
    pub class_id: Option<u32>,
    #[serde(rename = "parentCategoryId")]
    pub parent_category_id: Option<u32>,
    #[serde(rename = "displayIndex")]
    pub display_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModAuthor {
    pub id: u32,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModAsset {
    pub id: u32,
    #[serde(rename = "modId")]
    pub mod_id: u32,
    pub title: String,
    pub description: String,
    #[serde(rename = "thumbnailUrl")]
    pub thumbnail_url: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CurseForgeFile {
    pub id: u32,
    #[serde(rename = "gameId")]
    pub game_id: u32,
    #[serde(rename = "modId")]
    pub mod_id: u32,
    #[serde(rename = "isAvailable")]
    pub is_available: bool,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "releaseType")]
    pub release_type: u32,
    #[serde(rename = "fileStatus")]
    pub file_status: u32,
    #[serde(default)]
    pub hashes: Vec<FileHash>,
    #[serde(rename = "fileDate")]
    pub file_date: String,
    #[serde(rename = "fileLength")]
    pub file_length: u64,
    #[serde(rename = "downloadCount")]
    pub download_count: u64,
    #[serde(rename = "downloadUrl")]
    pub download_url: String,
    #[serde(rename = "gameVersions", default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<FileDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileHash {
    pub value: String,
    pub algo: u32, // 1 = Sha1, 2 = Md5
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileDependency {
    #[serde(rename = "modId")]
    pub mod_id: u32,
    #[serde(rename = "relationType")]
    pub relation_type: u32, // 1 = EmbeddedLibrary, 2 = OptionalDependency, 3 = RequiredDependency, 4 = Tool, 5 = Incompatible, 6 = Include
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileIndex {
    #[serde(rename = "gameVersion")]
    pub game_version: String,
    #[serde(rename = "fileId")]
    pub file_id: u32,
    pub filename: String,
    #[serde(rename = "releaseType")]
    pub release_type: u32,
    #[serde(rename = "gameVersionTypeId")]
    pub game_version_type_id: Option<u32>,
    #[serde(rename = "modLoader")]
    pub mod_loader: u32,
}

/// Response structure for search mods API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchModsResponse {
    pub data: Vec<CurseForgeInfo>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub index: u32,
    #[serde(rename = "pageSize")]
    pub page_size: u32,
    #[serde(rename = "resultCount")]
    pub result_count: u32,
    #[serde(rename = "totalCount")]
    pub total_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModFilesResponse {
    pub data: Vec<CurseForgeFile>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurseForgeProvider {
    pub limit: usize,
    pub filter: CurseForgeFilter,
    pub cache: ModCache<Vec<CurseForgeInfo>>,
    pub cache_path: PathBuf,
}

impl Default for CurseForgeProvider {
    fn default() -> Self {
        let cache_path = match crate::get_minecraft_kable_dir() {
            Ok(dir) => dir.join("curseforge_cache.json"),
            Err(_) => PathBuf::from("curseforge_cache.json"),
        };
        let cache = ModCache::load_from_disk(&cache_path).unwrap_or_else(|_| ModCache::new(3600));
        Self {
            limit: 20,
            filter: CurseForgeFilter {
                query: None,
                category_id: None,
                game_version: None,
                mod_loader_type: None,
                sort_field: None,
                sort_order: None,
            },
            cache,
            cache_path,
        }
    }
}

#[async_trait::async_trait]
impl ModProvider for CurseForgeProvider {
    fn set_limit(&mut self, limit: usize) {
        self.limit = limit;
    }

    #[log_result(log_values = true, max_length = 150)]
    async fn get(&mut self, offset: usize) -> Result<Vec<ModInfoKind>, String> {
        let cache_key = format!(
            "offset:{}:query:{}:category:{}:loader:{}:version:{}:sort:{}:order:{}",
            offset,
            self.filter.query.as_deref().unwrap_or(""),
            self.filter.category_id.unwrap_or(0),
            self.filter
                .mod_loader_type
                .as_ref()
                .map(|_| "loader")
                .unwrap_or(""),
            self.filter.game_version.as_deref().unwrap_or(""),
            self.filter
                .sort_field
                .as_ref()
                .map(|_| "sort")
                .unwrap_or(""),
            self.filter
                .sort_order
                .as_ref()
                .map(|_| "order")
                .unwrap_or("")
        );
        println!("[CurseForgeProvider] Using cache key: {}", cache_key);

        if let Some(entry) = self.cache.get(&cache_key) {
            if !self.cache.is_stale(&cache_key) {
                println!(
                    "[CurseForgeProvider] Returning cached results for key: {}",
                    cache_key
                );
                return Ok(entry
                    .value
                    .clone()
                    .into_iter()
                    .map(ModInfoKind::CurseForge)
                    .collect());
            } else {
                println!(
                    "[CurseForgeProvider] Cache entry is stale for key: {}",
                    cache_key
                );
            }
        } else {
            println!(
                "[CurseForgeProvider] No cache entry found for key: {}",
                cache_key
            );
        }

        let mods = search_mods(&self.filter, offset, self.limit).await?;
        self.cache.insert(cache_key.clone(), mods.clone());
        let _ = self.cache.save_to_disk(&self.cache_path);
        Ok(mods.into_iter().map(ModInfoKind::CurseForge).collect())
    }

    fn filter(
        &mut self,
        installation: Option<&KableInstallation>,
        filter: Option<crate::mods::manager::ModFilter>,
    ) {
        println!(
            "[CurseForgeProvider] Filtering called with installation: {:?}, filter: {:?}",
            installation.map(|i| &i.name),
            filter
        );

        if let Some(crate::mods::manager::ModFilter::CurseForge(cf_filter)) = filter {
            self.filter = cf_filter;
            println!("[CurseForgeProvider] Set filter: {}", self.filter);
        }

        if let Some(installation) = installation {
            // Extract loader from version_id if not already set
            if self.filter.mod_loader_type.is_none() {
                if let Some(loader) = extract_loader_from_version_id(&installation.version_id) {
                    self.filter.mod_loader_type = Some(loader);
                    println!(
                        "[CurseForgeProvider] Set loader filter from installation: {:?}",
                        self.filter.mod_loader_type
                    );
                }
            }

            // Extract Minecraft version from version_id if not already set
            if self.filter.game_version.is_none() {
                if let Some(mc_version) = extract_minecraft_version(&installation.version_id) {
                    self.filter.game_version = Some(mc_version.clone());
                    println!(
                        "[CurseForgeProvider] Set mc_version filter from installation: {}",
                        mc_version
                    );
                } else {
                    // Fallback: use the version_id as-is if we can't extract a proper version
                    self.filter.game_version = Some(installation.version_id.clone());
                    println!("[CurseForgeProvider] Set mc_version filter (fallback) from installation: {}", installation.version_id);
                }
            }
        }

        println!("[CurseForgeProvider] Current filters: {}", self.filter);
    }

    #[log_result]
    async fn download(
        &self,
        mod_id: &str,
        version_id: Option<&str>,
        installation: &KableInstallation,
    ) -> Result<(), String> {
        // Determine the mods directory for the installation
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

        let mod_id_u32: u32 = mod_id
            .parse()
            .map_err(|_| format!("Invalid mod ID: {}", mod_id))?;

        // Disable any existing versions of this mod (same project_id)
        disable_old_mod_versions(&mods_dir, mod_id).await?;

        let files = get_mod_files(mod_id_u32).await?;
        let file = if let Some(version_id) = version_id {
            let version_id_u32: u32 = version_id
                .parse()
                .map_err(|_| format!("Invalid version ID: {}", version_id))?;
            files.into_iter().find(|f| f.id == version_id_u32)
        } else {
            files.into_iter().next()
        };

        let file = file.ok_or("Mod file not found")?;
        let download_url = get_mod_file_download_url(mod_id_u32, file.id).await?;
        
        // Download the file
        download_mod_file(&download_url, &mods_dir.join(&file.file_name)).await?;
        
        // Save metadata
        save_mod_metadata(&mods_dir, &file.file_name, mod_id, &file.display_name).await?;
        
        Ok(())
    }

    fn set_index(&mut self, index: Option<String>) {
        if let Some(index_str) = index {
            // Parse the index string to determine sort field and order
            match index_str.as_str() {
                "featured" => {
                    self.filter.sort_field = Some(ModsSearchSortField::Featured);
                    self.filter.sort_order = Some(SortOrder::Descending);
                }
                "popularity" => {
                    self.filter.sort_field = Some(ModsSearchSortField::Popularity);
                    self.filter.sort_order = Some(SortOrder::Descending);
                }
                "updated" => {
                    self.filter.sort_field = Some(ModsSearchSortField::LastUpdated);
                    self.filter.sort_order = Some(SortOrder::Descending);
                }
                "downloads" => {
                    self.filter.sort_field = Some(ModsSearchSortField::TotalDownloads);
                    self.filter.sort_order = Some(SortOrder::Descending);
                }
                "name" => {
                    self.filter.sort_field = Some(ModsSearchSortField::Name);
                    self.filter.sort_order = Some(SortOrder::Ascending);
                }
                _ => {
                    self.filter.sort_field = Some(ModsSearchSortField::Popularity);
                    self.filter.sort_order = Some(SortOrder::Descending);
                }
            }
        } else {
            self.filter.sort_field = None;
            self.filter.sort_order = None;
        }
    }

    fn get_index(&self) -> Option<&String> {
        // Convert current sort settings back to string format
        // This is a bit tricky since we store the parsed values, but we need to return a string
        // For simplicity, return None here and rely on set_index for proper handling
        None
    }
}

/// Get API key from environment variable
fn get_api_key() -> Result<String, String> {
    match std::env::var("CURSEFORGE_API_KEY") {
        Ok(key) => {
            if key.is_empty() {
                Err("CURSEFORGE_API_KEY environment variable is empty. Please add a valid API key to your .env file.".to_string())
            } else {
                println!(
                    "[CurseForgeAPI] API key loaded successfully (length: {})",
                    key.len()
                );
                Ok(key)
            }
        }
        Err(_) => Err(
            "CURSEFORGE_API_KEY environment variable not set. Please add it to your .env file."
                .to_string(),
        ),
    }
}

/// Search mods on CurseForge
#[log_result(log_values = true, max_length = 100, debug_only = false)]
pub async fn search_mods(
    filter: &CurseForgeFilter,
    offset: usize,
    limit: usize,
) -> Result<Vec<CurseForgeInfo>, String> {
    let api_key = get_api_key()?;
    let client = Client::new();
    let mut url = format!(
        "https://api.curseforge.com/v1/mods/search?gameId={}&index={}&pageSize={}",
        MINECRAFT_GAME_ID, offset, limit
    );

    // Add search filter
    if let Some(ref query) = filter.query {
        url.push_str(&format!("&searchFilter={}", urlencoding::encode(query)));
    }

    // Add category filter
    if let Some(category_id) = filter.category_id {
        url.push_str(&format!("&categoryId={}", category_id));
    }

    // Add game version filter
    if let Some(ref game_version) = filter.game_version {
        url.push_str(&format!(
            "&gameVersion={}",
            urlencoding::encode(game_version)
        ));
    }

    // Add mod loader type filter
    if let Some(ref mod_loader) = filter.mod_loader_type {
        url.push_str(&format!("&modLoaderType={}", *mod_loader as u32));
    }

    // Add sort field
    if let Some(ref sort_field) = filter.sort_field {
        url.push_str(&format!("&sortField={}", *sort_field as u32));
    }

    // Add sort order
    if let Some(ref sort_order) = filter.sort_order {
        let order_str = match sort_order {
            SortOrder::Ascending => "asc",
            SortOrder::Descending => "desc",
        };
        url.push_str(&format!("&sortOrder={}", order_str));
    }

    println!("[CurseForgeAPI] Calling URL: {}", url);

    let resp = client
        .get(&url)
        .header("x-api-key", &api_key)
        .send()
        .await
        .map_err(|e| format!("CurseForge search failed: {}", e))?;

    println!("[CurseForgeAPI] Response status: {}", resp.status());

    // Check if the response is successful
    let status = resp.status();
    if !status.is_success() {
        let error_text = resp
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error response".to_string());
        return Err(format!("CurseForge API error {}: {}", status, error_text));
    }

    let response_text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    println!(
        "[CurseForgeAPI] Response body length: {}",
        response_text.len()
    );

    if response_text.is_empty() {
        return Err("CurseForge API returned empty response".to_string());
    }

    let search_response: SearchModsResponse =
        serde_json::from_str(&response_text).map_err(|e| {
            format!(
                "CurseForge search parse failed: {} (body: {})",
                e, response_text
            )
        })?;

    println!(
        "[CurseForgeAPI] Received {} mods from API",
        search_response.data.len()
    );
    Ok(search_response.data)
}

/// Get all files for a given CurseForge mod ID
#[log_result]
pub async fn get_mod_files(mod_id: u32) -> Result<Vec<CurseForgeFile>, String> {
    let api_key = get_api_key()?;
    let client = Client::new();
    let url = format!("https://api.curseforge.com/v1/mods/{}/files", mod_id);

    let resp = client
        .get(&url)
        .header("x-api-key", &api_key)
        .send()
        .await
        .map_err(|e| format!("CurseForge get files failed: {}", e))?;

    let files_response: ModFilesResponse = resp
        .json()
        .await
        .map_err(|e| format!("CurseForge get files parse failed: {}", e))?;

    Ok(files_response.data)
}

/// Get download URL for a specific file
#[log_result]
pub async fn get_mod_file_download_url(mod_id: u32, file_id: u32) -> Result<String, String> {
    let api_key = get_api_key()?;
    let client = Client::new();
    let url = format!(
        "https://api.curseforge.com/v1/mods/{}/files/{}/download-url",
        mod_id, file_id
    );

    let resp = client
        .get(&url)
        .header("x-api-key", &api_key)
        .send()
        .await
        .map_err(|e| format!("CurseForge get download URL failed: {}", e))?;

    #[derive(Deserialize)]
    struct DownloadUrlResponse {
        data: String,
    }

    let download_response: DownloadUrlResponse = resp
        .json()
        .await
        .map_err(|e| format!("CurseForge get download URL parse failed: {}", e))?;

    Ok(download_response.data)
}

/// Download a mod file from CurseForge and save to the given path
#[log_result]
pub async fn download_mod_file(url: &str, save_path: &std::path::Path) -> Result<(), String> {
    let client = Client::new();
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("CurseForge download failed: {}", e))?;
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("CurseForge download bytes failed: {}", e))?;
    if let Some(parent) = save_path.parent() {
        crate::ensure_parent_dir_exists_async(parent).await?;
    }
    crate::write_file_atomic_async(save_path, &bytes).await?;
    Ok(())
}

/// Extract Minecraft version from a version_id string (same logic as Modrinth)
fn extract_minecraft_version(version_id: &str) -> Option<String> {
    if let Ok(mc_version_regex) = regex::Regex::new(r"\b(\d+\.\d+(?:\.\d+)?)\b") {
        for cap in mc_version_regex.captures_iter(version_id) {
            if let Some(version) = cap.get(1) {
                let version_str = version.as_str();
                if version_str.starts_with("1.") {
                    println!(
                        "[CurseForgeProvider] Extracted MC version '{}' from version_id '{}'",
                        version_str, version_id
                    );
                    return Some(version_str.to_string());
                }
            }
        }
    }

    println!(
        "[CurseForgeProvider] Could not extract MC version from version_id '{}'",
        version_id
    );
    None
}

/// Extract loader type from version_id string and convert to CurseForge ModLoaderType
fn extract_loader_from_version_id(version_id: &str) -> Option<ModLoaderType> {
    let version_lower = version_id.to_lowercase();

    let loader = if version_lower.contains("fabric") {
        Some(ModLoaderType::Fabric)
    } else if version_lower.contains("neoforge") {
        Some(ModLoaderType::NeoForge)
    } else if version_lower.contains("forge") {
        Some(ModLoaderType::Forge)
    } else if version_lower.contains("quilt") {
        Some(ModLoaderType::Quilt)
    } else {
        None
    };

    if let Some(ref loader_type) = loader {
        println!(
            "[CurseForgeProvider] Extracted loader '{:?}' from version_id '{}'",
            loader_type, version_id
        );
    } else {
        println!(
            "[CurseForgeProvider] Could not extract loader from version_id '{}'",
            version_id
        );
    }

    loader
}

/// Remove old versions of a mod before downloading a new version
/// Checks for mods with the same project_id and removes them completely
async fn disable_old_mod_versions(mods_dir: &std::path::Path, project_id: &str) -> Result<(), String> {
    use tokio::fs;
    
    // Read all files in the mods directory
    let mut entries = fs::read_dir(mods_dir)
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
                    if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&metadata_content) {
                        // Check if this metadata belongs to the same project
                        if let Some(stored_project_id) = metadata.get("project_id").and_then(|v| v.as_str()) {
                            if stored_project_id == project_id {
                                // Get the associated jar file name
                                if let Some(jar_file) = metadata.get("file_name").and_then(|v| v.as_str()) {
                                    let jar_path = mods_dir.join(jar_file);
                                    
                                    // Delete the old jar file completely
                                    if jar_path.exists() {
                                        fs::remove_file(&jar_path)
                                            .await
                                            .map_err(|e| format!("Failed to remove old mod version: {}", e))?;
                                        
                                        println!(
                                            "[CurseForgeProvider] Removed old version: {} (project: {})",
                                            jar_file, project_id
                                        );
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
    
    Ok(())
}

/// Save metadata for a downloaded mod
async fn save_mod_metadata(
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
