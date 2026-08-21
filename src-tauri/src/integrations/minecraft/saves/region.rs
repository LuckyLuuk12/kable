use api_types::worlds::{WorldChunk, WorldChunkCompression, WorldRegion, WorldRegionStorage};
use std::path::Path;

const SECTOR_SIZE: u64 = 4096;
const HEADER_SIZE: u64 = SECTOR_SIZE * 2;
const CHUNK_COUNT: usize = 1024;

pub async fn load_directory(path: impl AsRef<Path>) -> Result<WorldRegionStorage, String> {
    let path = path.as_ref();

    if !crate::system::fs::is_dir(path).await? {
        return Ok(WorldRegionStorage {
            path: path.to_string_lossy().into_owned(),
            region_count: "0".to_owned(),
            chunk_count: "0".to_owned(),
            total_size: "0".to_owned(),
            regions: Vec::new(),
        });
    }

    let entries = crate::system::fs::read_dir(path).await?;

    let mut regions = Vec::new();
    let mut total_size = 0u64;
    let mut chunk_count = 0u64;

    for region_path in entries {
        if region_path.extension().and_then(|value| value.to_str()) != Some("mca") {
            continue;
        }

        let Some((x, z)) = parse_region_filename(&region_path) else {
            continue;
        };

        if let Ok(region) = load(&region_path, x, z).await {
            total_size += region.size.parse::<u64>().unwrap_or(0);
            chunk_count += region.chunk_count as u64;
            regions.push(region);
        }
    }

    regions.sort_by_key(|region| (region.x, region.z));

    Ok(WorldRegionStorage {
        path: path.to_string_lossy().into_owned(),
        region_count: regions.len().to_string(),
        chunk_count: chunk_count.to_string(),
        total_size: total_size.to_string(),
        regions,
    })
}

pub async fn load(path: impl AsRef<Path>, x: i32, z: i32) -> Result<WorldRegion, String> {
    let path = path.as_ref();

    let data = crate::system::fs::read(path).await?;

    if data.len() < HEADER_SIZE as usize {
        return Err(format!("Region file is smaller than its required header: {}", path.display()));
    }

    let metadata = tokio::fs::metadata(path).await.map_err(|e| format!("Failed to read region metadata: {e}"))?;

    let mut chunks = Vec::with_capacity(CHUNK_COUNT);
    let mut chunk_count = 0i32;

    for index in 0..CHUNK_COUNT {
        let location_offset = index * 4;

        let sector_offset = read_u24(&data[location_offset..location_offset + 3]);
        let sector_count = data[location_offset + 3];

        let timestamp_offset = SECTOR_SIZE as usize + index * 4;

        let timestamp = u32::from_be_bytes([
            data[timestamp_offset],
            data[timestamp_offset + 1],
            data[timestamp_offset + 2],
            data[timestamp_offset + 3],
        ]);

        let local_x = (index % 32) as i32;
        let local_z = (index / 32) as i32;

        let chunk_x = x * 32 + local_x;
        let chunk_z = z * 32 + local_z;

        if sector_offset == 0 || sector_count == 0 {
            continue;
        }

        chunk_count += 1;

        chunks.push(WorldChunk {
            x: chunk_x,
            z: chunk_z,
            region_x: x,
            region_z: z,
            sector_offset: sector_offset as i32,
            sector_count: sector_count as i32,
            timestamp: if timestamp == 0 { None } else { Some(timestamp.to_string()) },
            compression: None,
            compressed_size: None,
            data: None,
        });
    }

    Ok(WorldRegion { x, z, path: path.to_string_lossy().into_owned(), size: metadata.len().to_string(), chunk_count, chunks })
}

pub async fn load_chunk(path: impl AsRef<Path>, local_x: u8, local_z: u8) -> Result<WorldChunk, String> {
    if local_x >= 32 || local_z >= 32 {
        return Err(format!("Invalid local chunk coordinates: {}, {}", local_x, local_z));
    }

    let path = path.as_ref();
    let data = crate::system::fs::read(path).await?;

    if data.len() < HEADER_SIZE as usize {
        return Err("Region file is smaller than its header".into());
    }

    let index = local_z as usize * 32 + local_x as usize;
    let location_offset = index * 4;

    let sector_offset = read_u24(&data[location_offset..location_offset + 3]);
    let sector_count = data[location_offset + 3];

    if sector_offset == 0 || sector_count == 0 {
        return Err(format!("Chunk {}:{} does not exist in {}", local_x, local_z, path.display()));
    }

    let timestamp_offset = SECTOR_SIZE as usize + index * 4;

    let timestamp =
        u32::from_be_bytes([data[timestamp_offset], data[timestamp_offset + 1], data[timestamp_offset + 2], data[timestamp_offset + 3]]);

    let start = sector_offset as usize * SECTOR_SIZE as usize;
    let end = start + sector_count as usize * SECTOR_SIZE as usize;

    if end > data.len() {
        return Err(format!("Chunk extends beyond region file: {}", path.display()));
    }

    if start + 5 > data.len() {
        return Err("Chunk header is incomplete".into());
    }

    let length = u32::from_be_bytes([data[start], data[start + 1], data[start + 2], data[start + 3]]);

    if length < 1 {
        return Err("Invalid chunk length: missing compression byte".into());
    }

    let compression = WorldChunkCompression::from_id(data[start + 4])?;

    let compressed_end = start
        .checked_add(4)
        .and_then(|value| value.checked_add(length as usize))
        .ok_or_else(|| "Invalid chunk length".to_string())?;

    if compressed_end > end || compressed_end > data.len() {
        return Err("Chunk data extends beyond allocated sectors".into());
    }

    let compressed_size = length - 1;

    Ok(WorldChunk {
        x: local_x as i32,
        z: local_z as i32,
        region_x: 0,
        region_z: 0,
        sector_offset: sector_offset as i32,
        sector_count: sector_count as i32,
        timestamp: if timestamp == 0 { None } else { Some(timestamp.to_string()) },
        compression: Some(compression),
        compressed_size: Some(compressed_size as i32),
        data: Some(data[start + 5..compressed_end].to_vec()),
    })
}

fn parse_region_filename(path: &Path) -> Option<(i32, i32)> {
    let stem = path.file_stem()?.to_str()?;
    let mut parts = stem.split('.');

    if parts.next()? != "r" {
        return None;
    }

    let x = parts.next()?.parse().ok()?;
    let z = parts.next()?.parse().ok()?;

    if parts.next().is_some() {
        return None;
    }

    Some((x, z))
}

fn read_u24(bytes: &[u8]) -> u32 {
    ((bytes[0] as u32) << 16) | ((bytes[1] as u32) << 8) | bytes[2] as u32
}
