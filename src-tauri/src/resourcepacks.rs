use kable_macros::log_result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tokio::fs as async_fs;

// Filter structure for resourcepack searching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ResourcePackFilterFacets {
    pub query: Option<String>,                      // User search string
    pub categories: Option<Vec<(String, String)>>,  // (operation, value) - each AND'd
    pub game_versions: Option<Vec<String>>,         // From installation
    // Note: Performance and features are Modrinth categories/tags
}

impl ResourcePackFilterFacets {
    /// Build facets array for Modrinth API from ResourcePackFilterFacets + installation info
    /// Returns Vec<Vec<String>> where:
    /// - Each inner Vec is OR'd together
    /// - Outer Vec items are AND'd together
    ///   Each filter is AND'd as separate array
    pub fn to_modrinth_facets(&self, mc_version: Option<&str>) -> Vec<Vec<String>> {
        let mut facets: Vec<Vec<String>> = Vec::new();

        // Categories - each filter is AND'd (separate array per filter)
        if let Some(ref cats) = self.categories {
            for (op, val) in cats {
                facets.push(vec![format!("categories{}{}", op, val)]);
            }
        }

        // MC Version from installation - AND (separate array)
        if let Some(mc_version) = mc_version {
            facets.push(vec![format!("versions:{}", mc_version)]);
        }

        // Game versions - each AND'd
        if let Some(ref versions) = self.game_versions {
            for version in versions {
                facets.push(vec![format!("versions:{}", version)]);
            }
        }

        // Project type - always resourcepack
        facets.push(vec!["project_type:resourcepack".to_string()]);

        facets
    }
}

