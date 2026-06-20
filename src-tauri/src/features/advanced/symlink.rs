use crate::Logger;
use api_types::symlinks::{SymlinkEntry, SymlinkState, SymlinkView};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkManager {
    entries: Vec<SymlinkEntry>,
}

impl SymlinkManager {
    pub async fn load() -> Result<Self, String> {
        let config_path = crate::system::fs::kable_dir().unwrap().join("symlinks.json");

        if !config_path.exists() {
            return Ok(Self { entries: Vec::new() });
        }

        let data = tokio::fs::read(&config_path).await.map_err(|e| e.to_string())?;
        let entries: Vec<SymlinkEntry> = serde_json::from_slice(&data).map_err(|e| e.to_string())?;

        Ok(Self { entries })
    }

    pub async fn save(&self) -> Result<(), String> {
        let config_path = crate::system::fs::kable_dir().unwrap().join("symlinks.json");
        let data = serde_json::to_vec(&self.entries).map_err(|e| e.to_string())?;
        tokio::fs::write(&config_path, &data).await.map_err(|e| e.to_string())
    }

    pub async fn list(&self) -> Result<Vec<SymlinkView>, String> {
        let mut views = Vec::new();
        for entry in &self.entries {
            let mut view: SymlinkView = entry.clone().into();
            // Compute state, kind, source_exists & link_exists using system::symlinks functions and the entry data:
            let state = state(entry).await?;
            let kind =
                if entry.source.is_dir() { api_types::symlinks::SymlinkKind::Directory } else { api_types::symlinks::SymlinkKind::File };
            let source_exists = crate::system::fs::exists(&entry.source).await?;
            let link_exists = crate::system::symlinks::is_symlink(&entry.destination).await?;
            // Apply the fields that had to be calculated outside of the api_types crate:
            view.state = state;
            view.kind = kind;
            view.source_exists = source_exists;
            view.link_exists = link_exists;
            views.push(view);
        }
        Ok(views)
    }

    pub async fn create(&mut self, entry: SymlinkEntry) -> Result<(), String> {
        self.entries.push(entry);
        Ok(())
    }

    pub async fn remove(&mut self, id: &str) -> Result<(), String> {
        self.entries.retain(|e| e.id != id);
        Ok(())
    }

    pub async fn enable(&mut self, id: &str) -> Result<(), String> {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.enabled = true;
        }
        Ok(())
    }

    pub async fn disable(&mut self, id: &str) -> Result<(), String> {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.enabled = false;
        }
        Ok(())
    }

    pub async fn apply(&self, installation_id: Option<&str>) -> Result<(), String> {
        for entry in &self.entries {
            apply_entry(entry).await?;
        }
        Ok(())
    }

    pub async fn cleanup(&self, installation_id: Option<&str>) -> Result<(), String> {
        for entry in &self.entries {
            if let Err(e) = crate::system::symlinks::remove_link(&entry.destination).await {
                Logger::warn_global(&format!("Failed to cleanup symlink {}: {}", entry.id, e), None);
            }
        }
        Ok(())
    }
}

pub async fn state(entry: &SymlinkEntry) -> Result<SymlinkState, String> {
    if !entry.enabled {
        return Ok(SymlinkState::Disabled);
    }

    let source_exists = tokio::fs::try_exists(&entry.source).await.map_err(|e| e.to_string())?;

    if !source_exists {
        return Ok(SymlinkState::MissingTarget);
    }

    let link_exists = crate::system::symlinks::is_symlink(&entry.destination).await?;

    if !link_exists {
        return Ok(SymlinkState::MissingLink);
    }

    Ok(SymlinkState::Enabled)
}

pub async fn apply_entry(entry: &SymlinkEntry) -> Result<(), String> {
    if !entry.enabled {
        return Ok(());
    }

    if !tokio::fs::try_exists(&entry.source).await.map_err(|e| e.to_string())? {
        return Ok(());
    }

    if entry.source.is_dir() {
        crate::system::symlinks::link_dir(&entry.source, &entry.destination).await?;
    } else {
        crate::system::symlinks::link_file(&entry.source, &entry.destination).await?;
    }

    Ok(())
}
