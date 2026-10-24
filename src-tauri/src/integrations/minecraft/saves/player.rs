use super::nbt;
use api_types::worlds::{NbtValue, WorldGameMode, WorldInventoryItem, WorldPlayer, WorldPosition, WorldRotation};
use std::path::Path;
use uuid::Uuid;

pub async fn load_all(path: impl AsRef<Path>) -> Result<Vec<WorldPlayer>, String> {
    let playerdata_path = path.as_ref().join("playerdata");

    if !crate::system::fs::is_dir(&playerdata_path).await? {
        return Ok(Vec::new());
    }

    let entries = crate::system::fs::read_dir(&playerdata_path).await?;
    let mut players = Vec::new();

    for file in entries {
        if file.extension().and_then(|value| value.to_str()) != Some("dat") {
            continue;
        }

        let Some(uuid) = parse_uuid(&file) else {
            continue;
        };

        match load(&file, uuid).await {
            Ok(player) => players.push(player),
            Err(_) => continue,
        }
    }

    players.sort_by_key(|player| player.uuid.clone());

    Ok(players)
}

pub async fn load(path: impl AsRef<Path>, uuid: Uuid) -> Result<WorldPlayer, String> {
    let root = nbt::load_gzip(path.as_ref()).await?;

    let data = match &root {
        NbtValue::Compound(entries) => entries.as_slice(),
        _ => {
            return Err(format!("Invalid player NBT: root is not a compound: {}", path.as_ref().display()));
        }
    };

    Ok(WorldPlayer {
        uuid: uuid.to_string(),
        path: path.as_ref().to_string_lossy().into_owned(),

        name: nbt::string(data, "playerName"),
        dimension: nbt::string(data, "Dimension"),

        position: parse_position(data),
        rotation: parse_rotation(data),

        health: nbt::f32(data, "Health"),
        food_level: nbt::i32(data, "foodLevel"),
        food_saturation: nbt::f32(data, "foodSaturationLevel"),

        experience_level: nbt::i32(data, "XpLevel"),
        experience: nbt::f32(data, "XpP"),
        total_experience: nbt::i32(data, "XpTotal"),

        game_mode: parse_game_mode(data),

        inventory: parse_inventory(data),

        ender_items: nbt::value(data, "EnderItems").cloned(),

        nbt: Some(root),
    })
}

pub async fn load_raw(path: impl AsRef<Path>) -> Result<NbtValue, String> {
    nbt::load_gzip(path).await
}

fn parse_uuid(path: &Path) -> Option<Uuid> {
    let stem = path.file_stem()?.to_str()?;
    Uuid::parse_str(stem).ok()
}

fn parse_position(entries: &[api_types::worlds::NbtEntry]) -> Option<WorldPosition> {
    let value = nbt::value(entries, "Pos")?;

    match value {
        NbtValue::List(values) if values.len() >= 3 => {
            Some(WorldPosition { x: nbt::f64_value(&values[0])?, y: nbt::f64_value(&values[1])?, z: nbt::f64_value(&values[2])? })
        }

        _ => None,
    }
}

fn parse_rotation(entries: &[api_types::worlds::NbtEntry]) -> Option<WorldRotation> {
    let value = nbt::value(entries, "Rotation")?;

    match value {
        NbtValue::List(values) if values.len() >= 2 => {
            Some(WorldRotation { yaw: nbt::f32_value(&values[0])?, pitch: nbt::f32_value(&values[1])? })
        }

        _ => None,
    }
}

fn parse_game_mode(entries: &[api_types::worlds::NbtEntry]) -> Option<WorldGameMode> {
    let value = nbt::i32(entries, "playerGameType")?;

    Some(match value {
        0 => WorldGameMode::Survival,
        1 => WorldGameMode::Creative,
        2 => WorldGameMode::Adventure,
        3 => WorldGameMode::Spectator,
        _ => WorldGameMode::Unknown,
    })
}

fn parse_inventory(entries: &[api_types::worlds::NbtEntry]) -> Vec<WorldInventoryItem> {
    let Some(NbtValue::List(items)) = nbt::value(entries, "Inventory") else {
        return Vec::new();
    };

    items.iter().filter_map(parse_inventory_item).collect()
}

fn parse_inventory_item(value: &NbtValue) -> Option<WorldInventoryItem> {
    let NbtValue::Compound(entries) = value else {
        return None;
    };

    let slot = nbt::i8(entries, "Slot")?;
    let count = nbt::i32(entries, "Count").or_else(|| nbt::i8(entries, "Count").map(|value| value as i32))?;

    let item_id = nbt::string(entries, "id")?;

    let nbt = nbt::value(entries, "tag").cloned();

    Some(WorldInventoryItem { slot, item_id, count, nbt })
}
