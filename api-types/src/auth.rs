use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::Timestamp;

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u32,
    pub interval: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
pub struct MicrosoftToken {
    pub access_token: String,
    pub expires_at: Timestamp,
    /// Raw refresh token from device code flow. Should be encrypted if stored persistently!!
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
pub struct KableAccount {
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
    /// This is usually the same as the user's UUID but for correctness use the Minecraft Profile's ID!
    pub local_id: String,
    pub minecraft_profile: KableMinecraftProfile,
    pub persistent: bool,
    pub remote_id: String,
    #[serde(rename = "type")]
    pub account_type: String,
    pub user_properties: Vec<()>,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
#[serde(rename_all = "snake_case")]
pub struct KableMinecraftProfile {
    pub id: String,
    pub name: String,
    pub requires_profile_name_change: bool,
    pub requires_skin_change: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
#[serde(rename_all = "snake_case")]
pub struct KableAccountsJson {
    pub accounts: HashMap<String, KableAccount>,
    pub active_account_local_id: String,
    pub mojang_client_token: String,
}

// ? Official launcher types:

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, facet::Facet, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LauncherAccountsJson {
    pub accounts: HashMap<String, LauncherAccount>,
    pub mojang_client_token: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, facet::Facet, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LauncherAccount {
    pub access_token: String,
    pub access_token_expires_at: String,
    pub avatar: String,
    pub eligible_for_free_trials: bool,
    pub eligible_for_migration: bool,
    pub franchise_inventory_id: String,
    pub has_multiple_profiles: bool,
    pub in_forced_migration: bool,
    pub legacy: bool,
    pub license_product_ids: Vec<Option<String>>,
    pub local_id: String,
    pub minecraft_profile: MinecraftProfile,
    pub persistent: bool,
    pub remote_id: String,
    #[serde(rename = "type")]
    pub account_type: String,
    pub user_properites: Vec<Option<String>>,
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, facet::Facet, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
    pub requires_profile_name_change: bool,
    pub requires_skin_change: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, facet::Facet, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftProfileResponse {
    pub id: String,
    pub name: String,
}

// ? Impl launcher account json into kable account json
impl From<LauncherAccount> for KableAccount {
    fn from(launcher_account: LauncherAccount) -> Self {
        KableAccount {
            access_token: launcher_account.access_token,
            access_token_expires_at: launcher_account.access_token_expires_at,
            encrypted_refresh_token: None, // LauncherAccount does not have a refresh token
            avatar: launcher_account.avatar,
            eligible_for_free_trials: launcher_account.eligible_for_free_trials,
            eligible_for_migration: launcher_account.eligible_for_migration,
            franchise_inventory_id: launcher_account.franchise_inventory_id,
            has_multiple_profiles: launcher_account.has_multiple_profiles,
            in_forced_migration: launcher_account.in_forced_migration,
            legacy: launcher_account.legacy,
            license_product_ids: launcher_account.license_product_ids.into_iter().map(|opt| opt.unwrap_or_default()).collect(),
            local_id: launcher_account.local_id,
            minecraft_profile: KableMinecraftProfile {
                id: launcher_account.minecraft_profile.id,
                name: launcher_account.minecraft_profile.name,
                requires_profile_name_change: launcher_account.minecraft_profile.requires_profile_name_change,
                requires_skin_change: launcher_account.minecraft_profile.requires_skin_change,
            },
            persistent: launcher_account.persistent,
            remote_id: launcher_account.remote_id,
            account_type: launcher_account.account_type,
            user_properties: vec![], // LauncherAccount does not have user properties
            username: launcher_account.username,
        }
    }
}

impl From<LauncherAccountsJson> for KableAccountsJson {
    fn from(launcher_accounts_json: LauncherAccountsJson) -> Self {
        // Find the active account by looking at what token expires last, if there are multiple accounts with the same expiration, just pick the first one. If there are no accounts, set active_account_local_id to empty string.
        let active_account_local_id = launcher_accounts_json
            .accounts
            .values()
            .max_by_key(|account| DateTime::parse_from_rfc3339(&account.access_token_expires_at).unwrap_or_else(|_| Utc::now().into()))
            .map(|account| account.local_id.clone())
            .unwrap_or_else(String::new);

        KableAccountsJson {
            accounts: launcher_accounts_json.accounts.into_iter().map(|(k, v)| (k, v.into())).collect(),
            active_account_local_id,
            mojang_client_token: launcher_accounts_json.mojang_client_token,
        }
    }
}
