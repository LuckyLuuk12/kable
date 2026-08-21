use super::region;
use api_types::worlds::{WorldDimension, WorldDimensionKind};
use std::path::{Path, PathBuf};

pub async fn load_all(path: impl AsRef<Path>) -> Result<Vec<WorldDimension>, String> {
    let path = path.as_ref();
    let mut dimensions = Vec::new();

    add_dimension(&mut dimensions, path, "minecraft:overworld", WorldDimensionKind::Overworld, PathBuf::new()).await?;

    add_dimension(&mut dimensions, path, "minecraft:the_nether", WorldDimensionKind::Nether, PathBuf::from("DIM-1")).await?;

    add_dimension(&mut dimensions, path, "minecraft:the_end", WorldDimensionKind::End, PathBuf::from("DIM1")).await?;

    let dimensions_dir = path.join("dimensions");

    if crate::system::fs::is_dir(&dimensions_dir).await? {
        discover_custom_dimensions(&mut dimensions, &dimensions_dir, "").await?;
    }

    Ok(dimensions)
}

pub async fn load_region_storage(path: impl AsRef<Path>) -> Result<Vec<api_types::worlds::WorldRegionStorage>, String> {
    let dimensions = load_all(path).await?;

    Ok(dimensions.into_iter().filter_map(|dimension| dimension.region_storage).collect())
}

async fn add_dimension(
    dimensions: &mut Vec<WorldDimension>,
    world_path: &Path,
    id: &str,
    kind: WorldDimensionKind,
    relative_path: PathBuf,
) -> Result<(), String> {
    let dimension_path = world_path.join(relative_path);
    let region_path = dimension_path.join("region");

    if !crate::system::fs::is_dir(&dimension_path).await? && !crate::system::fs::is_dir(&region_path).await? {
        return Ok(());
    }

    let storage = region::load_directory(&region_path).await?;

    dimensions.push(WorldDimension {
        id: id.to_owned(),
        kind,
        path: dimension_path.to_string_lossy().into_owned(),
        chunk_count: storage.clone().chunk_count,
        region_count: storage.clone().region_count,
        region_storage: Some(storage),
    });

    Ok(())
}

async fn discover_custom_dimensions(dimensions: &mut Vec<WorldDimension>, directory: &Path, namespace: &str) -> Result<(), String> {
    let entries = crate::system::fs::read_dir(directory).await?;

    for path in entries {
        if !crate::system::fs::is_dir(&path).await? {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("Invalid dimension directory: {}", path.display()))?
            .to_owned();

        let id = if namespace.is_empty() { name.clone() } else { format!("{namespace}:{name}") };

        let region_path = path.join("region");

        if crate::system::fs::is_dir(&region_path).await? {
            let storage = region::load_directory(&region_path).await?;

            dimensions.push(WorldDimension {
                id: id.clone(),
                kind: WorldDimensionKind::Custom,
                path: path.to_string_lossy().into_owned(),
                chunk_count: storage.clone().chunk_count,
                region_count: storage.clone().region_count,
                region_storage: Some(storage),
            });
        }

        Box::pin(discover_custom_dimensions(dimensions, &path, &id)).await?;
    }

    Ok(())
}
