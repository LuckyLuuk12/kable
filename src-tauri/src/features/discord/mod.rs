use api_types::profiles::KableProfile;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use discord_rich_presence::activity::{Activity, Assets, Timestamps};

use crate::constants::DISCORD_APP_ID;
use crate::integrations::discord::DiscordClient;

use api_types::discord::{ActivityPriority, PresenceState};

static DISCORD: Lazy<Mutex<DiscordManager>> = Lazy::new(|| Mutex::new(DiscordManager::new()));

pub struct DiscordManager {
    client: DiscordClient,
    current_state: PresenceState,
    enabled: bool,
}

impl DiscordManager {
    // Default state
    pub fn default_presence_state() -> PresenceState {
        PresenceState {
            state: "In Launcher".to_string(),
            details: "Launcher".to_string(),
            priority: ActivityPriority::Idle,
            large_image: Some("kable_logo".to_string()),
            large_text: Some("Launcher".to_string()),
            small_image: None,
            small_text: None,
            start_timestamp: None,
        }
    }

    fn new() -> Self {
        Self { client: DiscordClient::new(DISCORD_APP_ID), current_state: Self::default_presence_state(), enabled: true }
    }

    // Lifecycle
    fn initialize(&mut self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        self.client.connect()?;
        self.update_presence()?;
        Ok(())
    }

    fn set_enabled(&mut self, enabled: bool) -> Result<(), String> {
        self.enabled = enabled;

        if enabled {
            self.initialize()?;
        } else {
            self.client.disconnect()?;
        }

        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), String> {
        self.client.disconnect()
    }

    fn clear(&mut self) -> Result<(), String> {
        self.current_state = Self::default_presence_state();
        self.update_presence()
    }

    // State handling
    fn set_state(&mut self, new_state: PresenceState) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        if new_state.priority >= self.current_state.priority {
            self.current_state = new_state;
            self.update_presence()?;
        }

        Ok(())
    }

    // Presence update
    fn update_presence(&mut self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        let mut assets = Assets::new();

        if let Some(v) = self.current_state.large_image.as_deref() {
            assets = assets.large_image(v);
        }
        if let Some(v) = self.current_state.large_text.as_deref() {
            assets = assets.large_text(v);
        }
        if let Some(v) = self.current_state.small_image.as_deref() {
            assets = assets.small_image(v);
        }
        if let Some(v) = self.current_state.small_text.as_deref() {
            assets = assets.small_text(v);
        }

        let mut activity = Activity::new().state(&self.current_state.state).details(&self.current_state.details).assets(assets);

        if let Some(ts) = self.current_state.start_timestamp {
            activity = activity.timestamps(Timestamps::new().start(ts.into()));
        }

        self.client.set_activity(activity)
    }
}

pub fn initialize() -> Result<(), String> {
    let mut m = DISCORD.lock().map_err(|e| e.to_string())?;
    m.initialize()
}

pub fn set_enabled(enabled: bool) -> Result<(), String> {
    let mut m = DISCORD.lock().map_err(|e| e.to_string())?;
    m.set_enabled(enabled)
}

pub fn set_playing(profile: &KableProfile) -> Result<(), String> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i32;

    let profile_name = &profile.name;
    let version = &profile.version.minecraft_version.clone().unwrap_or_default(); // e.g. 1.20.1
    let loader = profile.version.loader;

    let small_image = Some(profile.version.loader.to_string());
    let small_text = profile.version.loader_version.clone(); // e.g. 47.0.0 for version.id == 1.20.1-forge-47.0.0

    let state = PresenceState {
        state: format!("{profile_name} • {loader}"),
        details: "Playing Minecraft launched with Kable".to_string(),
        priority: ActivityPriority::Playing,
        large_image: Some("minecraft_logo".to_string()),
        large_text: Some(format!("Minecraft Java Edition {version}")),
        small_image,
        small_text,
        start_timestamp: Some(timestamp),
    };

    let mut m = DISCORD.lock().map_err(|e| e.to_string())?;
    m.set_state(state)
}

pub fn set_browsing(section: &str) -> Result<(), String> {
    let state = match section {
        "mods" => "Browsing Mods",
        "shaders" => "Browsing Shaders",
        "resourcepacks" | "resource-packs" => "Browsing Resource Packs",
        "maps" => "Browsing Maps",
        "installations" => "Managing Installations",
        "settings" => "Configuring Settings",
        "skins" => "Customizing Skins",
        "profile" => "Viewing Profile",
        _ => "In Launcher",
    };

    let presence = PresenceState {
        state: state.to_string(),
        details: "Launcher".to_string(),
        priority: ActivityPriority::Browsing,
        large_image: Some("kable_logo".to_string()),
        large_text: Some("Launcher".to_string()),
        small_image: None,
        small_text: None,
        start_timestamp: None,
    };

    let mut m = DISCORD.lock().map_err(|e| e.to_string())?;
    m.set_state(presence)
}

pub fn clear_playing() -> Result<(), String> {
    let mut m = DISCORD.lock().map_err(|e| e.to_string())?;
    m.clear()
}

pub fn clear() -> Result<(), String> {
    let mut m = DISCORD.lock().map_err(|e| e.to_string())?;
    m.clear()
}

pub fn disconnect() -> Result<(), String> {
    let mut m = DISCORD.lock().map_err(|e| e.to_string())?;
    m.disconnect()
}
