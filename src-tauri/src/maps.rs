use crate::logging::{debug, error, info};
use fastnbt::from_bytes;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use tokio::fs as async_fs;

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
#[derive(Debug, Deserialize)]
struct LevelDatRoot {
    #[serde(rename = "Data")]
    pub data: Option<LevelData>,
}

#[derive(Debug, Deserialize)]
struct LevelData {
    #[serde(rename = "LevelName")]
    pub level_name: Option<String>,
    #[serde(rename = "GameType")]
    pub game_type: Option<i32>,
    #[serde(rename = "Difficulty")]
    pub difficulty: Option<i8>,
    #[serde(rename = "LastPlayed")]
    pub last_played: Option<i64>,
    #[serde(rename = "RandomSeed")]
    pub random_seed: Option<i64>,
    #[serde(rename = "allowCommands")]
    pub allow_commands: Option<i8>,
    #[serde(rename = "generatorName")]
    pub generator_name: Option<String>,
    #[serde(rename = "Version")]
    pub version: Option<VersionData>,
}

#[derive(Debug, Deserialize)]
struct VersionData {
    #[serde(rename = "Name")]
    pub name: Option<String>,
}

// Get all local worlds from the saves directory
#[tauri::command]
pub async fn get_local_worlds(minecraft_path: String) -> Result<Vec<LocalWorld>, String> {
    let saves_dir = PathBuf::from(&minecraft_path).join("saves");

    if !saves_dir.exists() {
        return Ok(Vec::new());
    }

    let mut worlds = Vec::new();

    let mut dir = async_fs::read_dir(&saves_dir)
        .await
        .map_err(|e| e.to_string())?;
    while let Some(entry) = dir.next_entry().await.map_err(|e| e.to_string())? {
        let world_path = entry.path();
        if world_path.is_dir() {
            if let Ok(world) = parse_world_folder(&world_path, &minecraft_path).await {
                worlds.push(world);
            }
        }
    }

    // Sort by last played (most recent first)
    worlds.sort_by(|a, b| b.last_played.cmp(&a.last_played));

    Ok(worlds)
}

// Parse individual world folder
async fn parse_world_folder(
    world_path: &PathBuf,
    minecraft_path: &str,
) -> Result<LocalWorld, String> {
    let folder_name = world_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let level_dat = world_path.join("level.dat");
    let icon_path = world_path.join("icon.png");

    // Calculate world size using blocking recursive helper in a spawned blocking task
    let path_clone = world_path.clone();
    let size_bytes: u64 = tokio::task::spawn_blocking(move || calculate_folder_size(&path_clone))
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or(0);
    let size_mb = size_bytes / (1024 * 1024);

    // Count backups for this world (async)
    let backup_count = count_world_backups_async(minecraft_path, &folder_name).await;

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
        icon: if icon_path.exists() {
            Some(icon_path.to_string_lossy().to_string())
        } else {
            None
        },
        backup_count,
        has_cheats: false,
        world_type: "default".to_string(),
    };

    // Try to read level.dat for more information
    if level_dat.exists() {
        if let Ok(level_data) = parse_level_dat_async(&level_dat).await {
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
                world.has_cheats = cheats != 0;
            }

            if let Some(generator) = level_data.generator_name {
                world.world_type = generator;
            }

            if let Some(version_data) = level_data.version {
                if let Some(version_str) = version_data.name {
                    world.version = version_str;
                }
            }
        }
    }

    // Set creation time from folder metadata if not available
    if world.created == 0 {
        if let Ok(metadata) = async_fs::metadata(world_path).await {
            if let Ok(created) = metadata.created() {
                if let Ok(duration) = created.duration_since(std::time::UNIX_EPOCH) {
                    world.created = duration.as_secs() as i64;
                }
            }
        }
    }

    // Count backups for this world (async)
    world.backup_count = count_world_backups_async(minecraft_path, &folder_name).await;

    Ok(world)
}

// NOTE: removed legacy synchronous `parse_level_dat` in favor of the
// async variant `parse_level_dat_async` above. Keeping only the async
// parser avoids unused code and encourages non-blocking file reads.

