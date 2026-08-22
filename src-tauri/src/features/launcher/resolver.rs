use crate::{
    integrations::minecraft::{
        assets::AssetResolver,
        libraries::LibraryResolver,
        log_config::LogConfigResolver,
        natives::NativeResolver,
        versions::types::{Arg, Arguments, McVersionManifest, Rule, RuleArg, StringOrList},
    },
    Logger,
};
use api_types::profiles::{KableProfile, ProfileVersionType};
use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
    process::Command,
};

fn classpath_separator() -> &'static str {
    if cfg!(windows) {
        ";"
    } else {
        ":"
    }
}

fn current_os_name() -> &'static str {
    match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "osx",
        "linux" => "linux",
        other => other,
    }
}

fn current_os_arch() -> &'static str {
    std::env::consts::ARCH
}

#[derive(Debug, Clone)]
struct RuleContext {
    os_name: String,
    os_arch: String,
    os_version: Option<String>,

    is_demo_user: bool,
    has_custom_resolution: bool,

    has_quick_plays_support: bool,
    is_quick_play_singleplayer: bool,
    is_quick_play_multiplayer: bool,
    is_quick_play_realms: bool,
}

impl Default for RuleContext {
    fn default() -> Self {
        Self {
            os_name: current_os_name().to_string(),
            os_arch: current_os_arch().to_string(),
            os_version: current_os_version(),

            is_demo_user: false,
            has_custom_resolution: false,

            has_quick_plays_support: false,
            is_quick_play_singleplayer: false,
            is_quick_play_multiplayer: false,
            is_quick_play_realms: false,
        }
    }
}

fn current_os_version() -> Option<String> {
    #[cfg(windows)]
    {
        let version = windows_version::OsVersion::current();

        Some(format!("{}.{}.{}", version.major, version.minor, version.build))
    }

    #[cfg(not(windows))]
    {
        None
    }
}

fn parse_version(version: &str) -> Vec<u64> {
    version
        .split('.')
        .map(|part| part.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse::<u64>().unwrap_or(0))
        .collect()
}

fn compare_versions(left: &str, right: &str) -> Ordering {
    let left = parse_version(left);
    let right = parse_version(right);

    let length = left.len().max(right.len());

    for index in 0..length {
        let left_part = left.get(index).copied().unwrap_or(0);
        let right_part = right.get(index).copied().unwrap_or(0);

        match left_part.cmp(&right_part) {
            Ordering::Equal => {}
            ordering => return ordering,
        }
    }

    Ordering::Equal
}

fn version_at_least(version: &str, minimum: &str) -> bool {
    compare_versions(version, minimum) != Ordering::Less
}

fn version_at_most(version: &str, maximum: &str) -> bool {
    compare_versions(version, maximum) != Ordering::Greater
}

fn os_rule_matches(os: &crate::integrations::minecraft::versions::types::OsRule, context: &RuleContext) -> bool {
    if let Some(name) = &os.name {
        if name != &context.os_name {
            return false;
        }
    }

    if let Some(arch) = &os.arch {
        if arch != &context.os_arch {
            return false;
        }
    }

    if let Some(version_range) = &os.version_range {
        let Some(version) = &context.os_version else {
            return false;
        };

        if let Some(min) = &version_range.min {
            if !version_at_least(version, min) {
                return false;
            }
        }

        if let Some(max) = &version_range.max {
            if !version_at_most(version, max) {
                return false;
            }
        }
    }

    true
}

fn feature_matches(features: &std::collections::HashMap<String, bool>, context: &RuleContext) -> bool {
    for (feature, required) in features {
        let actual = match feature.as_str() {
            "is_demo_user" => context.is_demo_user,

            "has_custom_resolution" => context.has_custom_resolution,

            "has_quick_plays_support" => context.has_quick_plays_support,

            "is_quick_play_singleplayer" => context.is_quick_play_singleplayer,

            "is_quick_play_multiplayer" => context.is_quick_play_multiplayer,

            "is_quick_play_realms" => context.is_quick_play_realms,

            _ => false,
        };

        if actual != *required {
            return false;
        }
    }

    true
}

fn rule_matches(rule: &Rule, context: &RuleContext) -> bool {
    if let Some(os) = &rule.os {
        if !os_rule_matches(os, context) {
            return false;
        }
    }

    if let Some(features) = &rule.features {
        if !feature_matches(features, context) {
            return false;
        }
    }

    true
}

