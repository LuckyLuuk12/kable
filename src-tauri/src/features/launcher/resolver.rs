// // Here we take the "parsed Manifests" of a KableProfile's version field and ensure:
// // 1. The libraries are downloaded
// // 2. The assets are downloaded
// // 3. The natives are extracted and ready to be used
// // 4. We build the classpath (argument for the JVM)
// // 5. our crate::system::java module can now find the java executable path
// // 6. finally we make the Command using the manifest info, java path and profile info. App settings might also be used and we need to get the active account info to resolve some variables in the arguments.

// use crate::integrations::minecraft::versions::{Manifests, VersionManifest};
// use api_types::profiles::KableProfile;
// use api_types::profiles::VersionType;
// use tauri::Manager;
// use tauri::Window;

// fn get_primary_monitor_size() -> Option<(u32, u32)> {
//     let monitor = crate::app_handle().primary_monitor().ok()??;
//     let size = monitor.size();

//     Some((size.width, size.height))
// }

// /// Here we replace variables like ${clientId} with the actual values from profile, settings and global account state.
// /// This will be a HUGE match on the variable names inside the ${} and mostly hardcoded
// /// We accept any manifest (as Vanilla and neoforge seem to have variables) and attempt to replace them all
// async fn replace_variables(profile: KableProfile) -> Result<String, String> {
//     let mut loader_contents = crate::system::fs::read_str(
//         crate::system::fs::mc_dir()?.join("versions").join(&profile.version.id).join(format!("{}.json", profile.version.id)),
//     )
//     .await?;
//     // The now we need to build classpath and also extract the asset index id to replace those variables:

//     // Get the active account from accounts module:
//     let active_account = crate::features::accounts::management::get_active_account().await?.ok_or("No active account found")?;

//     // Now we replace as many variable names as possible with regex on ${variable_name} and replace with the actual value from profile, settings and global account state.
//     loader_contents = loader_contents.replace("${assets_index_name}", asset_index_id);
//     loader_contents = loader_contents
//         .replace("${assets_root}", crate::system::fs::mc_dir()?.join(crate::constants::ASSETS_DIR).to_string_lossy().to_string().as_str());
//     loader_contents = loader_contents.replace("${auth_access_token}", &active_account.access_token);
//     loader_contents = loader_contents.replace("${auth_player_name}", &active_account.minecraft_profile.name);
//     loader_contents = loader_contents.replace("${auth_uuid}", &active_account.minecraft_profile.id);
//     loader_contents = loader_contents.replace("${auth_xuid}", &active_account.remote_id);
//     loader_contents = loader_contents.replace("${clientId}", &active_account.minecraft_profile.id);
//     loader_contents = loader_contents.replace("${game_directory}", crate::system::fs::mc_dir()?.to_string_lossy().to_string().as_str());
//     loader_contents = loader_contents.replace("${user_type}", &active_account.account_type);
//     loader_contents = loader_contents.replace("${version_name}", &profile.version.id);
//     loader_contents =
//         loader_contents.replace("${version_type}", &profile.version.version_type.clone().unwrap_or(VersionType::Release).to_string());
//     // TODO: implement quick play
//     loader_contents = loader_contents.replace("${quickPlayMultiplayer}", "false");
//     loader_contents = loader_contents.replace("${quickPlayPath}", "");
//     loader_contents = loader_contents.replace("${quickPlayRealms}", "false");
//     loader_contents = loader_contents.replace("${quickPlaySingleplayer}", "false");
//     loader_contents = loader_contents.replace("${resolution_height}", &get_primary_monitor_size().unwrap_or((800, 600)).1.to_string());
//     loader_contents = loader_contents.replace("${resolution_width}", &get_primary_monitor_size().unwrap_or((800, 600)).0.to_string());
//     loader_contents = loader_contents.replace("${launcher_name}", env!("CARGO_PKG_NAME"));
//     loader_contents = loader_contents.replace("${launcher_version}", env!("CARGO_PKG_VERSION"));
//     loader_contents = loader_contents.replace(
//         "${natives_directory}",
//         crate::system::fs::mc_dir()?.join(crate::constants::NATIVES_DIR).to_string_lossy().to_string().as_str(),
//     );
//     loader_contents = loader_contents
//         .replace("${path}", crate::system::fs::mc_dir()?.join(crate::constants::LOGS_DIR).to_string_lossy().to_string().as_str());
//     // ? REPLACE NEOFORGE VARIABLES
//     loader_contents = loader_contents.replace("${classpath_separator}", if cfg!(target_os = "windows") { ";" } else { ":" });
//     loader_contents = loader_contents.replace(
//         "${library_directory}",
//         crate::system::fs::mc_dir()?.join(crate::constants::LIBRARIES_DIR).to_string_lossy().to_string().as_str(),
//     );
//     loader_contents = loader_contents.replace("${version_name}", &profile.version.id);
//     // ? Finally we need to replace the classpath variable with the actual classpath built from the manifest:
//     loader_contents = loader_contents.replace("${classpath}", classpath);
//     Ok(loader_contents)
// }

// async fn build_command(manifests: Manifests) -> Result<std::process::Command, String> {
//     // 1. use global general settings to get java path or find it with our crate::system::java module
//     // 2. using manifest typed structure build the full command respecting rules (per OS), classpath separators, natives directory, etc.
//     // 3. build the Command struct and return for resolve()
//     todo!()
// }

