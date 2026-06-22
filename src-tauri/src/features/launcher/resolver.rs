// Here we take the "parsed Manifests" of a KableProfile's version field and ensure:
// 1. The libraries are downloaded
// 2. The assets are downloaded
// 3. The natives are extracted and ready to be used
// 4. We build the classpath (argument for the JVM)
// 5. our crate::system::java module can now find the java executable path
// 6. finally we make the Command using the manifest info, java path and profile info. App settings might also be used and we need to get the active account info to resolve some variables in the arguments.

use crate::integrations::minecraft::versions::{Manifests, VersionManifest};
use api_types::profiles::KableProfile;
use api_types::profiles::VersionType;
use tauri::Manager;
use tauri::Window;

fn get_primary_monitor_size() -> Option<(u32, u32)> {
    let monitor = crate::app_handle().primary_monitor().ok()??;
    let size = monitor.size();

    Some((size.width, size.height))
}

async fn ensure_libraries(profile: KableProfile) -> Result<(), String> {
    let manifest_contents = replace_variables(profile).await?;
    let manifests = crate::integrations::minecraft::versions::load_version_manifest(manifest_contents).await?;
    // If the version has a parent version (for loader != Vanilla), we need to ensure the libraries for the parent version as well.
    if let Some(parent_manifest) = manifests.parent_version {
        crate::integrations::minecraft::libraries::ensure_libraries(&parent_manifest.libraries).await?;
    }
    // Now we ensure the libraries for the version itself:
    match manifests.version {
        crate::integrations::minecraft::versions::VersionManifest::Vanilla(m) => {
            crate::integrations::minecraft::libraries::ensure_libraries(&m.libraries).await?
        }
        crate::integrations::minecraft::versions::VersionManifest::Fabric(m) => {
            crate::integrations::minecraft::libraries::ensure_libraries(&m.libraries).await?
        }
        crate::integrations::minecraft::versions::VersionManifest::IrisFabric(m) => {
            crate::integrations::minecraft::libraries::ensure_libraries(&m.libraries).await?
        }
        crate::integrations::minecraft::versions::VersionManifest::Forge(m) => {
            crate::integrations::minecraft::libraries::ensure_libraries(&m.libraries).await?
        }
        crate::integrations::minecraft::versions::VersionManifest::NeoForge(m) => {
            crate::integrations::minecraft::libraries::ensure_libraries(&m.libraries).await?
        }
        crate::integrations::minecraft::versions::VersionManifest::Quilt(m) => {
            crate::integrations::minecraft::libraries::ensure_libraries(&m.libraries).await?
        }
    };
    Ok(())
}

// Assets are only indexed in vanilla manifests but all loaders "should" have a parent pointing to a vanilla version so we can just use the vanilla asset index for all loaders.
async fn ensure_assets(profile: KableProfile) -> Result<(), String> {
    let manifest_contents = replace_variables(profile.clone()).await?;
    let manifests = crate::integrations::minecraft::versions::load_version_manifest(manifest_contents).await?;
    if let Some(parent_manifest) = manifests.parent_version {
        crate::integrations::minecraft::assets::ensure_assets(&parent_manifest.asset_index).await?;
    }
    match manifests.version {
        crate::integrations::minecraft::versions::VersionManifest::Vanilla(m) => {
            crate::integrations::minecraft::assets::ensure_assets(&m.asset_index).await?
        }
        _ => {
            // For non-vanilla loaders, we should have already ensured the assets from the parent version above.
        }
    };
    Ok(())
}

async fn ensure_natives(profile: KableProfile) -> Result<(), String> {
    // TODO: Somehow we need to get "some" .dll's from some of the libraries I guess?
    Ok(())
}

/// This should be called withing replace_variables as part of replacing is making the classpath but we assume all manifest variables besides classpath are replaced before this is called,
/// so we can just read the manifest contents from the .minecraft versions folder and replace the variables in it, then we can parse the manifest and build the classpath from it.
async fn build_classpath(profile: KableProfile, manifest_contents: String) -> Result<String, String> {
    let manifests = crate::integrations::minecraft::versions::load_version_manifest(manifest_contents).await?;
    let mut classpath_entries: Vec<String> = Vec::new();
    todo!()
}

