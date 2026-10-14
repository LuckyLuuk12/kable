use api_types::worlds::WorldDatapack;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct PackMcMeta {
    pack: PackInfo,
}

#[derive(Debug, Deserialize)]
struct PackInfo {
    description: serde_json::Value,

    #[serde(rename = "pack_format")]
    pack_format: Option<i32>,

    #[serde(rename = "min_format")]
    min_format: Option<i32>,

    #[serde(rename = "max_format")]
    max_format: Option<i32>,
}

pub async fn load_all(path: impl AsRef<Path>) -> Result<Vec<WorldDatapack>, String> {
    let datapacks_path = path.as_ref().join("datapacks");

    if !crate::system::fs::is_dir(&datapacks_path).await? {
        return Ok(Vec::new());
    }

    let entries = crate::system::fs::read_dir(&datapacks_path).await?;
    let mut datapacks = Vec::new();

    for entry in entries {
        let Some(name) = entry.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        if crate::system::fs::is_dir(&entry).await? {
            if let Ok(pack) = load_directory(&entry, name).await {
                datapacks.push(pack);
            }

            continue;
        }

        if entry.extension().and_then(|value| value.to_str()) == Some("zip") {
            if let Ok(pack) = load_zip(&entry, name).await {
                datapacks.push(pack);
            }
        }
    }

    datapacks.sort_by_key(|pack| pack.name.clone().unwrap_or_default().to_lowercase());

    Ok(datapacks)
}

async fn load_directory(path: &Path, name: &str) -> Result<WorldDatapack, String> {
    let meta_path = path.join("pack.mcmeta");
    let meta = read_metadata(&meta_path).await?;

    Ok(WorldDatapack {
        id: name.to_owned(),
        name: Some(name.to_owned()),
        description: Some(description_to_string(&meta.pack.description)),
        pack_format: meta.pack.pack_format,
        min_format: meta.pack.min_format,
        max_format: meta.pack.max_format,
        enabled: true,
        path: path.to_string_lossy().into_owned(),
    })
}

async fn load_zip(path: &Path, name: &str) -> Result<WorldDatapack, String> {
    let bytes = crate::system::fs::read(path).await?;
    let reader = std::io::Cursor::new(bytes);

    let mut archive = zip::ZipArchive::new(reader).map_err(|e| format!("Failed to open datapack {}: {e}", path.display()))?;

    let mut meta_file = archive.by_name("pack.mcmeta").map_err(|e| format!("Datapack {} has no pack.mcmeta: {e}", path.display()))?;

    let mut contents = String::new();

    std::io::Read::read_to_string(&mut meta_file, &mut contents).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

    let meta: PackMcMeta = serde_json::from_str(&contents).map_err(|e| format!("Failed to parse {}: {e}", path.display()))?;

    Ok(WorldDatapack {
        id: name.to_owned(),
        name: Some(name.to_owned()),
        description: Some(description_to_string(&meta.pack.description)),
        pack_format: meta.pack.pack_format,
        min_format: meta.pack.min_format,
        max_format: meta.pack.max_format,
        enabled: true,
        path: path.to_string_lossy().into_owned(),
    })
}

async fn read_metadata(path: &Path) -> Result<PackMcMeta, String> {
    let contents = crate::system::fs::read_str(path).await?;

    serde_json::from_str(&contents).map_err(|e| format!("Failed to parse {}: {e}", path.display()))
}

fn description_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.clone(),

        serde_json::Value::Object(object) => object.get("text").and_then(serde_json::Value::as_str).unwrap_or_default().to_owned(),

        _ => value.to_string(),
    }
}
