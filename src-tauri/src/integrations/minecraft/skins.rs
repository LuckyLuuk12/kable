use api_types::skins::{AccountSkin, CustomSkinEntry, CustomSkinsRoot, SkinModel, SkinUploadConfig};
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use serde_json;
use std::path::{Path, PathBuf};

use crate::constants::CUSTOM_SKINS_FILE;
use crate::system::fs;

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
const DATA_URL_PREFIX: &str = "data:image/png;base64,";

/// Returns the legacy custom-skins file path in the Minecraft directory.
fn custom_skins_path() -> Result<PathBuf, String> {
    Ok(fs::mc_dir()?.join(CUSTOM_SKINS_FILE))
}

fn is_png(data: &[u8]) -> bool {
    data.starts_with(PNG_SIGNATURE)
}

fn timestamp() -> String {
    Utc::now().to_rfc3339()
}

async fn read_root_unlocked(path: &Path) -> Result<CustomSkinsRoot, String> {
    if !fs::exists(path).await? {
        return Ok(CustomSkinsRoot::default());
    }

    let contents = fs::read_str(path).await?;
    serde_json::from_str(&contents).map_err(|error| format!("Failed to parse custom skins file: {error}"))
}

async fn write_root_unlocked(path: &Path, root: &CustomSkinsRoot) -> Result<(), String> {
    let contents = serde_json::to_string_pretty(root).map_err(|error| format!("Failed to serialize custom skins: {error}"))?;

    fs::write_str(path, &contents, false).await?;
    Ok(())
}

/// Resolves a stored skin image to PNG bytes.
///
/// Supported values:
/// - `data:image/png;base64,...`
/// - raw base64-encoded PNG data
/// - a file path, absolute or relative to the Minecraft directory
async fn decode_skin_image(skin_image: &str, minecraft_dir: &Path) -> Result<Vec<u8>, String> {
    if let Some(encoded) = skin_image.strip_prefix(DATA_URL_PREFIX) {
        let data = general_purpose::STANDARD.decode(encoded).map_err(|error| format!("Invalid skin data URL: {error}"))?;

        if !is_png(&data) {
            return Err("Skin data is not a valid PNG file".into());
        }

        return Ok(data);
    }

    // Try raw base64 before treating the value as a path.
    if let Ok(data) = general_purpose::STANDARD.decode(skin_image) {
        if is_png(&data) {
            return Ok(data);
        }
    }

    let stored_path = PathBuf::from(skin_image);
    let resolved_path = if stored_path.is_absolute() { stored_path } else { minecraft_dir.join(stored_path) };

    let data = fs::read(&resolved_path)
        .await
        .map_err(|error| format!("Failed to read local skin '{}': {error}", resolved_path.display()))?;

    if !is_png(&data) {
        return Err(format!("Local skin '{}' is not a valid PNG file", resolved_path.display()));
    }

    Ok(data)
}

fn to_data_url(data: &[u8]) -> String {
    format!("{DATA_URL_PREFIX}{}", general_purpose::STANDARD.encode(data))
}

fn entry_to_account_skin(key: &str, entry: &CustomSkinEntry, data_url: Option<String>) -> AccountSkin {
    let uploaded_date = chrono::DateTime::parse_from_rfc3339(&entry.created).ok().and_then(|value| i32::try_from(value.timestamp()).ok());

    AccountSkin {
        id: if entry.id.is_empty() { key.to_string() } else { entry.id.clone() },
        name: entry.name.clone(),
        url: data_url,
        model: if entry.slim { SkinModel::Slim } else { SkinModel::Classic },
        is_current: false,
        uploaded_date,
    }
}

/// Lists locally stored skins from launcher_custom_skins.json.
pub async fn get_local_skins() -> Result<Vec<AccountSkin>, String> {
    let path = custom_skins_path()?;
    let lock_path = path.clone();

    fs::with_file_lock(&lock_path, move || async move {
        let root = read_root_unlocked(&path).await?;
        let minecraft_dir = fs::mc_dir()?;

        let mut entries: Vec<_> = root.custom_skins.into_iter().collect();

        // Preserve launcher ordering for IDs such as skin_1, skin_2, skin_10.
        entries.sort_by_key(|(key, _)| key.strip_prefix("skin_").and_then(|suffix| suffix.parse::<u64>().ok()).unwrap_or(u64::MAX));

        let mut skins = Vec::with_capacity(entries.len());

        for (key, entry) in entries {
            let data_url = match decode_skin_image(&entry.skin_image, &minecraft_dir).await {
                Ok(data) => Some(to_data_url(&data)),
                // Keep listing resilient if one old entry references a missing file.
                Err(_) if entry.skin_image.starts_with("http://") || entry.skin_image.starts_with("https://") => {
                    Some(entry.skin_image.clone())
                }
                Err(_) => None,
            };

            skins.push(entry_to_account_skin(&key, &entry, data_url));
        }

        Ok(skins)
    })
    .await
}

