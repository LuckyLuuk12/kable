use crate::{
    features::launcher::{resolver, runtime_inject},
    Logger,
};
use api_types::{launcher::LaunchResult, profiles::KableProfile};
use std::process::Command;
use std::process::Stdio;
use tokio::process::Command as TokioCommand;

pub async fn launch_game(
    profile: KableProfile,
    // _settings: &api_types::settings::CategorizedLauncherSettings,
) -> Result<LaunchResult, String> {
    // 1. Resolve and prepare the command
    let cmd = resolver::resolve(profile.clone()).await?;
    log_command(&cmd, &profile);
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
    let _ = crate::features::profiles::management::update_last_used(profile.clone()).await?;
    Logger::debug_global(format!("Game launched with PID: {}, Runtime ID: {}", pid, runtime_id).as_str(), Some(profile.id.as_str()));
    Ok(LaunchResult { pid, runtime_id: runtime_id.to_string(), command: format!("{:?}", cmd) })
}

fn log_command(cmd: &Command, profile: &KableProfile) {
    let program = cmd.get_program().to_string_lossy();

    let mut redact_next = false;

    let args = cmd
        .get_args()
        .map(|arg| {
            let arg = arg.to_string_lossy();

            if redact_next {
                redact_next = false;
                return "<redacted>".to_string();
            }

            if matches!(arg.as_ref(), "--accessToken" | "--clientId" | "--xuid") {
                redact_next = true;
            }

            if arg.contains(' ') || arg.contains(';') {
                format!("{arg:?}")
            } else {
                arg.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    Logger::debug_global(format!("Launching game: {program:?} {args}").as_str(), Some(profile.id.as_str()));
}
