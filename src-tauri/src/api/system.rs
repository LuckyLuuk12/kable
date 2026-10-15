use crate::system::{fs, java, net, processes};

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

#[tauri::command]
#[specta::specta]
pub async fn get_max_ram() -> Result<u32, String> {
    java::get_max_ram().await
}

#[tauri::command]
#[specta::specta]
pub fn open_developer_settings() -> Result<(), String> {
    processes::open_developer_settings()
}
