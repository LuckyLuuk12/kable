use discord_rich_presence::{activity::Activity, DiscordIpc, DiscordIpcClient};

pub struct DiscordClient {
    client: Option<DiscordIpcClient>,
    app_id: &'static str,
}

impl DiscordClient {
    pub fn new(app_id: &'static str) -> Self {
        Self { client: None, app_id }
    }

    pub fn connect(&mut self) -> Result<(), String> {
        if self.client.is_some() {
            return Ok(());
        }

        let mut client = DiscordIpcClient::new(self.app_id);

        client.connect().map_err(|e| format!("Failed to connect to Discord: {e}"))?;

        self.client = Some(client);
        Ok(())
    }

    pub fn set_activity(&mut self, activity: Activity) -> Result<(), String> {
        let client = self.ensure_connected()?;

        client.set_activity(activity).map_err(|e| format!("Failed to set Discord activity: {e}"))?;

        Ok(())
    }

    pub fn clear_activity(&mut self) -> Result<(), String> {
        let client = self.ensure_connected()?;

        client.clear_activity().map_err(|e| format!("Failed to clear Discord activity: {e}"))?;

        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<(), String> {
        if let Some(mut client) = self.client.take() {
            client.close().map_err(|e| format!("Failed to close Discord connection: {e}"))?;
        }

        Ok(())
    }

    fn ensure_connected(&mut self) -> Result<&mut DiscordIpcClient, String> {
        if self.client.is_none() {
            self.connect()?;
        }

        Ok(self.client.as_mut().unwrap())
    }
}
