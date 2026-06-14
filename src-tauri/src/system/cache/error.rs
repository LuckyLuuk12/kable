use std::path::PathBuf;

use thiserror::Error;

/// Errors produced by kable-cache.
#[derive(Debug, Error)]
pub enum CacheError {
    /// Cache system was initialized more than once.
    #[error("cache has already been initialized")]
    AlreadyInitialized,

    /// Cache root has not been initialized.
    #[error("cache has not been initialized")]
    NotInitialized,

    /// Failed to create/read/write a cache file.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to serialize a value.
    #[error("serialization error: {0}")]
    Serialize(#[from] bincode::error::EncodeError),

    /// Failed to deserialize a value.
    #[error("deserialization error: {0}")]
    Deserialize(#[from] bincode::error::DecodeError),

    /// Cache file exists but is invalid.
    #[error("cache entry is corrupted: {path}")]
    CorruptedEntry { path: PathBuf },

    /// Generated cache key was invalid.
    #[error("invalid cache key")]
    InvalidKey,
}
