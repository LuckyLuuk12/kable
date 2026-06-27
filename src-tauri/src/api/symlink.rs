use crate::features::advanced::symlink;
use api_types::symlinks::{Symlink, SymlinkCreateRequest};

#[tauri::command]
#[specta::specta]
pub async fn symlink_initialize() -> Result<(), String> {
    symlink::initialize().await
}

#[tauri::command]
#[specta::specta]
pub async fn symlink_create(req: SymlinkCreateRequest) -> Result<Symlink, String> {
    symlink::create(req).await
}

#[tauri::command]
#[specta::specta]
pub async fn symlink_remove(link: Symlink) -> Result<(), String> {
    symlink::remove(link).await
}

#[tauri::command]
#[specta::specta]
pub async fn symlink_toggle(link: Symlink) -> Result<bool, String> {
    symlink::toggle(link).await
}
