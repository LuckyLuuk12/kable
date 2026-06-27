use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::system::symlinks as sys;
use api_types::symlinks::{Symlink, SymlinkCreateRequest};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SymlinkConfig {
    pub symlinks: Vec<Symlink>,
}

impl Default for SymlinkConfig {
    fn default() -> Self {
        Self { symlinks: vec![] }
    }
}

fn config_path() -> Result<PathBuf, String> {
    Ok(crate::system::fs::launcher_dir()?.join(crate::constants::CUSTOM_SYMLINKS_FILE))
}

async fn read_config() -> Result<SymlinkConfig, String> {
    let path = config_path()?;

    if !path.exists() {
        return Ok(SymlinkConfig::default());
    }

    let raw = tokio::fs::read_to_string(path).await.map_err(|e| e.to_string())?;

    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

async fn write_config(cfg: &SymlinkConfig) -> Result<(), String> {
    let path = config_path()?;

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }

    let raw = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;

    tokio::fs::write(path, raw).await.map_err(|e| e.to_string())
}

// ===== STATE =====

pub struct SymlinkFeature {
    config: SymlinkConfig,
}

impl SymlinkFeature {
    pub fn new() -> Self {
        Self { config: SymlinkConfig::default() }
    }

    /// REQUIRED ENTRY POINT
    pub async fn initialize(&mut self) -> Result<(), String> {
        self.config = read_config().await?;

        for link in &self.config.symlinks {
            let _ = sys::create(link).await;
        }

        Ok(())
    }

    pub async fn create(&mut self, req: SymlinkCreateRequest) -> Result<Symlink, String> {
        let file = req.source.file_name().ok_or("Invalid source")?;

        let link = Symlink { source: req.clone().source, destination: req.destination_parent.join(file) };

        sys::validate(&link)?;
        sys::create(&link).await?;

        self.config.symlinks.push(link.clone());
        write_config(&self.config).await?;

        Ok(link)
    }

    pub async fn remove(&mut self, link: Symlink) -> Result<(), String> {
        sys::remove(&link).await?;

        self.config.symlinks.retain(|s| s.destination != link.destination);
        write_config(&self.config).await
    }

    pub async fn toggle(&mut self, link: Symlink) -> Result<bool, String> {
        if sys::exists(&link).await? {
            sys::remove(&link).await?;
            return Ok(false);
        }

        sys::create(&link).await?;
        Ok(true)
    }
}

// ===== SINGLE GLOBAL INSTANCE =====

static FEATURE: Lazy<Mutex<SymlinkFeature>> = Lazy::new(|| Mutex::new(SymlinkFeature::new()));

fn with_feature<T>(f: impl FnOnce(&mut SymlinkFeature) -> T) -> Result<T, String> {
    let mut m = FEATURE.lock().map_err(|e| e.to_string())?;
    Ok(f(&mut m))
}

async fn with_feature_async<T, Fut>(f: impl FnOnce(&mut SymlinkFeature) -> Fut) -> Result<T, String>
where
    Fut: std::future::Future<Output = Result<T, String>>,
{
    let mut m = FEATURE.lock().map_err(|e| e.to_string())?;
    f(&mut m).await
}
