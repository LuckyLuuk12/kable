/// Specta is a library for generating TypeScript types from Rust types. This module is responsible for generating the TypeScript types for the API commands.
/// As well as any other types that are used in the API commands. This is done by using the `specta` crate to collect all the types that are used in the API
/// commands and then generating the TypeScript types for those types.
// use tauri_specta::ts;
use crate::api;
use specta_typescript::Typescript;
use tauri_specta::{collect_commands, collect_events, Builder};

pub fn generate() -> Result<(), Box<dyn std::error::Error>> {
    let builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            // TODO: make sure all commands are registered in lib.rs AND in typegen module
            // #region Accounts
            api::start_authentication,
            api::poll_authentication,
            api::list_accounts,
            api::add_account,
            api::remove_account,
            api::set_active_account,
            api::get_active_account,
            // #endregion Accounts
            // #region Discord
            api::initialize_discord_rpc,
            api::set_discord_enabled,
            api::set_discord_playing,
            api::set_discord_browsing,
            api::clear_discord_playing,
            api::clear_discord_presence,
            api::disconnect_discord,
            // #endregion Discord
            // #region Icons
            api::get_custom_icon_templates,
            api::save_custom_icon_template,
            api::delete_custom_icon_template,
            api::open_icons_directory,
            // #endregion Icons
            // #region Images
            api::resolve_image_path,
            // #endregion Images
            // #region Launcher
            api::launch_game,
            api::auto_detect_java,
            api::get_java_path,
            // #endregion Launcher
            // #region Projects
            api::browse,
            api::list_projects,
            api::remove_project,
            api::download_project,
            api::enable_project,
            api::disable_project,
            api::toggle_project,
            api::check_for_project_update,
            api::check_for_project_updates,
            api::update_project,
            api::update_all_projects,
            // #endregion Projects
            // #region Profiles
            api::get_profiles,
            api::get_profile,
            // api::create_profile,
            api::modify_profile,
            api::delete_profile,
            api::get_versions,
            // #endregion Profiles
            // #region Settings
            api::get_settings,
            api::set_settings,
            // #endregion Settings
            // #region Sounds
            api::list_soundpacks,
            api::get_soundpack_metadata,
            api::load_soundpack_file,
            api::import_soundpack_zip,
            api::get_sounds_directory_path,
            api::open_sounds_directory,
            // #endregion Sounds
            // #region Symlinks
            api::get_symlinks,
            api::temporary_symlinks,
            api::create,
            api::remove,
            api::toggle,
            api::update,
            // #endregion Symlinks
            // #region System
            api::open_url,
            api::open_path,
            // #endregion System
            // #region Updater
            api::check_for_updates,
            api::install_update,
            api::download_update,
            api::apply_downloaded_update,
            api::get_current_version,
            // #endregion Updater
        ])
        .events(collect_events![]);

    #[cfg(debug_assertions)]
    {
        builder.export(Typescript::default(), "../src/lib/api.ts").expect("failed to export TS");
    }

    Ok(())
}
