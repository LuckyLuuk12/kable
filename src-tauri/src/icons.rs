use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs as async_fs;

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
    let launcher_dir = crate::get_kable_launcher_dir()?;
    Ok(launcher_dir.join("config").join("icons"))
}

/// Ensure the icons directory exists
async fn ensure_icons_dir() -> Result<PathBuf, String> {
    let icons_dir = get_icons_dir().await?;
    // Use centralized helper to ensure the directory exists and return the path
    match crate::ensure_folder(&icons_dir).await {
        Ok(p) => Ok(p),
        Err(err) => Err(format!("Failed to ensure icons directory exists: {}", err)),
    }
}

/// Get all custom icon templates
pub async fn get_custom_icon_templates() -> Result<Vec<CustomIconTemplate>, String> {
    // Ensure icons directory exists (creates it if missing)
    let icons_dir = ensure_icons_dir().await?;

    let mut templates = Vec::new();

    // Read all .json and .yml/.yaml files in the icons directory
    let mut dir = async_fs::read_dir(&icons_dir)
        .await
        .map_err(|e| format!("Failed to read icons directory: {}", e))?;

    while let Ok(Some(entry)) = dir.next_entry().await {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        match extension {
            "json" => {
                if let Ok(template) = load_json_template(&path).await {
                    templates.push(template);
                }
            }
            "yml" | "yaml" => {
                if let Ok(template) = load_yaml_template(&path).await {
                    templates.push(template);
                }
            }
            _ => {}
        }
    }

    Ok(templates)
}

/// Load a JSON icon template (async)
async fn load_json_template(path: &PathBuf) -> Result<CustomIconTemplate, String> {
    let content = async_fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read template file: {}", e))?;

    let template: CustomIconTemplate = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON template: {}", e))?;

    Ok(template)
}

/// Load a YAML icon template (async)
async fn load_yaml_template(path: &PathBuf) -> Result<CustomIconTemplate, String> {
    let content = async_fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read template file: {}", e))?;

    let template: CustomIconTemplate = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse YAML template: {}", e))?;

    Ok(template)
}

/// Save a custom icon template
pub async fn save_custom_icon_template(template: CustomIconTemplate) -> Result<String, String> {
    let icons_dir = ensure_icons_dir().await?;
    let file_path = icons_dir.join(format!("{}.json", template.name));

    let json_content = serde_json::to_string_pretty(&template)
        .map_err(|e| format!("Failed to serialize template: {}", e))?;

    // Ensure parent and write atomically
    crate::ensure_parent_dir_exists_async(&file_path)
        .await
        .map_err(|e| format!("Failed to ensure icons directory: {}", e))?;
    crate::write_file_atomic_async(&file_path, json_content.as_bytes())
        .await
        .map_err(|e| format!("Failed to write template file: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

/// Delete a custom icon template
pub async fn delete_custom_icon_template(template_name: String) -> Result<(), String> {
    let icons_dir = ensure_icons_dir().await?;

    // Try both .json and .yml/.yaml extensions
    let extensions = ["json", "yml", "yaml"];
    let mut found = false;

    for ext in &extensions {
        let file_path = icons_dir.join(format!("{}.{}", template_name, ext));
        if file_path.exists() {
            tokio::fs::remove_file(&file_path)
                .await
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
pub async fn validate_icon_template(
    template_content: String,
    format: String,
) -> Result<CustomIconTemplate, String> {
    let template: CustomIconTemplate = match format.as_str() {
        "json" => serde_json::from_str(&template_content)
            .map_err(|e| format!("Invalid JSON format: {}", e))?,
        "yaml" | "yml" => serde_yaml::from_str(&template_content)
            .map_err(|e| format!("Invalid YAML format: {}", e))?,
        _ => return Err("Unsupported format. Use 'json', 'yaml', or 'yml'".to_string()),
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

    // If this is an SVG template, validate all SVG content
    if template.icon_type == "svg" {
        // Validate fallback icon
        if !is_valid_svg(&template.fallback_icon) {
            return Err("Invalid or potentially unsafe SVG in fallback icon".to_string());
        }

        // Validate all icon SVG content
        for (name, svg_content) in &template.icons {
            if !is_valid_svg(svg_content) {
                return Err(format!(
                    "Invalid or potentially unsafe SVG in icon '{}': {}",
                    name,
                    svg_content.chars().take(50).collect::<String>()
                ));
            }
        }
    }

    Ok(template)
}

/// Validate SVG content for security
fn is_valid_svg(content: &str) -> bool {
    if content.is_empty() {
        return false;
    }

    let content = content.trim();

    // Must start with <svg and end with </svg>
    if !content.starts_with("<svg") || !content.ends_with("</svg>") {
        return false;
    }

    // Check for potentially dangerous content (case-insensitive)
    let content_lower = content.to_lowercase();
    let dangerous_patterns = [
        "<script",
        "javascript:",
        "onclick=",
        "onload=",
        "onerror=",
        "onmouseover=",
        "onfocus=",
        "onblur=",
        "<iframe",
        "<object",
        "<embed",
        "<link",
        "<style",
        "<meta",
        "data:text/html",
        "data:application/javascript",
        "data:text/javascript",
        "href=\"javascript:",
        "src=\"javascript:",
        "&#x",
        "&#",          // HTML entities that could be used for obfuscation
        "expression(", // CSS expressions
        "import(",     // ES6 imports
        "eval(",       // JavaScript eval
        "settimeout",
        "setinterval", // Timer functions
    ];

    for pattern in &dangerous_patterns {
        if content_lower.contains(pattern) {
            return false;
        }
    }

    // Additional check for event handlers with various quote styles
    let event_handler_regex = regex::Regex::new(r"(?i)on\w+\s*=").unwrap();
    if event_handler_regex.is_match(content) {
        return false;
    }

    true
}

/// Get the icons directory path for frontend file operations
pub async fn get_icons_directory_path() -> Result<String, String> {
    let icons_dir = ensure_icons_dir().await?;
    Ok(icons_dir.to_string_lossy().to_string())
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
