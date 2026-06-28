use crate::system::{fs, net};

#[tauri::command]
#[specta::specta]
pub async fn open_url(url: String) -> Result<(), String> {
    net::open_url(url).await
}

#[tauri::command]
#[specta::specta]
pub async fn open_path(path: String) -> Result<(), String> {
    fs::open_path(path).await
}
