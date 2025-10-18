// launcher/trait.rs

use crate::auth::LauncherAccount;
use crate::installations::get_version;
use crate::installations::kable_profiles::KableInstallation;
use crate::settings::CategorizedLauncherSettings;
use crate::versions::LoaderKind;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LaunchContext {
    pub installation: KableInstallation,
    pub settings: CategorizedLauncherSettings,
    pub account: LauncherAccount,
    pub minecraft_dir: String,
    // Add more as needed (manifest, paths, etc.)
}

impl LaunchContext {
    pub fn new(
        installation: KableInstallation,
        settings: CategorizedLauncherSettings,
        account: LauncherAccount,
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
        let mut version_id = self.clone().installation.version_id;
        // Trim whitespace/newlines that might be present from conversion or user data
        version_id = version_id.trim().to_string();
        // Diagnostic logging to help understand packaged vs dev behavior
        crate::logging::Logger::debug_global(
            &format!("detect_loader_type: version_id='{}'", version_id),
            None,
        );
        if let Some(version) = get_version(version_id.clone()).await {
            crate::logging::Logger::debug_global(
                &format!(
                    "detect_loader_type: found version entry for '{}'",
                    version_id
                ),
                None,
            );
            match version.loader {
                LoaderKind::Vanilla => Ok(LoaderType::Vanilla),
                LoaderKind::Fabric => Ok(LoaderType::Fabric),
                LoaderKind::IrisFabric => Ok(LoaderType::IrisFabric), // Iris is a Fabric mod but has its own loader which is identical to Fabric
                LoaderKind::Quilt => Ok(LoaderType::Quilt),           // Quilt is a fork of Fabric
                LoaderKind::Forge => Ok(LoaderType::Forge),
                LoaderKind::NeoForge => Ok(LoaderType::NeoForge),
            }
        } else {
            // Some installation version IDs are special placeholders like "latest-release" or
            // "latest-snapshot" which don't exist in the versions list; treat these as
            // Vanilla by default instead of failing detection.
            crate::logging::Logger::debug_global(
                &format!(
                    "detect_loader_type: no version entry for '{}', checking fallback",
                    version_id
                ),
                None,
            );
            if version_id.starts_with("latest")
                || version_id == "latest-release"
                || version_id == "latest-snapshot"
            {
                Ok(LoaderType::Vanilla)
            } else {
                Err("Failed to detect loader type".into())
            }
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
