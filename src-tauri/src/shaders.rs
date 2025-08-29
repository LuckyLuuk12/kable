use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// Shader management structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShaderPack {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: Option<String>,
    pub file_path: String,
    pub file_name: String,
    pub file_size: u64,
    pub compatible_versions: Vec<String>,
    pub enabled: bool,
    pub source_url: Option<String>,
    pub thumbnail: Option<String>,
    pub shader_loader: ShaderLoader,
    pub installed_date: i64,
    pub last_used: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ShaderLoader {
    OptiFine,
    Iris,
    Sodium,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShaderSettings {
    pub quality: ShaderQuality,
    pub shadows: bool,
    pub shadow_resolution: u32,
    pub anti_aliasing: bool,
    pub bloom: bool,
    pub motion_blur: bool,
    pub custom_settings: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ShaderQuality {
    Low,
    Medium,
    High,
    Ultra,
    Custom,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShaderDownload {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub download_url: String,
    pub thumbnail: Option<String>,
    pub tags: Vec<String>,
    pub minecraft_versions: Vec<String>,
    pub shader_loader: ShaderLoader,
    pub rating: f32,
    pub downloads: u64,
    pub size_mb: u64,
    pub source: ShaderSource,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ShaderSource {
    Modrinth,
    CurseForge,
    Other(String),
}

// Get all installed shaders from the shaderpacks directory
#[tauri::command]
pub async fn get_installed_shaders(minecraft_path: String) -> Result<Vec<ShaderPack>, String> {
    let shaderpacks_dir = PathBuf::from(minecraft_path).join("shaderpacks");

    if !shaderpacks_dir.exists() {
        return Ok(Vec::new());
    }

    let mut shaders = Vec::new();

    for entry in fs::read_dir(&shaderpacks_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_path = entry.path();

        if file_path.is_file() {
            if let Some(extension) = file_path.extension() {
                if extension == "zip" || extension == "jar" {
                    if let Ok(shader) = parse_shader_pack(&file_path).await {
                        shaders.push(shader);
                    }
                }
            }
        }
    }

    // Sort by name
    shaders.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(shaders)
}

// Parse shader pack file
async fn parse_shader_pack(shader_path: &PathBuf) -> Result<ShaderPack, String> {
    let file_name = shader_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let file_size = fs::metadata(shader_path).map_err(|e| e.to_string())?.len();

    // Extract shader info from filename (basic approach)
    let name = extract_shader_name(&file_name);
    let version = extract_shader_version(&file_name);

    // Get installation date from file metadata
    let installed_date = if let Ok(metadata) = fs::metadata(shader_path) {
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

    Ok(ShaderPack {
        id: file_name.clone(),
        name,
        version,
        author: "Unknown".to_string(),
        description: None,
        file_path: shader_path.to_string_lossy().to_string(),
        file_name,
        file_size,
        compatible_versions: vec!["1.20".to_string(), "1.19".to_string()], // Default compatibility
        enabled: false, // Would need to check against options.txt or similar
        source_url: None,
        thumbnail: None,
        shader_loader: ShaderLoader::OptiFine, // Default to OptiFine
        installed_date,
        last_used: None,
    })
}

// Extract shader name from filename
fn extract_shader_name(filename: &str) -> String {
    // Remove file extension
    let name_without_ext = filename.trim_end_matches(".zip").trim_end_matches(".jar");

    // Try to extract name before version indicators
    let version_indicators = ["_v", "_V", "-v", "-V", "_", "-"];

    for indicator in &version_indicators {
        if let Some(pos) = name_without_ext.find(indicator) {
            let potential_name = &name_without_ext[..pos];
            if !potential_name.is_empty() {
                return potential_name.replace("_", " ").replace("-", " ");
            }
        }
    }

    // If no version indicator found, use the whole name
    name_without_ext.replace("_", " ").replace("-", " ")
}

// Extract version from filename
fn extract_shader_version(filename: &str) -> String {
    let name_without_ext = filename.trim_end_matches(".zip").trim_end_matches(".jar");

    // Simple version extraction - look for patterns like v1.0, 1.0, etc.
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

// Enable/disable shader pack
#[tauri::command]
pub async fn toggle_shader(
    minecraft_path: String,
    shader_file: String,
    enabled: bool,
) -> Result<(), String> {
    let options_file = PathBuf::from(minecraft_path).join("optionsshaders.txt");

    // This is a simplified implementation
    // In reality, you'd need to properly parse and modify the OptiFine shaders config
    if enabled {
        let content = format!("shaderPack={}\n", shader_file);
        fs::write(&options_file, content).map_err(|e| e.to_string())?;
    } else {
        let content = "shaderPack=\n";
        fs::write(&options_file, content).map_err(|e| e.to_string())?;
    }

    Ok(())
}

// Delete shader pack
#[tauri::command]
pub async fn delete_shader(minecraft_path: String, shader_file: String) -> Result<(), String> {
    let shader_path = PathBuf::from(minecraft_path)
        .join("shaderpacks")
        .join(shader_file);

    if !shader_path.exists() {
        return Err("Shader file does not exist".to_string());
    }

    fs::remove_file(&shader_path).map_err(|e| format!("Failed to delete shader: {}", e))?;

    Ok(())
}

// Install shader pack from file
#[tauri::command]
pub async fn install_shader_pack(
    minecraft_path: String,
    shader_file_path: String,
) -> Result<String, String> {
    let source_path = PathBuf::from(shader_file_path);
    let shaderpacks_dir = PathBuf::from(minecraft_path).join("shaderpacks");

    if !source_path.exists() {
        return Err("Source shader file does not exist".to_string());
    }

    // Create shaderpacks directory if it doesn't exist
    if !shaderpacks_dir.exists() {
        fs::create_dir_all(&shaderpacks_dir).map_err(|e| e.to_string())?;
    }

    let file_name = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid file name")?;

    let destination_path = shaderpacks_dir.join(file_name);

    // Copy shader pack to shaderpacks directory
    fs::copy(&source_path, &destination_path)
        .map_err(|e| format!("Failed to install shader: {}", e))?;

    Ok(file_name.to_string())
}

// Get shader pack info
#[tauri::command]
pub async fn get_shader_info(
    minecraft_path: String,
    shader_file: String,
) -> Result<ShaderPack, String> {
    let shader_path = PathBuf::from(minecraft_path)
        .join("shaderpacks")
        .join(shader_file);

    if !shader_path.exists() {
        return Err("Shader file does not exist".to_string());
    }

    parse_shader_pack(&shader_path)
        .await
        .map_err(|e| e.to_string())
}
