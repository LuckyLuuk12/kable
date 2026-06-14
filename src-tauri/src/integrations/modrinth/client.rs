// implementation of the Modrinth API: https://docs.modrinth.com/api/
// Previously I implemented this myself but I found this crate: https://crates.io/crates/modrinth-api
// Now this file just contains some configuration / initialization to easily use this crate
use modrinth_api::apis::Configuration;

/**
 * Configuration {
 *     base_path: "https://api.modrinth.com/v2".to_owned(),
 *     user_agent: Some("OpenAPI-Generator/v2.7.0/15cf3fc/rust".to_owned()),
 *     client: reqwest::Client::new(),
 *     basic_auth: None,
 *     oauth_access_token: None,
 *     bearer_access_token: None,
 *     api_key: None,
 * }
 */
pub const MODRINTH_CONFIGURATION: Configuration = Configuration {
    user_agent: Some("minecraft-launcher/Kable".to_owned()),
    ..Configuration::default()
};
