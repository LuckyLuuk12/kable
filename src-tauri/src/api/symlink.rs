use crate::features::advanced::symlink::SymlinkManager;
use api_types::symlinks::{CreateSymlinkRequest, SymlinkView};

#[tauri::command]
pub async fn get_symlinks() -> Result<Vec<SymlinkView>, String> {
    SymlinkManager::load().await?.list().await
}

#[tauri::command]
pub async fn create_symlink(request: CreateSymlinkRequest) -> Result<(), String> {
    SymlinkManager::load().await?.create(request.into()).await
}

#[tauri::command]
pub async fn delete_symlink(id: String) -> Result<(), String> {
    SymlinkManager::load().await?.remove(&id).await
}

#[tauri::command]
pub async fn enable_symlink(id: String) -> Result<(), String> {
    SymlinkManager::load().await?.enable(&id).await
}

#[tauri::command]
pub async fn disable_symlink(id: String) -> Result<(), String> {
    SymlinkManager::load().await?.disable(&id).await
}

// #[tauri::command]
// pub async fn repair_symlink(id: String) -> Result<(), String> {
//     SymlinkManager::load().await?.repair(&id).await
// }
