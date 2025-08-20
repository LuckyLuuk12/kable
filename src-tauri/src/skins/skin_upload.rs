use crate::auth::{get_minecraft_account, AuthMethod, LauncherAccount};
use crate::logging::{LogLevel, Logger};
use crate::skins::types::{CurrentSkin, SkinModel, SkinUploadConfig, SkinUploadResponse, AccountSkin};
use base64::{Engine as _, engine::general_purpose};
use reqwest;
use serde_json::Value;
use std::fs;
use tauri_plugin_dialog::DialogExt;

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
    let current_skin = get_current_skin(&account).await?;
    
    if !current_skin.has_skin {
        return Err("No current skin found. Upload a skin first.".to_string());
    }

    // If no URL available, we can't re-upload the same skin with a different model
    let skin_url = current_skin.url.ok_or_else(|| {
        "Cannot change model: current skin URL not available".to_string()
    })?;

    // Download current skin
    let skin_data = download_skin_from_url(&skin_url).await?;

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

/// Get the current skin information from Mojang
pub async fn get_current_skin_info() -> Result<CurrentSkin, String> {
    Logger::console_log(
        LogLevel::Info,
        "🔍 Fetching current skin information",
        None,
    );

    // Get the authenticated account
    let account = get_minecraft_account(Some(AuthMethod::DeviceCodeFlow))
        .await
        .map_err(|e| format!("Authentication required: {}", e))?;

    get_current_skin(&account).await
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

/// Get all skins stored in the user's Microsoft/Mojang account
pub async fn get_all_account_skins() -> Result<Vec<AccountSkin>, String> {
    Logger::console_log(
        LogLevel::Info,
        "🎨 Fetching all skins from Microsoft account",
        None,
    );

    // Get the authenticated account
    let account = get_minecraft_account(Some(AuthMethod::DeviceCodeFlow))
        .await
        .map_err(|e| format!("Authentication required: {}", e))?;

    // Unfortunately, Microsoft/Mojang doesn't provide a direct API to get all user skins
    // The skin history feature in the official launcher likely uses internal APIs
    // 
    // For now, we can only get the current skin. Future implementation could:
    // 1. Store previously uploaded skins locally
    // 2. Use unofficial APIs if available
    // 3. Parse skin history from other sources
    
    let current_skin = get_current_skin(&account).await?;
    
    if current_skin.has_skin {
        let account_skin = AccountSkin {
            id: "current".to_string(),
            name: "Current Skin".to_string(),
            url: current_skin.url,
            model: current_skin.model,
            is_current: true,
            uploaded_date: None, // We don't have this information from the API
        };
        Ok(vec![account_skin])
    } else {
        Ok(vec![])
    }
}

/// Apply an account skin (set it as the current skin)
pub async fn apply_account_skin(skin_id: String) -> Result<SkinUploadResponse, String> {
    Logger::console_log(
        LogLevel::Info,
        &format!("🎯 Applying account skin: {}", skin_id),
        None,
    );

    // For the "current" skin, there's nothing to do
    if skin_id == "current" {
        return Ok(SkinUploadResponse {
            success: true,
            message: "Current skin is already active".to_string(),
            model_used: SkinModel::Classic, // We'd need to get the actual model
        });
    }

    // For future implementation when we have access to skin history:
    // 1. Get the skin data from Microsoft's API using the skin_id
    // 2. Re-upload it to set as current
    
    Err("Skin history management not yet implemented".to_string())
}

/// Internal function to upload skin data to Mojang API
async fn upload_skin_to_mojang(
    account: &LauncherAccount,
    skin_data: &[u8],
    model: SkinModel,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    
    // Create form data
    let form = reqwest::multipart::Form::new()
        .part("file", reqwest::multipart::Part::bytes(skin_data.to_vec())
            .file_name("skin.png")
            .mime_str("image/png")
            .map_err(|e| format!("Failed to create form data: {}", e))?)
        .text("model", model.to_api_string());

    let url = format!(
        "https://api.mojang.com/user/profile/{}/skin",
        account.minecraft_profile.id
    );

    Logger::console_log(
        LogLevel::Debug,
        &format!("🌐 Making API request to: {}", url),
        None,
    );

    let response = client
        .put(&url)
        .header("Authorization", format!("Bearer {}", account.access_token))
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;

    match response.status() {
        reqwest::StatusCode::OK => Ok(()),
        reqwest::StatusCode::NO_CONTENT => Ok(()),
        status => {
            let error_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("API request failed with status {}: {}", status, error_body))
        }
    }
}

/// Get current skin information from Mojang profile API
async fn get_current_skin(account: &LauncherAccount) -> Result<CurrentSkin, String> {
    let client = reqwest::Client::new();
    
    // Get profile with textures
    let url = format!(
        "https://sessionserver.mojang.com/session/minecraft/profile/{}",
        account.minecraft_profile.id
    );

    Logger::console_log(
        LogLevel::Debug,
        &format!("🌐 Fetching profile from: {}", url),
        None,
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch profile: {}", e))?;

    if response.status() != reqwest::StatusCode::OK {
        return Err(format!("Profile request failed with status: {}", response.status()));
    }

    let profile_data: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse profile response: {}", e))?;

    // Parse the profile data to extract skin information
    let properties = profile_data
        .get("properties")
        .and_then(|p| p.as_array())
        .ok_or_else(|| "No properties found in profile".to_string())?;

    for property in properties {
        if property.get("name").and_then(|n| n.as_str()) == Some("textures") {
            let value = property
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "No texture value found".to_string())?;

            // Decode base64
            let decoded = general_purpose::STANDARD
                .decode(value)
                .map_err(|e| format!("Failed to decode texture data: {}", e))?;

            let texture_data: Value = serde_json::from_slice(&decoded)
                .map_err(|e| format!("Failed to parse texture JSON: {}", e))?;

            // Extract skin information
            if let Some(skin) = texture_data.get("textures").and_then(|t| t.get("SKIN")) {
                let url = skin.get("url").and_then(|u| u.as_str()).map(|s| s.to_string());
                let metadata = skin.get("metadata");
                
                let model = if let Some(meta) = metadata {
                    let model_str = meta.get("model").and_then(|m| m.as_str()).unwrap_or("classic");
                    SkinModel::from_api_string(model_str).unwrap_or(SkinModel::Classic)
                } else {
                    SkinModel::Classic
                };

                return Ok(CurrentSkin {
                    model,
                    url,
                    has_skin: true,
                });
            }
        }
    }

    Ok(CurrentSkin {
        model: SkinModel::Classic,
        url: None,
        has_skin: false,
    })
}

/// Download skin data from a URL
async fn download_skin_from_url(url: &str) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::new();
    
    Logger::console_log(
        LogLevel::Debug,
        &format!("⬇️ Downloading skin from: {}", url),
        None,
    );

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download skin: {}", e))?;

    if response.status() != reqwest::StatusCode::OK {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let data = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read skin data: {}", e))?;

    Ok(data.to_vec())
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
