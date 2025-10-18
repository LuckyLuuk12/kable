use crate::auth::{get_minecraft_account, AuthMethod, LauncherAccount};
use crate::logging::{LogLevel, Logger};
use crate::skins::types::{CustomSkinEntry, CustomSkinsRoot};
use crate::skins::types::{SkinModel, SkinUploadConfig, SkinUploadResponse};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::Utc;
use reqwest;
use serde_json;
use std::collections::HashMap;
use std::fs;
use tauri_plugin_dialog::DialogExt;
use tokio::fs as async_fs;

/// Modify a skin entry by its id in launcher_custom_skins.json
pub fn modify_skin_by_id(
    skin_id: &str,
    new_name: Option<&str>,
    new_cape_id: Option<&str>,
    new_slim: Option<bool>,
) -> Result<(), String> {
    // Find Minecraft directory and skins file
    let minecraft_dir = crate::get_default_minecraft_dir()?;
    let skins_path = minecraft_dir.join("launcher_custom_skins.json");

    // Read or create CustomSkinsRoot (sync)
    let mut root: CustomSkinsRoot = if skins_path.exists() {
        match fs::read_to_string(&skins_path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or(CustomSkinsRoot {
                custom_skins: HashMap::new(),
                version: Some(1),
            }),
            Err(_) => CustomSkinsRoot {
                custom_skins: HashMap::new(),
                version: Some(1),
            },
        }
    } else {
        // Create empty file and structure if missing
        let empty = CustomSkinsRoot {
            custom_skins: HashMap::new(),
            version: Some(1),
        };
        let json = serde_json::to_string_pretty(&empty)
            .map_err(|e| format!("Serialization error: {}", e))?;
        if let Some(parent) = skins_path.parent() {
            crate::ensure_folder_sync(parent)
                .map_err(|e| format!("Failed to create skins dir: {}", e))?;
        }
        crate::write_file_atomic_sync(&skins_path, json.as_bytes())?;
        empty
    };

    // Find and modify the skin entry
    if let Some(skin) = root.custom_skins.get_mut(skin_id) {
        if let Some(name) = new_name {
            skin.name = name.to_string();
        }
        if let Some(cape_id) = new_cape_id {
            skin.cape_id = cape_id.to_string();
        }
        if let Some(slim) = new_slim {
            skin.slim = slim;
        }
        skin.updated = Utc::now().to_rfc3339();
    } else {
        return Err(format!("Skin id '{}' not found", skin_id));
    }

    // Sort custom_skins by skin number
    let mut sorted: Vec<(String, crate::skins::types::CustomSkinEntry)> =
        root.custom_skins.drain().collect();
    sorted.sort_by_key(|(k, _)| {
        k.strip_prefix("skin_")
            .and_then(|num| num.parse::<usize>().ok())
            .unwrap_or(usize::MAX)
    });
    root.custom_skins = sorted.into_iter().collect();

    // Write back to file (sync atomic)
    let json =
        serde_json::to_string_pretty(&root).map_err(|e| format!("Serialization error: {}", e))?;
    if let Some(parent) = skins_path.parent() {
        crate::ensure_folder_sync(parent)
            .map_err(|e| format!("Failed to create skins dir: {}", e))?;
    }
    crate::write_file_atomic_sync(&skins_path, json.as_bytes())?;

    Logger::console_log(
        LogLevel::Info,
        &format!("✏️ Modified skin {} in launcher_custom_skins.json", skin_id),
        None,
    );
    Ok(())
}

