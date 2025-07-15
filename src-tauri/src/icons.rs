use std::path::PathBuf;
use std::fs;
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomIconTemplate {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    #[serde(rename = "iconType")]
    pub icon_type: String,
    #[serde(rename = "fallbackIcon")]
    pub fallback_icon: String,
    pub icons: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IconSettings {
    #[serde(rename = "selectedTemplate")]
    pub selected_template: String,
    #[serde(rename = "customTemplates")]
    pub custom_templates: Vec<CustomIconTemplate>,
    #[serde(rename = "builtinTemplates")]
    pub builtin_templates: Vec<String>,
}

/// Get the icons configuration directory
async fn get_icons_dir() -> Result<PathBuf, String> {
    let launcher_dir = crate::get_launcher_dir().await?;
    Ok(PathBuf::from(launcher_dir).join("config").join("icons"))
}

/// Ensure the icons directory exists
async fn ensure_icons_dir() -> Result<PathBuf, String> {
    let icons_dir = get_icons_dir().await?;
    if !icons_dir.exists() {
        fs::create_dir_all(&icons_dir)
            .map_err(|e| format!("Failed to create icons directory: {}", e))?;
    }
    Ok(icons_dir)
}

/// Get all custom icon templates
#[command]
pub async fn get_custom_icon_templates() -> Result<Vec<CustomIconTemplate>, String> {
    let icons_dir = get_icons_dir().await?;
    if !icons_dir.exists() {
        return Ok(Vec::new());
    }

    let mut templates = Vec::new();

    // Read all .json and .yml/.yaml files in the icons directory
    let entries = fs::read_dir(&icons_dir)
        .map_err(|e| format!("Failed to read icons directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if !path.is_file() {
            continue;
        }

        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "json" => {
                if let Ok(template) = load_json_template(&path) {
                    templates.push(template);
                }
            },
            "yml" | "yaml" => {
                if let Ok(template) = load_yaml_template(&path) {
                    templates.push(template);
                }
            },
            _ => {}
        }
    }

    Ok(templates)
}

/// Load a JSON icon template
fn load_json_template(path: &PathBuf) -> Result<CustomIconTemplate, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read template file: {}", e))?;
    
    let template: CustomIconTemplate = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON template: {}", e))?;
    
    Ok(template)
}

/// Load a YAML icon template
fn load_yaml_template(path: &PathBuf) -> Result<CustomIconTemplate, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read template file: {}", e))?;
    
    let template: CustomIconTemplate = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse YAML template: {}", e))?;
    
    Ok(template)
}

/// Save a custom icon template
#[command]
pub async fn save_custom_icon_template(template: CustomIconTemplate) -> Result<String, String> {
    let icons_dir = ensure_icons_dir().await?;
    let file_path = icons_dir.join(format!("{}.json", template.name));
    
    let json_content = serde_json::to_string_pretty(&template)
        .map_err(|e| format!("Failed to serialize template: {}", e))?;
    
    fs::write(&file_path, json_content)
        .map_err(|e| format!("Failed to write template file: {}", e))?;
    
    Ok(file_path.to_string_lossy().to_string())
}

/// Delete a custom icon template
#[command]
pub async fn delete_custom_icon_template(template_name: String) -> Result<(), String> {
    let icons_dir = get_icons_dir().await?;
    
    // Try both .json and .yml/.yaml extensions
    let extensions = ["json", "yml", "yaml"];
    let mut found = false;
    
    for ext in &extensions {
        let file_path = icons_dir.join(format!("{}.{}", template_name, ext));
        if file_path.exists() {
            fs::remove_file(&file_path)
                .map_err(|e| format!("Failed to delete template file: {}", e))?;
            found = true;
            break;
        }
    }
    
    if !found {
        return Err(format!("Template '{}' not found", template_name));
    }
    
    Ok(())
}

/// Validate an icon template
#[command]
pub async fn validate_icon_template(template_content: String, format: String) -> Result<CustomIconTemplate, String> {
    let template: CustomIconTemplate = match format.as_str() {
        "json" => {
            serde_json::from_str(&template_content)
                .map_err(|e| format!("Invalid JSON format: {}", e))?
        },
        "yaml" | "yml" => {
            serde_yaml::from_str(&template_content)
                .map_err(|e| format!("Invalid YAML format: {}", e))?
        },
        _ => return Err("Unsupported format. Use 'json', 'yaml', or 'yml'".to_string())
    };
    
    // Basic validation
    if template.name.is_empty() {
        return Err("Template name is required".to_string());
    }
    
    if template.icon_type.is_empty() {
        return Err("Icon type is required".to_string());
    }
    
    if template.fallback_icon.is_empty() {
        return Err("Fallback icon is required".to_string());
    }
    
    if template.icons.is_empty() {
        return Err("At least one icon mapping is required".to_string());
    }
    
    Ok(template)
}

/// Get the icons directory path for frontend file operations
#[command]
pub async fn get_icons_directory_path() -> Result<String, String> {
    let icons_dir = ensure_icons_dir().await?;
    Ok(icons_dir.to_string_lossy().to_string())
}

/// Open the icons directory in the system file explorer
#[command]
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
