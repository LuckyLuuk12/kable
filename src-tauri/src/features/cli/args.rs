// here we parse the command line arguments and run match-based whatever command the user has given us
// for now we only support running a "profile" (which refreshes auth for default account and launches the game)
use clap::Parser;

use crate::features::accounts::management::{get_active_account, list_accounts};
use crate::features::customization::settings::load_settings;
use crate::features::launcher::launch::launch_game;
use crate::features::profiles::{kable_profile::load_profiles, management::get_profile};
use crate::integrations::loaders::get_versions;

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
    // 1. First, we load app settings
    let _settings = load_settings().await?;
    // 2. Second, we load all profiles
    let _profiles = load_profiles().await?;
    // 3. Then we refresh auth for default account if possible
    let _accounts = list_accounts().await?;
    // 4. Then in async we can also fetch/load version info from loader manifests, once caching has been implemented this should improve initial loading time of the launcher
    let _versions = get_versions().await?;

    // 5. Now we check for args, if any present that prevent UI this here should end process as well
    if let Some(profile_id) = args.launch_profile {
        // Find profile by id, launch with loaded settings and active account.
        let profile = get_profile(&profile_id).await?;
        // let account = get_active_account().await?.expect("No active account found!");
        launch_game(profile).await?;
        std::process::exit(0); // Exit after launching the game, no UI needed
    }

    // 5. If no args we first load customizations if needed.

    // 6. If we reached this point we assume the lib.rs run() will continue and open UI.
    Ok(())
}
