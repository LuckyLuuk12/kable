use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet)]
pub struct MicrosoftToken {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
    /// Raw refresh token from device code flow. Should be encrypted if stored persistently!!
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet)]
pub struct LauncherAccount {
    pub access_token: String,
    pub access_token_expires_at: String,
    /// AES-encrypted refresh token for "persistent" accounts.
    pub encrypted_refresh_token: Option<String>,
    pub avatar: String,
    pub eligible_for_free_trials: bool,
    pub eligible_for_migration: bool,
    pub franchise_inventory_id: String,
    pub has_multiple_profiles: bool,
    pub in_forced_migration: bool,
    pub legacy: bool,
    pub license_product_ids: Vec<String>,
    pub local_id: String,
    pub minecraft_profile: MinecraftProfile,
    pub persistent: bool,
    pub remote_id: String,
    #[serde(rename = "type")]
    pub account_type: String,
    pub user_properties: Vec<()>,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet)]
#[serde(rename_all = "snake_case")]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
    pub requires_profile_name_change: bool,
    pub requires_skin_change: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet)]
#[serde(rename_all = "snake_case")]
pub struct LauncherAccountsJson {
    pub accounts: HashMap<String, LauncherAccount>,
    pub active_account_local_id: String,
    pub mojang_client_token: String,
}
