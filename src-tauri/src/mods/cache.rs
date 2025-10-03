use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModCacheEntry<T> {
    pub value: T,
    pub last_updated: u64, // unix timestamp (secs)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModCache<T> {
    pub entries: HashMap<String, ModCacheEntry<T>>, // key: mod id or search key
    pub default_ttl_secs: u64,
}

impl<T: Clone + Serialize + for<'de> Deserialize<'de>> ModCache<T> {
    pub fn new(default_ttl_secs: u64) -> Self {
        Self {
            entries: HashMap::new(),
            default_ttl_secs,
        }
    }

    pub fn get(&self, key: &str) -> Option<&ModCacheEntry<T>> {
        self.entries.get(key)
    }

    pub fn insert(&mut self, key: String, value: T) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.entries.insert(
            key,
            ModCacheEntry {
                value,
                last_updated: now,
            },
        );
    }

    pub fn is_stale(&self, key: &str) -> bool {
        if let Some(entry) = self.entries.get(key) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now > entry.last_updated + self.default_ttl_secs
        } else {
            true
        }
    }

    pub fn remove(&mut self, key: &str) {
        self.entries.remove(key);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn save_to_disk(&self, path: &Path) -> Result<(), String> {
        let data = serde_json::to_vec(self).map_err(|e| format!("Serialize cache: {e}"))?;
        // Use atomic sync writer to avoid partial write issues in sync contexts
        crate::write_file_atomic_sync(path, &data).map_err(|e| format!("Write cache: {e}"))
    }

    /// Async version: save the cache to disk using async filesystem APIs and
    /// ensuring parent directories exist.
    pub async fn save_to_disk_async(&self, path: &std::path::Path) -> Result<(), String> {
        let data = serde_json::to_vec(self).map_err(|e| format!("Serialize cache: {e}"))?;
        crate::ensure_parent_dir_exists_async(path)
            .await
            .map_err(|e| format!("Failed to ensure parent dir: {}", e))?;
        tokio::fs::write(path, data)
            .await
            .map_err(|e| format!("Write cache async: {e}"))
    }

    pub fn load_from_disk(path: &Path) -> Result<Self, String> {
        let data = fs::read(path).map_err(|e| format!("Read cache: {e}"))?;
        serde_json::from_slice(&data).map_err(|e| format!("Deserialize cache: {e}"))
    }

    /// Async version: load the cache using async filesystem APIs.
    pub async fn load_from_disk_async(path: &std::path::Path) -> Result<Self, String> {
        let data = tokio::fs::read(path).await.map_err(|e| format!("Read cache async: {e}"))?;
        serde_json::from_slice(&data).map_err(|e| format!("Deserialize cache: {e}"))
    }

    /// Remove all stale entries (older than ttl)
    pub fn purge_stale(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.entries
            .retain(|_, entry| now <= entry.last_updated + self.default_ttl_secs);
    }
}
