// launcher/trait.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::get_version;
use crate::installations::kable_profiles::KableInstallation;
use crate::settings::CategorizedLauncherSettings;
use crate::auth::MinecraftAccount;
use crate::versions::LoaderKind;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LaunchContext {
    pub installation: KableInstallation,
    pub settings: CategorizedLauncherSettings,
    pub account: MinecraftAccount,
    pub minecraft_dir: String,
    // Add more as needed (manifest, paths, etc.)
}

impl LaunchContext {
    pub fn new(
        installation: KableInstallation,
        settings: CategorizedLauncherSettings,
        account: MinecraftAccount,
        minecraft_dir: String,
    ) -> Result<Self, String> {
        Ok(Self {
            installation,
            settings,
            account,
            minecraft_dir,
        })
    }

    pub async fn detect_loader_type(&self) -> Result<LoaderType, String> {
        if let Some(version) = get_version(self.clone().installation.version_id).await {
            match version.loader {
                LoaderKind::Vanilla => Ok(LoaderType::Vanilla),
                LoaderKind::Fabric => Ok(LoaderType::Fabric),
                LoaderKind::IrisFabric => Ok(LoaderType::IrisFabric),
                LoaderKind::Quilt => Ok(LoaderType::Quilt),
                LoaderKind::Forge => Ok(LoaderType::Forge),
                LoaderKind::NeoForge => Ok(LoaderType::NeoForge)
            }   
        } else {
            Err("Failed to detect loader type".into())
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderType {
    Vanilla,
    Fabric,
    IrisFabric,
    Quilt,
    Forge,
    NeoForge,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LaunchResult {
    pub pid: u32,
    pub command: String,
}

#[async_trait]
pub trait Launchable: Send + Sync {
    async fn prepare(&self, context: &LaunchContext) -> Result<(), String>;
    async fn launch(&self, context: &LaunchContext) -> Result<LaunchResult, String>;
}
