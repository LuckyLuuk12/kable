use serde::{Deserialize, Serialize};

/// Current cache file format version.
///
/// Increment this whenever the on-disk format changes
/// in a non-backwards-compatible way.
pub const CACHE_FORMAT_VERSION: u32 = 1;

/// A persisted cache entry.
///
/// Stored as a single binary blob on disk using bincode.
///
/// Layout:
///
/// ```text
/// CacheEntry<T>
/// ├── format_version
/// ├── created_at
/// ├── ttl_secs
/// └── value
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    /// Cache file format version.
    pub format_version: u32,

    /// Unix timestamp when the entry was created.
    pub created_at: u64,

    /// Optional expiration time.
    ///
    /// None = never expires.
    pub ttl_secs: Option<u64>,

    /// Cached value.
    pub value: T,
}

impl<T> CacheEntry<T> {
    /// Creates a new cache entry.
    pub fn new(value: T, ttl_secs: Option<u64>, created_at: u64) -> Self {
        Self {
            format_version: CACHE_FORMAT_VERSION,
            created_at,
            ttl_secs,
            value,
        }
    }

    /// Returns true if the cache entry has expired.
    pub fn is_expired(&self, now: u64) -> bool {
        match self.ttl_secs {
            Some(ttl) => now.saturating_sub(self.created_at) > ttl,
            None => false,
        }
    }

    /// Returns true if the format version matches
    /// the current crate version.
    pub fn is_current_format(&self) -> bool {
        self.format_version == CACHE_FORMAT_VERSION
    }
}
