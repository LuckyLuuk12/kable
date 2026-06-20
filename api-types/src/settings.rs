use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default, facet::Facet)]
pub struct CategorizedLauncherSettings {
    #[serde(default)]
    pub general: GeneralSettings,
    #[serde(default)]
    pub appearance: AppearanceSettings,
    #[serde(default)]
    pub logging: LoggingSettings,
    #[serde(default)]
    pub network: NetworkSettings,
    #[serde(default)]
    pub content: ContentSettings,
    #[serde(default)]
    pub advanced: AdvancedSettings,
    #[serde(default)]
    pub misc: MiscSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet)]
pub struct GeneralSettings {
    #[serde(default)]
    pub java_path: Option<String>,
    #[serde(default)]
    pub game_directory: Option<String>,
    #[serde(default = "default_on_game_close")]
    pub on_game_close: String,
    #[serde(default = "default_on_game_crash")]
    pub on_game_crash: String,
    #[serde(default = "default_on_game_launch")]
    pub on_game_launch: String,
    #[serde(default = "default_auto_update")]
    pub auto_update_launcher: bool,
    #[serde(default)]
    pub show_ads: bool,
    #[serde(default = "default_update_mode")]
    pub update_mode: String,
    #[serde(default = "default_update_notification_style")]
    pub update_notification_style: String,
}

fn default_on_game_close() -> String {
    "open_home".to_string()
}
fn default_on_game_crash() -> String {
    "open_logs".to_string()
}
fn default_on_game_launch() -> String {
    "open_logs".to_string()
}
fn default_auto_update() -> bool {
    true
}
fn default_update_mode() -> String {
    "on_confirm".to_string()
}
fn default_update_notification_style() -> String {
    "notification".to_string()
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            java_path: None,
            game_directory: None,
            on_game_close: default_on_game_close(),
            on_game_crash: default_on_game_crash(),
            on_game_launch: default_on_game_launch(),
            auto_update_launcher: default_auto_update(),
            show_ads: false,
            update_mode: default_update_mode(),
            update_notification_style: default_update_notification_style(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet)]
pub struct SoundSettings {
    #[serde(default = "default_sound_enabled")]
    pub enabled: bool,
    #[serde(default = "default_sound_enabled")]
    pub music_enabled: bool,
    #[serde(default = "default_volume")]
    pub master_volume: u32,
    #[serde(default = "default_volume")]
    pub sound_volume: u32,
    #[serde(default = "default_volume")]
    pub music_volume: u32,
    #[serde(default = "default_soundpack")]
    pub selected_soundpack: String,
}

fn default_sound_enabled() -> bool {
    true
}
fn default_volume() -> u32 {
    50
}
fn default_soundpack() -> String {
    "default".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, facet::Facet)]
pub struct AppearanceSettings {
    #[serde(default)]
    pub selected_css_theme: String,
}

// #[derive(Debug, Serialize, Deserialize, Clone, facet::Facet)]
// pub struct LoggingSettings {
//     #[serde(default)]
//     pub enabled: bool,
//     #[serde(default)]
//     pub enable_persistent_logging: bool,
//     #[serde(default)]
//     pub enable_log_compression: bool,
//     #[serde(default)]
//     pub log_file_size_limit_mb: u64,
//     #[serde(default)]
//     pub log_retention_days: u64,
//     #[serde(default)]
//     pub max_memory_logs: usize,
//     #[serde(default)]
//     pub dedupe_window_size: usize,
//     #[serde(default)]
//     pub enable_dedupe: bool,
// }

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet)]
pub struct LoggingSettings {
    pub enabled: bool,
    pub persistent: bool,
    pub compression: bool,
    pub retention_days: u64,
    pub max_file_size_mb: u64,

    pub frontend_batch_size: usize,
    pub frontend_batch_interval_ms: u64,
    pub frontend_max_per_second: usize,

    pub max_memory_logs: usize,

    pub dedupe_enabled: bool,
    pub dedupe_window_size: usize,
}

impl Default for LoggingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            persistent: true,
            compression: true,
            retention_days: 30,
            max_file_size_mb: 10,
            frontend_batch_size: 400,
            frontend_batch_interval_ms: 1000,
            frontend_max_per_second: 10,
            max_memory_logs: 5000,
            dedupe_enabled: true,
            dedupe_window_size: 50,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, facet::Facet)]
pub struct NetworkSettings {
    #[serde(default)]
    pub use_proxy: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, facet::Facet)]
pub struct ContentSettings {
    #[serde(default)]
    pub allow_adult_content: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, facet::Facet)]
pub struct AdvancedSettings {
    #[serde(default)]
    pub developer_mode: bool,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, facet::Facet)]
pub struct MiscSettings {
    #[serde(default)]
    pub check_for_updates_on_start: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet)]
pub struct MinecraftDirectoryInfo {
    pub path: String,
    pub exists: bool,
}
