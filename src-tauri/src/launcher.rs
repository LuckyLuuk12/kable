pub mod fabric;
pub mod forge;
pub mod launchables;
pub mod utils;
pub mod vanilla;

pub use fabric::FabricLaunchable;
pub use forge::ForgeLaunchable;
pub use launchables::{LaunchContext, LaunchResult, Launchable, LoaderType};
pub use vanilla::VanillaLaunchable;

use crate::logging::Logger;
use crate::LauncherAccount;
use crate::{
    get_default_minecraft_dir, kable_profiles::KableInstallation, CategorizedLauncherSettings,
};

use once_cell::sync::OnceCell;
use std::collections::HashSet;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;
use tauri::async_runtime::spawn_blocking;

static MINECRAFT_PIDS: OnceCell<Mutex<HashSet<u32>>> = OnceCell::new();

fn get_pid_set() -> &'static Mutex<HashSet<u32>> {
    MINECRAFT_PIDS.get_or_init(|| Mutex::new(HashSet::new()))
}

async fn get_launchable_for_installation(
    context: &LaunchContext,
) -> Result<Box<dyn Launchable>, String> {
    // Detect loader type from context.installation or context.manifest
    match context.detect_loader_type().await? {
        LoaderType::Vanilla => Ok(Box::new(VanillaLaunchable)),
        LoaderType::Fabric => Ok(Box::new(FabricLaunchable)),
        LoaderType::IrisFabric => Ok(Box::new(FabricLaunchable)), // Iris is a Fabric mod but has its own loader which is identical to Fabric
        LoaderType::Quilt => Ok(Box::new(FabricLaunchable)),      // Quilt is a fork of Fabric
        LoaderType::Forge => Ok(Box::new(ForgeLaunchable)),       // Forge can be
        LoaderType::NeoForge => Ok(Box::new(ForgeLaunchable)),    // NeoForge is a fork of Forge
                                                                   // Add more as needed
    }
}

#[tauri::command]
pub async fn launch_installation(
    installation: KableInstallation,
    settings: CategorizedLauncherSettings,
    account: LauncherAccount,
) -> Result<LaunchResult, String> {
    // Use version_id for log grouping
    let version_id = installation.version_id.as_str();
    let instance_id = Some(version_id);
    Logger::info_global(
        &format!("Launching installation: {}", installation.name),
        None,
    );
    // Validate installation
    let minecraft_dir = match get_default_minecraft_dir() {
        Ok(dir) => dir.to_string_lossy().to_string(),
        Err(e) => {
            Logger::error_global(
                &format!("Failed to get default Minecraft dir: {}", e),
                instance_id,
            );
            return Err(format!("Failed to get default Minecraft dir: {}", e));
        }
    };
    // Build context
    let context = match LaunchContext::new(installation.clone(), settings, account, minecraft_dir) {
        Ok(ctx) => ctx,
        Err(e) => {
            Logger::error_global(
                &format!("Failed to build launch context: {}", e),
                instance_id,
            );
            return Err(format!("Failed to build launch context: {}", e));
        }
    };
    // Detect loader and get Launchable
    let launchable = match get_launchable_for_installation(&context).await {
        Ok(l) => l,
        Err(e) => {
            Logger::error_global(&format!("Failed to detect loader: {}", e), instance_id);
            return Err(format!("Failed to detect loader: {}", e));
        }
    };
    // Prepare (download, patch, etc.)
    if let Err(e) = launchable.prepare(&context).await {
        Logger::error_global(&format!("Failed to prepare launch: {}", e), instance_id);
        return Err(format!("Failed to prepare launch: {}", e));
    }
    // Build and run the launch command
    let result = match launchable.launch(&context).await {
        Ok(res) => {
            Logger::info_global(
                &format!("Minecraft launched successfully (PID: {})", res.pid),
                None,
            );
            res
        }
        Err(e) => {
            Logger::error_global(&format!("Failed to launch Minecraft: {}", e), None);
            return Err(format!("Failed to launch Minecraft: {}", e));
        }
    };
    // Track the launched PID
    let mut pids = get_pid_set().lock().unwrap();
    pids.insert(result.pid);
    Ok(result)
}

/// Kill a Minecraft process by PID (only if tracked)
#[tauri::command]
pub async fn kill_minecraft_process(process_id: u32) -> Result<(), String> {
    let mut pids = get_pid_set().lock().unwrap();
    if !pids.contains(&process_id) {
        return Err(format!(
            "Process {} is not tracked by the launcher",
            process_id
        ));
    }
    // Try to kill the process
    match Command::new("taskkill")
        .args(["/PID", &process_id.to_string(), "/F"])
        .status()
    {
        Ok(status) if status.success() => {
            pids.remove(&process_id);
            Ok(())
        }
        _ => Err(format!("Failed to kill process {}", process_id)),
    }
}

/// Get all running Minecraft process IDs (tracked by launcher)
#[tauri::command]
pub async fn get_running_minecraft_processes() -> Result<Vec<u32>, String> {
    let pids = get_pid_set().lock().unwrap();
    // Optionally, check if the process is still alive
    let mut running = Vec::new();
    for &pid in pids.iter() {
        if is_process_alive(pid) {
            running.push(pid);
        }
    }
    Ok(running)
}

fn is_process_alive(pid: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
        };
        const STILL_ACTIVE: u32 = 259;
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle.is_null() {
                return false;
            }
            let mut code = 0u32;
            let ok = GetExitCodeProcess(handle, &mut code as *mut u32);
            CloseHandle(handle);
            ok != 0 && code == STILL_ACTIVE
        }
    }
    #[cfg(unix)]
    {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
}

/// Check if any Minecraft process is running (tracked by launcher)
#[tauri::command]
pub async fn is_minecraft_running() -> Result<bool, String> {
    let running = get_running_minecraft_processes().await?;
    Ok(!running.is_empty())
}

/// Wait for a Minecraft process to exit (tracked by launcher)
#[tauri::command]
pub async fn wait_for_minecraft_exit(process_id: u32) -> Result<(), String> {
    let mut found = false;
    {
        let pids = get_pid_set().lock().unwrap();
        if pids.contains(&process_id) {
            found = true;
        }
    }
    if !found {
        return Err(format!(
            "Process {} is not tracked by the launcher",
            process_id
        ));
    }
    spawn_blocking(move || {
        while is_process_alive(process_id) {
            std::thread::sleep(Duration::from_millis(500));
        }
        // Remove from tracked set after exit
        let mut pids = get_pid_set().lock().unwrap();
        pids.remove(&process_id);
        Ok(())
    })
    .await
    .unwrap()
}
