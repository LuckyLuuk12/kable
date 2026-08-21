use crate::integrations::minecraft::saves;

use api_types::worlds::{NbtValue, World, WorldDatapack, WorldDimension, WorldLevel, WorldPlayer, WorldRegionStorage, WorldSummary};

use std::path::PathBuf;
use uuid::Uuid;

#[tauri::command]
#[specta::specta]
pub async fn load_worlds() -> Result<Vec<WorldSummary>, String> {
    saves::world::load_worlds().await
}

#[tauri::command]
#[specta::specta]
pub async fn load_world(path: String) -> Result<World, String> {
    saves::world::load_world(path).await
}

#[tauri::command]
#[specta::specta]
pub async fn load_world_level(path: String) -> Result<WorldLevel, String> {
    saves::level::load(PathBuf::from(path).join("level.dat")).await
}

#[tauri::command]
#[specta::specta]
pub async fn load_world_players(path: String) -> Result<Vec<WorldPlayer>, String> {
    saves::player::load_all(path).await
}

#[tauri::command]
#[specta::specta]
pub async fn load_world_player(path: String, uuid: String) -> Result<WorldPlayer, String> {
    let uuid = Uuid::parse_str(&uuid).map_err(|e| format!("Invalid player UUID: {e}"))?;

    saves::player::load(path, uuid).await
}

#[tauri::command]
#[specta::specta]
pub async fn load_world_dimensions(path: String) -> Result<Vec<WorldDimension>, String> {
    saves::dimension::load_all(path).await
}

#[tauri::command]
#[specta::specta]
pub async fn load_world_region_storage(path: String) -> Result<Vec<WorldRegionStorage>, String> {
    saves::dimension::load_region_storage(path).await
}

#[tauri::command]
#[specta::specta]
pub async fn load_world_datapacks(path: String) -> Result<Vec<WorldDatapack>, String> {
    saves::datapack::load_all(path).await
}

#[tauri::command]
#[specta::specta]
pub async fn load_world_nbt(path: String) -> Result<NbtValue, String> {
    saves::nbt::load_gzip(PathBuf::from(path)).await
}
