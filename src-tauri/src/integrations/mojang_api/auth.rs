/**
 * This file contains Microsoft Authentication backend logic using the minecraft-msa-auth crate.
 * @see https://github.com/minecraft-rs/minecraft-msa-auth/blob/826a6846d4e1109a7acfa1a989aa77533aa01fc9/examples/device_flow.rs
 * Here we handle the oauth / device code flow authentication with Microsoft/Xbox for getting Minecraft accounts, as well as token management and refreshing.
*/
use crate::constants::{
    DEFAULT_AZURE_CLIENT_ID, DEFAULT_AZURE_REDIRECT_URI, DEFAULT_OAUTH_PORT, DEVICE_CODE_URL,
    MSA_AUTHORIZE_URL, MSA_TOKEN_URL,
};
use crate::system::net::async_http_client;
use api_types::auth::{DeviceCodeResponse, MicrosoftToken};
use chrono::{Duration, Utc};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, DeviceAuthorizationUrl, Scope,
    StandardDeviceAuthorizationResponse, TokenResponse, TokenUrl,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env::var;
use std::sync::{Arc, Mutex};

///! Global state to store device authorization responses for polling
static DEVICE_AUTH_STORAGE: Lazy<Arc<Mutex<HashMap<String, StandardDeviceAuthorizationResponse>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub fn get_client_id() -> Result<String, String> {
    var("AZURE_CLIENT_ID")
        .or(var("CLIENT_ID"))
        .or(Ok(DEFAULT_AZURE_CLIENT_ID.to_string()))
        .map_err(|_: String| {
            "AZURE_CLIENT_ID / CLIENT_ID not set and no fallback available".to_string()
        })
}

pub fn get_redirect_uri() -> Result<String, String> {
    var("AZURE_REDIRECT_URI")
        .or(var("REDIRECT_URI"))
        .or(Ok(DEFAULT_AZURE_REDIRECT_URI.to_string()))
        .map_err(|_: String| {
            "AZURE_REDIRECT_URI / REDIRECT_URI not set and no fallback available".to_string()
        })
}

pub fn get_oauth_port() -> u16 {
    var("OAUTH_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_OAUTH_PORT)
}

/**
 * Start the device code flow authentication process.
 * This function initiates the device code flow by requesting a device code from the Microsoft OAuth2 endpoint.
 * Returns A `DeviceCodeResponse` containing the device code, user code, verification URI, expiration time, and polling interval.
 */
pub async fn start_authentication() -> Result<DeviceCodeResponse, String> {
    // Load client ID from environment
    let client_id = get_client_id()?;
    let client = BasicClient::new(ClientId::new(client_id))
        .set_auth_uri(AuthUrl::new(MSA_AUTHORIZE_URL.to_string()).unwrap())
        .set_token_uri(TokenUrl::new(MSA_TOKEN_URL.to_string()).unwrap())
        .set_device_authorization_url(
            DeviceAuthorizationUrl::new(DEVICE_CODE_URL.to_string()).unwrap(),
        );
    let details: StandardDeviceAuthorizationResponse = client
        .exchange_device_code()
        .add_scope(Scope::new("XboxLive.signin offline_access".to_string()))
        .request_async(&async_http_client)
        .await
        .map_err(|e| e.to_string())?;

    let response = DeviceCodeResponse {
        device_code: details.device_code().secret().to_string(),
        user_code: details.user_code().secret().to_string(),
        verification_uri: details.verification_uri().to_string(),
        expires_in: details.expires_in().as_secs(),
        interval: details.interval().as_secs(),
    };
    {
        let mut storage = DEVICE_AUTH_STORAGE.lock().unwrap();
        storage.insert(response.device_code.clone(), details);
    }
    Ok(response)
}

pub async fn poll_authentication(device_code: String) -> Result<MicrosoftToken, String> {
    // Retrieve the stored device authorization response
    let device_auth_response = {
        let storage = DEVICE_AUTH_STORAGE.lock().unwrap();
        storage.get(&device_code).cloned()
    };
    if let Some(details) = device_auth_response {
        let client_id = get_client_id()?;
        let token = BasicClient::new(ClientId::new(client_id))
            .set_auth_uri(AuthUrl::new(MSA_AUTHORIZE_URL.to_string()).unwrap())
            .set_token_uri(TokenUrl::new(MSA_TOKEN_URL.to_string()).unwrap())
            .set_device_authorization_url(
                DeviceAuthorizationUrl::new(DEVICE_CODE_URL.to_string()).unwrap(),
            )
            .exchange_device_access_token(&details)
            .request_async(&async_http_client, tokio::time::sleep, None)
            .await
            .map_or(
                Err("Failed to exchange device code for access token".to_string()),
                |token| Ok(token),
            )?;
        // Clean up stored device auth response
        {
            let mut storage = DEVICE_AUTH_STORAGE.lock().unwrap();
            storage.remove(&device_code);
        }
        let expires_in_secs = token.expires_in().map_or(3600, |d| d.as_secs() as i64);
        let expires_at = Utc::now() + Duration::seconds(expires_in_secs);
        return Ok(MicrosoftToken {
            access_token: token.access_token().secret().to_string(),
            expires_at,
            refresh_token: token.refresh_token().map(|rt| rt.secret().to_string()),
        });
    }
    Err("No device authorization found for the provided device code".to_string())
}

/// Refreshes the Microsoft access token using the provided refresh token, still needs to be stored securely if you want to use it!
pub async fn refresh_microsoft_token(token: MicrosoftToken) -> Result<MicrosoftToken, String> {
    let refresh_token = token
        .refresh_token
        .ok_or_else(|| "No refresh token available".to_string())?;

    let client_id = get_client_id()?;
    let client = BasicClient::new(ClientId::new(client_id))
        .set_auth_uri(
            AuthUrl::new(MSA_AUTHORIZE_URL.to_string())
                .map_err(|e| format!("Failed to create auth URL: {}", e))?,
        )
        .set_token_uri(
            TokenUrl::new(MSA_TOKEN_URL.to_string())
                .map_err(|e| format!("Failed to create token URL: {}", e))?,
        );

    let token_result = client
        .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token))
        .request_async(&async_http_client)
        .await
        .map_err(|e| format!("Failed to refresh token: {}", e))?;

    let new_expires_at = Utc::now()
        + chrono::Duration::seconds(
            token_result
                .expires_in()
                .map(|d| d.as_secs() as i64)
                .unwrap_or(3600),
        );

    Ok(MicrosoftToken {
        access_token: token_result.access_token().secret().to_string(),
        expires_at: new_expires_at,
        refresh_token: token_result
            .refresh_token()
            .map(|rt| rt.secret().to_string()),
    })
}
