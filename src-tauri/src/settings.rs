use crate::logging::{LogLevel, Logger};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategorizedLauncherSettings {
    pub general: GeneralSettings,
    pub appearance: AppearanceSettings,
    pub logging: LoggingSettings,
    pub network: NetworkSettings,
    pub content: ContentSettings,
    pub advanced: AdvancedSettings,
    pub misc: MiscSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralSettings {
    pub java_path: Option<String>,
    pub game_directory: Option<String>,
    /// 'open_logs' | 'open_home' | 'exit' | 'minimize' | 'ask'
    pub on_game_close: String,
    /// 'restart' | 'open_logs' | 'open_home' | 'exit' | 'minimize' | 'ask'
    pub on_game_crash: String,
    /// 'keep_open' | 'exit' | 'open_logs' | 'minimize' | 'ask'
    pub on_game_launch: String,
    pub auto_update_launcher: bool,
    pub show_ads: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppearanceSettings {
    pub theme: String, // 'light' | 'dark' | 'system'
    pub language: String,
    pub extra_spacing: u32,
    pub sidebar_width: u32,
    pub selected_icon_template: String,
    // For simplicity, icon_settings as raw JSON
    pub icon_settings: serde_json::Value,
    pub selected_css_theme: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingSettings {
    pub show_logs_page_in_nav: bool,
    pub enable_persistent_logging: bool,
    pub enable_log_compression: bool,
    pub log_file_size_limit_mb: serde_json::Value, // number or "disabled"
    pub log_retention_days: serde_json::Value,     // number or "disabled"
    pub merge_log_tabs: bool,
    pub default_log_levels: Vec<String>, // 'debug' | 'info' | 'warn' | 'error'
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkSettings {
    pub parallel_downloads: u32,
    pub connection_timeout: u32,
    pub download_speed_limit: serde_json::Value, // number or "unlimited"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentSettings {
    pub max_world_backups: serde_json::Value, // number or "disabled"
    pub auto_backup_worlds: bool,
    pub use_per_installation_mods_folder: bool,
    pub use_per_installation_resource_packs: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdvancedSettings {
    pub enable_experimental_features: bool,
    pub default_memory: u32,
    pub separate_logs_window: bool,
    pub auto_save_interval: u32, // in seconds, 0 means no auto save
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MiscSettings {
    pub use_titlebar: bool,
    pub auth_preference: String, // 'code' | 'device_code'
}

impl Default for CategorizedLauncherSettings {
    fn default() -> Self {
        Self {
            general: GeneralSettings {
                java_path: None,
                game_directory: None,
                on_game_close: "exit".to_string(),
                on_game_crash: "restart".to_string(),
                on_game_launch: "keep_open".to_string(),
                auto_update_launcher: true,
                show_ads: false,
            },
            appearance: AppearanceSettings {
                theme: "dark".to_string(),
                language: "en".to_string(),
                extra_spacing: 0,
                sidebar_width: 250,
                selected_icon_template: "emoji".to_string(),
                icon_settings: serde_json::Value::Object(serde_json::Map::new()),
                selected_css_theme: "default".to_string(),
            },
            logging: LoggingSettings {
                show_logs_page_in_nav: true,
                enable_persistent_logging: false,
                enable_log_compression: true,
                log_file_size_limit_mb: serde_json::json!(10),
                log_retention_days: serde_json::json!(30),
                merge_log_tabs: false,
                default_log_levels: vec![
                    "info".to_string(),
                    "warn".to_string(),
                    "error".to_string(),
                ],
            },
            network: NetworkSettings {
                parallel_downloads: 3,
                connection_timeout: 30,
                download_speed_limit: serde_json::json!("unlimited"),
            },
            content: ContentSettings {
                max_world_backups: serde_json::json!(5),
                auto_backup_worlds: false,
                use_per_installation_mods_folder: false,
                use_per_installation_resource_packs: false,
            },
            advanced: AdvancedSettings {
                enable_experimental_features: false,
                default_memory: 2048,
                separate_logs_window: false,
                auto_save_interval: 30, // in seconds, 0 means no auto save
            },
            misc: MiscSettings {
                use_titlebar: true,
                auth_preference: "code".to_string(),
            },
        }
    }
}

// Helper functions
// pub fn get_kable_launcher_dir() -> Result<PathBuf, AppError> {
//     let base_dir = if let Some(appdata) = dirs::data_dir() {
//         appdata
//     } else {
//         return Err(AppError::Io(std::io::Error::new(
//             std::io::ErrorKind::NotFound,
//             "Could not find data directory"
//         )));
//     };

//     let launcher_dir = base_dir.join("kable-launcher");

//     if !launcher_dir.exists() {
//         fs::create_dir_all(&launcher_dir)?;
//     }

//     Ok(launcher_dir)
// }

fn get_settings_path() -> Result<PathBuf, String> {
    // Ok(crate::get_kable_launcher_dir()?.join("settings.json"))
    let kable_dir = crate::get_kable_launcher_dir().map_err(|e| e.to_string())?;
    // ensure the file exists
    let settings_path = kable_dir.join("settings.json");
    if !settings_path.exists() {
        fs::create_dir_all(&kable_dir).map_err(|e| e.to_string())?;
        fs::write(
            &settings_path,
            serde_json::to_string_pretty(&CategorizedLauncherSettings::default())
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(settings_path)
}

// Settings management
#[tauri::command]
pub async fn load_settings() -> Result<CategorizedLauncherSettings, String> {
    let settings_path = get_settings_path().map_err(|e| e.to_string())?;

    if settings_path.exists() {
        let contents = fs::read_to_string(&settings_path).map_err(|e| e.to_string())?;
        let settings: CategorizedLauncherSettings =
            serde_json::from_str(&contents).map_err(|e| e.to_string())?;
        Ok(settings)
    } else {
        let default_settings = CategorizedLauncherSettings::default();
        save_settings(default_settings.clone())?;
        Ok(default_settings)
    }
}

#[tauri::command]
pub fn save_settings(settings: CategorizedLauncherSettings) -> Result<(), String> {
    let settings_path = get_settings_path().map_err(|e| e.to_string())?;
    let contents = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&settings_path, contents).map_err(|e| e.to_string())?;
    Logger::console_log(
        LogLevel::Info,
        &format!("Settings saved to: {}", settings_path.display()),
        None,
    );
    Ok(())
}

// Get launcher data directory
// #[tauri::command]
// pub async fn get_launcher_directory() -> Result<String, String> {
//     let launcher_dir = get_kable_launcher_dir().map_err(|e| e.to_string())?;
//     Ok(launcher_dir.to_string_lossy().to_string())
// }

// Get default Minecraft directory
// pub async fn get_default_minecraft_directory() -> Result<String, String> {
//     let minecraft_dir = if cfg!(target_os = "windows") {
//         if let Some(appdata) = dirs::data_dir() {
//             appdata.join(".minecraft")
//         } else {
//             return Err("Could not find AppData directory".to_string());
//         }
//     } else if cfg!(target_os = "macos") {
//         if let Some(home) = dirs::home_dir() {
//             home.join("Library").join("Application Support").join("minecraft")
//         } else {
//             return Err("Could not find home directory".to_string());
//         }
//     } else {
//         // Linux
//         if let Some(home) = dirs::home_dir() {
//             home.join(".minecraft")
//         } else {
//             return Err("Could not find home directory".to_string());
//         }
//     };

//     Ok(minecraft_dir.to_string_lossy().to_string())
// }

// Validate Minecraft directory
#[tauri::command]
pub async fn validate_minecraft_directory(path: String) -> Result<MinecraftDirectoryInfo, String> {
    let minecraft_path = PathBuf::from(path);

    if !minecraft_path.exists() {
        return Err("Directory does not exist".to_string());
    }

    let saves_dir = minecraft_path.join("saves");
    let shaderpacks_dir = minecraft_path.join("shaderpacks");
    let resourcepacks_dir = minecraft_path.join("resourcepacks");

    let has_saves = saves_dir.exists();
    let has_shaderpacks = shaderpacks_dir.exists();
    let has_resourcepacks = resourcepacks_dir.exists();

    // Count directories/files
    let saves_count = if has_saves {
        count_directories(&saves_dir).unwrap_or(0)
    } else {
        0
    };

    let shaderpacks_count = if has_shaderpacks {
        count_files_with_extensions(&shaderpacks_dir, &[".zip", ".jar"]).unwrap_or(0)
    } else {
        0
    };

    let resourcepacks_count = if has_resourcepacks {
        count_files_with_extensions(&resourcepacks_dir, &[".zip", ".jar"]).unwrap_or(0)
    } else {
        0
    };

    // Calculate directory size (approximation)
    let size_mb = calculate_directory_size(&minecraft_path).unwrap_or(0) / (1024 * 1024);

    let is_valid = minecraft_path.join("launcher_profiles.json").exists()
        || minecraft_path.join("versions").exists()
        || has_saves
        || has_shaderpacks
        || has_resourcepacks;

    Ok(MinecraftDirectoryInfo {
        path: minecraft_path.to_string_lossy().to_string(),
        is_valid,
        has_saves,
        has_shaderpacks,
        has_resourcepacks,
        saves_count,
        shaderpacks_count,
        resourcepacks_count,
        size_mb,
    })
}

// // Helper function to count directories
fn count_directories(path: &PathBuf) -> Result<u32, std::io::Error> {
    let mut count = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            count += 1;
        }
    }
    Ok(count)
}

// // Helper function to count files with specific extensions
fn count_files_with_extensions(path: &PathBuf, extensions: &[&str]) -> Result<u32, std::io::Error> {
    let mut count = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            if let Some(file_name) = entry.file_name().to_str() {
                for ext in extensions {
                    if file_name.to_lowercase().ends_with(ext) {
                        count += 1;
                        break;
                    }
                }
            }
        }
    }
    Ok(count)
}

// // Helper function to calculate directory size
fn calculate_directory_size(path: &PathBuf) -> Result<u64, std::io::Error> {
    let mut size = 0;
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                size += entry.metadata()?.len();
            } else if path.is_dir() {
                size += calculate_directory_size(&path)?;
            }
        }
    }
    Ok(size)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftDirectoryInfo {
    pub path: String,
    pub is_valid: bool,
    pub has_saves: bool,
    pub has_shaderpacks: bool,
    pub has_resourcepacks: bool,
    pub saves_count: u32,
    pub shaderpacks_count: u32,
    pub resourcepacks_count: u32,
    pub size_mb: u64,
}

/// Load custom CSS content from a theme name
#[tauri::command]
pub async fn load_custom_css(theme_name: String, app: tauri::AppHandle) -> Result<String, String> {
    load_css_theme(theme_name, app).await
}

/// Set the selected CSS theme in settings
#[tauri::command]
pub async fn set_selected_css_theme(theme_name: String) -> Result<(), String> {
    let mut settings = load_settings().await?;
    settings.appearance.selected_css_theme = theme_name;
    save_settings(settings)?;
    Ok(())
}

/// Get the current selected CSS theme from settings
#[tauri::command]
pub async fn get_selected_css_theme() -> Result<String, String> {
    let settings = load_settings().await?;
    Ok(settings.appearance.selected_css_theme)
}

/// Select a CSS file using the system file dialog
#[tauri::command]
pub async fn select_css_file() -> Result<Option<String>, String> {
    // use tauri_plugin_dialog::DialogExt;

    // We need the app handle to use the dialog plugin
    // For now, return an error indicating this needs to be implemented differently
    Err("File dialog not implemented yet - please enter path manually".to_string())
}

/// Get the CSS themes configuration directory
async fn get_css_themes_dir() -> Result<PathBuf, String> {
    let launcher_dir = crate::get_kable_launcher_dir()?;
    Ok(launcher_dir.join("config").join("themes"))
}

/// Ensure the CSS themes directory exists
async fn ensure_css_themes_dir() -> Result<PathBuf, String> {
    let themes_dir = get_css_themes_dir().await?;
    if !themes_dir.exists() {
        fs::create_dir_all(&themes_dir)
            .map_err(|e| format!("Failed to create themes directory: {}", e))?;
    }
    Ok(themes_dir)
}

/// Get all available CSS themes (built-in + custom)
#[tauri::command]
pub async fn get_css_themes(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let mut themes = vec!["default".to_string()]; // Always include default

    // Add built-in themes from static/themes directory
    if let Ok(builtin_themes) = get_builtin_css_themes(&app).await {
        themes.extend(builtin_themes);
    }

    // Add custom themes from user directory
    let themes_dir = get_css_themes_dir().await?;
    if themes_dir.exists() {
        let entries = fs::read_dir(&themes_dir)
            .map_err(|e| format!("Failed to read themes directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if let Some(extension) = path.extension() {
                if extension == "css" {
                    if let Some(theme_name) = path.file_stem().and_then(|stem| stem.to_str()) {
                        themes.push(theme_name.to_string());
                    }
                }
            }
        }
    }

    // Remove duplicates (in case a custom theme has the same name as built-in)
    themes.sort();
    themes.dedup();

    Ok(themes)
}

/// Get built-in CSS themes from static/themes directory
async fn get_builtin_css_themes(app: &tauri::AppHandle) -> Result<Vec<String>, String> {
    let mut builtin_themes = Vec::new();

    // First try using the Tauri resource resolution method
    match app
        .path()
        .resolve("static/themes", tauri::path::BaseDirectory::Resource)
    {
        Ok(themes_dir) => {
            Logger::console_log(
                LogLevel::Debug,
                &format!("Resolved themes directory: {:?}", themes_dir),
                None,
            );

            if themes_dir.exists() {
                let entries = fs::read_dir(&themes_dir)
                    .map_err(|e| format!("Failed to read built-in themes directory: {}", e))?;

                for entry in entries {
                    let entry =
                        entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                    let path = entry.path();

                    if !path.is_file() {
                        continue;
                    }

                    if let Some(extension) = path.extension() {
                        if extension == "css" {
                            if let Some(theme_name) =
                                path.file_stem().and_then(|stem| stem.to_str())
                            {
                                Logger::console_log(
                                    LogLevel::Info,
                                    &format!("Found built-in theme: {}", theme_name),
                                    None,
                                );
                                builtin_themes.push(theme_name.to_string());
                            }
                        }
                    }
                }
                return Ok(builtin_themes);
            } else {
                Logger::console_log(
                    LogLevel::Debug,
                    &format!("Built-in themes directory does not exist: {:?}", themes_dir),
                    None,
                );
            }
        }
        Err(e) => {
            Logger::console_log(
                LogLevel::Debug,
                &format!("Failed to resolve static/themes path: {}", e),
                None,
            );
        }
    }

    // Fallback: Try to find static/themes in development mode (relative to the workspace root)
    if cfg!(debug_assertions) {
        Logger::console_log(
            LogLevel::Debug,
            "Development mode: trying to find static/themes relative to workspace",
            None,
        );

        // Try to find the workspace root by looking for package.json + src-tauri directory
        let current_exe = std::env::current_exe().unwrap_or_default();
        let mut search_dir = current_exe.parent().unwrap_or(std::path::Path::new("."));

        for level in 0..10 {
            // Search up to 10 levels up
            Logger::console_log(
                LogLevel::Debug,
                &format!("Searching level {}: {:?}", level, search_dir),
                None,
            );

            let package_json = search_dir.join("package.json");
            let src_tauri_dir = search_dir.join("src-tauri");

            // Look for workspace markers: package.json at root level AND src-tauri directory
            if package_json.exists() && src_tauri_dir.exists() {
                let static_themes_dir = search_dir.join("static").join("themes");

                Logger::console_log(
                    LogLevel::Debug,
                    &format!(
                        "Found workspace root, checking static themes path: {:?}",
                        static_themes_dir
                    ),
                    None,
                );

                if static_themes_dir.exists() {
                    Logger::console_log(
                        LogLevel::Info,
                        &format!(
                            "Found development static themes directory: {:?}",
                            static_themes_dir
                        ),
                        None,
                    );

                    let entries = fs::read_dir(&static_themes_dir)
                        .map_err(|e| format!("Failed to read built-in themes directory: {}", e))?;

                    for entry in entries {
                        let entry =
                            entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                        let path = entry.path();

                        if !path.is_file() {
                            continue;
                        }

                        if let Some(extension) = path.extension() {
                            if extension == "css" {
                                if let Some(theme_name) =
                                    path.file_stem().and_then(|stem| stem.to_str())
                                {
                                    Logger::console_log(
                                        LogLevel::Info,
                                        &format!("Found built-in theme: {}", theme_name),
                                        None,
                                    );
                                    builtin_themes.push(theme_name.to_string());
                                }
                            }
                        }
                    }
                } else {
                    Logger::console_log(
                        LogLevel::Debug,
                        &format!(
                            "Static themes directory not found at: {:?}",
                            static_themes_dir
                        ),
                        None,
                    );
                }
                break;
            }

            if let Some(parent) = search_dir.parent() {
                search_dir = parent;
            } else {
                break;
            }
        }
    }

    Ok(builtin_themes)
}

/// Save a CSS theme file
#[tauri::command]
pub async fn save_css_theme(theme_name: String, css_content: String) -> Result<String, String> {
    let themes_dir = ensure_css_themes_dir().await?;
    let file_path = themes_dir.join(format!("{}.css", theme_name));

    fs::write(&file_path, css_content).map_err(|e| format!("Failed to write theme file: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

/// Delete a custom CSS theme
#[tauri::command]
pub async fn delete_css_theme(theme_name: String) -> Result<(), String> {
    if theme_name == "default" {
        return Err("Cannot delete the default theme".to_string());
    }

    let themes_dir = get_css_themes_dir().await?;
    let file_path = themes_dir.join(format!("{}.css", theme_name));

    if file_path.exists() {
        fs::remove_file(&file_path).map_err(|e| format!("Failed to delete theme file: {}", e))?;
    }

    Ok(())
}

/// Load CSS content for a theme
#[tauri::command]
pub async fn load_css_theme(theme_name: String, app: tauri::AppHandle) -> Result<String, String> {
    if theme_name == "default" {
        return Ok(String::new()); // Default theme has no custom CSS
    }

    // First try to load from custom themes directory
    let themes_dir = get_css_themes_dir().await?;
    let custom_file_path = themes_dir.join(format!("{}.css", theme_name));

    if custom_file_path.exists() {
        Logger::console_log(
            LogLevel::Info,
            &format!("Loading custom theme: {}", theme_name),
            None,
        );
        return fs::read_to_string(&custom_file_path)
            .map_err(|e| format!("Failed to read custom theme file: {}", e));
    }

    // If not found in custom themes, try built-in themes
    match app.path().resolve(
        format!("static/themes/{}.css", theme_name),
        tauri::path::BaseDirectory::Resource,
    ) {
        Ok(builtin_file_path) => {
            Logger::console_log(
                LogLevel::Debug,
                &format!("Resolved built-in theme path: {:?}", builtin_file_path),
                None,
            );

            if builtin_file_path.exists() {
                Logger::console_log(
                    LogLevel::Info,
                    &format!(
                        "Loading built-in theme '{}' from: {:?}",
                        theme_name, builtin_file_path
                    ),
                    None,
                );
                return fs::read_to_string(&builtin_file_path)
                    .map_err(|e| format!("Failed to read built-in theme file: {}", e));
            } else {
                Logger::console_log(
                    LogLevel::Debug,
                    &format!(
                        "Built-in theme file does not exist: {:?}",
                        builtin_file_path
                    ),
                    None,
                );
            }
        }
        Err(e) => {
            Logger::console_log(
                LogLevel::Debug,
                &format!(
                    "Failed to resolve built-in theme path for '{}': {}",
                    theme_name, e
                ),
                None,
            );
        }
    }

    // Fallback: Try development mode path
    if cfg!(debug_assertions) {
        Logger::console_log(
            LogLevel::Debug,
            &format!(
                "Development mode: trying to find theme '{}' in workspace",
                theme_name
            ),
            None,
        );

        let current_exe = std::env::current_exe().unwrap_or_default();
        let mut search_dir = current_exe.parent().unwrap_or(std::path::Path::new("."));

        // Search up the directory tree to find workspace root
        for level in 0..10 {
            // Increased search depth
            Logger::console_log(
                LogLevel::Debug,
                &format!("Searching level {}: {:?}", level, search_dir),
                None,
            );

            let package_json = search_dir.join("package.json");
            let src_tauri_dir = search_dir.join("src-tauri");

            // Look for workspace markers: package.json at root level AND src-tauri directory
            if package_json.exists() && src_tauri_dir.exists() {
                let theme_file_path = search_dir
                    .join("static")
                    .join("themes")
                    .join(format!("{}.css", theme_name));

                Logger::console_log(
                    LogLevel::Debug,
                    &format!(
                        "Found workspace root, checking theme path: {:?}",
                        theme_file_path
                    ),
                    None,
                );

                if theme_file_path.exists() {
                    Logger::console_log(
                        LogLevel::Info,
                        &format!(
                            "Loading built-in theme '{}' from development path: {:?}",
                            theme_name, theme_file_path
                        ),
                        None,
                    );
                    return fs::read_to_string(&theme_file_path)
                        .map_err(|e| format!("Failed to read built-in theme file: {}", e));
                } else {
                    Logger::console_log(
                        LogLevel::Debug,
                        &format!(
                            "Theme file not found at expected path: {:?}",
                            theme_file_path
                        ),
                        None,
                    );
                }
                break;
            }

            if let Some(parent) = search_dir.parent() {
                search_dir = parent;
            } else {
                break;
            }
        }
    }

    Err(format!(
        "Theme '{}' not found in custom or built-in themes",
        theme_name
    ))
}

/// Open the CSS themes directory in file explorer
#[tauri::command]
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
