use api_types::worlds::{NbtEntry, NbtValue, WorldAdvancement, WorldAdvancementProgress};
use serde::Deserialize;
use std::{collections::HashMap, path::Path};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct AdvancementFile {
    #[serde(flatten)]
    advancements: HashMap<String, AdvancementProgress>,
}

#[derive(Debug, Deserialize)]
struct AdvancementProgress {
    #[serde(default)]
    done: bool,

    #[serde(default)]
    criteria: HashMap<String, serde_json::Value>,
}

pub async fn load_all(path: impl AsRef<Path>) -> Result<Vec<WorldAdvancement>, String> {
    let advancements_path = path.as_ref().join("advancements");

    if !crate::system::fs::is_dir(&advancements_path).await? {
        return Ok(Vec::new());
    }

    let entries = crate::system::fs::read_dir(&advancements_path).await?;
    let mut advancements = Vec::new();

    for file in entries {
        if file.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }

        let Some(uuid) = parse_uuid(&file) else {
            continue;
        };

        match load_player(&file, uuid).await {
            Ok(mut player_advancements) => {
                advancements.append(&mut player_advancements);
            }
            Err(_) => continue,
        }
    }

    Ok(advancements)
}

async fn load_player(path: &Path, uuid: Uuid) -> Result<Vec<WorldAdvancement>, String> {
    let contents = crate::system::fs::read_str(path).await?;

    let file: AdvancementFile =
        serde_json::from_str(&contents).map_err(|e| format!("Failed to parse advancements {}: {e}", path.display()))?;

    Ok(file
        .advancements
        .into_iter()
        .map(|(id, progress)| WorldAdvancement {
            id,
            player_uuid: uuid.to_string(),
            done: progress.done,
            criteria: progress
                .criteria
                .into_iter()
                .map(|(id, value)| WorldAdvancementProgress {
                    id,
                    achieved_at: value.get("obtained").and_then(serde_json::Value::as_str).map(ToOwned::to_owned),
                    raw: Some(json_to_nbt(value)),
                })
                .collect(),
        })
        .collect())
}

fn parse_uuid(path: &Path) -> Option<Uuid> {
    let stem = path.file_stem()?.to_str()?;
    Uuid::parse_str(stem).ok()
}

fn json_to_nbt(value: serde_json::Value) -> NbtValue {
    match value {
        serde_json::Value::Null => NbtValue::String("null".to_owned()),

        serde_json::Value::Bool(value) => NbtValue::Byte(if value { 1 } else { 0 }),

        serde_json::Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                NbtValue::Long(value.to_string())
            } else if let Some(value) = value.as_f64() {
                NbtValue::Double(value)
            } else {
                NbtValue::String(value.to_string())
            }
        }

        serde_json::Value::String(value) => NbtValue::String(value),

        serde_json::Value::Array(values) => NbtValue::List(values.into_iter().map(json_to_nbt).collect()),

        serde_json::Value::Object(values) => {
            NbtValue::Compound(values.into_iter().map(|(name, value)| NbtEntry { name, value: json_to_nbt(value) }).collect())
        }
    }
}