// Async variant of the level.dat parser - reads and parses NBT data
async fn parse_level_dat_async(level_dat_path: &std::path::Path) -> Result<LevelData, String> {
    // Read the file in a blocking task since NBT parsing is CPU-bound
    let path = level_dat_path.to_path_buf();

    tokio::task::spawn_blocking(move || {
        // Read the gzipped NBT file
        let file = fs::File::open(&path).map_err(|e| format!("Failed to open level.dat: {}", e))?;

        let mut decoder = GzDecoder::new(file);
        let mut contents = Vec::new();
        decoder
            .read_to_end(&mut contents)
            .map_err(|e| format!("Failed to decompress level.dat: {}", e))?;

        // Parse NBT data
        let root: LevelDatRoot =
            from_bytes(&contents).map_err(|e| format!("Failed to parse NBT data: {}", e))?;

        root.data
            .ok_or_else(|| "No Data tag found in level.dat".to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
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

// (Removed async recursive folder-size helper; use blocking `calculate_folder_size` inside spawn_blocking)

// Delete a world
#[tauri::command]
pub async fn delete_world(minecraft_path: String, world_folder: String) -> Result<(), String> {
    info(&format!(
        "Deleting world: {} from {}",
        world_folder, minecraft_path
    ));

    let world_path = PathBuf::from(minecraft_path)
        .join("saves")
        .join(&world_folder);

    if !world_path.exists() {
        let error_msg = "World folder does not exist".to_string();
        error(&error_msg);
        return Err(error_msg);
    }

    match async_fs::remove_dir_all(&world_path).await {
        Ok(_) => {
            info(&format!("Successfully deleted world: {}", world_folder));
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Failed to delete world: {}", e);
            error(&error_msg);
            Err(error_msg)
        }
    }
}

// Create backup of a world
#[tauri::command]
pub async fn backup_world(minecraft_path: String, world_folder: String) -> Result<String, String> {
    info(&format!(
        "Creating backup for world: {} from {}",
        world_folder, minecraft_path
    ));

    let minecraft_dir = PathBuf::from(minecraft_path);
    let saves_dir = minecraft_dir.join("saves");
    let world_path = saves_dir.join(&world_folder);

    // Use .minecraft/kable/world-backups for backup storage
    let kable_dir = minecraft_dir.join("kable");
    let backups_dir = kable_dir.join("world-backups");

    if !world_path.exists() {
        let error_msg = "World folder does not exist".to_string();
        error(&error_msg);
        return Err(error_msg);
    }

    // Ensure kable and backups directories exist using centralized helper
    debug(&format!(
        "Ensuring backups directory exists: {}",
        backups_dir.display()
    ));
    if let Err(e) = crate::ensure_folder(&backups_dir).await {
        let error_msg = format!("Failed to create backups directory: {}", e);
        error(&error_msg);
        return Err(error_msg);
    }

    // Create backup name with timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}_{}", world_folder, timestamp);
    let backup_path = backups_dir.join(&backup_name);

    debug(&format!(
        "Copying world from {} to {}",
        world_path.display(),
        backup_path.display()
    ));

    // Copy world folder to backup location
    match copy_dir_all_async(world_path.as_path(), backup_path.as_path()).await {
        Ok(_) => {
            info(&format!("Successfully created backup: {}", backup_name));
            Ok(backup_name)
        }
        Err(e) => {
            let error_msg = format!("Failed to create backup: {}", e);
            error(&error_msg);
            Err(error_msg)
        }
    }
}

// Helper function to count backups for a world
async fn count_world_backups_async(minecraft_path: &str, world_folder: &str) -> u32 {
    let minecraft_dir = PathBuf::from(minecraft_path);
    let backups_dir = minecraft_dir.join("kable").join("world-backups");

    if !backups_dir.exists() {
        return 0;
    }

    let mut count = 0u32;
    let mut dir = match async_fs::read_dir(&backups_dir).await {
        Ok(d) => d,
        Err(_) => return 0,
    };
    while let Ok(Some(entry)) = dir.next_entry().await {
        if let Some(file_name) = entry.file_name().to_str() {
            if file_name.starts_with(&format!("{}_", world_folder)) {
                count += 1;
            }
        }
    }
    count
}

// Helper function to copy directory recursively
async fn copy_dir_all_async(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    // For recursive copying, delegate to a blocking task that uses std::fs to avoid
    // recursive async functions which require boxing.
    let src = src.to_path_buf();
    let dst = dst.to_path_buf();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        fn copy_sync(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
            crate::ensure_folder_sync(dst).map_err(|e| e.to_string())?;
            for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                let out_path = dst.join(entry.file_name());
                if path.is_dir() {
                    copy_sync(&path, &out_path)?;
                } else {
                    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
                    if let Some(parent) = out_path.parent() {
                        crate::ensure_folder_sync(parent).map_err(|e| e.to_string())?;
                    }
                    crate::write_file_atomic_sync(&out_path, &bytes)
                        .map_err(|e| format!("atomic write failed: {}", e))?;
                }
            }
            Ok(())
        }
        copy_sync(&src, &dst)
    })
    .await
    .map_err(|e| format!("spawn_blocking failed: {}", e))??;
    Ok(())
}
