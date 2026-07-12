use crate::features::customization::icons;
use crate::features::customization::images;
use crate::features::customization::settings;
use crate::features::customization::sounds;
use crate::features::customization::themes;
use api_types::icons::IconTemplate;
use api_types::settings::CategorizedLauncherSettings;
use api_types::sounds::SoundpackMetadata;

#[tauri::command]
#[specta::specta]
pub async fn get_icon_templates() -> Result<Vec<IconTemplate>, String> {
    icons::get_icon_templates().await
}

#[tauri::command]
#[specta::specta]
pub async fn save_custom_icon_template(template: IconTemplate) -> Result<String, String> {
    icons::save_custom_icon_template(template).await
}

#[tauri::command]
#[specta::specta]
pub async fn delete_custom_icon_template(template_name: String) -> Result<(), String> {
    icons::delete_custom_icon_template(template_name).await
}

#[tauri::command]
#[specta::specta]
pub async fn open_icons_directory() -> Result<(), String> {
    icons::open_icons_directory().await
}

#[tauri::command]
#[specta::specta]
pub async fn resolve_image_path(key: String) -> Result<String, String> {
    images::resolve_image_path(key).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_settings() -> Result<CategorizedLauncherSettings, String> {
    settings::load_settings().await
}

#[tauri::command]
#[specta::specta]
pub async fn set_settings(settings: CategorizedLauncherSettings) -> Result<(), String> {
    settings::save_settings(settings).await
}

#[tauri::command]
#[specta::specta]
pub async fn list_soundpacks() -> Result<Vec<String>, String> {
    sounds::list_soundpacks().await
}

#[tauri::command]
#[specta::specta]
pub async fn get_soundpack_metadata(pack: String) -> Result<SoundpackMetadata, String> {
    sounds::get_soundpack_metadata(pack).await
}

#[tauri::command]
#[specta::specta]
pub async fn load_soundpack_file(pack: String, file: String) -> Result<Vec<u8>, String> {
    sounds::load_soundpack_file(pack, file).await
}

#[tauri::command]
#[specta::specta]
pub async fn import_soundpack_zip(path: String) -> Result<String, String> {
    sounds::import_soundpack_zip(path).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_sounds_directory_path() -> Result<String, String> {
    sounds::get_sounds_directory_path().await
}

#[tauri::command]
#[specta::specta]
pub async fn open_sounds_directory() -> Result<(), String> {
    sounds::open_sounds_directory().await
}

#[tauri::command]
#[specta::specta]
pub async fn list_css_themes() -> Result<Vec<String>, String> {
    themes::list_css_themes().await
}

#[tauri::command]
#[specta::specta]
pub async fn load_css_theme(name: String) -> Result<String, String> {
    themes::load_css_theme(name).await
}

#[tauri::command]
#[specta::specta]
pub async fn save_css_theme(name: String, css: String) -> Result<String, String> {
    themes::save_css_theme(name, css).await
}

#[tauri::command]
#[specta::specta]
pub async fn delete_css_theme(name: String) -> Result<(), String> {
    themes::delete_css_theme(name).await
}

#[tauri::command]
#[specta::specta]
pub async fn open_css_themes_directory() -> Result<(), String> {
    themes::open_css_themes_directory().await
}
