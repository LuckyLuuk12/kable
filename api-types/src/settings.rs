use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::icons::IconTemplate;

#[derive(Debug, Serialize, Deserialize, Clone, Default, Type)]
#[serde(default)]
pub struct CategorizedLauncherSettings {
    pub general: GeneralSettings,
    pub appearance: AppearanceSettings,
    pub content: ContentSettings,
    pub logging: LoggingSettings,
    pub network: NetworkSettings,
    pub advanced: AdvancedSettings,
    pub misc: MiscSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(default)]
pub struct GeneralSettings {
    pub java_path: Option<String>,
    pub game_directory: Option<String>,
    pub on_game_close: OnGameAction,
    pub on_game_crash: OnGameAction,
    pub on_game_launch: OnGameAction,
    pub update_mode: UpdateMode,
    pub update_detection: UpdateDetection,
    pub update_notification_style: UpdateNotificationStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct SoundSettings {
    pub enabled: bool,
    pub music_enabled: bool,
    pub master_volume: u32,
    pub sound_volume: u32,
    pub music_volume: u32,
    pub selected_soundpack: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct AppearanceSettings {
    /// Optional selected CSS theme ID.
    ///
    /// `None` means the application default styling should be used.
    /// Built-in IDs: `builtin:kasai-sora`, `builtin:modrinth`, `builtin:official`.
    /// Custom IDs: `custom:<filename>`.
    pub theme: Option<String>,
    pub language: Language,

    /// Selected icon template ID. `None` uses the default icon template.
    pub icon_template: Option<String>,
    pub custom_icon_templates: Vec<IconTemplate>,
    pub sound_settings: SoundSettings,
}

/// The origin of a CSS theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum CssThemeType {
    Builtin,
    Custom,
}

/// Metadata for a CSS theme available to Kable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CssTheme {
    /// Stable identifier, e.g. `builtin:official` or `custom:my-theme`.
    pub id: String,

    /// Human-readable name shown in the appearance settings.
    pub name: String,

    /// Whether this theme ships with Kable or was added by the user.
    #[serde(rename = "type")]
    pub theme_type: CssThemeType,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct LoggingSettings {
    pub enabled: bool,
    pub persistent: bool,
    pub compression: bool,
    pub retention_days: u32,
    pub max_file_size_mb: u32,

    pub frontend_batch_size: u32,
    pub frontend_batch_interval_ms: u32,
    pub frontend_max_per_second: u32,

    pub max_memory_logs: u32,

    pub dedupe_enabled: bool,
    pub dedupe_window_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct NetworkSettings {
    pub max_download_threads: Option<u32>,
    pub max_download_speed_kbps: Option<u32>,
    pub max_requests_per_second: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct ContentSettings {
    pub allow_adult_content: bool,
    pub allow_ads: bool,
    pub enable_recommendations: bool,
    pub enable_notifications: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct AdvancedSettings {
    pub enable_advanced_features: bool,
    pub enable_nightly_updates: bool,
    pub developer_mode: bool,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct MiscSettings {
    /// I keep this mostly undocumented because it is open-source and I don't want to spoil the fun features (:
    pub enable_fun: bool,
}

//?---------------------------------------------------------------------
//? custom enums to avoid using strings
//?---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum OnGameAction {
    Ask,
    Exit,
    Minimize,
    MinimizeToTray,
    Nothing,
    /// Opens a specific page by name, e.g. "home", "logs", etc.
    Open(String),
    Restart,
}

/// This determines HOW, once checked, to perform the update.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum UpdateMode {
    /// After a check the user gets prompted to confirm, skip/cancel or only download and wait for restart.
    OnConfirm,

    /// Download and install automatically, without asking the user.
    Automatic,

    /// No automatic checks or prompts. Users must manage updates manually.
    Manual,
}

/// This determines WHEN to check for updates.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum UpdateDetection {
    OnStartup,
    OnClose,

    /// Checks for updates every N seconds.
    Periodically(u32),

    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum UpdateNotificationStyle {
    Notification,
    Modal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    English,
    // TODO: add more languages when a localization system exists.
}

//?---------------------------------------------------------------------
//? impl blocks, default, etc.
//?---------------------------------------------------------------------

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            java_path: None,
            game_directory: None,
            on_game_close: OnGameAction::Nothing,
            on_game_crash: OnGameAction::Open("open_logs".to_string()),
            on_game_launch: OnGameAction::Open("open_logs".to_string()),
            update_mode: UpdateMode::OnConfirm,
            update_detection: UpdateDetection::OnStartup,
            update_notification_style: UpdateNotificationStyle::Modal,
        }
    }
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            music_enabled: true,
            // Avoid blasting users' ears when the game starts.
            master_volume: 25,
            sound_volume: 100,
            music_volume: 100,
            selected_soundpack: "default".to_string(),
        }
    }
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: None,
            language: Language::English,
            icon_template: None,
            custom_icon_templates: vec![],
            sound_settings: SoundSettings::default(),
        }
    }
}

impl Default for ContentSettings {
    fn default() -> Self {
        Self { allow_adult_content: false, allow_ads: true, enable_recommendations: true, enable_notifications: true }
    }
}

impl Default for LoggingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            persistent: false,
            compression: true,
            retention_days: 30,
            max_file_size_mb: 10,
            frontend_batch_size: 400,
            frontend_batch_interval_ms: 1000,
            frontend_max_per_second: 10,
            max_memory_logs: 5000,
            dedupe_enabled: true,
            dedupe_window_size: 200,
        }
    }
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self { max_download_threads: Some(8), max_download_speed_kbps: None, max_requests_per_second: None }
    }
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self { enable_nightly_updates: false, developer_mode: false, extra: HashMap::new(), enable_advanced_features: true }
    }
}

impl Default for MiscSettings {
    fn default() -> Self {
        Self { enable_fun: true }
    }
}