// Resource pack management structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourcePack {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: Option<String>,
    pub file_path: String,
    pub file_name: String,
    pub file_size: u64,
    pub compatible_versions: Vec<String>,
    pub pack_format: u32,
    pub enabled: bool,
    pub source_url: Option<String>,
    pub thumbnail: Option<String>,
    pub installed_date: i64,
    pub last_used: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourcePackDownload {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub download_url: String,
    pub thumbnail: Option<String>,
    pub gallery: Option<Vec<String>>,
    pub featured_gallery: Option<String>,
    pub tags: Vec<String>,
    pub minecraft_versions: Vec<String>,
    pub resolution: Option<String>,
    pub rating: f32,
    pub downloads: u64,
    pub size_mb: u64,
    pub source: ResourcePackSource,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ResourcePackSource {
    Modrinth,
    CurseForge,
    Other(String),
}

#[derive(Debug, Deserialize)]
struct ModrinthSearchResponse {
    hits: Vec<ModrinthProject>,
    #[allow(dead_code)]
    total_hits: u32,
}

#[derive(Debug, Deserialize)]
struct ModrinthProject {
    project_id: String,
    #[allow(dead_code)]
    slug: String,
    title: String,
    description: String,
    author: String,
    icon_url: Option<String>,
    gallery: Option<Vec<String>>,
    featured_gallery: Option<String>,
    downloads: u64,
    #[allow(dead_code)]
    versions: Vec<String>,
    #[serde(default)]
    categories: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ModrinthVersion {
    id: String,
    #[allow(dead_code)]
    project_id: String,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    version_number: String,
    game_versions: Vec<String>,
    #[allow(dead_code)]
    loaders: Vec<String>,
    files: Vec<ModrinthFile>,
}

#[derive(Debug, Deserialize)]
struct ModrinthFile {
    url: String,
    #[allow(dead_code)]
    filename: String,
    size: u64,
    primary: bool,
}

/// Get all installed resource packs from the resourcepacks directory
#[log_result(log_values = true, max_length = 100)]
pub async fn get_installed_resourcepacks(minecraft_path: String) -> Result<Vec<ResourcePack>, String> {
    let resourcepacks_dir = PathBuf::from(minecraft_path).join("resourcepacks");

    if !resourcepacks_dir.exists() {
        return Ok(Vec::new());
    }

    let mut packs = Vec::new();

    for entry in fs::read_dir(&resourcepacks_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_path = entry.path();

        if file_path.is_file() {
            if let Some(extension) = file_path.extension() {
                if extension == "zip" {
                    if let Ok(pack) = parse_resource_pack(&file_path).await {
                        packs.push(pack);
                    }
                }
            }
        }
    }

    // Sort by name
    packs.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(packs)
}

/// Parse resource pack file
async fn parse_resource_pack(pack_path: &PathBuf) -> Result<ResourcePack, String> {
    let file_name = pack_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let file_size = fs::metadata(pack_path).map_err(|e| e.to_string())?.len();

    // Extract pack info from filename (basic approach)
    let name = extract_pack_name(&file_name);
    let version = extract_pack_version(&file_name);

    // Get installation date from file metadata
    let installed_date = if let Ok(metadata) = fs::metadata(pack_path) {
        if let Ok(created) = metadata.created() {
            if let Ok(duration) = created.duration_since(std::time::UNIX_EPOCH) {
                duration.as_secs() as i64
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    Ok(ResourcePack {
        id: file_name.clone(),
        name,
        version,
        author: "Unknown".to_string(),
        description: None,
        file_path: pack_path.to_string_lossy().to_string(),
        file_name,
        file_size,
        compatible_versions: vec!["1.20".to_string(), "1.19".to_string()],
        pack_format: 15,
        enabled: false,
        source_url: None,
        thumbnail: None,
        installed_date,
        last_used: None,
    })
}

/// Extract resource pack name from filename
fn extract_pack_name(filename: &str) -> String {
    let name_without_ext = filename.trim_end_matches(".zip");

    let version_indicators = ["_v", "_V", "-v", "-V", "_", "-"];

    for indicator in &version_indicators {
        if let Some(pos) = name_without_ext.find(indicator) {
            let potential_name = &name_without_ext[..pos];
            if !potential_name.is_empty() {
                return potential_name.replace("_", " ").replace("-", " ");
            }
        }
    }

    name_without_ext.replace("_", " ").replace("-", " ")
}

/// Extract version from filename
fn extract_pack_version(filename: &str) -> String {
    let name_without_ext = filename.trim_end_matches(".zip");

    if let Some(v_pos) = name_without_ext.find('v') {
        let after_v = &name_without_ext[v_pos + 1..];
        if let Some(space_pos) = after_v.find(' ') {
            return after_v[..space_pos].to_string();
        } else {
            return after_v.to_string();
        }
    }

    "Unknown".to_string()
}

/// Delete resource pack
#[log_result]
pub async fn delete_resourcepack(minecraft_path: String, pack_file: String) -> Result<(), String> {
    let pack_path = PathBuf::from(minecraft_path)
        .join("resourcepacks")
        .join(pack_file);

    if !pack_path.exists() {
        return Err("Resource pack file does not exist".to_string());
    }

    async_fs::remove_file(&pack_path)
        .await
        .map_err(|e| format!("Failed to delete resource pack: {}", e))?;

    Ok(())
}

/// Install resource pack from file
#[log_result]
pub async fn install_resourcepack(
    minecraft_path: String,
    pack_file_path: String,
) -> Result<String, String> {
    let source_path = PathBuf::from(pack_file_path);
    let resourcepacks_dir = PathBuf::from(minecraft_path).join("resourcepacks");

    if !source_path.exists() {
        return Err("Source resource pack file does not exist".to_string());
    }

    crate::ensure_folder(&resourcepacks_dir)
        .await
        .map_err(|e| e.to_string())?;

    let file_name = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid file name")?;

    let destination_path = resourcepacks_dir.join(file_name);

    async_fs::copy(&source_path, &destination_path)
        .await
        .map_err(|e| format!("Failed to install resource pack: {}", e))?;

    Ok(file_name.to_string())
}

/// Get resource pack info
#[log_result]
pub async fn get_resourcepack_info(
    minecraft_path: String,
    pack_file: String,
) -> Result<ResourcePack, String> {
    let pack_path = PathBuf::from(minecraft_path)
        .join("resourcepacks")
        .join(pack_file);

    if !pack_path.exists() {
        return Err("Resource pack file does not exist".to_string());
    }

    parse_resource_pack(&pack_path)
        .await
        .map_err(|e| e.to_string())
}

/// Search for resource packs on Modrinth
#[log_result(log_values = true, max_length = 100, debug_only = false)]
pub async fn search_modrinth_resourcepacks(
    query: String,
    minecraft_version: Option<String>,
    limit: u32,
    offset: u32,
) -> Result<Vec<ResourcePackDownload>, String> {
    search_modrinth_resourcepacks_with_facets(query, minecraft_version, None, limit, offset).await
}

/// Search for resource packs on Modrinth with custom filter facets
#[log_result(log_values = true, max_length = 100, debug_only = false)]
pub async fn search_modrinth_resourcepacks_with_facets(
    query: String,
    minecraft_version: Option<String>,
    facets: Option<ResourcePackFilterFacets>,
    limit: u32,
    offset: u32,
) -> Result<Vec<ResourcePackDownload>, String> {
    let client = reqwest::Client::new();
    let mut url = format!(
        "https://api.modrinth.com/v2/search?limit={}&offset={}&facets=[[\"project_type:resourcepack\"]]",
        limit, offset
    );

    // Add query if present
    if !query.is_empty() {
        let encoded = query.replace(' ', "%20").replace('&', "%26");
        url.push_str(&format!("&query={}", encoded));
    }

    // Build facets if custom filters provided
    if let Some(filter_facets) = facets {
        let facet_arrays = filter_facets.to_modrinth_facets(minecraft_version.as_deref());
        if !facet_arrays.is_empty() {
            let facets_json: Vec<String> = facet_arrays
                .iter()
                .map(|inner_array| {
                    let items: Vec<String> = inner_array
                        .iter()
                        .map(|item| format!("\"{}\"", item))
                        .collect();
                    format!("[{}]", items.join(","))
                })
                .collect();
            // Replace the default facets with our custom ones
            let facets_param = format!("&facets=[{}]", facets_json.join(","));
            // Remove default facets and add custom ones
            url = url.replace("&facets=[[\"project_type:resourcepack\"]]", &facets_param);
        }
    } else if let Some(version) = minecraft_version {
        // Simple version filter without custom facets
        url.push_str(&format!(
            "&facets=[[\"versions:{}\"]]",
            version
        ));
    }

    println!("[ModrinthAPI] Calling resourcepack search URL: {}", url);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to search Modrinth resource packs: {}", e))?;

    let search_result: ModrinthSearchResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Modrinth resourcepack search response: {}", e))?;

    let mut packs = Vec::new();
    for project in search_result.hits {
        // Get version details
        let version_url = format!(
            "https://api.modrinth.com/v2/project/{}/version",
            project.project_id
        );
        let version_response = match client.get(&version_url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                println!(
                    "[ModrinthAPI] Failed to fetch versions for resourcepack {}: {}",
                    project.project_id, e
                );
                continue;
            }
        };

        let versions: Vec<ModrinthVersion> = match version_response.json().await {
            Ok(v) => v,
            Err(e) => {
                println!(
                    "[ModrinthAPI] Failed to parse versions for resourcepack {}: {}",
                    project.project_id, e
                );
                continue;
            }
        };

        let latest_version = match versions.first() {
            Some(v) => v,
            None => {
                println!(
                    "[ModrinthAPI] No versions found for resourcepack {}",
                    project.project_id
                );
                continue;
            }
        };

        let primary_file = match latest_version
            .files
            .iter()
            .find(|f| f.primary)
            .or_else(|| latest_version.files.first())
        {
            Some(f) => f,
            None => {
                println!(
                    "[ModrinthAPI] No files found for resourcepack {} version {}",
                    project.project_id, latest_version.id
                );
                continue;
            }
        };

        // Extract resolution from categories (e.g., "16x", "32x", "64x", etc.)
        let resolution = project.categories.iter()
            .find(|cat| cat.ends_with('x') && cat.chars().take_while(|c| c.is_numeric()).count() > 0)
            .cloned();

        packs.push(ResourcePackDownload {
            id: project.project_id,
            name: project.title,
            author: project.author,
            description: project.description,
            download_url: primary_file.url.clone(),
            thumbnail: project.icon_url,
            gallery: project.gallery,
            featured_gallery: project.featured_gallery,
            tags: project.categories,
            minecraft_versions: latest_version.game_versions.clone(),
            resolution,
            rating: 0.0,
            downloads: project.downloads,
            size_mb: primary_file.size / 1024 / 1024,
            source: ResourcePackSource::Modrinth,
        });
    }

    println!("[ModrinthAPI] Received {} resource packs from API", packs.len());
    Ok(packs)
}

/// Get resource pack details from Modrinth
#[log_result(log_values = true, max_length = 100)]
pub async fn get_modrinth_resourcepack_details(project_id: String) -> Result<ResourcePackDownload, String> {
    let client = reqwest::Client::new();

    let project_url = format!("https://api.modrinth.com/v2/project/{}", project_id);
    let project_response = client
        .get(&project_url)
        .header("User-Agent", "kable-launcher")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch project: {}", e))?;

    let project: ModrinthProject = project_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse project: {}", e))?;

    let version_url = format!("https://api.modrinth.com/v2/project/{}/version", project_id);
    let version_response = client
        .get(&version_url)
        .header("User-Agent", "kable-launcher")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch versions: {}", e))?;

    let versions: Vec<ModrinthVersion> = version_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse versions: {}", e))?;

    let latest_version = versions.first().ok_or("No versions found")?;
    let primary_file = latest_version
        .files
        .iter()
        .find(|f| f.primary)
        .ok_or("No primary file found")?;

    // Extract resolution from categories
    let resolution = project.categories.iter()
        .find(|cat| cat.ends_with('x') && cat.chars().take_while(|c| c.is_numeric()).count() > 0)
        .cloned();

    Ok(ResourcePackDownload {
        id: project.project_id,
        name: project.title,
        author: project.author,
        description: project.description,
        download_url: primary_file.url.clone(),
        thumbnail: project.icon_url,
        gallery: project.gallery,
        featured_gallery: project.featured_gallery,
        tags: project.categories,
        minecraft_versions: latest_version.game_versions.clone(),
        resolution,
        rating: 0.0,
        downloads: project.downloads,
        size_mb: primary_file.size / (1024 * 1024),
        source: ResourcePackSource::Modrinth,
    })
}

/// Download and install resource pack from Modrinth
#[log_result]
pub async fn download_and_install_resourcepack(
    minecraft_path: String,
    download_url: String,
    filename: String,
) -> Result<String, String> {
    let resourcepacks_dir = PathBuf::from(&minecraft_path).join("resourcepacks");

    crate::ensure_folder(&resourcepacks_dir)
        .await
        .map_err(|e| e.to_string())?;

    let destination = resourcepacks_dir.join(&filename);

    let client = reqwest::Client::new();
    let response = client
        .get(&download_url)
        .header("User-Agent", "kable-launcher")
        .send()
        .await
        .map_err(|e| format!("Failed to download resource pack: {}", e))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read resource pack data: {}", e))?;

    async_fs::write(&destination, bytes)
        .await
        .map_err(|e| format!("Failed to write resource pack file: {}", e))?;

    Ok(filename)
}

/// Download and install resource pack from Modrinth to a dedicated folder
#[log_result]
pub async fn download_and_install_resourcepack_to_dedicated(
    _minecraft_path: String,
    dedicated_folder: String,
    download_url: String,
    filename: String,
) -> Result<String, String> {
    let kable_dir = crate::get_minecraft_kable_dir()?;
    let dedicated_path = PathBuf::from(&dedicated_folder);
    
    // Support both absolute paths and relative paths from .minecraft/kable
    let packs_dir = if dedicated_path.is_absolute() {
        dedicated_path
    } else {
        kable_dir.join(&dedicated_folder)
    };

    crate::ensure_folder(&packs_dir)
        .await
        .map_err(|e| e.to_string())?;

    let destination = packs_dir.join(&filename);

    let client = reqwest::Client::new();
    let response = client
        .get(&download_url)
        .header("User-Agent", "kable-launcher")
        .send()
        .await
        .map_err(|e| format!("Failed to download resource pack: {}", e))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read resource pack data: {}", e))?;

    async_fs::write(&destination, bytes)
        .await
        .map_err(|e| format!("Failed to write resource pack file: {}", e))?;

    Ok(filename)
}

/// Setup symbolic link from dedicated resource pack folder to .minecraft/resourcepacks
#[log_result]
pub async fn setup_resourcepack_symlink(
    minecraft_path: String,
    dedicated_folder: String,
    symlink_name: String,
) -> Result<(), String> {
    let minecraft_dir = PathBuf::from(&minecraft_path);
    let kable_dir = crate::get_minecraft_kable_dir()?;
    
    // Ensure symlinks are allowed in Minecraft
    crate::ensure_symlinks_enabled(&minecraft_dir).await?;

    let dedicated_path = PathBuf::from(&dedicated_folder);
    // Support both absolute paths and relative paths from .minecraft/kable
    let source_dir = if dedicated_path.is_absolute() {
        dedicated_path
    } else {
        kable_dir.join(&dedicated_folder)
    };

    // Ensure the dedicated folder exists
    crate::ensure_folder(&source_dir).await?;

    let target_link = minecraft_dir.join("resourcepacks").join(symlink_name);

    // Create the symlink
    crate::create_directory_symlink(&source_dir, &target_link).await?;

    Ok(())
}

/// Remove symbolic link from .minecraft/resourcepacks
pub async fn remove_resourcepack_symlink(
    minecraft_path: String,
    symlink_name: String,
) -> Result<(), String> {
    let minecraft_dir = PathBuf::from(&minecraft_path);
    let target_link = minecraft_dir.join("resourcepacks").join(symlink_name);

    crate::remove_symlink_if_exists(&target_link).await?;

    Ok(())
}

/// Delete resource pack from dedicated folder and clean up symlink
pub async fn delete_resourcepack_from_dedicated(
    minecraft_path: String,
    dedicated_folder: String,
    pack_file: String,
    symlink_name: Option<String>,
) -> Result<(), String> {
    let kable_dir = crate::get_minecraft_kable_dir()?;
    let dedicated_path = PathBuf::from(&dedicated_folder);
    
    // Support both absolute paths and relative paths from .minecraft/kable
    let packs_dir = if dedicated_path.is_absolute() {
        dedicated_path
    } else {
        kable_dir.join(&dedicated_folder)
    };

    let pack_path = packs_dir.join(&pack_file);

    if !pack_path.exists() {
        return Err("Resource pack file does not exist".to_string());
    }

    async_fs::remove_file(&pack_path)
        .await
        .map_err(|e| format!("Failed to delete resource pack: {}", e))?;

    // If a symlink name was provided, try to remove it
    if let Some(link_name) = symlink_name {
        let _ = remove_resourcepack_symlink(minecraft_path, link_name).await;
    }

    Ok(())
}
