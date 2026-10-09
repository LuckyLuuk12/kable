use crate::features::accounts::{management, secure_token};
use crate::integrations::mojang_api::auth;
use api_types::auth::{KableAccount, KableMinecraftProfile, MicrosoftToken};
use chrono::Utc;
use minecraft_msa_auth::MinecraftAuthorizationFlow;
use reqwest::Client;

pub async fn authenticate(token: MicrosoftToken) -> Result<KableAccount, String> {
    let account = from_microsoft_token(token).await?;
    management::add_account(account.clone()).await?;
    Ok(account)
}

pub async fn from_microsoft_token(token: MicrosoftToken) -> Result<KableAccount, String> {
    let minecraft_flow = MinecraftAuthorizationFlow::new(Client::new());

    let minecraft_token = minecraft_flow
        .exchange_microsoft_token(&token.access_token)
        .await
        .map_err(|e| format!("Failed to exchange Microsoft token for Minecraft token: {}", e))?;

    // This value is not the Minecraft player's UUID.
    // Do not assume it is the numeric Xbox XUID used by ${auth_xuid}.
    let remote_id = minecraft_token.username().clone();

    let access_token = minecraft_token.access_token().as_ref().to_string();

    // The account stores a Minecraft access token, so its expiry must be
    // derived from the Minecraft token lifetime, not the Microsoft token.
    let minecraft_expires_at = Utc::now() + chrono::Duration::seconds(minecraft_token.expires_in() as i64);

    let profile = auth::fetch_minecraft_profile(&access_token).await?;

    let encrypted_refresh_token = match token.refresh_token {
        Some(refresh_token) => {
            Some(secure_token::encrypt_token(&refresh_token).await.map_err(|e| format!("Failed to encrypt refresh token: {}", e))?)
        }
        None => None,
    };

    Ok(KableAccount {
        access_token,
        access_token_expires_at: minecraft_expires_at.to_rfc3339(),
        encrypted_refresh_token,
        avatar: format!("https://crafatar.com/avatars/{}?size=64", profile.id),
        eligible_for_free_trials: true,
        eligible_for_migration: false,
        franchise_inventory_id: "1/Mg==".to_string(),
        has_multiple_profiles: false,
        in_forced_migration: false,
        legacy: false,
        license_product_ids: Vec::new(),
        local_id: profile.id.clone(),
        minecraft_profile: KableMinecraftProfile {
            id: profile.id.clone(),
            name: profile.name.clone(),
            requires_profile_name_change: false,
            requires_skin_change: false,
        },
        persistent: true,
        remote_id,
        account_type: "msa".to_string(),
        user_properties: Vec::new(),
        username: profile.name,
    })
}
