use crate::features::customization::images;

#[tauri::command]
pub async fn resolve_image_path(key: String) -> Result<String, String> {
    images::resolve_image_path(key).await
}
