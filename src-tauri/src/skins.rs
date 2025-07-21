use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::AppError;

// Skin management structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinecraftSkin {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub file_name: String,
    pub is_slim: bool, // Alex or Steve model
    pub preview_url: Option<String>,
    pub source: SkinSource,
    pub created_date: i64,
    pub last_used: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SkinSource {
    Local,
    Mojang,
    Custom,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkinDownload {
    pub id: String,
    pub name: String,
    pub author: String,
    pub download_url: String,
    pub preview_url: Option<String>,
    pub tags: Vec<String>,
    pub is_slim: bool,
    pub rating: f32,
    pub downloads: u64,
    pub source: SkinDownloadSource,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SkinDownloadSource {
    NameMC,
    MinecraftSkins,
    Other(String),
}

// Get all local skins from the launcher directory
#[tauri::command]
pub async fn get_local_skins() -> Result<Vec<MinecraftSkin>, String> {
    let launcher_dir = crate::get_kable_launcher_dir().map_err(|e| e.to_string())?;
    let skins_dir = launcher_dir.join("skins");
    
    if !skins_dir.exists() {
        fs::create_dir_all(&skins_dir).map_err(|e| e.to_string())?;
        return Ok(Vec::new());
    }
    
    let mut skins = Vec::new();
    
    for entry in fs::read_dir(&skins_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_path = entry.path();
        
        if file_path.is_file() {
            if let Some(extension) = file_path.extension() {
                if extension == "png" {
                    if let Ok(skin) = parse_skin_file(&file_path).await {
                        skins.push(skin);
                    }
                }
            }
        }
    }
    
    // Sort by name
    skins.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(skins)
}

// Parse skin file
async fn parse_skin_file(skin_path: &PathBuf) -> Result<MinecraftSkin, AppError> {
    let file_name = skin_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    let name = extract_skin_name(&file_name);
    let is_slim = detect_slim_model(skin_path).unwrap_or(false);
    
    // Get creation date from file metadata
    let created_date = if let Ok(metadata) = fs::metadata(skin_path) {
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
    
    Ok(MinecraftSkin {
        id: file_name.clone(),
        name,
        file_path: skin_path.to_string_lossy().to_string(),
        file_name,
        is_slim,
        preview_url: None,
        source: SkinSource::Local,
        created_date,
        last_used: None,
    })
}

// Extract skin name from filename
fn extract_skin_name(filename: &str) -> String {
    // Remove file extension
    let name_without_ext = filename.trim_end_matches(".png");
    
    // Replace underscores and hyphens with spaces
    name_without_ext.replace("_", " ").replace("-", " ")
}

// Detect if skin uses slim (Alex) model
fn detect_slim_model(skin_path: &std::path::Path) -> Result<bool, AppError> {
    // This is a simplified implementation. 
    // In reality, you'd analyze the skin texture to determine if it's slim or classic model
    // For now, we'll use filename heuristics
    let filename = skin_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    // Look for indicators in filename
    let slim_indicators = ["alex", "slim", "thin"];
    let classic_indicators = ["steve", "classic", "wide"];
    
    for indicator in &slim_indicators {
        if filename.contains(indicator) {
            return Ok(true);
        }
    }
    
    for indicator in &classic_indicators {
        if filename.contains(indicator) {
            return Ok(false);
        }
    }
    
    // Default to classic model if unknown
    Ok(false)
}

// Save skin to local storage
#[tauri::command]
pub async fn save_skin(skin_data: Vec<u8>, skin_name: String, is_slim: bool) -> Result<String, String> {
    let launcher_dir = crate::get_kable_launcher_dir().map_err(|e| e.to_string())?;
    let skins_dir = launcher_dir.join("skins");
    
    // Create skins directory if it doesn't exist
    if !skins_dir.exists() {
        fs::create_dir_all(&skins_dir).map_err(|e| e.to_string())?;
    }
    
    // Create filename with model type indicator
    let model_suffix = if is_slim { "_alex" } else { "_steve" };
    let file_name = format!("{}{}.png", skin_name.replace(" ", "_"), model_suffix);
    let skin_path = skins_dir.join(&file_name);
    
    // Write skin data to file
    fs::write(&skin_path, skin_data).map_err(|e| format!("Failed to save skin: {}", e))?;
    
    Ok(file_name)
}

// Delete skin from local storage
#[tauri::command]
pub async fn delete_skin(skin_file: String) -> Result<(), String> {
    let launcher_dir = crate::get_kable_launcher_dir().map_err(|e| e.to_string())?;
    let skin_path = launcher_dir.join("skins").join(skin_file);
    
    if !skin_path.exists() {
        return Err("Skin file does not exist".to_string());
    }
    
    fs::remove_file(&skin_path).map_err(|e| format!("Failed to delete skin: {}", e))?;
    
    Ok(())
}

// Install skin from file path
#[tauri::command]
pub async fn install_skin(skin_file_path: String, skin_name: Option<String>) -> Result<String, String> {
    let source_path = PathBuf::from(skin_file_path);
    
    if !source_path.exists() {
        return Err("Source skin file does not exist".to_string());
    }
    
    // Read skin data
    let skin_data = fs::read(&source_path).map_err(|e| format!("Failed to read skin file: {}", e))?;
    
    // Validate that it's a PNG file (basic check)
    if skin_data.len() < 8 || &skin_data[1..4] != b"PNG" {
        return Err("Invalid PNG file".to_string());
    }
    
    // Determine skin name
    let name = if let Some(custom_name) = skin_name {
        custom_name
    } else {
        source_path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("Custom Skin")
            .to_string()
    };
    
    // Detect model type
    let is_slim = detect_slim_model(&source_path).unwrap_or(false);
    
    // Save skin
    save_skin(skin_data, name, is_slim).await
}

// Get skin file data for preview/use
#[tauri::command]
pub async fn get_skin_data(skin_file: String) -> Result<Vec<u8>, String> {
    let launcher_dir = crate::get_kable_launcher_dir().map_err(|e| e.to_string())?;
    let skin_path = launcher_dir.join("skins").join(skin_file);
    
    if !skin_path.exists() {
        return Err("Skin file does not exist".to_string());
    }
    
    fs::read(&skin_path).map_err(|e| format!("Failed to read skin file: {}", e))
}

// Get current Minecraft skin from account
#[tauri::command]
pub async fn get_current_minecraft_skin(access_token: String, uuid: String) -> Result<Option<String>, String> {
    // This would make an API call to Mojang to get the current skin
    // For now, return None as a placeholder
    let _ = (access_token, uuid); // Suppress unused variable warnings
    Ok(None)
}

// Upload skin to Minecraft account
#[tauri::command]
pub async fn upload_skin_to_minecraft(access_token: String, skin_file: String, is_slim: bool) -> Result<(), String> {
    // This would make an API call to Mojang to upload the skin
    // For now, this is a placeholder implementation
    let _ = (access_token, skin_file, is_slim); // Suppress unused variable warnings
    
    // In a real implementation, you would:
    // 1. Get the skin data from the local file
    // 2. Make a POST request to https://api.mojang.com/user/profile/{uuid}/skin
    // 3. Include the access token in Authorization header
    // 4. Send the skin file data in the request body
    
    Err("Skin upload not implemented yet".to_string())
}

// Select skin file using file dialog
#[tauri::command]
pub async fn select_skin_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    
    let file_path = app.dialog()
        .file()
        .add_filter("PNG Images", &["png"])
        .set_title("Select Skin File")
        .blocking_pick_file();
    
    match file_path {
        Some(path) => {
            match path.as_path() {
                Some(path_buf) => Ok(Some(path_buf.to_string_lossy().to_string())),
                None => Err("Invalid file path".to_string()),
            }
        },
        None => Ok(None), // User cancelled
    }
}
