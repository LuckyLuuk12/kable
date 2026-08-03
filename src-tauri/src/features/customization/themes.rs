use api_types::settings::Theme;

use crate::constants::{CONFIG_DIR, THEMES_DIR};
use crate::system::fs;
use std::path::PathBuf;

fn css_themes_dir() -> Result<PathBuf, String> {
    Ok(fs::launcher_dir()?.join(CONFIG_DIR).join(THEMES_DIR))
}

async fn ensure_css_themes_dir() -> Result<PathBuf, String> {
    let dir = css_themes_dir()?;
    fs::create_dir(&dir).await?;
    Ok(dir)
}

// Lists all Theme enum variants except of course the Custom(String) variant, which is not a builtin theme
fn builtin_themes() -> Result<Vec<Theme>, String> {
    Ok(vec![Theme::Light, Theme::Dark, Theme::System])
}

pub async fn list_css_themes() -> Result<Vec<Theme>, String> {
    let mut themes = builtin_themes()?;

    let dir = ensure_css_themes_dir().await?;

    let entries = fs::read_dir(&dir).await?;
    // Iterate over the entries and look for .css files, then add them as Theme::Custom(name) to the list
    for path in entries {
        if fs::is_file(&path).await? {
            let ext = path.extension().and_then(|e| e.to_str());

            if ext == Some("css") {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    let theme = Theme::Custom(name.to_string());

                    if !themes.contains(&theme) {
                        themes.push(theme);
                    }
                }
            }
        }
    }

    Ok(themes)
}

// Loads the CSS content of a theme given its name, which can be either a builtin theme (e.g. "builtin:dark") or a custom theme (e.g. "custom:my_theme")
// pub async fn load_css_theme(name: String) -> Result<String, String> {
//     if let Some(id) = name.strip_prefix("builtin:") {
//         return Ok(format!("/* builtin theme: {} */", id));
//     }

//     if let Some(id) = name.strip_prefix("custom:") {
//         let dir = css_themes_dir()?;
//         let path = dir.join(format!("{}.css", id));

//         return fs::read_str(&path).await;
//     }

//     Err(format!("invalid theme name: {}", name))
// }
pub async fn load_css(theme: Theme) -> Result<String, String> {
    match theme {
        Theme::Light => Ok("/* builtin theme: light */".to_string()),
        Theme::Dark => Ok("/* builtin theme: dark */".to_string()),
        Theme::System => Ok("/* builtin theme: system */".to_string()),
        Theme::Custom(name) => {
            let dir = css_themes_dir()?;
            let path = dir.join(format!("{}.css", name));

            fs::read_str(&path).await
        }
    }
}

pub async fn save_css_theme(name: String, css: String) -> Result<String, String> {
    let dir = ensure_css_themes_dir().await?;
    let path = dir.join(format!("{}.css", name));

    fs::write(&path, css.as_bytes(), false).await?;

    Ok(format!("custom:{}", name))
}

pub async fn delete_css_theme(name: String) -> Result<(), String> {
    let Some(id) = name.strip_prefix("custom:") else {
        return Err("only custom themes can be deleted".to_string());
    };

    let dir = css_themes_dir()?;
    let path = dir.join(format!("{}.css", id));

    if fs::exists(&path).await? {
        fs::remove_file(&path).await?;
    }

    Ok(())
}

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
