// here we parse the command line arguments and run match-based whatever command the user has given us
// for now we only support running a "profile" (which refreshes auth for default account and launches the game)
use clap::Parser;

use crate::features::accounts::management::refresh_accounts;
use crate::features::customization::settings::load_settings;
use crate::features::launcher::launch::launch_game;
use crate::features::profiles::management::get_profile;
use crate::features::profiles::minimalize::cleanup_orphaned_resources;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long)]
    pub launch_profile: Option<String>,
}

pub fn parse_args() -> Args {
    Args::parse()
}

/**
 * This is called in lib.rs run() before any UI is initialized, we should preload settings and required data here and
 * based on args maybe skip UI, or work as a CLI launcher.
 */
pub async fn handle_args(args: Args) -> Result<(), String> {
    crate::system::cache::initialize(crate::system::fs::cache_dir()?).await?;

    // 1. Load app settings
    let _settings = load_settings().await?;

    // 2. Refresh authentication
    refresh_accounts().await?;

    // 3. Handle launch arguments
    if let Some(profile_id) = args.launch_profile {
        let profile = get_profile(&profile_id).await?;
        launch_game(profile).await?;
        std::process::exit(0);
    }

    // 4. Schedule cleanup in the background
    tokio::spawn(async {
        if let Err(error) = cleanup_orphaned_resources().await {
            print!("[STARTUP] Cleanup of orphaned resources failed: {error}");
        }
    });

    // 5. Continue startup without waiting for cleanup
    Ok(())
}