/// Reads a local skin's PNG data and model by key or stored entry ID.
pub async fn get_local_skin_data_by_id(skin_id: &str) -> Result<(Vec<u8>, SkinModel), String> {
    let path = custom_skins_path()?;
    let lock_path = path.clone();
    let skin_id = skin_id.to_string();

    let entry = fs::with_file_lock(&lock_path, move || async move {
        let root = read_root_unlocked(&path).await?;

        root.custom_skins
            .into_iter()
            .find(|(key, entry)| key == &skin_id || entry.id == skin_id)
            .map(|(_, entry)| entry)
            .ok_or_else(|| format!("Local skin '{skin_id}' was not found"))
    })
    .await?;

    let minecraft_dir = fs::mc_dir()?;
    let data = decode_skin_image(&entry.skin_image, &minecraft_dir).await?;
    let model = if entry.slim { SkinModel::Slim } else { SkinModel::Classic };

    Ok((data, model))
}

/// Adds a PNG file to the local custom-skin collection.
///
/// This does not upload the skin to Mojang.
pub async fn add_local_skin(config: SkinUploadConfig) -> Result<AccountSkin, String> {
    let source_path = PathBuf::from(&config.file_path);

    let data = fs::read(&source_path).await.map_err(|error| format!("Failed to read skin file '{}': {error}", source_path.display()))?;

    if !is_png(&data) {
        return Err("Skin file must be a PNG image".into());
    }

    let file_name = source_path
        .file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("Custom Skin")
        .to_string();

    let data_url = to_data_url(&data);
    let path = custom_skins_path()?;
    let lock_path = path.clone();

    let entry = fs::with_file_lock(&lock_path, move || async move {
        let mut root = read_root_unlocked(&path).await?;

        let next_number = root
            .custom_skins
            .keys()
            .filter_map(|key| key.strip_prefix("skin_"))
            .filter_map(|suffix| suffix.parse::<u64>().ok())
            .max()
            .unwrap_or(0)
            + 1;

        let key = format!("skin_{next_number}");
        let now = timestamp();

        let entry = CustomSkinEntry {
            cape_id: String::new(),
            created: now.clone(),
            id: key.clone(),
            model_image: String::new(),
            name: file_name,
            skin_image: data_url,
            slim: config.model == SkinModel::Slim,
            texture_id: String::new(),
            updated: now,
        };

        root.version.get_or_insert(1);
        root.custom_skins.insert(key.clone(), entry.clone());

        write_root_unlocked(&path, &root).await?;
        Ok((key, entry))
    })
    .await?;

    Ok(entry_to_account_skin(&entry.0, &entry.1, Some(entry.1.skin_image.clone())))
}

/// Updates a local skin's editable metadata.
///
/// Pass `Some("")` for `new_cape_id` to clear the stored cape ID.
pub async fn modify_skin_by_id(
    skin_id: &str,
    new_name: Option<String>,
    new_cape_id: Option<String>,
    new_slim: Option<bool>,
) -> Result<(), String> {
    let path = custom_skins_path()?;
    let lock_path = path.clone();
    let skin_id = skin_id.to_string();

    fs::with_file_lock(&lock_path, move || async move {
        let mut root = read_root_unlocked(&path).await?;

        let entry = root
            .custom_skins
            .iter_mut()
            .find(|(key, entry)| key.as_str() == skin_id || entry.id == skin_id)
            .map(|(_, entry)| entry)
            .ok_or_else(|| format!("Local skin '{skin_id}' was not found"))?;

        if let Some(name) = new_name {
            entry.name = name;
        }

        if let Some(cape_id) = new_cape_id {
            entry.cape_id = cape_id;
        }

        if let Some(slim) = new_slim {
            entry.slim = slim;
        }

        entry.updated = timestamp();

        write_root_unlocked(&path, &root).await
    })
    .await
}

/// Removes a skin from local custom-skin storage.
pub async fn remove_skin_by_id(skin_id: &str) -> Result<(), String> {
    let path = custom_skins_path()?;
    let lock_path = path.clone();
    let skin_id = skin_id.to_string();

    fs::with_file_lock(&lock_path, move || async move {
        let mut root = read_root_unlocked(&path).await?;

        let key = root
            .custom_skins
            .iter()
            .find(|(key, entry)| key.as_str() == skin_id || entry.id == skin_id)
            .map(|(key, _)| key.clone())
            .ok_or_else(|| format!("Local skin '{skin_id}' was not found"))?;

        root.custom_skins.remove(&key);
        write_root_unlocked(&path, &root).await
    })
    .await
}
