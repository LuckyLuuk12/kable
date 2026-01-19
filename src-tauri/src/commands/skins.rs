/// Get all local skins from the Minecraft directory
#[tauri::command]
pub async fn get_local_skins() -> Result<Vec<AccountSkin>, String> {
    crate::skins::get_skins::get_local_skins().await
}
// Skin commands - Tauri command wrappers for skin functionality
// These commands call the functions in the skins module
use crate::skins::*;

/// Get the full player profile (id, name, skins, capes) from Mojang API
#[tauri::command]
pub async fn get_player_profile() -> Result<crate::skins::types::PlayerProfile, String> {
    crate::skins::get_player_profile().await
}

/// Remove a skin entry by its id from launcher_custom_skins.json
#[tauri::command]
pub fn remove_skin_by_id(skin_id: String) -> Result<(), String> {
    crate::skins::modify_skins::remove_skin_by_id(&skin_id)
}

/// Modify a skin entry by its id in launcher_custom_skins.json
#[tauri::command]
pub fn modify_skin_by_id(
    skin_id: String,
    new_name: Option<String>,
    new_cape_id: Option<String>,
    new_slim: Option<bool>,
) -> Result<(), String> {
    crate::skins::modify_skins::modify_skin_by_id(
        &skin_id,
        new_name.as_deref(),
        new_cape_id.as_deref(),
        new_slim,
    )
}

/// Upload a skin file to the authenticated Microsoft/Mojang account
#[tauri::command]
pub async fn upload_skin_to_account(
    config: SkinUploadConfig,
) -> Result<SkinUploadResponse, String> {
    crate::skins::upload_skin_to_account(config).await
}

/// Change the skin model (slim/classic) for the current skin
#[tauri::command]
pub async fn change_skin_model(new_model: SkinModel) -> Result<SkinUploadResponse, String> {
    crate::skins::change_skin_model(new_model).await
}

/// Get skin URL for a specific player by UUID
#[tauri::command]
pub async fn get_skin_url_by_uuid(uuid: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    // Note: Minecraft UUIDs are public identifiers, not sensitive data.
    // This is the official Mojang Session Server API endpoint - the UUID
    // must be in the URL path as per Mojang's API specification.
    // See: https://minecraft.wiki/w/Mojang_API#Query_player_profile
    let url = format!(
        "https://sessionserver.mojang.com/session/minecraft/profile/{}",
        uuid
    );

    // codeql[rust/cleartext-transmission] - Minecraft UUIDs are public data
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch profile: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Profile fetch failed with status: {}",
            response.status()
        ));
    }

    let profile: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse profile: {}", e))?;

    // Extract skin URL from profile properties
    if let Some(properties) = profile["properties"].as_array() {
        for prop in properties {
            if prop["name"].as_str() == Some("textures") {
                if let Some(value) = prop["value"].as_str() {
                    let decoded =
                        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, value)
                            .map_err(|e| format!("Failed to decode textures: {}", e))?;
                    let textures: serde_json::Value = serde_json::from_slice(&decoded)
                        .map_err(|e| format!("Failed to parse textures: {}", e))?;

                    if let Some(skin_url) = textures["textures"]["SKIN"]["url"].as_str() {
                        return Ok(skin_url.to_string());
                    }
                }
            }
        }
    }

    Err("No skin URL found in profile".to_string())
}

/// Get the current skin information from Mojang
#[tauri::command]
pub async fn get_current_skin_info() -> Result<CurrentSkin, String> {
    crate::skins::get_current_skin_info().await
}

/// Select a skin file using the system file dialog
#[tauri::command]
pub async fn select_skin_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    crate::skins::select_skin_file(app).await
}

/// Get all skins stored in the user's Microsoft/Mojang account
#[tauri::command]
pub async fn get_all_account_skins() -> Result<Vec<AccountSkin>, String> {
    crate::skins::get_all_account_skins().await
}

/// Apply an account skin (set it as the current skin)
#[tauri::command]
pub async fn apply_account_skin(skin_id: String) -> Result<SkinUploadResponse, String> {
    crate::skins::apply_account_skin(skin_id).await
}

/// Get the currently active cape for the authenticated user
#[tauri::command]
pub async fn get_active_cape() -> Result<Option<crate::skins::types::AccountCape>, String> {
    crate::skins::get_active_cape().await
}

/// Apply a cape to the authenticated user's profile (or remove it if None)
#[tauri::command]
pub async fn apply_cape(cape_id: Option<String>) -> Result<String, String> {
    crate::skins::apply_cape(cape_id).await
}
