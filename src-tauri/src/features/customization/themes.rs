use crate::constants::{CONFIG_DIR, THEMES_DIR};
use crate::system::fs::{ensure_folder, get_kable_launcher_dir};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

/// Get the CSS themes directory
pub fn get_css_themes_dir() -> Result<PathBuf, String> {
    Ok(get_kable_launcher_dir()?.join(CONFIG_DIR).join(THEMES_DIR))
}

/// Ensure the CSS themes directory exists
pub async fn ensure_css_themes_dir() -> Result<PathBuf, String> {
    let themes_dir = get_css_themes_dir()?;
    match ensure_folder(&themes_dir).await {
        Ok(p) => Ok(p),
        Err(err) => Err(format!("Failed to ensure themes directory exists: {}", err)),
    }
}

/// List all available CSS themes (including built-in)
pub async fn get_css_themes(app: AppHandle) -> Result<Vec<String>, String> {
    let mut themes = get_builtin_css_themes(&app)?;

    let themes_dir = ensure_css_themes_dir().await?;
    if let Ok(entries) = fs::read_dir(themes_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("css") {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    let theme_name = format!("custom:{}", name);
                    if !themes.contains(&theme_name) {
                        themes.push(theme_name);
                    }
                }
            }
        }
    }

    Ok(themes)
}

/// Get a list of built-in CSS themes from assets
pub fn get_builtin_css_themes(_app: &AppHandle) -> Result<Vec<String>, String> {
    // In a real implementation, this would list files from tauri::AssetResolver or similar
    // For now, return a hardcoded list matching the old implementation's logic
    Ok(vec![
        "builtin:default".to_string(),
        "builtin:dark".to_string(),
        "builtin:light".to_string(),
        "builtin:ocean".to_string(),
        "builtin:forest".to_string(),
        "builtin:crimson".to_string(),
        "builtin:amber".to_string(),
    ])
}

/// Load a CSS theme's content
pub async fn load_css_theme(theme_name: String, app: AppHandle) -> Result<String, String> {
    if theme_name.starts_with("builtin:") {
        let name = theme_name.strip_prefix("builtin:").unwrap();
        // This would normally load from assets
        return Ok(format!("/* Builtin theme: {} */", name));
    }

    if theme_name.starts_with("custom:") {
        let name = theme_name.strip_prefix("custom:").unwrap();
        let themes_dir = get_css_themes_dir()?;
        let theme_path = themes_dir.join(format!("{}.css", name));

        let content = fs::read_to_string(theme_path)
            .map_err(|e| format!("Failed to read theme file: {}", e))?;
        return Ok(content);
    }

    Err(format!("Invalid theme name: {}", theme_name))
}

/// Save a custom CSS theme
pub async fn save_css_theme(theme_name: String, css_content: String) -> Result<String, String> {
    let themes_dir = ensure_css_themes_dir().await?;
    let theme_path = themes_dir.join(format!("{}.css", theme_name));

    fs::write(&theme_path, css_content).map_err(|e| format!("Failed to save theme: {}", e))?;

    Ok(format!("custom:{}", theme_name))
}

/// Delete a custom CSS theme
pub async fn delete_css_theme(theme_name: String) -> Result<(), String> {
    if !theme_name.starts_with("custom:") {
        return Err("Only custom themes can be deleted".to_string());
    }

    let name = theme_name.strip_prefix("custom:").unwrap();
    let themes_dir = get_css_themes_dir()?;
    let theme_path = themes_dir.join(format!("{}.css", name));

    if theme_path.exists() {
        fs::remove_file(theme_path).map_err(|e| format!("Failed to delete theme: {}", e))?;
    }

    Ok(())
}

/// Open the themes directory in the system file explorer
pub async fn open_css_themes_directory() -> Result<(), String> {
    let themes_dir = ensure_css_themes_dir().await?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&themes_dir)
            .spawn()
            .map_err(|e| format!("Failed to open themes directory: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&themes_dir)
            .spawn()
            .map_err(|e| format!("Failed to open themes directory: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&themes_dir)
            .spawn()
            .map_err(|e| format!("Failed to open themes directory: {}", e))?;
    }

    Ok(())
}
