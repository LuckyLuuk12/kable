use api_types::worlds::{WorldPlayerStatistics, WorldStatistic};
use serde_json::Value;
use std::path::Path;
use uuid::Uuid;

pub async fn load_all(path: impl AsRef<Path>) -> Result<Vec<WorldPlayerStatistics>, String> {
    let stats_path = path.as_ref().join("stats");

    if !crate::system::fs::is_dir(&stats_path).await? {
        return Ok(Vec::new());
    }

    let entries = crate::system::fs::read_dir(&stats_path).await?;
    let mut statistics = Vec::new();

    for file in entries {
        if file.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }

        let Some(uuid) = parse_uuid(&file) else {
            continue;
        };

        match load(&file, uuid).await {
            Ok(stats) => statistics.push(stats),
            Err(_) => continue,
        }
    }

    statistics.sort_by(|a, b| a.uuid.cmp(&b.uuid));

    Ok(statistics)
}

pub async fn load(path: impl AsRef<Path>, uuid: Uuid) -> Result<WorldPlayerStatistics, String> {
    let path = path.as_ref();

    let contents = crate::system::fs::read_str(path).await?;

    let root: Value = serde_json::from_str(&contents).map_err(|e| format!("Failed to parse statistics {}: {e}", path.display()))?;

    let stats = root
        .get("stats")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("Invalid statistics file {}: missing stats object", path.display()))?;

    let mut entries = Vec::new();

    for (category, category_value) in stats {
        let Some(category_stats) = category_value.as_object() else {
            continue;
        };

        for (name, value) in category_stats {
            let Some(value) = value.as_u64() else {
                continue;
            };

            entries.push(WorldStatistic { category: category.clone(), name: name.clone(), value: value.to_string() });
        }
    }

    entries.sort_by(|a, b| a.category.cmp(&b.category).then_with(|| a.name.cmp(&b.name)));

    Ok(WorldPlayerStatistics { uuid: uuid.to_string(), path: path.to_string_lossy().into_owned(), statistics: entries })
}

fn parse_uuid(path: &Path) -> Option<Uuid> {
    let stem = path.file_stem()?.to_str()?;
    Uuid::parse_str(stem).ok()
}
