use tauri_plugin_dialog::DialogExt;

// Re-export types from symlink_manager
pub use crate::symlink_manager::{CustomSymlink, CustomSymlinksConfig, SymlinkInfo};

/// List all symlinks - combines custom symlinks from config with detected managed symlinks
#[tauri::command]
pub async fn list_symlinks() -> Result<Vec<SymlinkInfo>, String> {
    crate::symlink_manager::list_all_symlinks().await
}

/// Create a custom symlink
#[tauri::command]
pub async fn create_custom_symlink(
    source: String,
    destination_parent: String,
    installation_id: Option<String>,
) -> Result<String, String> {
    crate::symlink_manager::create_custom_symlink(source, destination_parent, installation_id).await
}

/// Remove a symlink by its destination path or ID
#[tauri::command]
pub async fn remove_symlink(destination: String, id: Option<String>) -> Result<(), String> {
    crate::symlink_manager::remove_symlink(destination, id).await
}

/// Toggle symlink disabled state
#[tauri::command]
pub async fn toggle_symlink_disabled(destination: String, id: Option<String>) -> Result<bool, String> {
    crate::symlink_manager::toggle_symlink_disabled(destination, id).await
}

/// Update an existing symlink
#[tauri::command]
pub async fn update_symlink(
    id: Option<String>,
    old_destination: String,
    new_source: String,
    new_destination_parent: String,
    new_installation_id: Option<Option<String>>,
) -> Result<(), String> {
    crate::symlink_manager::update_symlink(id, old_destination, new_source, new_destination_parent, new_installation_id).await
}

/// Select a file using the system file dialog
#[tauri::command]
pub async fn select_file_for_symlink(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let file_path = app
        .dialog()
        .file()
        .set_title("Select File")
        .blocking_pick_file();

    match file_path {
        Some(path) => match path.as_path() {
            Some(path_buf) => Ok(Some(path_buf.to_string_lossy().to_string())),
            None => Err("Invalid file path".to_string()),
        },
        None => Ok(None), // User cancelled
    }
}

/// Select a folder using the system file dialog
#[tauri::command]
pub async fn select_folder_for_symlink(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let folder_path = app
        .dialog()
        .file()
        .set_title("Select Folder")
        .blocking_pick_folder();

    match folder_path {
        Some(path) => match path.as_path() {
            Some(path_buf) => Ok(Some(path_buf.to_string_lossy().to_string())),
            None => Err("Invalid folder path".to_string()),
        },
        None => Ok(None), // User cancelled
    }
}


