use crate::icons;

/// Get all custom icon templates
#[tauri::command]
pub async fn get_custom_icon_templates() -> Result<Vec<icons::CustomIconTemplate>, String> {
    icons::get_custom_icon_templates().await
}

/// Save a custom icon template
#[tauri::command]
pub async fn save_custom_icon_template(
    template: icons::CustomIconTemplate,
) -> Result<String, String> {
    icons::save_custom_icon_template(template).await
}

/// Delete a custom icon template
#[tauri::command]
pub async fn delete_custom_icon_template(template_name: String) -> Result<(), String> {
    icons::delete_custom_icon_template(template_name).await
}

/// Validate an icon template
#[tauri::command]
pub async fn validate_icon_template(
    template_content: String,
    format: String,
) -> Result<icons::CustomIconTemplate, String> {
    icons::validate_icon_template(template_content, format).await
}

/// Get the icons directory path for frontend file operations
#[tauri::command]
pub async fn get_icons_directory_path() -> Result<String, String> {
    icons::get_icons_directory_path().await
}

/// Open the icons directory in the system file explorer
#[tauri::command]
pub async fn open_icons_directory() -> Result<(), String> {
    icons::open_icons_directory().await
}
