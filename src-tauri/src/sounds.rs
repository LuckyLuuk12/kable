use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundpackMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub sounds: HashMap<String, String>,
    pub music: Option<HashMap<String, Vec<String>>>,
}

/// Get the sounds configuration directory
async fn get_sounds_dir() -> Result<PathBuf, String> {
    let launcher_dir = crate::get_kable_launcher_dir()?;
    Ok(launcher_dir.join("config").join("sounds"))
}

/// Ensure the sounds directory exists
async fn ensure_sounds_dir() -> Result<PathBuf, String> {
    let sounds_dir = get_sounds_dir().await?;
    // Use centralized helper to ensure the directory exists and return the path
    match crate::ensure_folder(&sounds_dir).await {
        Ok(p) => Ok(p),
        Err(err) => Err(format!("Failed to ensure sounds directory exists: {}", err)),
    }
}

/// List all available soundpacks
pub async fn list_soundpacks() -> Result<Vec<String>, String> {
    let soundpacks_dir = ensure_sounds_dir().await?;

    let mut packs = vec!["default".to_string()];

    if soundpacks_dir.exists() {
        let entries = fs::read_dir(&soundpacks_dir)
            .map_err(|e| format!("Failed to read soundpacks directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Check if it has a soundpack.json
                if path.join("soundpack.json").exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        packs.push(name.to_string());
                    }
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("zip") {
                // Check if zip contains soundpack.json
                if let Ok(file) = fs::File::open(&path) {
                    if let Ok(mut archive) = ZipArchive::new(file) {
                        if archive.by_name("soundpack.json").is_ok() {
                            if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                                packs.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(packs)
}

/// Get metadata for a specific soundpack
pub async fn get_soundpack_metadata(pack: String) -> Result<SoundpackMetadata, String> {
    // Check for default soundpack
    if pack == "default" {
        return Ok(get_default_soundpack_metadata());
    }

    let soundpacks_dir = ensure_sounds_dir().await?;
    let pack_path = soundpacks_dir.join(&pack);

    // Try as directory first
    if pack_path.is_dir() {
        let metadata_path = pack_path.join("soundpack.json");
        let content = fs::read_to_string(&metadata_path)
            .map_err(|e| format!("Failed to read soundpack.json: {}", e))?;

        let metadata: SoundpackMetadata = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse soundpack.json: {}", e))?;

        return Ok(metadata);
    }

    // Try as ZIP file
    let zip_path = soundpacks_dir.join(format!("{}.zip", pack));
    if zip_path.exists() {
        let file =
            fs::File::open(&zip_path).map_err(|e| format!("Failed to open ZIP file: {}", e))?;

        let mut archive =
            ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

        let mut metadata_file = archive
            .by_name("soundpack.json")
            .map_err(|e| format!("soundpack.json not found in ZIP: {}", e))?;

        let mut content = String::new();
        std::io::Read::read_to_string(&mut metadata_file, &mut content)
            .map_err(|e| format!("Failed to read soundpack.json from ZIP: {}", e))?;

        let metadata: SoundpackMetadata = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse soundpack.json: {}", e))?;

        return Ok(metadata);
    }

    Err(format!("Soundpack not found: {}", pack))
}

/// Load a sound file from a soundpack
pub async fn load_soundpack_file(pack: String, file: String) -> Result<Vec<u8>, String> {
    // Check for default soundpack - return empty (will use built-in browser sounds)
    if pack == "default" {
        return Err("Default soundpack files should be bundled in frontend".to_string());
    }

    let soundpacks_dir = ensure_sounds_dir().await?;
    let pack_path = soundpacks_dir.join(&pack);

    // Try as directory first
    if pack_path.is_dir() {
        let file_path = pack_path.join(&file);
        let data = fs::read(&file_path)
            .map_err(|e| format!("Failed to read sound file {}: {}", file, e))?;
        return Ok(data);
    }

    // Try as ZIP file
    let zip_path = soundpacks_dir.join(format!("{}.zip", pack));
    if zip_path.exists() {
        let file_handle =
            fs::File::open(&zip_path).map_err(|e| format!("Failed to open ZIP file: {}", e))?;

        let mut archive = ZipArchive::new(file_handle)
            .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

        let mut sound_file = archive
            .by_name(&file)
            .map_err(|e| format!("Sound file {} not found in ZIP: {}", file, e))?;

        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut sound_file, &mut data)
            .map_err(|e| format!("Failed to read sound file from ZIP: {}", e))?;

        return Ok(data);
    }

    Err(format!("Sound file not found: {}", file))
}

/// Import a soundpack from a ZIP file
pub async fn import_soundpack_zip(path: String) -> Result<String, String> {
    let soundpacks_dir = ensure_sounds_dir().await?;
    let source_path = PathBuf::from(&path);

    if !source_path.exists() {
        return Err("Source ZIP file does not exist".to_string());
    }

    // Open and validate the ZIP
    let file =
        fs::File::open(&source_path).map_err(|e| format!("Failed to open ZIP file: {}", e))?;

    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    // Check for soundpack.json
    let mut metadata_file = archive
        .by_name("soundpack.json")
        .map_err(|_| "ZIP file does not contain soundpack.json".to_string())?;

    let mut content = String::new();
    std::io::Read::read_to_string(&mut metadata_file, &mut content)
        .map_err(|e| format!("Failed to read soundpack.json: {}", e))?;

    let metadata: SoundpackMetadata =
        serde_json::from_str(&content).map_err(|e| format!("Invalid soundpack.json: {}", e))?;

    let pack_name = metadata.name.clone();

    // Copy ZIP to soundpacks directory
    let dest_path = soundpacks_dir.join(format!("{}.zip", pack_name));
    fs::copy(&source_path, &dest_path).map_err(|e| format!("Failed to copy soundpack: {}", e))?;

    Ok(pack_name)
}

/// Get the sounds directory path for frontend file operations
pub async fn get_sounds_directory_path() -> Result<String, String> {
    let sounds_dir = ensure_sounds_dir().await?;
    Ok(sounds_dir.to_string_lossy().to_string())
}

/// Open the sounds directory in the system file explorer
pub async fn open_sounds_directory() -> Result<(), String> {
    let sounds_dir = ensure_sounds_dir().await?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&sounds_dir)
            .spawn()
            .map_err(|e| format!("Failed to open sounds directory: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&sounds_dir)
            .spawn()
            .map_err(|e| format!("Failed to open sounds directory: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&sounds_dir)
            .spawn()
            .map_err(|e| format!("Failed to open sounds directory: {}", e))?;
    }

    Ok(())
}

/// Get the default soundpack metadata (built-in sounds)
fn get_default_soundpack_metadata() -> SoundpackMetadata {
    let mut sounds = HashMap::new();
    sounds.insert("click".to_string(), "click.mp3".to_string());
    sounds.insert("hover".to_string(), "hover.mp3".to_string());
    sounds.insert("success".to_string(), "success.mp3".to_string());
    sounds.insert("error".to_string(), "error.mp3".to_string());
    sounds.insert("notification".to_string(), "notification.mp3".to_string());
    sounds.insert("launch".to_string(), "launch.mp3".to_string());

    let mut music = HashMap::new();
    music.insert(
        "menu".to_string(),
        vec!["music/menu1.mp3".to_string(), "music/menu2.mp3".to_string()],
    );

    SoundpackMetadata {
        name: "default".to_string(),
        version: "1.0.0".to_string(),
        author: "Kable".to_string(),
        sounds,
        music: Some(music),
    }
}
