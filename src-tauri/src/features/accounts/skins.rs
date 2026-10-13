use api_types::auth::KableAccount;
use api_types::skins::{AccountCape, AccountSkin, CurrentSkin, PlayerProfile, SkinModel, SkinUploadConfig, SkinUploadResponse};

use crate::features::accounts::management::get_active_account_for_launch;
use crate::integrations::{minecraft, mojang_api};

/// Returns the active account with a refreshed Minecraft access token when needed.
async fn require_active_account() -> Result<KableAccount, String> {
    get_active_account_for_launch(None).await?.ok_or_else(|| "No active Minecraft account is configured".to_string())
}

/// Retrieves the authenticated player's Minecraft profile, online skins and capes.
pub async fn get_player_profile(account: KableAccount) -> Result<PlayerProfile, String> {
    mojang_api::skins::get_player_profile(&account.access_token).await
}

/// Retrieves the current skin from the Minecraft session server.
pub async fn get_current_skin_info(account: KableAccount) -> Result<CurrentSkin, String> {
    mojang_api::skins::get_current_skin_info(&account.minecraft_profile.id).await
}

/// Returns online skins and locally saved skins together.
pub async fn get_all_account_skins(account: KableAccount) -> Result<Vec<AccountSkin>, String> {
    let profile = mojang_api::skins::get_player_profile(&account.access_token).await?;
    let mut skins = profile.skins;
    let local_skins = minecraft::skins::get_local_skins().await?;

    // Keep online skins first, followed by locally saved skins.
    skins.extend(local_skins);
    Ok(skins)
}

/// Returns only locally saved skins.
pub async fn get_local_skins() -> Result<Vec<AccountSkin>, String> {
    minecraft::skins::get_local_skins().await
}

/// Saves a PNG skin to the launcher's local custom-skin collection.
///
/// This does not upload the skin to Mojang.
pub async fn upload_skin_to_account(config: SkinUploadConfig) -> Result<SkinUploadResponse, String> {
    let skin = minecraft::skins::add_local_skin(config.clone()).await?;

    Ok(SkinUploadResponse { success: true, message: format!("Skin '{}' saved locally", skin.name), model_used: skin.model })
}

/// Updates the name, cape ID and/or model of a locally saved skin.
pub async fn modify_skin_by_id(
    skin_id: String,
    new_name: Option<String>,
    new_cape_id: Option<String>,
    new_slim: Option<bool>,
) -> Result<(), String> {
    minecraft::skins::modify_skin_by_id(&skin_id, new_name, new_cape_id, new_slim).await
}

/// Removes a skin from local custom-skin storage.
pub async fn remove_skin_by_id(skin_id: String) -> Result<(), String> {
    minecraft::skins::remove_skin_by_id(&skin_id).await
}

/// Applies a locally saved skin to the active Minecraft account.
///
/// Online skin-history entries cannot be selected directly through the supported
/// Mojang API. Selecting "current" does not require an API request.
pub async fn apply_account_skin(skin_id: String) -> Result<(), String> {
    if skin_id == "current" {
        return Ok(());
    }

    let account = require_active_account().await?;

    // Resolve a local skin by its stored ID or key. The integration handles both.
    let (skin_data, model) = minecraft::skins::get_local_skin_data_by_id(&skin_id).await?;

    mojang_api::skins::upload_skin_to_mojang(&account.access_token, &skin_data, model).await
}

/// Changes the active skin model by re-uploading the current skin image.
pub async fn change_skin_model(new_model: SkinModel) -> Result<(), String> {
    let account = require_active_account().await?;

    mojang_api::skins::change_skin_model(&account.access_token, &account.minecraft_profile.id, new_model).await
}

/// Returns the currently active cape, if any.
pub async fn get_active_cape() -> Result<Option<AccountCape>, String> {
    let account = require_active_account().await?;

    mojang_api::skins::get_active_cape(&account.access_token).await
}

/// Activates a cape by ID, or hides the currently active cape when given None.
pub async fn apply_cape(cape_id: Option<String>) -> Result<(), String> {
    let account = require_active_account().await?;

    mojang_api::skins::apply_cape(&account.access_token, cape_id.as_deref()).await
}

/// Downloads a skin image from a URL.
pub async fn download_skin_from_url(url: String) -> Result<Vec<u8>, String> {
    mojang_api::skins::download_skin_from_url(&url).await
}
