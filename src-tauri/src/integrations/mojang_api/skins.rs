use api_types::skins::{AccountCape, AccountSkin, CurrentSkin, PlayerProfile, SkinModel};
use base64::{engine::general_purpose, Engine as _};
use reqwest::multipart::{Form, Part};
use serde::Deserialize;
use serde_json::Value;

use crate::constants::{MINECRAFT_PROFILE_CAPES_URL, MINECRAFT_PROFILE_SKINS_URL, MINECRAFT_PROFILE_URL, MINECRAFT_SESSION_PROFILE_URL};

fn api_error(context: &str, error: impl std::fmt::Display) -> String {
    format!("{context}: {error}")
}

#[derive(Debug, Deserialize)]
struct MinecraftProfileResponse {
    id: String,
    name: String,
    #[serde(default)]
    skins: Vec<MinecraftSkinResponse>,
    #[serde(default)]
    capes: Vec<MinecraftCapeResponse>,
}

#[derive(Debug, Deserialize)]
struct MinecraftSkinResponse {
    id: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    variant: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MinecraftCapeResponse {
    id: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    alias: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SessionProfileResponse {
    #[serde(default)]
    properties: Vec<SessionProperty>,
}

#[derive(Debug, Deserialize)]
struct SessionProperty {
    name: String,
    value: String,
}

/// Retrieves the authenticated player's Minecraft profile, skins and capes.
pub async fn get_player_profile(access_token: &str) -> Result<PlayerProfile, String> {
    let response = reqwest::Client::new()
        .get(MINECRAFT_PROFILE_URL)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|error| api_error("Failed to request Minecraft profile", error))?
        .error_for_status()
        .map_err(|error| api_error("Minecraft profile request failed", error))?;

    let profile: MinecraftProfileResponse = response.json().await.map_err(|error| api_error("Failed to parse Minecraft profile", error))?;

    let skins = profile
        .skins
        .into_iter()
        .map(|skin| {
            let model = skin.variant.as_deref().and_then(SkinModel::from_api_string).unwrap_or_default();

            AccountSkin {
                id: skin.id,
                name: match model {
                    SkinModel::Classic => "Classic".to_string(),
                    SkinModel::Slim => "Slim".to_string(),
                },
                url: skin.url,
                model,
                is_current: skin.state.eq_ignore_ascii_case("ACTIVE"),
                uploaded_date: None,
            }
        })
        .collect();

    let capes = profile
        .capes
        .into_iter()
        .map(|cape| AccountCape { id: cape.id, state: cape.state, url: cape.url, alias: cape.alias })
        .collect();

    Ok(PlayerProfile { id: profile.id, name: profile.name, skins, capes })
}

/// Looks up the current skin using the Minecraft session server.
///
/// This endpoint does not require a bearer token, only the Minecraft profile ID.
pub async fn get_current_skin_info(profile_id: &str) -> Result<CurrentSkin, String> {
    let url = format!("{}/{}", MINECRAFT_SESSION_PROFILE_URL.trim_end_matches('/'), profile_id);

    let response = reqwest::Client::new()
        .get(&url)
        .query(&[("unsigned", "false")])
        .send()
        .await
        .map_err(|error| api_error("Failed to request session profile", error))?
        .error_for_status()
        .map_err(|error| api_error("Session profile request failed", error))?;

    let profile: SessionProfileResponse = response.json().await.map_err(|error| api_error("Failed to parse session profile", error))?;

    let Some(textures_property) = profile.properties.iter().find(|property| property.name == "textures") else {
        return Ok(CurrentSkin { model: SkinModel::Classic, url: None, has_skin: false });
    };

    let decoded = general_purpose::STANDARD
        .decode(&textures_property.value)
        .map_err(|error| api_error("Failed to decode skin texture data", error))?;

    let textures: Value = serde_json::from_slice(&decoded).map_err(|error| api_error("Failed to parse skin texture data", error))?;

    let skin = &textures["textures"]["SKIN"];

    let skin_url = skin["url"].as_str().map(str::to_string);

    let model = skin["metadata"]["model"].as_str().and_then(SkinModel::from_api_string).unwrap_or_default();

    Ok(CurrentSkin { has_skin: skin_url.is_some(), url: skin_url, model })
}

/// Downloads skin image bytes from a URL.
pub async fn download_skin_from_url(url: &str) -> Result<Vec<u8>, String> {
    let response = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .map_err(|error| api_error("Failed to download skin", error))?
        .error_for_status()
        .map_err(|error| api_error("Skin download failed", error))?;

    response.bytes().await.map(|bytes| bytes.to_vec()).map_err(|error| api_error("Failed to read downloaded skin", error))
}

/// Uploads a PNG skin to the authenticated Minecraft profile.
pub async fn upload_skin_to_mojang(access_token: &str, skin_data: &[u8], model: SkinModel) -> Result<(), String> {
    if !skin_data.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err("Skin data must be a PNG image".into());
    }

    let skin_part = Part::bytes(skin_data.to_vec())
        .file_name("skin.png")
        .mime_str("image/png")
        .map_err(|error| api_error("Failed to prepare skin upload", error))?;

    let form = Form::new().part("file", skin_part).text("variant", model.to_api_string().to_string());

    reqwest::Client::new()
        .post(MINECRAFT_PROFILE_SKINS_URL)
        .bearer_auth(access_token)
        .multipart(form)
        .send()
        .await
        .map_err(|error| api_error("Failed to upload skin", error))?
        .error_for_status()
        .map_err(|error| api_error("Mojang skin upload failed", error))?;

    Ok(())
}

/// Changes the active skin's model by downloading and re-uploading the image.
///
/// The Mojang API does not provide a direct model-only update operation.
pub async fn change_skin_model(access_token: &str, profile_id: &str, new_model: SkinModel) -> Result<(), String> {
    let current_skin = get_current_skin_info(profile_id).await?;

    let skin_url = current_skin.url.ok_or_else(|| "The Minecraft profile does not have a skin to change".to_string())?;

    if current_skin.model == new_model {
        return Ok(());
    }

    let skin_data = download_skin_from_url(&skin_url).await?;

    upload_skin_to_mojang(access_token, &skin_data, new_model).await
}

/// Returns the active cape, if one is active.
pub async fn get_active_cape(access_token: &str) -> Result<Option<AccountCape>, String> {
    let profile = get_player_profile(access_token).await?;

    Ok(profile.capes.into_iter().find(|cape| cape.state.eq_ignore_ascii_case("ACTIVE")))
}

/// Activates a cape by ID, or hides the active cape when `cape_id` is `None`.
pub async fn apply_cape(access_token: &str, cape_id: Option<&str>) -> Result<(), String> {
    let active_url = format!("{}/active", MINECRAFT_PROFILE_CAPES_URL.trim_end_matches('/'));

    let client = reqwest::Client::new();

    let response = match cape_id {
        Some(cape_id) => client
            .put(&active_url)
            .bearer_auth(access_token)
            .json(&serde_json::json!({ "capeId": cape_id }))
            .send()
            .await
            .map_err(|error| api_error("Failed to activate cape", error))?,
        None => client
            .delete(&active_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|error| api_error("Failed to deactivate cape", error))?,
    };

    response.error_for_status().map_err(|error| api_error("Cape operation failed", error))?;

    Ok(())
}
