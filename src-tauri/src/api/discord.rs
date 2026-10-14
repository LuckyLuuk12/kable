use api_types::profiles::KableProfile;

use crate::features::discord;

/// Initialize Discord Rich Presence
#[tauri::command]
#[specta::specta]
pub fn initialize_discord_rpc() -> Result<(), String> {
    discord::initialize()
}

/// Enable or disable Discord Rich Presence globally
#[tauri::command]
#[specta::specta]
pub fn set_discord_enabled(enabled: bool) -> Result<(), String> {
    discord::set_enabled(enabled)
}

/// Set presence to "playing Minecraft"
#[tauri::command]
#[specta::specta]
pub fn set_discord_playing(profile: KableProfile) -> Result<(), String> {
    discord::set_playing(&profile)
}

/// Set presence to browsing a launcher section
#[tauri::command]
#[specta::specta]
pub fn set_discord_browsing(section: String) -> Result<(), String> {
    discord::set_browsing(&section)
}

/// Clear "playing" state (revert to idle / launcher state)
#[tauri::command]
#[specta::specta]
pub fn clear_discord_playing() -> Result<(), String> {
    discord::clear_playing()
}

/// Clear all Discord presence (hard reset)
#[tauri::command]
#[specta::specta]
pub fn clear_discord_presence() -> Result<(), String> {
    discord::clear()
}

/// Disconnect completely from Discord IPC
#[tauri::command]
#[specta::specta]
pub fn disconnect_discord() -> Result<(), String> {
    discord::disconnect()
}