fn rules_allow(rules: Option<&Vec<Rule>>, context: &RuleContext) -> bool {
    let Some(rules) = rules else {
        return true;
    };

    let mut allowed = false;

    for rule in rules {
        if !rule_matches(rule, context) {
            continue;
        }

        match rule.action.as_deref() {
            Some("allow") => {
                allowed = true;
            }

            Some("disallow") => {
                allowed = false;
            }

            _ => {}
        }
    }

    allowed
}

async fn replace_variables(
    value: &str,
    profile: &KableProfile,
    manifest: &McVersionManifest,
    classpath: &str,
    log_config_path: Option<&Path>,
) -> Result<String, String> {
    let mc_root = crate::system::fs::mc_dir()?;

    let assets_root = mc_root.join(crate::constants::ASSETS_DIR);

    let natives_root = mc_root.join(crate::constants::NATIVES_DIR).join(&profile.version.id);

    let libraries_root = mc_root.join(crate::constants::LIBRARIES_DIR);

    let active_account = crate::features::accounts::management::get_active_account().await?;

    let monitor = crate::app_handle()?.primary_monitor().ok().flatten();

    let (width, height) = monitor
        .map(|monitor| {
            let work_area = monitor.work_area();

            (work_area.size.width.saturating_sub(100), work_area.size.height.saturating_sub(100))
        })
        .unwrap_or((800, 600));

    let asset_index = manifest.asset_index.as_ref().and_then(|index| index.id.as_deref()).unwrap_or("");

    let version_type = profile.version.version_type.clone().unwrap_or(ProfileVersionType::Release).to_string();

    let mut result = value.to_string();

    result = result.replace("${assets_index_name}", asset_index);

    result = result.replace("${assets_root}", &assets_root.to_string_lossy());

    result = result.replace("${game_directory}", &mc_root.to_string_lossy());

    result = result.replace("${version_name}", &profile.version.id);

    result = result.replace("${version_type}", &version_type);

    result = result.replace("${launcher_name}", env!("CARGO_PKG_NAME"));

    result = result.replace("${launcher_version}", env!("CARGO_PKG_VERSION"));

    result = result.replace("${natives_directory}", &natives_root.to_string_lossy());

    result = result.replace("${library_directory}", &libraries_root.to_string_lossy());

    result = result.replace("${classpath}", classpath);

    result = result.replace("${classpath_separator}", classpath_separator());

    result = result.replace("${resolution_width}", &width.to_string());

    result = result.replace("${resolution_height}", &height.to_string());

    result = result.replace("${quickPlayMultiplayer}", "false");
    result = result.replace("${quickPlaySingleplayer}", "false");
    result = result.replace("${quickPlayRealms}", "false");
    result = result.replace("${quickPlayPath}", "");

    if let Some(log_config_path) = log_config_path {
        result = result.replace("${path}", &log_config_path.to_string_lossy());
    } else {
        result = result.replace("${path}", "");
    }

    if let Some(account) = active_account {
        result = result.replace("${auth_access_token}", &account.access_token);

        result = result.replace("${auth_player_name}", &account.minecraft_profile.name);

        result = result.replace("${auth_uuid}", &account.minecraft_profile.id);

        result = result.replace("${auth_xuid}", &account.remote_id);

        /*
         * clientid/clientId is not necessarily the Minecraft UUID.
         *
         * Keep this mapping until the account model exposes the actual
         * launcher/client ID separately.
         */
        result = result.replace("${clientid}", &account.minecraft_profile.id);

        result = result.replace("${clientId}", &account.minecraft_profile.id);

        result = result.replace("${user_type}", &account.account_type);
    } else {
        for variable in [
            "${auth_access_token}",
            "${auth_player_name}",
            "${auth_uuid}",
            "${auth_xuid}",
            "${clientid}",
            "${clientId}",
            "${user_type}",
        ] {
            result = result.replace(variable, "");
        }
    }

    Ok(result)
}

async fn resolve_arg(
    arg: &Arg,
    profile: &KableProfile,
    manifest: &McVersionManifest,
    classpath: &str,
    context: &RuleContext,
    log_config_path: Option<&Path>,
) -> Result<Vec<String>, String> {
    match arg {
        Arg::String(value) => Ok(vec![replace_variables(value, profile, manifest, classpath, log_config_path).await?]),

        Arg::Rule(RuleArg { rules, value }) => {
            if !rules_allow(rules.as_ref(), context) {
                return Ok(Vec::new());
            }

            let Some(value) = value else {
                return Ok(Vec::new());
            };

            match value {
                StringOrList::Single(value) => Ok(vec![replace_variables(value, profile, manifest, classpath, log_config_path).await?]),

                StringOrList::List(values) => {
                    let mut resolved = Vec::with_capacity(values.len());

                    for value in values {
                        resolved.push(replace_variables(value, profile, manifest, classpath, log_config_path).await?);
                    }

                    Ok(resolved)
                }
            }
        }
    }
}

