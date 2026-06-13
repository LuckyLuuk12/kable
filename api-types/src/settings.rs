use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppearanceSettings {
    #[serde(default)]
    pub selected_css_theme: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LoggingSettings {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NetworkSettings {
    #[serde(default)]
    pub use_proxy: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ContentSettings {
    #[serde(default)]
    pub allow_adult_content: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AdvancedSettings {
    #[serde(default)]
    pub developer_mode: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MiscSettings {
    #[serde(default)]
    pub check_for_updates_on_start: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinecraftDirectoryInfo {
    pub path: String,
    pub exists: bool,
}
