use crate::resourcepacks;

/// Get all installed resource packs from the resourcepacks directory
#[tauri::command]
pub async fn get_installed_resourcepacks(
    minecraft_path: String,
) -> Result<Vec<resourcepacks::ResourcePack>, String> {
    resourcepacks::get_installed_resourcepacks(minecraft_path).await
}

/// Delete resource pack
#[tauri::command]
pub async fn delete_resourcepack(minecraft_path: String, pack_file: String) -> Result<(), String> {
    resourcepacks::delete_resourcepack(minecraft_path, pack_file).await
}

/// Install resource pack from file
#[tauri::command]
pub async fn install_resourcepack(
    minecraft_path: String,
    pack_file_path: String,
) -> Result<String, String> {
    resourcepacks::install_resourcepack(minecraft_path, pack_file_path).await
}

/// Get resource pack info
#[tauri::command]
pub async fn get_resourcepack_info(
    minecraft_path: String,
    pack_file: String,
) -> Result<resourcepacks::ResourcePack, String> {
    resourcepacks::get_resourcepack_info(minecraft_path, pack_file).await
}

/// Search for resource packs on Modrinth
#[tauri::command]
pub async fn search_modrinth_resourcepacks(
    query: String,
    minecraft_version: Option<String>,
    limit: u32,
    offset: u32,
) -> Result<Vec<resourcepacks::ResourcePackDownload>, String> {
    resourcepacks::search_modrinth_resourcepacks(query, minecraft_version, limit, offset).await
}

/// Search for resource packs on Modrinth with custom filter facets
#[tauri::command]
pub async fn search_modrinth_resourcepacks_with_facets(
    query: String,
    minecraft_version: Option<String>,
    facets: Option<resourcepacks::ResourcePackFilterFacets>,
    limit: u32,
    offset: u32,
) -> Result<Vec<resourcepacks::ResourcePackDownload>, String> {
    resourcepacks::search_modrinth_resourcepacks_with_facets(
        query,
        minecraft_version,
        facets,
        limit,
        offset,
    )
    .await
}

/// Get resource pack details from Modrinth
#[tauri::command]
pub async fn get_modrinth_resourcepack_details(
    project_id: String,
) -> Result<resourcepacks::ResourcePackDownload, String> {
    resourcepacks::get_modrinth_resourcepack_details(project_id).await
}

/// Download and install resource pack from Modrinth
#[tauri::command]
pub async fn download_and_install_resourcepack(
    minecraft_path: String,
    download_url: String,
    filename: String,
) -> Result<String, String> {
    resourcepacks::download_and_install_resourcepack(minecraft_path, download_url, filename).await
}

/// Download and install resource pack from Modrinth to a dedicated folder
#[tauri::command]
pub async fn download_and_install_resourcepack_to_dedicated(
    minecraft_path: String,
    dedicated_folder: String,
    download_url: String,
    filename: String,
) -> Result<String, String> {
    resourcepacks::download_and_install_resourcepack_to_dedicated(
        minecraft_path,
        dedicated_folder,
        download_url,
        filename,
    )
    .await
}

/// Setup symbolic link from dedicated resource pack folder to .minecraft/resourcepacks
#[tauri::command]
pub async fn setup_resourcepack_symlink(
    minecraft_path: String,
    dedicated_folder: String,
    symlink_name: String,
) -> Result<(), String> {
    resourcepacks::setup_resourcepack_symlink(minecraft_path, dedicated_folder, symlink_name).await
}

/// Remove symbolic link from .minecraft/resourcepacks
#[tauri::command]
pub async fn remove_resourcepack_symlink(
    minecraft_path: String,
    symlink_name: String,
) -> Result<(), String> {
    resourcepacks::remove_resourcepack_symlink(minecraft_path, symlink_name).await
}

/// Delete resource pack from dedicated folder and clean up symlink
#[tauri::command]
pub async fn delete_resourcepack_from_dedicated(
    minecraft_path: String,
    dedicated_folder: String,
    pack_file: String,
    symlink_name: Option<String>,
) -> Result<(), String> {
    resourcepacks::delete_resourcepack_from_dedicated(
        minecraft_path,
        dedicated_folder,
        pack_file,
        symlink_name,
    )
    .await
}