async fn resolve_arguments(
    args: &[Arg],
    profile: &KableProfile,
    manifest: &McVersionManifest,
    classpath: &str,
    context: &RuleContext,
    log_config_path: Option<&Path>,
) -> Result<Vec<String>, String> {
    let mut resolved = Vec::new();

    for arg in args {
        resolved.extend(resolve_arg(arg, profile, manifest, classpath, context, log_config_path).await?);
    }

    Ok(resolved)
}

async fn resolve_jvm_arguments(
    manifest: &McVersionManifest,
    profile: &KableProfile,
    classpath: &str,
    context: &RuleContext,
    log_config_path: Option<&Path>,
) -> Result<Vec<String>, String> {
    let Some(arguments) = &manifest.arguments else {
        return Ok(Vec::new());
    };

    match arguments {
        Arguments::Structured(arguments) => {
            let mut args = Vec::new();

            if let Some(jvm) = &arguments.jvm {
                args.extend(resolve_arguments(jvm, profile, manifest, classpath, context, log_config_path).await?);
            }

            if let Some(default_user_jvm) = &arguments.default_user_jvm {
                args.extend(resolve_arguments(default_user_jvm, profile, manifest, classpath, context, log_config_path).await?);
            }

            Ok(args)
        }

        Arguments::Mixed(arguments) => resolve_arguments(arguments, profile, manifest, classpath, context, log_config_path).await,

        Arguments::Flat(arguments) => {
            let mut args = Vec::with_capacity(arguments.len());

            for argument in arguments {
                args.push(replace_variables(argument, profile, manifest, classpath, log_config_path).await?);
            }

            Ok(args)
        }
    }
}

async fn resolve_game_arguments(
    manifest: &McVersionManifest,
    profile: &KableProfile,
    classpath: &str,
    context: &RuleContext,
    log_config_path: Option<&Path>,
) -> Result<Vec<String>, String> {
    let Some(arguments) = &manifest.arguments else {
        return Ok(Vec::new());
    };

    match arguments {
        Arguments::Structured(arguments) => {
            let mut args = Vec::new();

            if let Some(game) = &arguments.game {
                args.extend(resolve_arguments(game, profile, manifest, classpath, context, log_config_path).await?);
            }

            Ok(args)
        }

        Arguments::Mixed(arguments) => resolve_arguments(arguments, profile, manifest, classpath, context, log_config_path).await,

        Arguments::Flat(arguments) => {
            let mut args = Vec::with_capacity(arguments.len());

            for argument in arguments {
                args.push(replace_variables(argument, profile, manifest, classpath, log_config_path).await?);
            }

            Ok(args)
        }
    }
}

async fn ensure_client_jar(manifest: &McVersionManifest) -> Result<PathBuf, String> {
    let mc_root = crate::system::fs::mc_dir()?;

    let version_dir = mc_root.join(crate::constants::VERSIONS_DIR).join(&manifest.id);

    let client_jar = version_dir.join(format!("{}.jar", manifest.id));

    if client_jar.is_file() {
        return Ok(client_jar);
    }

    let Some(download) = manifest.downloads.as_ref().and_then(|downloads| downloads.client.as_ref()) else {
        return Err(format!("Minecraft version {} has no client download information", manifest.id));
    };

    let Some(url) = &download.url else {
        return Err(format!("Minecraft version {} has no client download URL", manifest.id));
    };

    crate::system::fs::create_dir(&version_dir).await?;

    Logger::debug_global(format!("Downloading Minecraft client {} from {}", manifest.id, url).as_str(), None);

    crate::system::net::download_to_file(url, &client_jar).await?;

    if !client_jar.is_file() {
        return Err(format!("Minecraft client download completed but {} does not exist", client_jar.display()));
    }

    Ok(client_jar)
}

fn add_client_jar_to_classpath(library_classpath: String, client_jar: &Path) -> String {
    if library_classpath.is_empty() {
        return client_jar.to_string_lossy().to_string();
    }

    format!("{}{}{}", library_classpath, classpath_separator(), client_jar.to_string_lossy())
}

