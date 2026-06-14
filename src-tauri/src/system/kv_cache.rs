use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Simple in-memory async cache with optional persistence hooks.
/// Use for feature-level caches. Not opinionated about persistence.
pub type SharedCache<K, V> = Lazy<RwLock<HashMap<K, V>>>;

// Example shared caches (feature code can create their own named caches)
pub static GLOBAL_STRING_CACHE: Lazy<RwLock<HashMap<String, String>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Async get helper
pub async fn cache_get(key: &str) -> Option<String> {
    let guard = GLOBAL_STRING_CACHE.read().await;
    guard.get(key).cloned()
}

/// Async set helper
pub async fn cache_set(key: String, val: String) {
    let mut guard = GLOBAL_STRING_CACHE.write().await;
    guard.insert(key, val);
}
