use crate::{features::launcher::resolver, features::launcher::runtime_inject};
use api_types::{launcher::LaunchResult, profiles::KableProfile};
use std::process::Stdio;
use tokio::process::Command as TokioCommand;

pub async fn launch_game(
    profile: KableProfile,
    // _settings: &api_types::settings::CategorizedLauncherSettings,
) -> Result<LaunchResult, String> {
    // 1. Resolve and prepare the command
    let cmd = resolver::resolve(profile.clone()).await?;
    // 3. Spawn process and track it with runtime injection
    let mut tokio_cmd = TokioCommand::new(cmd.get_program());
    tokio_cmd.args(cmd.get_args());
    tokio_cmd.current_dir(crate::system::fs::mc_dir()?);

    #[cfg(target_os = "windows")]
    {
        tokio_cmd.creation_flags(0x08000000);
    }
    tokio_cmd.stdout(Stdio::piped());
    tokio_cmd.stderr(Stdio::piped());

    let child = tokio_cmd.spawn().map_err(|e| format!("Failed to launch: {e}"))?;
    let pid = child.id().unwrap_or(0);

    crate::system::processes::track_process(pid);
    let (runtime_id, runtime, pipeline) = runtime_inject::handle_injections(profile.clone(), child);
    runtime.run(pipeline).await?;

    // Update last used timestamp for the profile
    let _ = crate::features::profiles::management::update_last_used(profile).await?;

    Ok(LaunchResult { pid, runtime_id: runtime_id.to_string(), command: format!("{:?}", cmd) })
}
