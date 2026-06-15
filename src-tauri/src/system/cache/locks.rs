use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::Mutex;

static LOCKS: once_cell::sync::Lazy<DashMap<String, Arc<Mutex<()>>>> = once_cell::sync::Lazy::new(DashMap::new);

pub fn get_lock(key: &str) -> Arc<Mutex<()>> {
    if let Some(lock) = LOCKS.get(key) {
        return lock.clone();
    }

    let lock = Arc::new(Mutex::new(()));
    LOCKS.insert(key.to_string(), lock.clone());
    lock
}
