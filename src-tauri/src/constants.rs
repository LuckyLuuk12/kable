// IDs and public keys:
pub const DISCORD_APP_ID: &str = "1432139549592649738";
pub const DEFAULT_AZURE_CLIENT_ID: &str = "4c27a19f-a3d0-4cd2-8e05-9fd961f905df";
// Trusted URLs for fetching required information
pub const KABLE_GITHUB_RELEASES: &str = "https://api.github.com/repos/LuckyLuuk12/kable/releases";
pub const MINECRAFT_VERSION_MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
pub const FABRIC_META_URL: &str = "https://meta.fabricmc.net";
pub const QUILT_META_URL: &str = "https://meta.quiltmc.org";
pub const FORGE_MAVEN_METADATA_URL: &str = "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";
pub const NEOFORGE_VERSION_URL: &str = "https://maven.neoforged.net/api/maven/versions/releases/net%2Fneoforged%2Fneoforge";
pub const DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
pub const MSA_AUTHORIZE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
pub const MSA_TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
pub const DEFAULT_AZURE_REDIRECT_URI: &str = "http://localhost:43110/callback";
pub const DEFAULT_OAUTH_PORT: u16 = 43110;

// App Folder names - all are relative to .kable except .kable itself which is relative to .minecraft
/// Root folder for all kable data, usually <user path>/.minecraft/.kable (on Windows: C:\Users\<User>\AppData\Roaming\.minecraft\.kable)
pub const KABLE_DIR_NAME: &str = ".kable";
/// folder in .kable for storing mods per profile like mods/<profile_id>/*.jar.
pub const EXPORTS_DIR: &str = "exports";
pub const LOGS_DIR: &str = "logs";
pub const MANIFESTS_DIR: &str = "manifests";
pub const MODS_DIR: &str = "mods";
pub const RESOURCEPACKS_DIR: &str = "resourcepacks";
pub const SHADERPACKS_DIR: &str = "shaderpacks";
/// Here we store launcher-specific config files: kable_accounts.json, settings.json & token.key, as well as other future config files
pub const LAUNCHER_DIR: &str = "launcher";
/// subfolder for the various settings/customizations: icons, images, sounds, themes.
pub const CONFIG_DIR: &str = "config";
pub const ICONS_DIR: &str = "icons";
pub const IMAGES_DIR: &str = "images";
pub const SOUNDS_DIR: &str = "sounds";
pub const THEMES_DIR: &str = "themes";
// .minecraft subfolders
pub const VERSIONS_DIR: &str = "versions";
pub const LIBRARIES_DIR: &str = "libraries";
pub const ASSETS_DIR: &str = "assets";
pub const NATIVES_DIR: &str = "natives";
// Other kind of util folders - usually still relate
pub const TEMP_DIR: &str = "tmp";
pub const CACHE_DIR: &str = "cache";

// File names
pub const SETTINGS_FILE: &str = "settings.json";
pub const SOUNDPACK_FILE: &str = "soundpack.json";
pub const CUSTOM_SYMLINKS_FILE: &str = "custom_symlinks.json";
pub const LAUNCHER_PROFILES_FILE: &str = "launcher_profiles.json";
pub const LAUNCHER_ACCOUNTS_FILE: &str = "launcher_accounts.json";
pub const PENDING_UPDATE_FILE: &str = "pending_update.json";
pub const KABLE_ACCOUNTS_FILE: &str = "kable_accounts.json";
pub const KABLE_PROFILES_FILE: &str = "kable_profiles.json";
pub const MODRINTH_CACHE_FILE: &str = "modrinth_cache.json";
pub const MODRINTH_VERSIONS_CACHE_FILE: &str = "modrinth_versions_cache.json";

// Launcher special values
pub const LATEST_RELEASE: &str = "latest-release";
pub const LATEST_SNAPSHOT: &str = "latest-snapshot";

/// Get the Fabric profile JSON URL for a specific Minecraft and loader version
pub fn fabric_profile_url(mc_version: &str, loader_version: &str) -> String {
    format!("{}/v2/versions/loader/{}/{}/profile/json", FABRIC_META_URL, mc_version, loader_version)
}

/// Get the Fabric profile JAR URL for a specific Minecraft and loader version
pub fn fabric_profile_jar_url(mc_version: &str, loader_version: &str) -> String {
    format!("{}/v2/versions/loader/{}/{}/profile/jar", FABRIC_META_URL, mc_version, loader_version)
}

/// Get the Quilt profile JSON URL for a specific Minecraft and loader version
pub fn quilt_profile_url(mc_version: &str, loader_version: &str) -> String {
    format!("{}/v3/versions/loader/{}/{}/profile/json", QUILT_META_URL, mc_version, loader_version)
}

// Note: We assume relative filenames to work from the .minecraft folder
// e.g. the kable dir is usually <user path>/.minecraft/.kable