async fn build_command(manifest: &McVersionManifest, profile: &KableProfile) -> Result<Command, String> {
    let asset_resolver = AssetResolver::new()?;
    let library_resolver = LibraryResolver::new()?;
    let native_resolver = NativeResolver::new()?;
    let log_config_resolver = LogConfigResolver::new()?;

    let libraries = manifest.libraries.as_deref().unwrap_or(&[]);

    Logger::debug_global(
        format!("Resolving Minecraft {} with {} libraries", manifest.id, libraries.len()).as_str(),
        Some(profile.id.as_str()),
    );

    asset_resolver.ensure_assets(manifest).await?;

    library_resolver.ensure_downloaded(libraries).await?;

    let natives_dir = native_resolver.ensure_natives(libraries, &profile.version.id).await?;

    let log_config_path = log_config_resolver.ensure_config(manifest).await?;

    let client_jar = ensure_client_jar(manifest).await?;

    let library_classpath = library_resolver.resolve_classpath(libraries)?;

    let classpath = add_client_jar_to_classpath(library_classpath, &client_jar);

    let java_path = crate::system::java::find_java_executable(
        crate::features::customization::settings::load_settings().await?.general.java_path.as_ref(),
    )?;

    let context = RuleContext {
        os_name: current_os_name().to_string(),
        os_arch: current_os_arch().to_string(),
        os_version: current_os_version(),

        is_demo_user: false,

        has_custom_resolution: true,

        has_quick_plays_support: false,
        is_quick_play_singleplayer: false,
        is_quick_play_multiplayer: false,
        is_quick_play_realms: false,
    };

    let mut jvm_args = resolve_jvm_arguments(manifest, profile, &classpath, &context, log_config_path.as_deref()).await?;

    let game_args = resolve_game_arguments(manifest, profile, &classpath, &context, log_config_path.as_deref()).await?;

    /*
     * The classpath is normally supplied by the manifest.
     * Keep this fallback for old manifests.
     */
    if !jvm_args.iter().any(|arg| arg == "-cp" || arg == "-classpath") {
        jvm_args.push("-cp".to_string());
        jvm_args.push(classpath.clone());
    }

    /*
     * Logging configuration is separate from the regular
     * JVM argument list in the version manifest.
     *
     * ${path} is replaced with the actual downloaded XML
     * configuration file under assets/log_configs.
     */
    if let Some(logging) = &manifest.logging {
        if let Some(client) = &logging.client {
            if let Some(argument) = &client.argument {
                jvm_args.push(replace_variables(argument, profile, manifest, &classpath, log_config_path.as_deref()).await?);
            }
        }
    }

    let main_class = manifest.main_class.as_deref().ok_or_else(|| format!("Minecraft manifest {} has no main class", manifest.id))?;

    let mc_root = crate::system::fs::mc_dir()?;

    let mut cmd = Command::new(&java_path);

    cmd.current_dir(mc_root);
    cmd.args(&jvm_args);
    cmd.arg(main_class);
    cmd.args(&game_args);

    Logger::debug_global(format!("Java executable: {}", java_path).as_str(), Some(profile.id.as_str()));

    Logger::debug_global(format!("Natives directory: {}", natives_dir.display()).as_str(), Some(profile.id.as_str()));

    if let Some(log_config_path) = &log_config_path {
        Logger::debug_global(format!("Log configuration: {}", log_config_path.display()).as_str(), Some(profile.id.as_str()));
    }

    Logger::debug_global(
        format!("Classpath entries: {}", classpath.split(classpath_separator()).count()).as_str(),
        Some(profile.id.as_str()),
    );

    Logger::debug_global(format!("Main class: {}", main_class).as_str(), Some(profile.id.as_str()));

    Logger::debug_global(format!("JVM arguments: {:?}", jvm_args).as_str(), Some(profile.id.as_str()));

    Logger::debug_global(format!("Game arguments: {:?}", game_args).as_str(), Some(profile.id.as_str()));

    Ok(cmd)
}

pub async fn resolve(profile: KableProfile) -> Result<Command, String> {
    let manifest_path = crate::system::fs::mc_dir()?
        .join(crate::constants::VERSIONS_DIR)
        .join(&profile.version.id)
        .join(format!("{}.json", profile.version.id));

    let raw = crate::system::fs::read_str(manifest_path).await?;

    let manifest = crate::integrations::minecraft::versions::load_version_manifest(raw).await?;

    let resolved = crate::integrations::minecraft::manifest::resolve_manifest_chain(manifest).await?;

    build_command(&resolved, &profile).await
}
