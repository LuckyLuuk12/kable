use crate::constants::{CONFIG_DIR, THEMES_DIR};
use crate::system::fs;
use std::path::PathBuf;
use tauri::AppHandle;

type FsResult<T> = Result<T, String>;

pub fn css_themes_dir() -> FsResult<PathBuf> {
    Ok(fs::launcher_dir()?.join(CONFIG_DIR).join(THEMES_DIR))
}

async fn ensure_css_themes_dir() -> FsResult<PathBuf> {
    let dir = css_themes_dir()?;
    fs::create_dir(&dir).await?;
    Ok(dir)
}

pub fn builtin_themes(_app: &AppHandle) -> FsResult<Vec<String>> {
    Ok(vec![
        "builtin:default".into(),
        "builtin:dark".into(),
        "builtin:light".into(),
        "builtin:ocean".into(),
        "builtin:forest".into(),
        "builtin:crimson".into(),
        "builtin:amber".into(),
    ])
}

pub async fn list_css_themes(app: AppHandle) -> FsResult<Vec<String>> {
    let mut themes = builtin_themes(&app)?;

    let dir = ensure_css_themes_dir().await?;

    let entries = fs::read_dir(&dir).await?;

    for path in entries {
        if fs::is_file(&path).await? {
            let ext = path.extension().and_then(|e| e.to_str());

            if ext == Some("css") {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    let theme = format!("custom:{}", name);

                    if !themes.contains(&theme) {
                        themes.push(theme);
                    }
                }
            }
        }
    }

    Ok(themes)
}

pub async fn load_css_theme(name: String, _app: AppHandle) -> FsResult<String> {
    if let Some(id) = name.strip_prefix("builtin:") {
        return Ok(format!("/* builtin theme: {} */", id));
    }

    if let Some(id) = name.strip_prefix("custom:") {
        let dir = css_themes_dir()?;
        let path = dir.join(format!("{}.css", id));

        return fs::read_str(&path).await;
    }

    Err(format!("invalid theme name: {}", name))
}

pub async fn save_css_theme(name: String, css: String) -> FsResult<String> {
    let dir = ensure_css_themes_dir().await?;
    let path = dir.join(format!("{}.css", name));

    fs::write(&path, css.as_bytes(), false).await?;

    Ok(format!("custom:{}", name))
}

pub async fn delete_css_theme(name: String) -> FsResult<()> {
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

pub async fn open_css_themes_directory() -> FsResult<()> {
    let dir = ensure_css_themes_dir().await?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer").arg(&dir).spawn().map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&dir).spawn().map_err(|e| e.to_string())?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&dir).spawn().map_err(|e| e.to_string())?;

    Ok(())
}