// pub async fn resolve(profile: KableProfile) -> Result<std::process::Command, String> {
//     ensure_libraries(profile.clone()).await?;
//     ensure_assets(profile.clone()).await?;
//     ensure_natives(profile.clone()).await?;
//     let manifest_contents = replace_variables(profile.clone()).await?;
//     let manifests = crate::integrations::minecraft::versions::load_version_manifest(manifest_contents).await?;
//     // let java_path = crate::system::java::find_java_executable(None)?;
//     // let mut cmd = std::process::Command::new(java_path);
//     build_command(manifests).await
// }

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
use crate::integrations::minecraft::versions::McVersionManifest;
use crate::integrations::minecraft::{assets::AssetResolver, libraries::LibraryResolver, natives::NativeResolver};
use api_types::profiles::{KableProfile, ProfileVersionType};
use std::process::Command;

fn classpath_sep() -> &'static str {
    if cfg!(windows) {
        ";"
    } else {
        ":"
    }
}

async fn replace_variables(profile: &KableProfile, manifest: &McVersionManifest, classpath: &str) -> Result<String, String> {
    let mc_root = crate::system::fs::mc_dir()?;
    let assets_root = mc_root.join(crate::constants::ASSETS_DIR);
    let libs_root = mc_root.join(crate::constants::LIBRARIES_DIR);
    let natives_root = mc_root.join(crate::constants::NATIVES_DIR);

    let active_account = crate::features::accounts::management::get_active_account().await?;

    let monitor = crate::app_handle().primary_monitor().ok().flatten();
    let (width, height) = monitor
        .map(|m| {
            let s = m.size();
            (s.width, s.height)
        })
        .unwrap_or((800, 600));

    let asset_index = manifest.asset_index.as_ref().and_then(|a| a.id.as_deref()).unwrap_or("unknown");

    let mut result = String::new();

    // we assume this is the "base command string" passed in externally
    result.push_str(classpath);

    // ----------------------------
    // vanilla variables
    // ----------------------------
    result = result.replace("${assets_index_name}", asset_index);
    result = result.replace("${assets_root}", &assets_root.to_string_lossy());

    result = result.replace("${auth_access_token}", &active_account.access_token);
    result = result.replace("${auth_player_name}", &active_account.minecraft_profile.name);
    result = result.replace("${auth_uuid}", &active_account.minecraft_profile.id);
    result = result.replace("${auth_xuid}", &active_account.remote_id);
    result = result.replace("${clientId}", &active_account.minecraft_profile.id);

    result = result.replace("${game_directory}", &mc_root.to_string_lossy());
    result = result.replace("${user_type}", &active_account.account_type);

    result = result.replace("${version_name}", &profile.version.id);
    result = result.replace("${version_type}", &profile.version.version_type.clone().unwrap_or(ProfileVersionType::Release).to_string());

    result = result.replace("${launcher_name}", env!("CARGO_PKG_NAME"));
    result = result.replace("${launcher_version}", env!("CARGO_PKG_VERSION"));

    result = result.replace("${natives_directory}", &natives_root.to_string_lossy());
    result = result.replace("${library_directory}", &libs_root.to_string_lossy());

    result = result.replace("${classpath_separator}", classpath_sep());

    result = result.replace("${classpath}", classpath);

    // quick play (optional, ignored for now)
    result = result.replace("${quickPlayMultiplayer}", "false");
    result = result.replace("${quickPlaySingleplayer}", "false");
    result = result.replace("${quickPlayRealms}", "false");
    result = result.replace("${quickPlayPath}", "");

    // resolution
    result = result.replace("${resolution_width}", &width.to_string());
    result = result.replace("${resolution_height}", &height.to_string());

    Ok(result)
}

async fn build_command(manifest: &McVersionManifest, profile: &KableProfile) -> Result<Command, String> {
    let asset_resolver = AssetResolver::new()?;
    let lib_resolver = LibraryResolver::new()?;
    let native_resolver = NativeResolver::new()?;

    let libs = manifest.libraries.as_deref().unwrap_or(&[]);

    asset_resolver.ensure_assets(manifest).await?;
    lib_resolver.ensure_downloaded(libs).await?;
    native_resolver.ensure_natives(libs, profile.version.id.as_str()).await?;

    let classpath = lib_resolver.resolve_classpath(libs)?;

    let command_string = replace_variables(profile, manifest, &classpath).await?;

    let java_path = crate::system::java::find_java_executable(
        crate::features::customization::settings::load_settings().await?.general.java_path.as_ref(),
    )?;

    let mut cmd = Command::new(java_path);
    // TODO: make function to figure out what (game) jar to use based on version manifest, from mc26 onwards mod loader jars are 0KB and we should run the parent jar instead...

    for arg in command_string.split_whitespace() {
        cmd.arg(arg);
    }

    Ok(cmd)
}

pub async fn resolve(profile: KableProfile) -> Result<Command, String> {
    let manifest_path = crate::system::fs::mc_dir()?
        .join(crate::constants::VERSIONS_DIR)
        .join(&profile.version.id)
        .join(format!("{}.json", profile.version.id));

    let raw = crate::system::fs::read_str(manifest_path).await?;

    let manifest: McVersionManifest = crate::integrations::minecraft::versions::load_version_manifest(raw).await?;

    let resolved = crate::integrations::minecraft::manifest::resolve_manifest_chain(manifest).await?;

    build_command(&resolved, &profile).await
}