/// Remove a skin entry by its id from launcher_custom_skins.json
pub fn remove_skin_by_id(skin_id: &str) -> Result<(), String> {
    // Find Minecraft directory and skins file
    let minecraft_dir = crate::get_default_minecraft_dir()?;
    let skins_path = minecraft_dir.join("launcher_custom_skins.json");

    // Read or create CustomSkinsRoot (sync)
    let mut root: CustomSkinsRoot = if skins_path.exists() {
        match fs::read_to_string(&skins_path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or(CustomSkinsRoot {
                custom_skins: HashMap::new(),
                version: Some(1),
            }),
            Err(_) => CustomSkinsRoot {
                custom_skins: HashMap::new(),
                version: Some(1),
            },
        }
    } else {
        // Create empty file and structure if missing
        let empty = CustomSkinsRoot {
            custom_skins: HashMap::new(),
            version: Some(1),
        };
        let json = serde_json::to_string_pretty(&empty)
            .map_err(|e| format!("Serialization error: {}", e))?;
        if let Some(parent) = skins_path.parent() {
            crate::ensure_folder_sync(parent)
                .map_err(|e| format!("Failed to create skins dir: {}", e))?;
        }
        crate::write_file_atomic_sync(&skins_path, json.as_bytes())?;
        empty
    };

    // Remove the skin entry
    if root.custom_skins.remove(skin_id).is_none() {
        return Err(format!("Skin id '{}' not found", skin_id));
    }

    // Sort custom_skins by skin number
    let mut sorted: Vec<(String, crate::skins::types::CustomSkinEntry)> =
        root.custom_skins.drain().collect();
    sorted.sort_by_key(|(k, _)| {
        k.strip_prefix("skin_")
            .and_then(|num| num.parse::<usize>().ok())
            .unwrap_or(usize::MAX)
    });
    root.custom_skins = sorted.into_iter().collect();

    // Write back to file (sync) — use sync atomic helper since this function is synchronous
    let json =
        serde_json::to_string_pretty(&root).map_err(|e| format!("Serialization error: {}", e))?;
    if let Some(parent) = skins_path.parent() {
        crate::ensure_folder_sync(parent)
            .map_err(|e| format!("Failed to create skins dir: {}", e))?;
    }
    crate::write_file_atomic_sync(&skins_path, json.as_bytes())?;

    Logger::console_log(
        LogLevel::Info,
        &format!(
            "🗑️ Removed skin {} from launcher_custom_skins.json",
            skin_id
        ),
        None,
    );
    Ok(())
}

/// Upload a skin file to the authenticated Microsoft/Mojang account
pub async fn upload_skin_to_account(
    config: SkinUploadConfig,
) -> Result<SkinUploadResponse, String> {
    Logger::console_log(
        LogLevel::Info,
        &format!("🎨 Adding skin locally with model: {:?}", config.model),
        None,
    );

    // Read the skin file (async)
    let skin_data = tokio::fs::read(&config.file_path)
        .await
        .map_err(|e| format!("Failed to read skin file: {}", e))?;

    // Validate it's a PNG file
    if !is_valid_skin_file(&skin_data) {
        return Err("Invalid skin file. Must be a valid PNG image.".to_string());
    }

    // Find Minecraft directory and skins file
    let minecraft_dir = crate::get_default_minecraft_dir()?;
    let skins_path = minecraft_dir.join("launcher_custom_skins.json");

    // Read or create CustomSkinsRoot (async)
    let mut root: CustomSkinsRoot = if skins_path.exists() {
        match async_fs::read_to_string(&skins_path).await {
            Ok(data) => serde_json::from_str(&data).unwrap_or(CustomSkinsRoot {
                custom_skins: HashMap::new(),
                version: Some(1),
            }),
            Err(_) => CustomSkinsRoot {
                custom_skins: HashMap::new(),
                version: Some(1),
            },
        }
    } else {
        CustomSkinsRoot {
            custom_skins: HashMap::new(),
            version: Some(1),
        }
    };

    // Find the first available skin number (fill gaps)
    let mut used_numbers: Vec<usize> = root
        .custom_skins
        .keys()
        .filter_map(|k| {
            if let Some(num_str) = k.strip_prefix("skin_") {
                num_str.parse::<usize>().ok()
            } else {
                None
            }
        })
        .collect();
    used_numbers.sort_unstable();
    let mut next_num = 1;
    for n in &used_numbers {
        match n.cmp(&next_num) {
            std::cmp::Ordering::Equal => next_num += 1,
            std::cmp::Ordering::Greater => break,
            std::cmp::Ordering::Less => {} // continue
        }
    }
    let skin_id = format!("skin_{}", next_num);
    let now = Utc::now().to_rfc3339();
    let skin_base64 = STANDARD.encode(&skin_data);
    let slim = config.model == crate::skins::types::SkinModel::Slim;

    let entry = CustomSkinEntry {
        cape_id: String::new(),
        created: now.clone(),
        id: skin_id.clone(),
        model_image: skin_base64.clone(), // modelImage same as skinImage
        name: format!("Custom Skin {}", now),
        skin_image: skin_base64.clone(),
        slim,
        texture_id: String::new(), // unknown purpose
        updated: now.clone(),
    };

    // Insert new skin with key = id
    root.custom_skins.insert(skin_id.clone(), entry.clone());

    // Sort custom_skins by skin number
    let mut sorted: Vec<(String, CustomSkinEntry)> = root.custom_skins.drain().collect();
    sorted.sort_by_key(|(k, _)| {
        k.strip_prefix("skin_")
            .and_then(|num| num.parse::<usize>().ok())
            .unwrap_or(usize::MAX)
    });
    root.custom_skins = sorted.into_iter().collect();

    // Write back to file (async)
    let json =
        serde_json::to_string_pretty(&root).map_err(|e| format!("Serialization error: {}", e))?;
    crate::ensure_parent_dir_exists_async(&skins_path).await?;
    crate::write_file_atomic_async(&skins_path, json.as_bytes())
        .await
        .map_err(|e| format!("Failed to write skin file: {}", e))?;

    Logger::console_log(
        LogLevel::Info,
        &format!("✅ Skin added locally as {}", entry.id),
        None,
    );

    Ok(SkinUploadResponse {
        success: true,
        message: format!("Skin added and applied locally as {}", entry.name),
        model_used: config.model,
    })
}

