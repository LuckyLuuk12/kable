use crate::features::customization::icons;
use icons::CustomIconTemplate;

#[tauri::command]
pub async fn get_custom_icon_templates() -> Result<Vec<CustomIconTemplate>, String> {
    icons::get_custom_icon_templates().await
}

#[tauri::command]
pub async fn save_custom_icon_template(template: CustomIconTemplate) -> Result<String, String> {
    icons::save_custom_icon_template(template).await
}

#[tauri::command]
pub async fn delete_custom_icon_template(template_name: String) -> Result<(), String> {
    icons::delete_custom_icon_template(template_name).await
}

#[tauri::command]
pub async fn open_icons_directory() -> Result<(), String> {
    icons::open_icons_directory().await
}