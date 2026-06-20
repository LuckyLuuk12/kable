use crate::constants::{CONFIG_DIR, SOUNDPACK_FILE, SOUNDS_DIR};
use crate::system::fs;
use api_types::sounds::SoundpackMetadata;
use std::collections::HashMap;
use std::path::PathBuf;
use zip::ZipArchive;

type FsResult<T> = Result<T, String>;

async fn sounds_dir() -> FsResult<PathBuf> {
    Ok(fs::launcher_dir()?.join(CONFIG_DIR).join(SOUNDS_DIR))
}

async fn ensure_sounds_dir() -> FsResult<PathBuf> {
    let dir = sounds_dir().await?;
    fs::create_dir(&dir).await?;
    Ok(dir)
}

pub async fn list_soundpacks() -> FsResult<Vec<String>> {
    let dir = ensure_sounds_dir().await?;

    let mut packs = vec!["default".to_string()];

    let entries = fs::read_dir(&dir).await?;

    for path in entries {
        // folder-based pack
        if fs::is_dir(&path).await? {
            let meta = path.join(SOUNDPACK_FILE);

            if fs::exists(&meta).await? {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    packs.push(name.to_string());
                }
            }
        }

        // zip-based pack
        if path.extension().and_then(|e| e.to_str()) == Some("zip") {
            let file = std::fs::File::open(&path).map_err(|e| format!("zip open failed {}: {}", path.display(), e))?;

            if let Ok(mut zip) = ZipArchive::new(file) {
                if zip.by_name(SOUNDPACK_FILE).is_ok() {
                    if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                        packs.push(name.to_string());
                    }
                }
            }
        }
    }

    Ok(packs)
}

pub async fn get_soundpack_metadata(pack: String) -> FsResult<SoundpackMetadata> {
    if pack == "default" {
        return Ok(default_metadata());
    }

    let base = ensure_sounds_dir().await?;

    let folder = base.join(&pack);

    if fs::exists(&folder).await? {
        let meta = folder.join(SOUNDPACK_FILE);
        let content = fs::read_str(&meta).await?;

        return serde_json::from_str(&content).map_err(|e| format!("json parse failed {}: {}", meta.display(), e));
    }

    let zip_path = base.join(format!("{}.zip", pack));

    if fs::exists(&zip_path).await? {
        let file = std::fs::File::open(&zip_path).map_err(|e| format!("zip open failed {}: {}", zip_path.display(), e))?;

        let mut zip = ZipArchive::new(file).map_err(|e| format!("zip read failed: {}", e))?;

        let mut entry = zip.by_name(SOUNDPACK_FILE).map_err(|e| format!("missing {}: {}", SOUNDPACK_FILE, e))?;

        let mut content = String::new();
        std::io::Read::read_to_string(&mut entry, &mut content).map_err(|e| format!("zip read error: {}", e))?;

        return serde_json::from_str(&content).map_err(|e| format!("json parse failed: {}", e));
    }

    Err(format!("soundpack not found: {}", pack))
}

pub async fn load_soundpack_file(pack: String, file: String) -> FsResult<Vec<u8>> {
    if pack == "default" {
        return Err("default sounds are frontend-bundled".to_string());
    }

    let base = ensure_sounds_dir().await?;
    let folder = base.join(&pack);

    if fs::exists(&folder).await? {
        let path = folder.join(&file);
        return fs::read(&path).await;
    }

    let zip_path = base.join(format!("{}.zip", pack));

    if fs::exists(&zip_path).await? {
        let file_handle = std::fs::File::open(&zip_path).map_err(|e| format!("zip open failed: {}", e))?;

        let mut zip = ZipArchive::new(file_handle).map_err(|e| format!("zip read failed: {}", e))?;

        let mut entry = zip.by_name(&file).map_err(|e| format!("file not in zip: {}", e))?;

        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut data).map_err(|e| format!("zip read error: {}", e))?;

        return Ok(data);
    }

    Err(format!("sound file not found: {}", file))
}

pub async fn import_soundpack_zip(path: String) -> FsResult<String> {
    let base = ensure_sounds_dir().await?;
    let src = PathBuf::from(&path);

    if !fs::exists(&src).await? {
        return Err("zip does not exist".to_string());
    }

    let file = std::fs::File::open(&src).map_err(|e| format!("zip open failed: {}", e))?;

    let mut zip = ZipArchive::new(file).map_err(|e| format!("zip read failed: {}", e))?;

    let mut meta = zip.by_name(SOUNDPACK_FILE).map_err(|_| format!("missing {}", SOUNDPACK_FILE))?;

    let mut content = String::new();
    std::io::Read::read_to_string(&mut meta, &mut content).map_err(|e| format!("meta read failed: {}", e))?;

    let parsed: SoundpackMetadata = serde_json::from_str(&content).map_err(|e| format!("invalid metadata: {}", e))?;

    let name = parsed.name.clone();

    let dest = base.join(format!("{}.zip", name));

    let zip_bytes = std::fs::read(&src).map_err(|e| e.to_string())?;

    fs::write(&dest, &zip_bytes, false).await?;

    Ok(name)
}

pub async fn get_sounds_directory_path() -> FsResult<String> {
    let dir = ensure_sounds_dir().await?;
    Ok(dir.to_string_lossy().to_string())
}

pub async fn open_sounds_directory() -> FsResult<()> {
    let dir = ensure_sounds_dir().await?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer").arg(&dir).spawn().map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&dir).spawn().map_err(|e| e.to_string())?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&dir).spawn().map_err(|e| e.to_string())?;

    Ok(())
}

fn default_metadata() -> SoundpackMetadata {
    let mut sounds = HashMap::new();
    sounds.insert("click".to_string(), "click.mp3".to_string());
    sounds.insert("hover".to_string(), "hover.mp3".to_string());
    sounds.insert("success".to_string(), "success.mp3".to_string());
    sounds.insert("error".to_string(), "error.mp3".to_string());
    sounds.insert("notification".to_string(), "notification.mp3".to_string());
    sounds.insert("launch".to_string(), "launch.mp3".to_string());

    let mut music = HashMap::new();
    music.insert("menu".to_string(), vec!["music/menu1.mp3".to_string(), "music/menu2.mp3".to_string()]);

    SoundpackMetadata {
        id: "default".to_string(),
        name: "default".to_string(),
        version: Some("1.0.0".to_string()),
        author: Some("Kable".to_string()),
        description: Some("default soundpack".to_string()),
        file_path: None,
    }
}
