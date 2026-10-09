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
pub async fn runtime_symlinks() -> Result<HashMap<String, Vec<Symlink>>, String> {
    symlink::runtime_symlinks().await
}

#[tauri::command]
#[specta::specta]
pub async fn runtime_symlinks_for_profile(profile_id: String) -> Result<Vec<Symlink>, String> {
    symlink::runtime_symlinks_for_profile(&profile_id).await
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
pub async fn update(old: Symlink, updated: Symlink) -> Result<Symlink, String> {
    symlink::update(old, updated).await
}

#[tauri::command]
#[specta::specta]
pub async fn create_runtime_symlink(profile_id: String, source: String, destination: String) -> Result<Symlink, String> {
    symlink::create_temporary(&profile_id, source.into(), destination.into()).await
}

#[tauri::command]
#[specta::specta]
pub async fn reconcile_runtime_symlinks(profile_id: String, desired: Vec<(String, String)>) -> Result<(), String> {
    symlink::reconcile_runtime(&profile_id, desired.into_iter().map(|(source, destination)| (source.into(), destination.into())).collect())
        .await
}

#[tauri::command]
#[specta::specta]
pub async fn cleanup_profile_symlinks(profile_id: String) -> Result<(), String> {
    symlink::cleanup_profile(&profile_id).await
}
