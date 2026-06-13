use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;

/// Download avatar from Crafatar and convert to base64 data URL
pub async fn download_avatar_as_base64(uuid: &str) -> Result<String, String> {
    // Note: Minecraft UUIDs are public identifiers, not sensitive data.
    // This is the official Mojang Session Server API endpoint - the UUID
    // must be in the URL path as per Mojang's API specification.
    // See: https://minecraft.wiki/w/Mojang_API#Query_player_profile
    let url = format!("https://crafatar.com/avatars/{}?size=64", uuid);

    let client = Client::new();
    // codeql[rust/cleartext-transmission] - Minecraft UUIDs are public data
    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| format!("Failed to download avatar: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Avatar download failed with status: {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read avatar bytes: {}", e))?;

    let base64_data = general_purpose::STANDARD.encode(&bytes);
    let data_url = format!("data:image/png;base64,{}", base64_data);

    Ok(data_url)
}
