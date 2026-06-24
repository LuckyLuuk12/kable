use crate::constants::{CONFIG_DIR, ICONS_DIR};
use crate::system::fs::{create_dir, launcher_dir, read_str};
use api_types::icons::CustomIconTemplate;
use std::fs;
use std::path::PathBuf;

/// Get the icons configuration directory
pub fn get_icons_dir() -> Result<PathBuf, String> {
    let launcher_dir = launcher_dir()?;
    Ok(launcher_dir.join(CONFIG_DIR).join(ICONS_DIR))
}

/// Ensure the icons directory exists
pub async fn ensure_icons_dir() -> Result<PathBuf, String> {
    let icons_dir = get_icons_dir()?;
    match create_dir(&icons_dir).await {
        Ok(p) => Ok(p),
        Err(err) => Err(format!("Failed to ensure icons directory exists: {}", err)),
    }
}

/// Get all custom icon templates
pub async fn get_custom_icon_templates() -> Result<Vec<CustomIconTemplate>, String> {
    let icons_dir = ensure_icons_dir().await?;
    let mut templates = Vec::new();

    if icons_dir.exists() {
        let entries = fs::read_dir(&icons_dir).map_err(|e| format!("Failed to read icons directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = read_str(&path).await {
                    if let Ok(template) = serde_json::from_str::<CustomIconTemplate>(&content) {
                        templates.push(template);
                    }
                }
            }
        }
    }

    Ok(templates)
}

/// Save a custom icon template
pub async fn save_custom_icon_template(template: CustomIconTemplate) -> Result<String, String> {
    let icons_dir = ensure_icons_dir().await?;
    let template_name = template.name.clone();
    let template_path = icons_dir.join(format!("{}.json", template_name));

    let content = serde_json::to_string_pretty(&template).map_err(|e| format!("Failed to serialize template: {}", e))?;

    fs::write(&template_path, content).map_err(|e| format!("Failed to write template file: {}", e))?;

    Ok(template_name)
}

/// Delete a custom icon template
pub async fn delete_custom_icon_template(template_name: String) -> Result<(), String> {
    let icons_dir = get_icons_dir()?;
    let template_path = icons_dir.join(format!("{}.json", template_name));

    if template_path.exists() {
        fs::remove_file(template_path).map_err(|e| format!("Failed to delete template file: {}", e))?;
    }

    Ok(())
}

/// Open the icons directory in the system file explorer
pub async fn open_icons_directory() -> Result<(), String> {
    let icons_dir = ensure_icons_dir().await?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&icons_dir)
            .spawn()
            .map_err(|e| format!("Failed to open icons directory: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&icons_dir)
            .spawn()
            .map_err(|e| format!("Failed to open icons directory: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&icons_dir)
            .spawn()
            .map_err(|e| format!("Failed to open icons directory: {}", e))?;
    }

    Ok(())
}
