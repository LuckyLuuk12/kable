use std::sync::Arc;

use dashmap::DashMap;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

static LOCKS: Lazy<DashMap<String, Arc<Mutex<()>>>> = Lazy::new(DashMap::new);

pub fn get_lock(key: &str) -> Arc<Mutex<()>> {
    LOCKS.entry(key.to_string()).or_insert_with(|| Arc::new(Mutex::new(()))).clone()
}
