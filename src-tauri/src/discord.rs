use discord_rich_presence::{
    activity::{Activity, Assets, Timestamps},
    DiscordIpc, DiscordIpcClient,
};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const DISCORD_APP_ID: &str = "1432139549592649738"; // Replace with actual Discord application ID

/// Priority levels for different activities
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActivityPriority {
    Idle = 0,
    Browsing = 1,
    Playing = 2, // Highest priority
}

/// Discord Rich Presence state
#[derive(Debug, Clone)]
pub struct PresenceState {
    pub state: String,
    pub details: String,
    pub priority: ActivityPriority,
    pub large_image: Option<String>,
    pub large_text: Option<String>,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
    pub start_timestamp: Option<i64>,
}

impl Default for PresenceState {
    fn default() -> Self {
        Self {
            state: "In Launcher".to_string(),
            details: "Kable - Minecraft Launcher".to_string(),
            priority: ActivityPriority::Idle,
            large_image: Some("kable_logo".to_string()),
            large_text: Some("Kable Launcher".to_string()),
            small_image: None,
            small_text: None,
            start_timestamp: None,
        }
    }
}

/// Global Discord RPC manager
pub struct DiscordRpcManager {
    client: Option<DiscordIpcClient>,
    current_state: PresenceState,
    enabled: bool,
}

static DISCORD_MANAGER: Lazy<Mutex<DiscordRpcManager>> = Lazy::new(|| {
    Mutex::new(DiscordRpcManager {
        client: None,
        current_state: PresenceState::default(),
        enabled: true,
    })
});

impl DiscordRpcManager {
    /// Initialize the Discord RPC client
    fn initialize(&mut self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        if self.client.is_some() {
            return Ok(());
        }

        let mut client = DiscordIpcClient::new(DISCORD_APP_ID);

        client
            .connect()
            .map_err(|e| format!("Failed to connect to Discord: {}", e))?;

        self.client = Some(client);
        crate::logging::Logger::info_global("Discord Rich Presence connected", None);

        // Set initial presence
        self.update_presence()?;

        Ok(())
    }

    /// Update the presence with the current state
    fn update_presence(&mut self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        let client = match &mut self.client {
            Some(c) => c,
            None => {
                self.initialize()?;
                self.client.as_mut().unwrap()
            }
        };

        let state = &self.current_state;

        let mut assets = Assets::new();
        if let Some(large_image) = &state.large_image {
            assets = assets.large_image(large_image);
        }
        if let Some(large_text) = &state.large_text {
            assets = assets.large_text(large_text);
        }
        if let Some(small_image) = &state.small_image {
            assets = assets.small_image(small_image);
        }
        if let Some(small_text) = &state.small_text {
            assets = assets.small_text(small_text);
        }

        let mut activity = Activity::new()
            .state(&state.state)
            .details(&state.details)
            .assets(assets);

        if let Some(timestamp) = state.start_timestamp {
            activity = activity.timestamps(Timestamps::new().start(timestamp));
        }

        client
            .set_activity(activity)
            .map_err(|e| format!("Failed to set Discord activity: {}", e))?;

        Ok(())
    }

    /// Set a new state if its priority is higher or equal to current
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

    /// Clear the presence
    fn clear(&mut self) -> Result<(), String> {
        if let Some(client) = &mut self.client {
            client
                .clear_activity()
                .map_err(|e| format!("Failed to clear Discord activity: {}", e))?;
        }
        Ok(())
    }

    /// Disconnect from Discord
    fn disconnect(&mut self) -> Result<(), String> {
        if let Some(mut client) = self.client.take() {
            client
                .close()
                .map_err(|e| format!("Failed to close Discord connection: {}", e))?;
            crate::logging::Logger::info_global("Discord Rich Presence disconnected", None);
        }
        Ok(())
    }
}

// Public API functions

/// Initialize Discord Rich Presence
pub fn initialize() -> Result<(), String> {
    let mut manager = DISCORD_MANAGER
        .lock()
        .map_err(|e| format!("Failed to lock Discord manager: {}", e))?;
    manager.initialize()
}

/// Enable or disable Discord Rich Presence
pub fn set_enabled(enabled: bool) -> Result<(), String> {
    let mut manager = DISCORD_MANAGER
        .lock()
        .map_err(|e| format!("Failed to lock Discord manager: {}", e))?;

    manager.enabled = enabled;

    if enabled {
        manager.initialize()?;
        manager.update_presence()?;
    } else {
        manager.disconnect()?;
    }

    Ok(())
}

/// Update presence for playing Minecraft
pub fn set_playing(installation_name: &str, version_id: &str, mod_loader: Option<&str>) -> Result<(), String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let loader_text = mod_loader.unwrap_or("Vanilla");
    let details = format!("Playing Minecraft {}", version_id);
    let state = format!("{} • {}", installation_name, loader_text);

    let (small_image, small_text) = match mod_loader {
        Some("fabric") => (Some("fabric".to_string()), Some("Fabric".to_string())),
        Some("forge") => (Some("forge".to_string()), Some("Forge".to_string())),
        Some("quilt") => (Some("quilt".to_string()), Some("Quilt".to_string())),
        Some("neoforge") => (Some("neoforge".to_string()), Some("NeoForge".to_string())),
        _ => (None, None),
    };

    let new_state = PresenceState {
        state,
        details,
        priority: ActivityPriority::Playing,
        large_image: Some("minecraft_logo".to_string()),
        large_text: Some(format!("Minecraft {}", version_id)),
        small_image,
        small_text,
        start_timestamp: Some(timestamp),
    };

    let mut manager = DISCORD_MANAGER
        .lock()
        .map_err(|e| format!("Failed to lock Discord manager: {}", e))?;
    manager.set_state(new_state)
}

/// Update presence for browsing a specific section
pub fn set_browsing(section: &str) -> Result<(), String> {
    let details = "In Launcher".to_string();
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

    let new_state = PresenceState {
        state: state.to_string(),
        details,
        priority: ActivityPriority::Browsing,
        large_image: Some("kable_logo".to_string()),
        large_text: Some("Kable Launcher".to_string()),
        small_image: None,
        small_text: None,
        start_timestamp: None,
    };

    let mut manager = DISCORD_MANAGER
        .lock()
        .map_err(|e| format!("Failed to lock Discord manager: {}", e))?;
    manager.set_state(new_state)
}

/// Clear playing status (reverts to last browsing state or idle)
pub fn clear_playing() -> Result<(), String> {
    let new_state = PresenceState {
        state: "In Launcher".to_string(),
        details: "Idle".to_string(),
        priority: ActivityPriority::Idle,
        large_image: Some("kable_logo".to_string()),
        large_text: Some("Kable Launcher".to_string()),
        small_image: None,
        small_text: None,
        start_timestamp: None,
    };

    let mut manager = DISCORD_MANAGER
        .lock()
        .map_err(|e| format!("Failed to lock Discord manager: {}", e))?;
    manager.current_state = new_state;
    manager.update_presence()
}

/// Clear all presence
pub fn clear() -> Result<(), String> {
    let mut manager = DISCORD_MANAGER
        .lock()
        .map_err(|e| format!("Failed to lock Discord manager: {}", e))?;
    manager.clear()
}

/// Disconnect from Discord
pub fn disconnect() -> Result<(), String> {
    let mut manager = DISCORD_MANAGER
        .lock()
        .map_err(|e| format!("Failed to lock Discord manager: {}", e))?;
    manager.disconnect()
}
