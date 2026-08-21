use api_types::worlds::{NbtEntry, NbtValue};
use flate2::read::{GzDecoder, ZlibDecoder};
use std::io::{Cursor, Read};

pub async fn load_gzip(path: impl AsRef<std::path::Path>) -> Result<NbtValue, String> {
    let bytes = crate::system::fs::read(path.as_ref()).await?;
    from_gzip(&bytes)
}

pub fn from_gzip(data: &[u8]) -> Result<NbtValue, String> {
    let mut decoder = GzDecoder::new(Cursor::new(data));
    let mut decompressed = Vec::new();

    decoder.read_to_end(&mut decompressed).map_err(|e| format!("Failed to decompress gzip NBT: {e}"))?;

    from_bytes(&decompressed)
}

pub fn from_zlib(data: &[u8]) -> Result<NbtValue, String> {
    let mut decoder = ZlibDecoder::new(Cursor::new(data));
    let mut decompressed = Vec::new();

    decoder.read_to_end(&mut decompressed).map_err(|e| format!("Failed to decompress zlib NBT: {e}"))?;

    from_bytes(&decompressed)
}

pub fn from_bytes(data: &[u8]) -> Result<NbtValue, String> {
    let value: fastnbt::Value = fastnbt::from_bytes(data).map_err(|e| format!("Failed to parse NBT: {e}"))?;

    from_fastnbt(value)
}

pub fn from_fastnbt(value: fastnbt::Value) -> Result<NbtValue, String> {
    match value {
        fastnbt::Value::Byte(value) => Ok(NbtValue::Byte(value)),
        fastnbt::Value::Short(value) => Ok(NbtValue::Short(value)),
        fastnbt::Value::Int(value) => Ok(NbtValue::Int(value)),
        fastnbt::Value::Long(value) => Ok(NbtValue::Long(value.to_string())),
        fastnbt::Value::Float(value) => Ok(NbtValue::Float(value)),
        fastnbt::Value::Double(value) => Ok(NbtValue::Double(value)),

        fastnbt::Value::ByteArray(value) => Ok(NbtValue::ByteArray(value.iter().copied().collect())),

        fastnbt::Value::String(value) => Ok(NbtValue::String(value)),

        fastnbt::Value::List(values) => values.into_iter().map(from_fastnbt).collect::<Result<Vec<_>, _>>().map(NbtValue::List),

        fastnbt::Value::Compound(entries) => entries
            .into_iter()
            .map(|(name, value)| Ok(NbtEntry { name, value: from_fastnbt(value)? }))
            .collect::<Result<Vec<_>, String>>()
            .map(NbtValue::Compound),

        fastnbt::Value::IntArray(value) => Ok(NbtValue::IntArray(value.to_vec())),

        fastnbt::Value::LongArray(value) => Ok(NbtValue::LongArray(value.iter().map(|v| v.to_string()).collect())),
    }
}

pub fn value<'a>(entries: &'a [NbtEntry], name: &str) -> Option<&'a NbtValue> {
    entries.iter().find(|entry| entry.name == name).map(|entry| &entry.value)
}

pub fn string(entries: &[NbtEntry], name: &str) -> Option<String> {
    match value(entries, name)? {
        NbtValue::String(value) => Some(value.clone()),
        _ => None,
    }
}

pub fn long(entries: &[NbtEntry], name: &str) -> Option<String> {
    match value(entries, name)? {
        NbtValue::Long(value) => Some(value.clone()),
        _ => None,
    }
}

pub fn bool(entries: &[NbtEntry], name: &str) -> Option<bool> {
    match value(entries, name)? {
        NbtValue::Byte(value) => Some(*value != 0),
        _ => None,
    }
}

pub fn byte(entries: &[NbtEntry], name: &str) -> Option<i8> {
    match value(entries, name)? {
        NbtValue::Byte(value) => Some(*value),
        _ => None,
    }
}

pub fn i8(entries: &[NbtEntry], name: &str) -> Option<i8> {
    byte(entries, name)
}

pub fn i32(entries: &[NbtEntry], name: &str) -> Option<i32> {
    match value(entries, name)? {
        NbtValue::Int(value) => Some(*value),
        _ => None,
    }
}

pub fn i64(entries: &[NbtEntry], name: &str) -> Option<i64> {
    match value(entries, name)? {
        NbtValue::Long(value) => value.parse().ok(),
        _ => None,
    }
}

pub fn f32(entries: &[NbtEntry], name: &str) -> Option<f32> {
    match value(entries, name)? {
        NbtValue::Float(value) => Some(*value),
        _ => None,
    }
}

pub fn f64(entries: &[NbtEntry], name: &str) -> Option<f64> {
    match value(entries, name)? {
        NbtValue::Double(value) => Some(*value),
        _ => None,
    }
}

pub fn compound<'a>(entries: &'a [NbtEntry], name: &str) -> Option<&'a [NbtEntry]> {
    match value(entries, name)? {
        NbtValue::Compound(value) => Some(value),
        _ => None,
    }
}

pub fn list<'a>(entries: &'a [NbtEntry], name: &str) -> Option<&'a [NbtValue]> {
    match value(entries, name)? {
        NbtValue::List(value) => Some(value),
        _ => None,
    }
}

pub fn f64_value(value: &NbtValue) -> Option<f64> {
    match value {
        NbtValue::Double(value) => Some(*value),
        NbtValue::Float(value) => Some(*value as f64),
        NbtValue::Int(value) => Some(*value as f64),
        NbtValue::Long(value) => value.parse::<f64>().ok(),
        _ => None,
    }
}

pub fn f32_value(value: &NbtValue) -> Option<f32> {
    match value {
        NbtValue::Float(value) => Some(*value),
        NbtValue::Double(value) => Some(*value as f32),
        NbtValue::Int(value) => Some(*value as f32),
        NbtValue::Long(value) => value.parse::<f32>().ok(),
        _ => None,
    }
}
