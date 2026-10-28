use std::path::PathBuf;

use api_types::settings::{CssTheme, CssThemeType};
use tauri::Manager;

use crate::{
    app_handle,
    constants::{CONFIG_DIR, THEMES_DIR},
    system::fs,
    Logger,
};

const BUILTIN_THEMES: &[(&str, &str, &str)] = &[
    ("builtin:kasai-sora", "Kasai Sora", "themes/KasaiSora-Theme.css"),
    ("builtin:modrinth", "Modrinth", "themes/Modrinth-Theme.css"),
    ("builtin:official", "Official", "themes/Official-Theme.css"),
    ("builtin:est", "EST", "themes/est.css"),
    ("builtin:brtd", "BRTD", "themes/brtd.css"),
    ("builtin:hlwn", "HLWN", "themes/hlwn.css"),
    ("builtin:crstms", "CRSTMS", "themes/crstms.css"),
    ("builtin:nwyr", "NWYR", "themes/nwyr.css"),
];

/// Get the directory containing user-created CSS themes.
fn css_themes_dir() -> Result<PathBuf, String> {
    Ok(fs::launcher_dir()?.join(CONFIG_DIR).join(THEMES_DIR))
}

/// Ensure the custom themes directory exists.
async fn ensure_css_themes_dir() -> Result<PathBuf, String> {
    let dir = css_themes_dir()?;
    fs::create_dir(&dir).await?;
    Ok(dir)
}

/// Resolve a bundled built-in theme to its packaged resource path.
fn builtin_theme_path(resource: &str) -> Result<PathBuf, String> {
    app_handle()?
        .path()
        .resolve(resource, tauri::path::BaseDirectory::Resource)
        .map_err(|e| format!("Failed to resolve built-in theme resource '{resource}': {e}"))
}

/// Find a built-in theme by its stable ID.
fn find_builtin_theme(id: &str) -> Option<&'static (&'static str, &'static str, &'static str)> {
    BUILTIN_THEMES.iter().find(|(builtin_id, _, _)| *builtin_id == id)
}

/// Validate a custom theme name to prevent path traversal and invalid filenames.
fn validate_custom_name(name: &str) -> Result<&str, String> {
    let name = name.trim();

    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || name.contains(':')
        || name.contains('\0')
    {
        return Err("Invalid custom theme name".to_string());
    }

    if name.ends_with(".css") {
        return Err("Provide the theme name without the .css extension".to_string());
    }

    Ok(name)
}

/// Parse a custom theme ID and validate its name.
fn custom_name_from_id(id: &str) -> Result<&str, String> {
    let name = id.strip_prefix("custom:").ok_or_else(|| "Only custom themes can be modified".to_string())?;

    validate_custom_name(name)
}

/// List built-in and custom CSS themes.
pub async fn list_css_themes() -> Result<Vec<CssTheme>, String> {
    let mut themes = BUILTIN_THEMES
        .iter()
        .map(|(id, name, _)| CssTheme { id: (*id).to_string(), name: (*name).to_string(), theme_type: CssThemeType::Builtin })
        .collect::<Vec<_>>();

    let dir = ensure_css_themes_dir().await?;
    let entries = fs::read_dir(&dir).await?;

    for path in entries {
        if !fs::is_file(&path).await? {
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("css") {
            continue;
        }

        let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };

        if validate_custom_name(name).is_err() {
            Logger::error_global(format!("Ignoring CSS theme with invalid filename: {:?}", path).as_str(), None);
            continue;
        }

        themes.push(CssTheme { id: format!("custom:{name}"), name: name.to_string(), theme_type: CssThemeType::Custom });
    }

    themes.sort_by(|a, b| {
        let a_builtin = a.theme_type == CssThemeType::Builtin;
        let b_builtin = b.theme_type == CssThemeType::Builtin;

        b_builtin.cmp(&a_builtin).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(themes)
}

/// Load CSS for a theme by its stable ID.
///
/// Built-in themes load from packaged application resources.
/// Custom themes load from the user's Kable configuration directory.
pub async fn load_css(theme_id: String) -> Result<String, String> {
    if let Some((_, _, resource)) = find_builtin_theme(&theme_id) {
        let path = builtin_theme_path(resource)?;

        return fs::read_str(&path)
            .await
            .map_err(|e| format!("Failed to load built-in CSS theme '{}' from {:?}: {}", theme_id, path, e));
    }

    if let Some(name) = theme_id.strip_prefix("custom:") {
        let name = validate_custom_name(name)?;
        let path = css_themes_dir()?.join(format!("{name}.css"));

        if !path.is_file() {
            return Err(format!("CSS theme '{name}' does not exist"));
        }

        return fs::read_str(&path).await.map_err(|e| format!("Failed to load CSS theme '{name}': {e}"));
    }

    Err(format!("Unknown CSS theme ID: '{theme_id}'"))
}

/// Save a custom CSS theme and return its stable ID.
pub async fn save_css_theme(name: String, css: String) -> Result<String, String> {
    let name = validate_custom_name(&name)?.to_string();
    let dir = ensure_css_themes_dir().await?;
    let path = dir.join(format!("{name}.css"));

    fs::write(&path, css.as_bytes(), false).await?;

    Ok(format!("custom:{name}"))
}

/// Delete a custom CSS theme by its stable ID.
///
/// Built-in themes cannot be deleted.
pub async fn delete_css_theme(theme_id: String) -> Result<(), String> {
    let name = custom_name_from_id(&theme_id)?;
    let path = css_themes_dir()?.join(format!("{name}.css"));

    if fs::exists(&path).await? {
        fs::remove_file(&path).await?;
    }

    Ok(())
}

/// Open the custom CSS themes directory in the system file explorer.
pub async fn open_css_themes_directory() -> Result<(), String> {
    let dir = ensure_css_themes_dir().await?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer").arg(&dir).spawn().map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&dir).spawn().map_err(|e| e.to_string())?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&dir).spawn().map_err(|e| e.to_string())?;

    Ok(())
}
