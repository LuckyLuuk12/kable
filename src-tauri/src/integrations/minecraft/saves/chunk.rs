use super::nbt;
use api_types::worlds::{NbtValue, WorldChunk};
use flate2::read::GzDecoder;
use std::io::{Cursor, Read};

pub fn parse_nbt(compression: api_types::worlds::WorldChunkCompression, data: &[u8]) -> Result<NbtValue, String> {
    let mut decompressed = Vec::new();

    match compression {
        api_types::worlds::WorldChunkCompression::Gzip => {
            let mut decoder = GzDecoder::new(Cursor::new(data));

            decoder.read_to_end(&mut decompressed).map_err(|e| format!("Failed to decompress gzip chunk data: {e}"))?;
        }

        api_types::worlds::WorldChunkCompression::Zlib => {
            let mut decoder = flate2::read::ZlibDecoder::new(Cursor::new(data));

            decoder.read_to_end(&mut decompressed).map_err(|e| format!("Failed to decompress zlib chunk data: {e}"))?;
        }

        api_types::worlds::WorldChunkCompression::Uncompressed => {
            decompressed.extend_from_slice(data);
        }

        api_types::worlds::WorldChunkCompression::Lz4 => {
            return Err("LZ4 chunk compression is not currently supported".into());
        }

        api_types::worlds::WorldChunkCompression::Custom(id) => {
            return Err(format!("Unsupported chunk compression type: {id}"));
        }
    }

    nbt::from_bytes(&decompressed)
}

pub fn load_nbt(chunk: &WorldChunk) -> Result<NbtValue, String> {
    let compression = chunk.compression.ok_or_else(|| "Chunk does not have a compression type".to_string())?;

    let data = chunk.data.as_deref().ok_or_else(|| "Chunk does not contain compressed data".to_string())?;

    parse_nbt(compression, data)
}
