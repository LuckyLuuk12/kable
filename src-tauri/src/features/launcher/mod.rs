//! Launcher feature
use api_types::auth::LauncherAccount;
use api_types::launcher::LaunchResult;
use api_types::profiles::KableProfile;
use api_types::settings::CategorizedLauncherSettings;
use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::process::Command;
use std::sync::Mutex;

pub mod assets;
pub mod fabric;
pub mod forge;
pub mod iris_fabric;
pub mod neoforge;
pub mod quilt;
pub mod utils;
pub mod vanilla;

static TRACKED_PIDS: Lazy<Mutex<HashSet<u32>>> = Lazy::new(|| Mutex::new(HashSet::new()));

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LaunchContext {
    pub installation: KableProfile,
    pub settings: CategorizedLauncherSettings,
    pub account: LauncherAccount,
    pub minecraft_dir: String,
}

#[async_trait]
pub trait Launchable: Send + Sync {
    async fn prepare(&self, context: &LaunchContext) -> Result<(), String>;
    async fn launch(&self, context: &LaunchContext) -> Result<LaunchResult, String>;
}

pub fn get_launchable_for_installation(
    installation: &KableProfile,
) -> Result<Box<dyn Launchable>, String> {
    if installation.version_id.contains("fabric") {
        Ok(Box::new(fabric::FabricLaunchable::default()))
    } else if installation.version_id.contains("forge") {
        Ok(Box::new(forge::ForgeLaunchable::default()))
    } else {
        Ok(Box::new(vanilla::VanillaLaunchable::default()))
    }
}

pub async fn launch_installation(
    profile: KableProfile,
    settings: CategorizedLauncherSettings,
    account: LauncherAccount,
) -> Result<LaunchResult, String> {
    let minecraft_dir = crate::system::fs::get_default_minecraft_dir()?
        .to_string_lossy()
        .to_string();

    let context = LaunchContext {
        installation: profile,
        settings,
        account,
        minecraft_dir,
    };

    let launchable = get_launchable_for_installation(&context.installation)?;
    launchable.prepare(&context).await?;
    let result = launchable.launch(&context).await?;

    crate::system::processes::track_process(result.pid);

    Ok(result)
}

pub async fn kill_minecraft_process(process_id: u32) -> Result<(), String> {
    crate::system::processes::kill_process(process_id).await
}

pub async fn get_running_minecraft_processes() -> Result<Vec<u32>, String> {
    let pids = crate::system::processes::get_tracked_pids();
    let mut running = Vec::new();
    for pid in pids {
        if crate::system::processes::is_process_alive(pid) {
            running.push(pid);
        }
    }
    Ok(running)
}

pub async fn is_minecraft_running() -> Result<bool, String> {
    let running = get_running_minecraft_processes().await?;
    Ok(!running.is_empty())
}

pub async fn wait_for_minecraft_exit(process_id: u32) -> Result<i32, String> {
    if !crate::system::processes::is_tracked(process_id) {
        return Err(format!("Process {} is not tracked", process_id));
    }

    tokio::task::spawn_blocking(move || {
        while crate::system::processes::is_process_alive(process_id) {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        let exit_code = crate::system::processes::get_process_exit_code(process_id).unwrap_or(-1);
        crate::system::processes::untrack_process(process_id);
        exit_code
    })
    .await
    .map_err(|e| format!("Wait task failed: {}", e))
}
