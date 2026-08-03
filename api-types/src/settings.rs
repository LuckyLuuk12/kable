use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::icons::IconTemplate;

#[derive(Debug, Serialize, Deserialize, Clone, Default, facet::Facet, specta::Type)]
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

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
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

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
#[serde(default)]
pub struct SoundSettings {
    pub enabled: bool,
    pub music_enabled: bool,
    pub master_volume: u32,
    pub sound_volume: u32,
    pub music_volume: u32,
    pub selected_soundpack: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
#[serde(default)]
pub struct AppearanceSettings {
    pub theme: Theme,
    pub language: Language,
    pub icon_template: Option<String>,
    pub custom_icon_templates: Vec<IconTemplate>,
    pub selected_css_theme: Option<Theme>,
    pub sound_settings: SoundSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
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

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
#[serde(default)]
pub struct NetworkSettings {
    pub max_download_threads: Option<u32>,
    pub max_download_speed_kbps: Option<u32>,
    pub max_requests_per_second: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
#[serde(default)]
pub struct ContentSettings {
    pub allow_adult_content: bool,
    pub allow_ads: bool,
    pub enable_recommendations: bool,
    pub enable_notifications: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
#[serde(default)]
pub struct AdvancedSettings {
    pub enable_advanced_features: bool,
    pub enable_nightly_updates: bool,
    pub developer_mode: bool,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
#[serde(default)]
pub struct MiscSettings {
    /// I keep this mostly undocumented because it is open-source and I don't want to spoil the fun features (:
    pub enable_fun: bool,
}

//?---------------------------------------------------------------------
//? custom enums to avoid using strings
//?---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
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
#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum UpdateMode {
    /// After a check the user gets prompted to confirm, skip/cancel or only download and wait for restart
    OnConfirm,
    /// Download + install automatically, without asking the user
    Automatic,
    /// No checks, no prompts, nothing. Users have to go to settings and do everything manually.
    Manual,
}
/// This determines WHEN to check for updates
#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum UpdateDetection {
    OnStartup,
    OnClose,
    /// Checks for updates every N seconds
    Periodically(u32),
    Manual,
}
#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum UpdateNotificationStyle {
    Notification,
    Modal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Theme {
    Light,
    Dark,
    System,
    /// Custom theme, specified by a path/name to a CSS file
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Language {
    English,
    // TODO: if we got some cool system for localization we could add more here
}

//?---------------------------------------------------------------------
//? impl blocks, default, etc.
//?---------------------------------------------------------------------

// pub struct CategorizedLauncherSettings {
//     pub general: GeneralSettings,
//     pub appearance: AppearanceSettings,
//     pub content: ContentSettings,
//     pub logging: LoggingSettings,
//     pub network: NetworkSettings,
//     pub advanced: AdvancedSettings,
//     pub misc: MiscSettings,
// }

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
            // let's not blast people's ears off as games tend to do...
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
            theme: Theme::System,
            icon_template: None,
            language: Language::English,
            custom_icon_templates: vec![],
            selected_css_theme: None,
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