/// Change the skin model (slim/classic) for the current skin
pub async fn change_skin_model(new_model: SkinModel) -> Result<SkinUploadResponse, String> {
    Logger::console_log(
        LogLevel::Info,
        &format!("🔄 Changing skin model to: {:?}", new_model),
        None,
    );

    // Get the authenticated account
    let account = get_minecraft_account(Some(AuthMethod::DeviceCodeFlow))
        .await
        .map_err(|e| format!("Authentication required: {}", e))?;

    // Get current skin
    let current_skin = crate::skins::get_skins::get_current_skin(&account).await?;

    if !current_skin.has_skin {
        return Err("No current skin found. Upload a skin first.".to_string());
    }

    // If no URL available, we can't re-upload the same skin with a different model
    let skin_url = current_skin
        .url
        .ok_or_else(|| "Cannot change model: current skin URL not available".to_string())?;

    // Download current skin
    let skin_data = crate::skins::get_skins::download_skin_from_url(&skin_url).await?;

    // Re-upload with new model
    match upload_skin_to_mojang(&account, &skin_data, new_model).await {
        Ok(_) => {
            Logger::console_log(
                LogLevel::Info,
                &format!("✅ Skin model changed to {:?}", new_model),
                None,
            );
            Ok(SkinUploadResponse {
                success: true,
                message: format!("Skin model changed to {}", new_model.to_api_string()),
                model_used: new_model,
            })
        }
        Err(e) => {
            Logger::console_log(
                LogLevel::Error,
                &format!("❌ Model change failed: {}", e),
                None,
            );
            Err(e)
        }
    }
}

/// Select a skin file using the system file dialog
pub async fn select_skin_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    Logger::console_log(
        LogLevel::Info,
        "📁 Opening file dialog for skin selection",
        None,
    );

    let file_path = app
        .dialog()
        .file()
        .add_filter("PNG Images", &["png"])
        .set_title("Select Minecraft Skin")
        .blocking_pick_file();

    match file_path {
        Some(path) => match path.as_path() {
            Some(path_buf) => {
                let path_str = path_buf.to_string_lossy().to_string();
                Logger::console_log(
                    LogLevel::Info,
                    &format!("📄 Selected skin file: {}", path_str),
                    None,
                );
                Ok(Some(path_str))
            }
            None => Err("Invalid file path".to_string()),
        },
        None => {
            Logger::console_log(LogLevel::Info, "❌ No file selected", None);
            Ok(None)
        }
    }
}

/// Internal function to update skin data on Mojang API
async fn upload_skin_to_mojang(
    account: &LauncherAccount,
    skin_data: &[u8],
    model: SkinModel, // Assume this can return "classic" or "slim"
) -> Result<(), String> {
    let client = reqwest::Client::new();

    // Multipart form: file + variant
    let form = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(skin_data.to_vec())
                .file_name("skin.png")
                .mime_str("image/png")
                .map_err(|e| format!("Failed to create form data: {}", e))?,
        )
        .text("variant", model.to_api_string()); // "classic" or "slim"

    let url = "https://api.minecraftservices.com/minecraft/profile/skins";

    Logger::console_log(
        LogLevel::Debug,
        &format!("🌐 Uploading skin to: {}", url),
        None,
    );

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", account.access_token))
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;

    match response.status() {
        reqwest::StatusCode::OK | reqwest::StatusCode::NO_CONTENT => Ok(()),
        status => {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!(
                "Skin upload failed with status {}: {}",
                status, error_body
            ))
        }
    }
}

