use crate::shaders;

/// Get all installed shaders from the shaderpacks directory
#[tauri::command]
pub async fn get_installed_shaders(
    minecraft_path: String,
) -> Result<Vec<shaders::ShaderPack>, String> {
    shaders::get_installed_shaders(minecraft_path).await
}

/// Enable/disable shader pack
#[tauri::command]
pub async fn toggle_shader(
    minecraft_path: String,
    shader_file: String,
    enabled: bool,
) -> Result<(), String> {
    shaders::toggle_shader(minecraft_path, shader_file, enabled).await
}

/// Delete shader pack
#[tauri::command]
pub async fn delete_shader(minecraft_path: String, shader_file: String) -> Result<(), String> {
    shaders::delete_shader(minecraft_path, shader_file).await
}

/// Install shader pack from file
#[tauri::command]
pub async fn install_shader_pack(
    minecraft_path: String,
    shader_file_path: String,
) -> Result<String, String> {
    shaders::install_shader_pack(minecraft_path, shader_file_path).await
}

/// Get shader pack info
#[tauri::command]
pub async fn get_shader_info(
    minecraft_path: String,
    shader_file: String,
) -> Result<shaders::ShaderPack, String> {
    shaders::get_shader_info(minecraft_path, shader_file).await
}

/// Search for shader packs on Modrinth
#[tauri::command]
pub async fn search_modrinth_shaders(
    query: String,
    minecraft_version: Option<String>,
    limit: u32,
    offset: u32,
) -> Result<Vec<shaders::ShaderDownload>, String> {
    shaders::search_modrinth_shaders(query, minecraft_version, limit, offset).await
}

/// Get shader pack details from Modrinth
#[tauri::command]
pub async fn get_modrinth_shader_details(
    project_id: String,
) -> Result<shaders::ShaderDownload, String> {
    shaders::get_modrinth_shader_details(project_id).await
}

/// Download and install shader from Modrinth
#[tauri::command]
pub async fn download_and_install_shader(
    minecraft_path: String,
    download_url: String,
    filename: String,
) -> Result<String, String> {
    shaders::download_and_install_shader(minecraft_path, download_url, filename).await
}
