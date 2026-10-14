use crate::system::cache::runtime;
use crate::system::cache::CacheEntryInfo;

#[tauri::command]
#[specta::specta]
pub async fn list_entries() -> Result<Vec<CacheEntryInfo>, String> {
    runtime::list_entries().await
}

#[tauri::command]
#[specta::specta]
pub async fn invalidate_entry(path: String) -> Result<(), String> {
    runtime::invalidate_entry(&path).await
}

#[tauri::command]
#[specta::specta]
pub async fn invalidate_namespace(namespace: String) -> Result<(), String> {
    runtime::invalidate_group(&namespace).await
}

#[tauri::command]
#[specta::specta]
pub async fn invalidate_memory() -> Result<(), String> {
    runtime::invalidate_memory().await;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn invalidate_all() -> Result<(), String> {
    runtime::invalidate_all().await
}