// TODO: Change return type to the expected SkinModel type (e.g. detect whether the file is probably a slim or classic skin)
/// Validate that the file is a valid PNG skin file
fn is_valid_skin_file(data: &[u8]) -> bool {
    // Check PNG signature
    if data.len() < 8 {
        return false;
    }

    // PNG signature: 89 50 4E 47 0D 0A 1A 0A
    let png_signature = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    if data[0..8] != png_signature {
        return false;
    }

    // Additional validation could be added here:
    // - Check image dimensions (should be 64x64, 64x32, or 128x128, 128x64)
    // - Validate PNG structure
    // - Check if it's a valid Minecraft skin format

    true
}

/// Apply an account skin (set it as the current skin)
pub async fn apply_account_skin(skin_id: String) -> Result<SkinUploadResponse, String> {
    Logger::console_log(
        LogLevel::Info,
        &format!("🎯 Applying skin: {}", skin_id),
        None,
    );

    // For the "current" skin, there's nothing to do
    if skin_id == "current" {
        return Ok(SkinUploadResponse {
            success: true,
            message: "Current skin is already active".to_string(),
            model_used: SkinModel::Classic, // Could fetch actual model if needed
        });
    }

    // Handle local skins
    if skin_id.starts_with("local_") {
        // Get all local skins
        let local_skins = crate::skins::get_skins::get_local_skins().await?;
        if let Some(skin) = local_skins.iter().find(|s| s.id == skin_id) {
            if let Some(url) = &skin.url {
                // Extract base64 from data URL
                let b64 = url.strip_prefix("data:image/png;base64,").unwrap_or(url);
                // Use the recommended base64 decode engine
                let skin_data = match base64::engine::general_purpose::STANDARD.decode(b64) {
                    Ok(data) => data,
                    Err(e) => return Err(format!("Failed to decode skin data: {}", e)),
                };
                // Get authenticated account
                let account = get_minecraft_account(Some(AuthMethod::DeviceCodeFlow))
                    .await
                    .map_err(|e| format!("Authentication required: {}", e))?;
                // Upload skin
                match upload_skin_to_mojang(&account, &skin_data, skin.model).await {
                    Ok(_) => {
                        Logger::console_log(
                            LogLevel::Info,
                            "✅ Local skin applied successfully",
                            None,
                        );
                        return Ok(SkinUploadResponse {
                            success: true,
                            message: "Local skin applied successfully".to_string(),
                            model_used: skin.model,
                        });
                    }
                    Err(e) => {
                        Logger::console_log(
                            LogLevel::Error,
                            &format!("❌ Failed to apply local skin: {}", e),
                            None,
                        );
                        return Err(e);
                    }
                }
            }
        }
        return Err(format!(
            "Local skin with ID '{}' not found or missing data URL",
            skin_id
        ));
    }

    // For future implementation when we have access to skin history:
    // 1. Get the skin data from Microsoft's API using the skin_id
    // 2. Re-upload it to set as current
    Err("Online skin history management not yet implemented".to_string())
}

/// Apply a cape to the authenticated user's profile
pub async fn apply_cape(cape_id: Option<String>) -> Result<String, String> {
    Logger::console_log(
        LogLevel::Info,
        &format!("🎽 Applying cape: {:?}", cape_id),
        None,
    );

    // Get the authenticated account
    let account = get_minecraft_account(Some(AuthMethod::DeviceCodeFlow))
        .await
        .map_err(|e| format!("Authentication required to apply cape: {}", e))?;

    let client = reqwest::Client::new();

    // Mojang API endpoint for changing active cape
    let url = if cape_id.is_some() {
        // Apply specific cape
        "https://api.minecraftservices.com/minecraft/profile/capes/active".to_string()
    } else {
        // Remove cape (hide)
        "https://api.minecraftservices.com/minecraft/profile/capes/active".to_string()
    };

    let response = if let Some(id) = &cape_id {
        // PUT request with cape ID in body
        let body = serde_json::json!({
            "capeId": id
        });

        client
            .put(&url)
            .header("Authorization", format!("Bearer {}", account.access_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to apply cape: {}", e))?
    } else {
        // DELETE request to remove cape
        client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", account.access_token))
            .send()
            .await
            .map_err(|e| format!("Failed to remove cape: {}", e))?
    };

    if response.status() == reqwest::StatusCode::OK
        || response.status() == reqwest::StatusCode::NO_CONTENT
    {
        let message = if cape_id.is_some() {
            "✅ Cape applied successfully".to_string()
        } else {
            "✅ Cape removed successfully".to_string()
        };

        Logger::console_log(LogLevel::Info, &message, None);
        Ok(message)
    } else {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Cape operation failed with status: {}", error_body))
    }
}
