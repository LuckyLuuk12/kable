use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::AppError;

// Map/World management structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorldDownload {
    pub id: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub download_url: String,
    pub thumbnail: Option<String>,
    pub tags: Vec<String>,
    pub minecraft_version: String,
    pub size_mb: u64,
    pub rating: f32,
    pub downloads: u64,
    pub source: WorldSource,
    pub created_date: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WorldSource {
    PlanetMinecraft,
    MinecraftMaps,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalWorld {
    pub id: String,
    pub name: String,
    pub folder_name: String,
    pub game_mode: GameMode,
    pub difficulty: Difficulty,
    pub version: String,
    pub size_mb: u64,
    pub last_played: i64,
    pub created: i64,
    pub seed: Option<String>,
    pub icon: Option<String>,
    pub backup_count: u32,
    pub has_cheats: bool,
    pub world_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

// NBT data structure for level.dat parsing
#[derive(Debug, Serialize, Deserialize)]
struct LevelData {
    #[serde(rename = "LevelName")]
    pub level_name: Option<String>,
    #[serde(rename = "GameType")]
    pub game_type: Option<i32>,
    #[serde(rename = "Difficulty")]
    pub difficulty: Option<i32>,
    #[serde(rename = "LastPlayed")]
    pub last_played: Option<i64>,
    #[serde(rename = "RandomSeed")]
    pub random_seed: Option<i64>,
    #[serde(rename = "allowCommands")]
    pub allow_commands: Option<bool>,
    #[serde(rename = "generatorName")]
    pub generator_name: Option<String>,
    #[serde(rename = "Version")]
    pub version: Option<HashMap<String, serde_json::Value>>,
}

// Get all local worlds from the saves directory
#[tauri::command]
pub async fn get_local_worlds(minecraft_path: String) -> Result<Vec<LocalWorld>, String> {
    let saves_dir = PathBuf::from(minecraft_path).join("saves");
    
    if !saves_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut worlds = Vec::new();
    
    for entry in fs::read_dir(&saves_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let world_path = entry.path();
        
        if world_path.is_dir() {
            if let Ok(world) = parse_world_folder(&world_path).await {
                worlds.push(world);
            }
        }
    }
    
    // Sort by last played (most recent first)
    worlds.sort_by(|a, b| b.last_played.cmp(&a.last_played));
    
    Ok(worlds)
}

// Parse individual world folder
async fn parse_world_folder(world_path: &PathBuf) -> Result<LocalWorld, AppError> {
    let folder_name = world_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    let level_dat = world_path.join("level.dat");
    let icon_path = world_path.join("icon.png");
    
    // Calculate world size
    let size_mb = calculate_folder_size(world_path).unwrap_or(0) / (1024 * 1024);
    
    // Default values
    let mut world = LocalWorld {
        id: folder_name.clone(),
        name: folder_name.clone(),
        folder_name: folder_name.clone(),
        game_mode: GameMode::Survival,
        difficulty: Difficulty::Normal,
        version: "Unknown".to_string(),
        size_mb,
        last_played: 0,
        created: 0,
        seed: None,
        icon: if icon_path.exists() { Some(icon_path.to_string_lossy().to_string()) } else { None },
        backup_count: 0,
        has_cheats: false,
        world_type: "default".to_string(),
    };
    
    // Try to read level.dat for more information
    if level_dat.exists() {
        if let Ok(level_data) = parse_level_dat(&level_dat) {
            if let Some(name) = level_data.level_name {
                world.name = name;
            }
            
            if let Some(game_type) = level_data.game_type {
                world.game_mode = match game_type {
                    0 => GameMode::Survival,
                    1 => GameMode::Creative,
                    2 => GameMode::Adventure,
                    3 => GameMode::Spectator,
                    _ => GameMode::Survival,
                };
            }
            
            if let Some(difficulty) = level_data.difficulty {
                world.difficulty = match difficulty {
                    0 => Difficulty::Peaceful,
                    1 => Difficulty::Easy,
                    2 => Difficulty::Normal,
                    3 => Difficulty::Hard,
                    _ => Difficulty::Normal,
                };
            }
            
            if let Some(last_played) = level_data.last_played {
                world.last_played = last_played;
            }
            
            if let Some(seed) = level_data.random_seed {
                world.seed = Some(seed.to_string());
            }
            
            if let Some(cheats) = level_data.allow_commands {
                world.has_cheats = cheats;
            }
            
            if let Some(generator) = level_data.generator_name {
                world.world_type = generator;
            }
            
            if let Some(version_data) = level_data.version {
                if let Some(version_name) = version_data.get("Name") {
                    if let Some(version_str) = version_name.as_str() {
                        world.version = version_str.to_string();
                    }
                }
            }
        }
    }
    
    // Set creation time from folder metadata if not available
    if world.created == 0 {
        if let Ok(metadata) = fs::metadata(world_path) {
            if let Ok(created) = metadata.created() {
                if let Ok(duration) = created.duration_since(std::time::UNIX_EPOCH) {
                    world.created = duration.as_secs() as i64;
                }
            }
        }
    }
    
    Ok(world)
}

// Parse level.dat file (simplified NBT parsing)
fn parse_level_dat(level_dat_path: &PathBuf) -> Result<LevelData, AppError> {
    // This is a simplified implementation. In a real scenario, you'd want to use
    // a proper NBT library like `nbt` crate for parsing Minecraft's NBT format
    
    // For now, we'll try to read basic information from the file
    // This is a placeholder implementation that would need proper NBT parsing
    let _contents = fs::read(level_dat_path)?;
    
    // Return default data for now - in real implementation, parse NBT
    Ok(LevelData {
        level_name: None,
        game_type: None,
        difficulty: None,
        last_played: None,
        random_seed: None,
        allow_commands: None,
        generator_name: None,
        version: None,
    })
}

// Calculate folder size recursively
fn calculate_folder_size(path: &PathBuf) -> Result<u64, std::io::Error> {
    let mut size = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            size += entry.metadata()?.len();
        } else if path.is_dir() {
            size += calculate_folder_size(&path)?;
        }
    }
    Ok(size)
}

// Delete a world
#[tauri::command]
pub async fn delete_world(minecraft_path: String, world_folder: String) -> Result<(), String> {
    let world_path = PathBuf::from(minecraft_path).join("saves").join(world_folder);
    
    if !world_path.exists() {
        return Err("World folder does not exist".to_string());
    }
    
    fs::remove_dir_all(&world_path).map_err(|e| format!("Failed to delete world: {}", e))?;
    
    Ok(())
}

// Create backup of a world
#[tauri::command]
pub async fn backup_world(minecraft_path: String, world_folder: String) -> Result<String, String> {
    let saves_dir = PathBuf::from(minecraft_path).join("saves");
    let world_path = saves_dir.join(&world_folder);
    let backups_dir = saves_dir.join("backups");
    
    if !world_path.exists() {
        return Err("World folder does not exist".to_string());
    }
    
    // Create backups directory if it doesn't exist
    if !backups_dir.exists() {
        fs::create_dir_all(&backups_dir).map_err(|e| e.to_string())?;
    }
    
    // Create backup name with timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}_{}", world_folder, timestamp);
    let backup_path = backups_dir.join(&backup_name);
    
    // Copy world folder to backup location
    copy_dir_all(&world_path, &backup_path).map_err(|e| format!("Failed to create backup: {}", e))?;
    
    Ok(backup_name)
}

// Helper function to copy directory recursively
fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<(), std::io::Error> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
