use crate::auth::{get_minecraft_account, AuthMethod, LauncherAccount};
use crate::logging::{LogLevel, Logger};
use crate::skins::types::{SkinModel, SkinUploadConfig, SkinUploadResponse};
use std::fs;
use base64::Engine;
use tauri_plugin_dialog::DialogExt;
use reqwest;

/// Upload a skin file to the authenticated Microsoft/Mojang account
pub async fn upload_skin_to_account(config: SkinUploadConfig) -> Result<SkinUploadResponse, String> {
    Logger::console_log(
        LogLevel::Info,
        &format!("🎨 Starting skin upload with model: {:?}", config.model),
        None,
    );

    // Get the authenticated account
    let account = get_minecraft_account(Some(AuthMethod::DeviceCodeFlow))
        .await
        .map_err(|e| format!("Authentication required: {}", e))?;

    // Validate the account has Minecraft
    if account.access_token.is_empty() {
        return Err("No valid access token found".to_string());
    }

    // Read the skin file
    let skin_data = fs::read(&config.file_path)
        .map_err(|e| format!("Failed to read skin file: {}", e))?;

    // Validate it's a PNG file
    if !is_valid_skin_file(&skin_data) {
        return Err("Invalid skin file. Must be a valid PNG image.".to_string());
    }

    // Upload to Mojang API
    match upload_skin_to_mojang(&account, &skin_data, config.model).await {
        Ok(_) => {
            Logger::console_log(
                LogLevel::Info,
                "✅ Skin uploaded successfully",
                None,
            );
            Ok(SkinUploadResponse {
                success: true,
                message: "Skin uploaded successfully".to_string(),
                model_used: config.model,
            })
        }
        Err(e) => {
            Logger::console_log(
                LogLevel::Error,
                &format!("❌ Skin upload failed: {}", e),
                None,
            );
            Err(e)
        }
    }
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
    let skin_url = current_skin.url.ok_or_else(|| {
        "Cannot change model: current skin URL not available".to_string()
    })?;

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
            Logger::console_log(
                LogLevel::Info,
                "❌ No file selected",
                None,
            );
            Ok(None)
        }
    }
}

/// Internal function to upload skin data to Mojang API
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
            let error_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Skin upload failed with status {}: {}", status, error_body))
        }
    }
}


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
        return Err(format!("Local skin with ID '{}' not found or missing data URL", skin_id));
    }

    // For future implementation when we have access to skin history:
    // 1. Get the skin data from Microsoft's API using the skin_id
    // 2. Re-upload it to set as current
    Err("Online skin history management not yet implemented".to_string())
}