/// Here we replace variables like ${clientId} with the actual values from profile, settings and global account state.
/// This will be a HUGE match on the variable names inside the ${} and mostly hardcoded
/// We accept any manifest (as Vanilla and neoforge seem to have variables) and attempt to replace them all
async fn replace_variables(profile: KableProfile) -> Result<String, String> {
    let mut loader_contents = crate::system::fs::read_str(
        crate::system::fs::mc_dir()?.join("versions").join(&profile.version.id).join(format!("{}.json", profile.version.id)),
    )
    .await?;
    // The manifest vanilla one contains the assets_index_name.
    let manifests = crate::integrations::minecraft::versions::load_version_manifest(loader_contents.clone()).await?;
    // we check if there is a parent or if it is a vanilla version, if so we get the vanilla (parent) manifest out of it:
    let version_manifest = match manifests.parent_version {
        Some(parent) => parent,
        None => match manifests.version {
            VersionManifest::Vanilla(v) => v,
            _ => return Err("No parent version found for non-vanilla loader".to_string()),
        },
    };

    // Get the active account from accounts module:
    let active_account = crate::features::accounts::management::get_active_account().await?.ok_or("No active account found")?;

    // Now we replace as many variable names as possible with regex on ${variable_name} and replace with the actual value from profile, settings and global account state.
    loader_contents = loader_contents.replace("${assets_index_name}", &version_manifest.asset_index.id);
    loader_contents = loader_contents
        .replace("${assets_root}", crate::system::fs::mc_dir()?.join(crate::constants::ASSETS_DIR).to_string_lossy().to_string().as_str());
    loader_contents = loader_contents.replace("${auth_access_token}", &active_account.access_token);
    loader_contents = loader_contents.replace("${auth_player_name}", &active_account.minecraft_profile.name);
    loader_contents = loader_contents.replace("${auth_uuid}", &active_account.minecraft_profile.id);
    loader_contents = loader_contents.replace("${auth_xuid}", &active_account.remote_id);
    loader_contents = loader_contents.replace("${clientId}", &active_account.minecraft_profile.id);
    loader_contents = loader_contents.replace("${game_directory}", crate::system::fs::mc_dir()?.to_string_lossy().to_string().as_str());
    loader_contents = loader_contents.replace("${user_type}", &active_account.account_type);
    loader_contents = loader_contents.replace("${version_name}", &profile.version.id);
    loader_contents =
        loader_contents.replace("${version_type}", &profile.version.version_type.clone().unwrap_or(VersionType::Release).to_string());
    // TODO: implement quick play
    loader_contents = loader_contents.replace("${quickPlayMultiplayer}", "false");
    loader_contents = loader_contents.replace("${quickPlayPath}", "");
    loader_contents = loader_contents.replace("${quickPlayRealms}", "false");
    loader_contents = loader_contents.replace("${quickPlaySingleplayer}", "false");
    loader_contents = loader_contents.replace("${resolution_height}", &get_primary_monitor_size().unwrap_or((800, 600)).1.to_string());
    loader_contents = loader_contents.replace("${resolution_width}", &get_primary_monitor_size().unwrap_or((800, 600)).0.to_string());
    loader_contents = loader_contents.replace("${launcher_name}", env!("CARGO_PKG_NAME"));
    loader_contents = loader_contents.replace("${launcher_version}", env!("CARGO_PKG_VERSION"));
    loader_contents = loader_contents.replace(
        "${natives_directory}",
        crate::system::fs::mc_dir()?.join(crate::constants::NATIVES_DIR).to_string_lossy().to_string().as_str(),
    );
    loader_contents = loader_contents
        .replace("${path}", crate::system::fs::mc_dir()?.join(crate::constants::LOGS_DIR).to_string_lossy().to_string().as_str());
    // ? REPLACE NEOFORGE VARIABLES
    loader_contents = loader_contents.replace("${classpath_separator}", if cfg!(target_os = "windows") { ";" } else { ":" });
    loader_contents = loader_contents.replace(
        "${library_directory}",
        crate::system::fs::mc_dir()?.join(crate::constants::LIBRARIES_DIR).to_string_lossy().to_string().as_str(),
    );
    loader_contents = loader_contents.replace("${version_name}", &profile.version.id);
    // ? Finally we need to replace the classpath variable with the actual classpath built from the manifest:
    loader_contents = loader_contents.replace("${classpath}", &build_classpath(profile, loader_contents.clone()).await?);
    Ok(loader_contents)
}

async fn build_command(manifests: Manifests) -> Result<std::process::Command, String> {
    // 1. use global general settings to get java path or find it with our crate::system::java module
    // 2. using manifest typed structure build the full command respecting rules (per OS), classpath separators, natives directory, etc.
    // 3. build the Command struct and return for resolve()
    todo!()
}

pub async fn resolve(profile: KableProfile) -> Result<std::process::Command, String> {
    ensure_libraries(profile.clone()).await?;
    ensure_assets(profile.clone()).await?;
    ensure_natives(profile.clone()).await?;
    let manifest_contents = replace_variables(profile.clone()).await?;
    let manifests = crate::integrations::minecraft::versions::load_version_manifest(manifest_contents).await?;
    // let java_path = crate::system::java::find_java_executable(None)?;
    // let mut cmd = std::process::Command::new(java_path);
    build_command(manifests).await
}

// ? Below I keep the output of `py .\scripts\version-manifest-variables.py` which is a list of all variables in the manifests:
/*
json path -> variable name
[VANILLA]
arguments.game           ->  assets_index_name
arguments.game           ->  assets_root
arguments.game           ->  auth_access_token
arguments.game           ->  auth_player_name
arguments.game           ->  auth_uuid
arguments.game           ->  auth_xuid
arguments.game           ->  clientid
arguments.game           ->  game_directory
arguments.game           ->  user_type
arguments.game           ->  version_name
arguments.game           ->  version_type
arguments.game.value     ->  quickPlayMultiplayer
arguments.game.value     ->  quickPlayPath
arguments.game.value     ->  quickPlayRealms
arguments.game.value     ->  quickPlaySingleplayer
arguments.game.value     ->  resolution_height
arguments.game.value     ->  resolution_width
arguments.jvm            ->  classpath
arguments.jvm            ->  launcher_name
arguments.jvm            ->  launcher_version
arguments.jvm            ->  natives_directory
logging.client.argument  ->  path

[NEOFORGE]
arguments.jvm  ->  classpath_separator
arguments.jvm  ->  library_directory
arguments.jvm  ->  version_name
*/
