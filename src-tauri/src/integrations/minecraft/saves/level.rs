use super::nbt;
use api_types::worlds::{
    NbtValue, WorldBorder, WorldDifficulty, WorldGameMode, WorldGameRule, WorldLevel, WorldSpawn, WorldTime, WorldVersion, WorldWeather,
};
use std::path::Path;

pub async fn load(path: impl AsRef<Path>) -> Result<WorldLevel, String> {
    let path = path.as_ref();
    let root = nbt::load_gzip(path).await?;

    let data = match &root {
        NbtValue::Compound(entries) => nbt::compound(entries, "Data"),
        _ => None,
    }
    .ok_or_else(|| format!("Invalid level.dat: missing root Data compound in {}", path.display()))?;

    Ok(WorldLevel {
        data_version: nbt::i32(data, "DataVersion"),
        version: parse_version(data),
        level_name: nbt::string(data, "LevelName"),
        game_mode: nbt::i32(data, "GameType").map(parse_game_mode),
        difficulty: nbt::byte(data, "Difficulty").map(parse_difficulty),
        hardcore: nbt::bool(data, "hardcore"),
        allow_commands: nbt::bool(data, "allowCommands"),
        seed: nbt::long(data, "RandomSeed"),
        time: parse_time(data),
        spawn: parse_spawn(data),
        weather: parse_weather(data),
        world_border: parse_world_border(data),
        game_rules: parse_game_rules(data),
        nbt: Some(root),
    })
}

fn parse_version(data: &[api_types::worlds::NbtEntry]) -> Option<WorldVersion> {
    let version = nbt::compound(data, "Version")?;

    Some(WorldVersion { name: nbt::string(version, "Name"), id: nbt::i32(version, "Id"), snapshot: nbt::bool(version, "Snapshot") })
}

fn parse_game_mode(value: i32) -> WorldGameMode {
    match value {
        0 => WorldGameMode::Survival,
        1 => WorldGameMode::Creative,
        2 => WorldGameMode::Adventure,
        3 => WorldGameMode::Spectator,
        _ => WorldGameMode::Unknown,
    }
}

fn parse_difficulty(value: i8) -> WorldDifficulty {
    match value {
        0 => WorldDifficulty::Peaceful,
        1 => WorldDifficulty::Easy,
        2 => WorldDifficulty::Normal,
        3 => WorldDifficulty::Hard,
        _ => WorldDifficulty::Unknown,
    }
}

fn parse_time(data: &[api_types::worlds::NbtEntry]) -> Option<WorldTime> {
    let game_time = nbt::long(data, "Time");
    let day_time = nbt::long(data, "DayTime");

    if game_time.is_none() && day_time.is_none() {
        return None;
    }

    Some(WorldTime { game_time, day_time })
}

fn parse_spawn(data: &[api_types::worlds::NbtEntry]) -> Option<WorldSpawn> {
    let x = nbt::i32(data, "SpawnX")?;
    let y = nbt::i32(data, "SpawnY")?;
    let z = nbt::i32(data, "SpawnZ")?;

    Some(WorldSpawn { x, y, z, angle: nbt::f32(data, "SpawnAngle") })
}

fn parse_weather(data: &[api_types::worlds::NbtEntry]) -> Option<WorldWeather> {
    let raining = nbt::bool(data, "raining");
    let rain_time = nbt::i32(data, "rainTime");
    let thundering = nbt::bool(data, "thundering");
    let thunder_time = nbt::i32(data, "thunderTime");

    if raining.is_none() && rain_time.is_none() && thundering.is_none() && thunder_time.is_none() {
        return None;
    }

    Some(WorldWeather { raining, rain_time, thundering, thunder_time })
}

fn parse_world_border(data: &[api_types::worlds::NbtEntry]) -> Option<WorldBorder> {
    let border = nbt::compound(data, "WorldBorder")?;

    Some(WorldBorder {
        center_x: nbt::f64(border, "CenterX"),
        center_z: nbt::f64(border, "CenterZ"),
        size: nbt::f64(border, "Size"),
        damage_per_block: nbt::f64(border, "DamagePerBlock"),
        safe_zone: nbt::f64(border, "SafeZone"),
        warning_blocks: nbt::i32(border, "WarningBlocks"),
        warning_time: nbt::i32(border, "WarningTime"),
        lerp_target: nbt::f64(border, "SizeLerpTarget"),
        lerp_time: nbt::long(data, "SizeLerpTime"),
    })
}

fn parse_game_rules(data: &[api_types::worlds::NbtEntry]) -> Vec<WorldGameRule> {
    let Some(rules) = nbt::compound(data, "GameRules") else {
        return Vec::new();
    };

    rules
        .iter()
        .filter_map(|entry| match &entry.value {
            NbtValue::String(value) => Some(WorldGameRule { name: entry.name.clone(), value: value.clone() }),
            _ => None,
        })
        .collect()
}
