// Discord Rich Presence commands

/// Set Discord Rich Presence for browsing a section
#[tauri::command]
pub async fn discord_set_browsing(section: String) -> Result<(), String> {
    crate::discord::set_browsing(&section)
}

/// Set Discord Rich Presence enabled/disabled
#[tauri::command]
pub async fn discord_set_enabled(enabled: bool) -> Result<(), String> {
    crate::discord::set_enabled(enabled)
}

/// Clear Discord Rich Presence
#[tauri::command]
pub async fn discord_clear() -> Result<(), String> {
    crate::discord::clear()
}
