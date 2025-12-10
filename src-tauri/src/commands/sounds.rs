use crate::sounds;

/// List all available soundpacks
#[tauri::command]
pub async fn list_soundpacks() -> Result<Vec<String>, String> {
    sounds::list_soundpacks().await
}

/// Get metadata for a specific soundpack
#[tauri::command]
pub async fn get_soundpack_metadata(pack: String) -> Result<sounds::SoundpackMetadata, String> {
    sounds::get_soundpack_metadata(pack).await
}

/// Load a sound file from a soundpack
#[tauri::command]
pub async fn load_soundpack_file(pack: String, file: String) -> Result<Vec<u8>, String> {
    sounds::load_soundpack_file(pack, file).await
}

/// Import a soundpack from a ZIP file
#[tauri::command]
pub async fn import_soundpack_zip(path: String) -> Result<String, String> {
    sounds::import_soundpack_zip(path).await
}

/// Get the sounds directory path for frontend file operations
#[tauri::command]
pub async fn get_sounds_directory_path() -> Result<String, String> {
    sounds::get_sounds_directory_path().await
}

/// Open the sounds directory in the system file explorer
#[tauri::command]
pub async fn open_sounds_directory() -> Result<(), String> {
    sounds::open_sounds_directory().await
}
