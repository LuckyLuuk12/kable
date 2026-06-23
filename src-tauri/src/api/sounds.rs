use crate::features::customization::sounds;
use api_types::sounds::SoundpackMetadata;

#[tauri::command]
pub async fn list_soundpacks() -> Result<Vec<String>, String> {
    sounds::list_soundpacks().await
}

#[tauri::command]
pub async fn get_soundpack_metadata(pack: String) -> Result<SoundpackMetadata, String> {
    sounds::get_soundpack_metadata(pack).await
}

#[tauri::command]
pub async fn load_soundpack_file(pack: String, file: String) -> Result<Vec<u8>, String> {
    sounds::load_soundpack_file(pack, file).await
}

#[tauri::command]
pub async fn import_soundpack_zip(path: String) -> Result<String, String> {
    sounds::import_soundpack_zip(path).await
}

#[tauri::command]
pub async fn get_sounds_directory_path() -> Result<String, String> {
    sounds::get_sounds_directory_path().await
}

#[tauri::command]
pub async fn open_sounds_directory() -> Result<(), String> {
    sounds::open_sounds_directory().await
}
