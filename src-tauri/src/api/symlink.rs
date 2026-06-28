use crate::features::advanced::symlink;
use api_types::symlinks::{Symlink, SymlinkCreateRequest};
use std::collections::HashMap;

#[tauri::command]
#[specta::specta]
pub async fn get_symlinks() -> Result<Vec<Symlink>, String> {
    symlink::symlinks().await
}

#[tauri::command]
#[specta::specta]
pub async fn temporary_symlinks() -> Result<HashMap<String, Symlink>, String> {
    symlink::temporary_symlinks().await
}

#[tauri::command]
#[specta::specta]
pub async fn create(link: SymlinkCreateRequest) -> Result<Symlink, String> {
    symlink::create(link).await
}

#[tauri::command]
#[specta::specta]
pub async fn remove(link: Symlink) -> Result<(), String> {
    symlink::remove(&link).await
}

#[tauri::command]
#[specta::specta]
pub async fn toggle(link: Symlink) -> Result<Symlink, String> {
    symlink::toggle(&link).await
}

#[tauri::command]
#[specta::specta]
pub async fn update(old: Symlink, new: Symlink) -> Result<Symlink, String> {
    symlink::update(old, new).await
}
