use super::{advancements, datapack, dimension, level, player, statistics};
use crate::system::fs::mc_dir;
use crate::Logger;
use api_types::worlds::{World, WorldSummary};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub async fn load_worlds() -> Result<Vec<WorldSummary>, String> {
    let saves_dir = mc_dir()?.join("saves");

    if !crate::system::fs::is_dir(&saves_dir).await? {
        return Ok(Vec::new());
    }

    let entries = crate::system::fs::read_dir(&saves_dir).await?;
    let mut worlds = Vec::new();

    for path in entries {
        if !crate::system::fs::is_dir(&path).await? {
            continue;
        }

        if !crate::system::fs::is_file(path.join("level.dat")).await? {
            continue;
        }

        match load_world_summary(&path).await {
            Ok(world) => worlds.push(world),
            Err(error) => {
                Logger::warn_global(format!("Failed to load world summary from {}: {}", path.display(), error).as_str(), None);
            }
        }
    }

    worlds.sort_by_key(|world| world.name.to_lowercase());

    Logger::debug_global(format!("Loaded {} world summaries from {}", worlds.len(), saves_dir.display()).as_str(), None);

    Ok(worlds)
}

pub async fn load_world_summary(path: impl AsRef<Path>) -> Result<WorldSummary, String> {
    let path = path.as_ref();

    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("Invalid world directory: {}", path.display()))?
        .to_owned();

    let icon_path = path.join("icon.png");

    let last_modified = tokio::fs::metadata(path)
        .await
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis().to_string());

    Ok(WorldSummary {
        id: world_id(path),
        name,
        path: path.to_string_lossy().into_owned(),
        icon: load_icon(&icon_path).await,
        last_modified,
    })
}

pub async fn load_world(path: impl AsRef<Path>) -> Result<World, String> {
    let path = path.as_ref();

    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("Invalid world directory: {}", path.display()))?
        .to_owned();

    let level = level::load(path.join("level.dat")).await?;

    let icon_path = path.join("icon.png");

    let last_modified = tokio::fs::metadata(path)
        .await
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis().to_string());

    Ok(World {
        id: world_id(path),
        name,
        path: path.to_string_lossy().into_owned(),
        icon: load_icon(&icon_path).await,
        last_modified,
        level: Some(level),
        players: player::load_all(path).await?,
        dimensions: dimension::load_all(path).await?,
        datapacks: datapack::load_all(path).await?,
        region_storage: dimension::load_region_storage(path).await?,
        advancements: advancements::load_all(path).await?,
        statistics: statistics::load_all(path).await?,
    })
}

fn world_id(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub async fn world_path(id: &str) -> Result<PathBuf, String> {
    let saves_dir = mc_dir()?.join("saves");
    let path = saves_dir.join(id);

    if !crate::system::fs::is_dir(&path).await? {
        return Err(format!("World does not exist: {id}"));
    }

    Ok(path)
}

async fn load_icon(path: &Path) -> Option<String> {
    let data = tokio::fs::read(path).await.ok()?;

    Some(format!("data:image/png;base64,{}", BASE64.encode(data)))
}
