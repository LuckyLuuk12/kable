/** Cape info from Mojang profile API */
export interface AccountCape {
  id: string;
  state: string;
  url?: string;
  alias?: string;
}

/** Full player profile from Mojang profile API */
export interface PlayerProfile {
  id: string;
  name: string;
  skins: AccountSkin[];
  capes: AccountCape[];
}
// Microsoft authentication types
export interface MicrosoftToken {
  access_token: string;
  expires_at: string;
  encrypted_refresh_token?: string;
}

// Authentication Methods
export type AuthMethod = "DeviceCodeFlow" | "Custom" | "Offline";

export interface DeviceCodeResponse {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
}
export interface MinecraftInstallation {
  id: string;
  name: string;
  path: string;
  version: string;
  is_valid: boolean;
  mod_loader: "vanilla" | "fabric" | "forge" | "quilt" | "neoforge"; // Backend uses mod_loader
  loader_version?: string;
  description?: string;
  game_directory?: string; // Backend uses game_directory
  java_path?: string; // Backend uses java_path
  jvm_args?: string; // Backend uses jvm_args as string
  memory?: number; // Memory allocation in MB for this installation
  last_played?: string; // Backend uses last_played (snake_case)
  created?: string; // ISO date string
}

// Keep the legacy interface for backwards compatibility if needed
export interface LegacyMinecraftInstallation {
  id: string;
  name: string;
  path: string;
  version: string;
  is_valid: boolean;
  type: "vanilla" | "fabric" | "forge" | "quilt" | "neoforge";
  modLoader: "vanilla" | "fabric" | "forge" | "quilt" | "neoforge";
  loader_version?: string;
  description?: string;
  gameDirectory?: string;
  javaPath?: string;
  jvmArgs?: string;
  lastPlayed?: string; // ISO date string
  created: string; // ISO date string
}

export interface LaunchOptions {
  version: string;
  username: string;
  uuid: string;
  access_token: string;
  memory: number; // in MB
  java_path?: string;
  jvm_args?: string[];
  game_args?: string[];
  window_width?: number;
  window_height?: number;
  fullscreen?: boolean;
}

export interface AuthResponse {
  username: string;
  uuid: string;
  access_token: string;
  refresh_token?: string;
  expires_at?: number;
}

// Microsoft Account Management - Enhanced for session management
export interface AdvancedSettings {
  enable_experimental_features?: boolean;
  default_memory?: number;
  separate_logs_window?: boolean;
  show_advanced_page?: boolean;
  check_nightly_updates?: boolean;
  extra?: Record<string, any>;
}

export interface MicrosoftAccount {
  id: string;
  username: string;
  uuid: string;
  access_token: string;
  refresh_token: string;
  expires_at: number;
  skin_url?: string;
  is_active: boolean;
  last_used: number;
  minecraft_access_token: string; // Minecraft-specific token
  minecraft_expires_at: number;
  xbox_user_hash: string;
}

// Launcher Accounts JSON structure (matches .minecraft/launcher_accounts.json)
export interface LauncherAccount {
  access_token: string;
  access_token_expires_at: string;
  encrypted_refresh_token?: string; // AES-encrypted refresh token
  avatar: string;
  eligible_for_free_trials: boolean;
  eligible_for_migration: boolean;
  franchise_inventory_id: string;
  has_multiple_profiles: boolean;
  in_forced_migration: boolean;
  legacy: boolean;
  license_product_ids: string[];
  local_id: string;
  minecraft_profile: {
    id: string;
    name: string;
    requires_profile_name_change: boolean;
    requires_skin_change: boolean;
  };
  persistent: boolean;
  remote_id: string;
  account_type: string; // "Xbox", "Offline", etc.
  user_properties: any[];
  username: string;
}

export interface LauncherAccountsJson {
  accounts: Record<string, LauncherAccount>;
  active_account_local_id: string;
  mojang_client_token: string;
}
// Mod Detection Service Types
export interface ModDetectionResult {
  hasActiveMods: boolean;
  modCount: number;
  detectedLoaders: string[];
  modLoaderType: "vanilla" | "fabric" | "forge" | "quilt" | "neoforge";
  loaderVersion?: string;
  modsList?: Array<{
    name: string;
    fileName: string;
    enabled: boolean;
  }>;
}

// Minecraft Session Data (launcher_profiles.json)
export interface MinecraftSession {
  access_token: string;
  client_token: string;
  uuid: string;
  username: string;
  user_type: string;
  user_properties: Record<string, any>;
}

/**
 * Launcher Profiles Data
 * ```ts
 * export interface LauncherProfiles {
 *  authentication_database: Record<string, MinecraftSession>;
 *  launcher_version: string;
 *  selected_user?: string;
 * }
 * ```
 */
export interface LauncherProfiles {
  authentication_database: Record<string, MinecraftSession>;
  launcher_version: string;
  selected_user?: string;
}

/**
 * Minecraft Version Information
 * ```ts
 * export interface MinecraftVersion {
 *  id: string;
 *  type: 'release' | 'snapshot' | 'old_beta' | 'old_alpha';
 *  url: string;
 *  time: string;
 *  releaseTime: string;
 * }
 * ```
 */
export interface MinecraftVersion {
  id: string;
  type: "release" | "snapshot" | "old_beta" | "old_alpha";
  url: string;
  time: string;
  releaseTime: string;
}

export interface MinecraftDirectoryInfo {
  path: string;
  is_valid: boolean;
  versions_folder?: string;
  launcher_profiles?: string;
}

// Additional type definitions for managers
export interface LocalWorld {
  id: string;
  name: string;
  folder_name: string;
  game_mode: "Survival" | "Creative" | "Adventure" | "Spectator";
  difficulty: "Peaceful" | "Easy" | "Normal" | "Hard";
  version: string;
  size_mb: number;
  last_played: number; // Unix timestamp in milliseconds
  created: number; // Unix timestamp in milliseconds
  seed: string | null;
  icon: string | null;
  backup_count: number;
  has_cheats: boolean;
  world_type: string;
}

export interface WorldDownload {
  id: string;
  name: string;
  url: string;
  description?: string;
  size?: number;
  version?: string;
}

export interface ShaderPack {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string | null;
  file_path: string;
  file_name: string;
  file_size: number;
  compatible_versions: string[];
  enabled: boolean;
  source_url: string | null;
  thumbnail: string | null;
  shader_loader: "Canvas" | "Iris" | "OptiFine" | "Vanilla";
  installed_date: number;
  last_used: number | null;
}

export interface ShaderDownload {
  id: string;
  name: string;
  author: string;
  description: string;
  download_url: string;
  thumbnail: string | null;
  gallery: string[] | null;
  featured_gallery: string | null;
  tags: string[];
  minecraft_versions: string[];
  shader_loader: "Canvas" | "Iris" | "OptiFine" | "Vanilla";
  rating: number;
  downloads: number;
  size_mb: number;
  source: "Modrinth" | "CurseForge" | { Other: string };
}

export interface ShaderSettings {
  quality: "Low" | "Medium" | "High" | "Ultra" | "Custom";
  shadows: boolean;
  shadow_resolution: number;
  anti_aliasing: boolean;
  bloom: boolean;
  motion_blur: boolean;
  custom_settings: Record<string, any>;
}

export interface MinecraftSkin {
  id: string;
  name: string;
  url?: string;
  type: "steve" | "alex";
  premium?: boolean;
  // Additional properties for local skin management
  file_name: string;
  is_slim: boolean;
  source: string;
  created_date: number;
  last_used?: number;
}

export interface SkinDownload {
  id: string;
  name: string;
  url: string;
  type: "steve" | "alex";
  description?: string;
}

// Mod Management Types
export interface ModInstallationConfig {
  id: string;
  name: string;
  installation_type: string;
  use_global_mods: boolean;
  mods_folder_path: string;
}

export interface InstalledMod {
  id: string;
  name: string;
  version: string;
  source: "Modrinth" | "CurseForge" | "Local";
  source_id: string;
  file_path: string;
  minecraft_version: string;
  mod_loader: "Fabric" | "Forge" | "Quilt" | "NeoForge";
  enabled: boolean;
  dependencies: string[];
  auto_update: boolean;
}

export interface ModProject {
  id: string;
  slug: string;
  title: string;
  description: string;
  categories: string[];
  client_side: "Required" | "Optional" | "Unsupported";
  server_side: "Required" | "Optional" | "Unsupported";
  downloads: number;
  icon_url?: string;
  source: "Modrinth" | "CurseForge" | "Other";
}

export interface ModVersion {
  id: string;
  version_number: string;
  version_type: "Release" | "Beta" | "Alpha";
  minecraft_versions: string[];
  mod_loaders: string[];
  date_published: string;
  downloads: number;
  changelog?: string;
  files: ModFile[];
}

export interface ModFile {
  url: string;
  filename: string;
  size: number;
  sha1: string;
  primary: boolean;
}

// Log system types
export interface GameInstance {
  id: string;
  installationId?: string;
  installationName?: string;
  installationPath: string;
  profileName: string;
  startTime?: Date;
  launchedAt: Date;
  lastActivity: Date;
  completedAt?: Date;
  status: "launching" | "running" | "crashed" | "stopped" | "closed";
  exitCode?: number;
  processId?: number;
  restartAttempts?: number;
}

export interface LogEntry {
  timestamp: Date;
  level: "debug" | "info" | "warn" | "error";
  source: "launcher" | "game";
  instanceId?: string; // For game logs
  message: string;
  raw?: string; // Raw log line for syntax highlighting
}

export interface GameInstanceLogs {
  instanceId: string;
  launcherLogs: LogEntry[];
  gameLogs: LogEntry[];
}

// _____________________________________________________________________________
//|                                                                             |
//|                              Settings Types                                 |
//|_____________________________________________________________________________|

/** Categorized Launcher Settings
 * ```ts
 * export interface CategorizedLauncherSettings {
 *   general: GeneralSettings;
 *   appearance: AppearanceSettings;
 *   logging: LoggingSettings;
 *   network: NetworkSettings;
 *   content: ContentSettings;
 *   advanced: AdvancedSettings;
 *   misc: MiscSettings;
 * }
 * ```
 */
export interface CategorizedLauncherSettings {
  general: GeneralSettings;
  appearance: AppearanceSettings;
  logging: LoggingSettings;
  network: NetworkSettings;
  content: ContentSettings;
  advanced: AdvancedSettings;
  misc: MiscSettings;
}

/** General Settings for the launcher
 * ```ts
 * export interface GeneralSettings {
 *  javaPath?: string;
 *  gameDirectory?: string;
 *  onGameClose: 'exit' | 'minimize' | 'ask';
 *  onGameCrash: 'restart' | 'close' | 'ask';
 *  onGameLaunch: 'keep_open' | 'close_launcher' | 'open_logs' | 'ask';
 *  autoUpdateLauncher: boolean;
 *  showAds: boolean;
 * }
 * ```
 */
export interface GeneralSettings {
  /** Optional different Java path for launching Minecraft */
  java_path?: string;
  /** The path to the .minecraft directory */
  game_directory?: string;
  /** What to do when the game is being closed (quit game / close window) */
  on_game_close: "open_logs" | "open_home" | "exit" | "minimize" | "ask";
  /** What to do when the game crashes */
  on_game_crash:
    | "restart"
    | "open_logs"
    | "open_home"
    | "exit"
    | "minimize"
    | "ask";
  /** Whether to keep the launcher open after launching the game */
  on_game_launch: "keep_open" | "exit" | "open_logs" | "minimize" | "ask";
  /** Whether to automatically check for updates on startup */
  auto_update_launcher: boolean;
  /** Whether to show ads; I am a nice guy, no paid subscription needed to disable ads */
  show_ads: boolean;
  /** Update behaviour: 'instant' | 'on_restart' | 'on_confirm' */
  update_mode: "instant" | "on_restart" | "on_confirm";
}

/** Custom Icon Template for the launcher
 * ```ts
 * export interface CustomIconTemplate {
 *   name: string;
 *   displayName: string;
 *   version: string;
 *   author?: string;
 *   description?: string;
 *   iconType: 'emoji' | 'fontawesome' | 'css' | 'svg' | 'image';
 *   fallbackIcon: string;
 *   icons: Record<string, string>;
 *   cssClasses?: Record<string, string>; // For CSS-based icons
 *   baseUrl?: string; // For image-based icons
 * }
 * ```
 */
export interface CustomIconTemplate {
  name: string;
  displayName: string;
  version: string;
  author?: string;
  description?: string;
  iconType: "emoji" | "fontawesome" | "css" | "svg" | "image";
  fallbackIcon: string;
  icons: Record<string, string>;
  cssClasses?: Record<string, string>; // For CSS-based icons
  baseUrl?: string; // For image-based icons
}

/** Icon Settings for the launcher
 * ```ts
 * export interface IconSettings {
 *    selectedTemplate: string;
 *    customTemplates: CustomIconTemplate[];
 *    builtinTemplates: string[];
 * }
 * ```
 */
export interface IconSettings {
  /** The currently selected template name */
  selected_template: string;
  /** User-uploaded templates */
  custom_templates: CustomIconTemplate[];
  /** Available built-in templates (emoji, fontawesome) */
  builtin_templates: string[];
}

/** Appearance Settings for the launcher
 * ```ts
 * export interface AppearanceSettings {
 *   theme: 'light' | 'dark' | 'system';
 *   language: string;
 *   extraSpacing: number;
 *   sidebarWidth: number;
 *   selectedIconTemplate: string;
 *   iconSettings: IconSettings;
 *   selectedCssTheme: string;
 * }
 * ```
 */
export interface AppearanceSettings {
  /** The theme to use for the launcher */
  theme: "light" | "dark" | "system";
  /** The language to use for the launcher */
  language: string;
  /** The amount of pixels to add in spacing containers and cards */
  extra_spacing: number;
  /** The width of the sidebar in pixels when the sidebar is open */
  sidebar_width: number;
  /** The icon template to use */
  selected_icon_template: string;
  /** Icon settings to allow user customization */
  icon_settings: IconSettings;
  /** The selected CSS theme name */
  selected_css_theme: string;
  /** Sound settings for the launcher */
  sound?: SoundSettings;
}

/** Sound Settings for the launcher
 * ```ts
 * export interface SoundSettings {
 *   enabled: boolean;
 *   music_enabled: boolean;
 *   master_volume: number;
 *   sound_volume: number;
 *   music_volume: number;
 *   selected_soundpack: string;
 * }
 * ```
 */
export interface SoundSettings {
  /** Whether sound effects are enabled */
  enabled: boolean;
  /** Whether background music is enabled */
  music_enabled: boolean;
  /** Master volume (0-100) */
  master_volume: number;
  /** Sound effects volume (0-100) */
  sound_volume: number;
  /** Background music volume (0-100) */
  music_volume: number;
  /** The selected soundpack name */
  selected_soundpack: string;
}

/** Logging Settings for the launcher
 * ```ts
 * export interface LoggingSettings {
 *   showLogsPageInNav: boolean;
 *   enablePersistentLogging: boolean;
 *   enableLogCompression: boolean;
 *   logFileSizeLimitMb: number | 'disabled';
 *   logRetentionDays: number | 'disabled';
 *   mergeLogTabs: boolean;
 *   defaultLogLevels: ('debug' | 'info' | 'warn' | 'error')[];
 *   maxMemoryLogs?: number;
 *   dedupeWindowSize?: number;
 *   enableDedupe?: boolean;
 * }
 * ```
 */
export interface LoggingSettings {
  /** Whether to show the logs page in the navigation */
  show_logs_page_in_nav: boolean;
  /** Whether to enable persistent logging: logs are saved to files */
  enable_persistent_logging: boolean;
  /** Whether to compress log files: only applies if persistent logging is enabled */
  enable_log_compression: boolean;
  /** The maximum size of a log file in MB before it is added to the zip */
  log_file_size_limit_mb: number | "disabled";
  /** The number of days to keep log files */
  log_retention_days: number | "disabled";
  /** Whether to try to "merge" log tabs into one tab if they are from the same game instance (but a different launch) */
  merge_log_tabs: boolean;
  /** Which log levels are shown by default */
  default_log_levels: ("debug" | "info" | "warn" | "error")[];
  /** Maximum number of log entries to keep in memory per instance (prevents memory overflow, default: 5000) */
  max_memory_logs?: number;
  /** Number of recent messages to check for duplicates (default: 50) */
  dedupe_window_size?: number;
  /** Enable automatic deduplication of log messages (default: true) */
  enable_dedupe?: boolean;
}

/** Network Settings for the launcher
 * ```ts
 * export interface NetworkSettings {
 *   parallelDownloads: number;
 *   connectionTimeout: number;
 *   downloadSpeedLimit: number | 'unlimited';
 * }
 * ```
 */
export interface NetworkSettings {
  /** The number of parallel downloads to use for downloading mods and shader packs */
  parallel_downloads: number;
  /** The connection timeout in seconds for network requests */
  connection_timeout: number;
  /** How much to throttle the download speed for parallel downloads */
  download_speed_limit: number | "unlimited";
}

/** Content Settings for the launcher
 * ```ts
 * export interface ContentSettings {
 *   maxWorldBackups: number | 'disabled';
 *   autoBackupWorlds: boolean;
 *   usePerInstallationModsFolder: boolean;
 *   usePerInstallationResourcePacks: boolean;
 * }
 * ```
 */
export interface ContentSettings {
  /** The maximum number of backups to keep for each world */
  max_world_backups: number | "disabled";
  /** Whether to zip all worlds at least once on startup (not if >= maxWorldBackups) */
  auto_backup_worlds: boolean;
  /** Whether to modify existing mod installations to use a per-installation mods folder in the kable directory */
  use_per_installation_mods_folder: boolean;
  /** Whether to have per-installation resource packs in the kable directory (this zips, copies and moves resource packs and is quite HEAVY) */
  use_per_installation_resource_packs: boolean;
}

/** Advanced Settings for the launcher
 * ```ts
 * export interface AdvancedSettings {
 *   enableExperimentalFeatures: boolean;
 *   defaultMemory: number;
 *   separateLogsWindow: boolean;
 *   autoSaveInterval: number | 'disabled';
 *   extra: Record<string, any>;
 * }
 * ```
 */
export interface AdvancedSettings {
  /** Whether to enable experimental features */
  enable_experimental_features: boolean;
  /** The amount of memory to allocate to the game by default, only used on newly created installation as default */
  default_memory: number; // in MB
  /** Whether to have the logs page on another window (experimental) */
  separate_logs_window: boolean;
  /** How frequently to auto save the settings */
  auto_save_interval: number | "disabled"; // in seconds, 'disabled' means no auto save
  /** Whether to show the advanced page in the navigation bar */
  show_advanced_page: boolean;
  /** Whether to check for nightly/prerelease updates */
  check_nightly_updates?: boolean;
  /** A map with string keys and any type of values for really advanced stuff */
  extra?: Record<string, any>;
}

/** Miscellaneous Settings for the launcher
 * ```ts
 * export interface MiscSettings {
 *   useTitlebar: boolean;
 *   authPreference: 'code' | 'device_code';
 * }
 * ```
 */
export interface MiscSettings {
  /** Whether a titlebar should be used (not handy for closing the app) */
  use_titlebar: boolean;
  /** Authentication preference (code flow is recommended) */
  auth_preference: "code" | "device_code";
}

// _____________________________________________________________________________
//|                                                                             |
//|                            Installation Types                               |
//|_____________________________________________________________________________|

/** LoaderKind enum (matches backend LoaderKind) */
export type LoaderKind =
  | "Vanilla"
  | "Fabric"
  | "IrisFabric"
  | "Forge"
  | "NeoForge"
  | "Quilt";

/** VersionData struct
 * ```ts
 * export interface VersionData {
 *   version_id: string;
 *   loader: LoaderKind;
 *   display_name: string;
 *   is_stable: boolean;
 *   extra: any;
 * }
 * ```
 */
export interface VersionData {
  version_id: string;
  loader: LoaderKind;
  display_name: string;
  is_stable: boolean;
  extra: any;
}

/** KableInstallation struct
 * ```ts
 * export interface KableInstallation {
 *   id: string;
 *   name: string;
 *   icon?: string | null;
 *   version_id: string;
 *   created: string;
 *   last_used: string;
 *   java_args: string[];
 *   dedicated_resource_pack_folder?: string | null;
 *   dedicated_shaders_folder?: string | null;
 *   favorite: boolean;
 *   total_time_played_ms: number;
 *   parameters_map: Record<string, string>;
 *   description?: string | null;
 *   times_launched: number;
 * }
 * ```
 */
export interface KableInstallation {
  id: string;
  name: string;
  icon?: string | null;
  version_id: string;
  created: string;
  last_used: string;
  java_args: string[];
  dedicated_mods_folder?: string | null;
  dedicated_resource_pack_folder?: string | null;
  dedicated_shaders_folder?: string | null;
  dedicated_config_folder?: string | null;
  favorite: boolean;
  total_time_played_ms: number;
  parameters_map: Record<string, string>;
  description?: string | null;
  times_launched: number;
  enable_pack_merging?: boolean;
  pack_order?: string[];
}

/** LauncherProfile struct
 * ```ts
 * export interface LauncherProfile {
 *   created: string;
 *   icon: string;
 *   java_args: string;
 *   last_used: string;
 *   last_version_id: string;
 *   name: string;
 *   profile_type: string;
 * }
 * ```
 */
export interface LauncherProfile {
  created: string;
  icon: string;
  java_args: string;
  last_used: string;
  last_version_id: string;
  name: string;
  profile_type: string;
}

/** InstallationForm struct - for creating/editing installations
 * ```ts
 * export interface InstallationForm {
 *   name: string;
 *   icon?: string | null;
 *   java_args?: string[];
 *   version_id: string;
 *   dedicated_resource_pack_folder?: string | null;
 *   dedicated_shaders_folder?: string | null;
 *   description?: string | null;
 * }
 * ```
 */
export interface InstallationForm {
  name: string;
  icon?: string | null;
  java_args?: string[];
  version_id: string;
  dedicated_resource_pack_folder?: string | null;
  dedicated_shaders_folder?: string | null;
  description?: string | null;
}

/** ModJarInfo struct - for mod jar file information
 * ```ts
 * export interface ModJarInfo {
 *   file_name: string;
 *   mod_name?: string | null;
 *   mod_version?: string | null;
 *   loader?: string | null;
 * }
 * ```
 */
export interface ModJarInfo {
  file_name: string;
  mod_name?: string | null;
  mod_version?: string | null;
  loader?: string | null;
  /** true when the JAR was found in the installation's disabled/ subfolder */
  disabled?: boolean;
}

// _____________________________________________________________________________
//|                                                                             |
//|                              Launcher Types                                 |
//|_____________________________________________________________________________|

/** Result of launching a game instance
 * ```ts
 * export interface LaunchResult {
 *   pid: number;
 *   success: boolean;
 *   error?: string;
 * }
 * ```
 */
export interface LaunchResult {
  pid: number;
  success: boolean;
  error?: string;
}

// _____________________________________________________________________________
//|                                                                             |
//|                                Mods Types                                   |
//|_____________________________________________________________________________|

/** Discriminated union for mod info returned by different providers.
 * Supports both Rust enum serialization format and TypeScript discriminated union format.
 * ```ts
 * export type ModInfoKind =
 *   | { kind: 'Modrinth'; data: ModrinthInfo }        // TypeScript discriminated union
 *   | { Modrinth: ModrinthInfo }                      // Rust enum serialization
 *   | { kind: 'CurseForge'; data: CurseForgeInfo }    // TypeScript discriminated union
 *   | { CurseForge: CurseForgeInfo };                 // Rust enum serialization
 * ```
 */
export type ModInfoKind =
  | { kind: "Modrinth"; data: ModrinthInfo } // TypeScript discriminated union format
  | { Modrinth: ModrinthInfo } // Rust enum serialization format
  | { kind: "CurseForge"; data: CurseForgeInfo } // TypeScript discriminated union format
  | { CurseForge: CurseForgeInfo }; // Rust enum serialization format

/** Discriminated union for mod filters for each provider.
 * Uses Rust externally tagged enum format for serde compatibility.
 * ```ts
 * export type ModFilter =
 *   | { Modrinth: FilterFacets }
 *   | { CurseForge: CurseForgeFilter };
 * ```
 */
export type ModFilter =
  | { Modrinth: FilterFacets }
  | { CurseForge: CurseForgeFilter };

/** Modrinth filter facets for searching mods.
 * ```ts
 * export interface FilterFacets {
 *   query?: string;
 *   categories?: [string, string][];
 *   client_side?: [string, string];
 *   server_side?: [string, string];
 *   index?: string;
 *   open_source?: boolean;
 *   license?: [string, string];
 *   downloads?: [string, number];
 * }
 * ```
 */
export interface FilterFacets {
  /** Free-text search query */
  query?: string;
  /** Array of [facet, value] pairs for categories */
  categories?: [string, string][];
  /** Client-side requirement filter */
  client_side?: [string, string];
  /** Server-side requirement filter */
  server_side?: [string, string];
  /** Index for sorting/filtering */
  index?: string;
  /** Whether to filter for open source mods */
  open_source?: boolean;
  /** License filter as [facet, value] */
  license?: [string, string];
  /** Downloads filter as [facet, minDownloads] */
  downloads?: [string, number];
}

/** Modrinth filter facets for searching shaders.
 * ```ts
 * export interface ShaderFilterFacets {
 *   query?: string;
 *   loaders?: [string, string][];
 *   categories?: [string, string][];
 *   game_versions?: string[];
 * }
 * ```
 */
export interface ShaderFilterFacets {
  /** Free-text search query */
  query?: string;
  /** Array of [operation, value] pairs for loaders (iris, optifine, canvas, vanilla) */
  loaders?: [string, string][];
  /** Array of [operation, value] pairs for categories (performance, features, style) */
  categories?: [string, string][];
  /** Game versions to filter by */
  game_versions?: string[];
}

// Resource Pack Types

export interface ResourcePack {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string | null;
  file_path: string;
  file_name: string;
  file_size: number;
  compatible_versions: string[];
  pack_format: number;
  enabled: boolean;
  source_url: string | null;
  thumbnail: string | null;
  installed_date: number;
  last_used: number | null;
}

export interface ResourcePackDownload {
  id: string;
  name: string;
  author: string;
  description: string;
  download_url: string;
  thumbnail: string | null;
  gallery: string[] | null;
  featured_gallery: string | null;
  tags: string[];
  minecraft_versions: string[];
  resolution: string | null;
  rating: number;
  downloads: number;
  size_mb: number;
  source: "Modrinth" | "CurseForge" | { Other: string };
}

/** Modrinth filter facets for searching resource packs.
 * ```ts
 * export interface ResourcePackFilterFacets {
 *   query?: string;
 *   categories?: [string, string][];
 *   game_versions?: string[];
 * }
 * ```
 */
export interface ResourcePackFilterFacets {
  /** Free-text search query */
  query?: string;
  /** Array of [operation, value] pairs for categories (resolutions, styles, etc) */
  categories?: [string, string][];
  /** Game versions to filter by */
  game_versions?: string[];
}

/** Modrinth mod project info.
 * ```ts
 * export interface ModrinthInfo {
 *   project_id: string;
 *   project_type: string;
 *   slug: string;
 *   title: string;
 *   description: string;
 *   author: string;
 *   categories: string[];
 *   display_categories: string[];
 *   versions: string[];
 *   downloads: number;
 *   follows?: number;
 *   icon_url?: string;
 *   date_created?: string;
 *   date_modified?: string;
 *   latest_version?: string;
 *   license?: string;
 *   client_side?: string;
 *   server_side?: string;
 *   gallery?: string[];
 *   featured_gallery?: string;
 *   color?: number;
 *   body?: string;
 *   additional_categories?: string[];
 *   issues_url?: string;
 *   source_url?: string;
 *   wiki_url?: string;
 *   discord_url?: string;
 *   donation_urls?: DonationUrl[];
 *   published?: string;
 *   updated?: string;
 *   approved?: string;
 *   owner?: string;
 *   team?: string;
 *   host?: string;
 *   license_obj?: ModrinthLicense;
 *   versions_obj?: ModrinthVersion[];
 *   game_versions?: string[];
 *   loaders?: string[];
 *   featured?: boolean;
 *   published_by?: string;
 *   approved_by?: string;
 *   moderation_message?: ModerationMessage;
 *   moderation_message_type?: string;
 * }
 * ```
 */
export interface ModrinthInfo {
  project_id: string;
  project_type: string;
  slug: string;
  title: string;
  description: string;
  author: string;
  categories: string[];
  display_categories: string[];
  versions: string[];
  downloads: number;
  follows?: number;
  icon_url?: string;
  date_created?: string;
  date_modified?: string;
  latest_version?: string;
  license?: string;
  client_side?: string;
  server_side?: string;
  gallery?: string[];
  featured_gallery?: string;
  color?: number;
  body?: string;
  additional_categories?: string[];
  issues_url?: string;
  source_url?: string;
  wiki_url?: string;
  discord_url?: string;
  donation_urls?: DonationUrl[];
  published?: string;
  updated?: string;
  approved?: string;
  owner?: string;
  team?: string;
  host?: string;
  license_obj?: ModrinthLicense;
  versions_obj?: ModrinthVersion[];
  game_versions?: string[];
  loaders?: string[];
  featured?: boolean;
  published_by?: string;
  approved_by?: string;
  moderation_message?: ModerationMessage;
  moderation_message_type?: string;
}

/** CurseForge mod project info.
 * ```ts
 * export interface CurseForgeInfo {
 *   id: number;
 *   game_id: number;
 *   name: string;
 *   slug: string;
 *   links: CurseForgeModLinks;
 *   summary: string;
 *   status: number;
 *   download_count: number;
 *   is_featured: boolean;
 *   primary_category_id: number;
 *   categories: CurseForgeCategory[];
 *   class_id?: number;
 *   authors: CurseForgeAuthor[];
 *   logo?: CurseForgeAsset;
 *   screenshots: CurseForgeAsset[];
 *   main_file_id: number;
 *   latest_files: CurseForgeFile[];
 *   latest_files_indexes: CurseForgeFileIndex[];
 *   date_created: string;
 *   date_modified: string;
 *   date_released?: string;
 *   allow_mod_distribution?: boolean;
 *   game_popularity_rank: number;
 *   is_available: boolean;
 *   thumbs_up_count: number;
 *   rating?: number;
 * }
 * ```
 */
export interface CurseForgeInfo {
  id: number;
  game_id: number;
  name: string;
  slug: string;
  links: CurseForgeModLinks;
  summary: string;
  status: number;
  download_count: number;
  is_featured: boolean;
  primary_category_id: number;
  categories: CurseForgeCategory[];
  class_id?: number;
  authors: CurseForgeAuthor[];
  logo?: CurseForgeAsset;
  screenshots: CurseForgeAsset[];
  main_file_id: number;
  latest_files: CurseForgeFile[];
  latest_files_indexes: CurseForgeFileIndex[];
  date_created: string;
  date_modified: string;
  date_released?: string;
  allow_mod_distribution?: boolean;
  game_popularity_rank: number;
  is_available: boolean;
  thumbs_up_count: number;
  rating?: number;
}

/** CurseForge mod links.
 * ```ts
 * export interface CurseForgeModLinks {
 *   website_url?: string;
 *   wiki_url?: string;
 *   issues_url?: string;
 *   source_url?: string;
 * }
 * ```
 */
export interface CurseForgeModLinks {
  website_url?: string;
  wiki_url?: string;
  issues_url?: string;
  source_url?: string;
}

/** CurseForge category info.
 * ```ts
 * export interface CurseForgeCategory {
 *   id: number;
 *   game_id: number;
 *   name: string;
 *   slug: string;
 *   url: string;
 *   icon_url: string;
 *   date_modified: string;
 *   is_class: boolean;
 *   class_id?: number;
 *   parent_category_id?: number;
 *   display_index: number;
 * }
 * ```
 */
export interface CurseForgeCategory {
  id: number;
  game_id: number;
  name: string;
  slug: string;
  url: string;
  icon_url: string;
  date_modified: string;
  is_class: boolean;
  class_id?: number;
  parent_category_id?: number;
  display_index: number;
}

/** CurseForge author info.
 * ```ts
 * export interface CurseForgeAuthor {
 *   id: number;
 *   name: string;
 *   url: string;
 * }
 * ```
 */
export interface CurseForgeAuthor {
  id: number;
  name: string;
  url: string;
}

/** CurseForge mod asset (logo, screenshots).
 * ```ts
 * export interface CurseForgeAsset {
 *   id: number;
 *   mod_id: number;
 *   title: string;
 *   description: string;
 *   thumbnail_url: string;
 *   url: string;
 * }
 * ```
 */
export interface CurseForgeAsset {
  id: number;
  mod_id: number;
  title: string;
  description: string;
  thumbnail_url: string;
  url: string;
}

/** CurseForge file info.
 * ```ts
 * export interface CurseForgeFile {
 *   id: number;
 *   game_id: number;
 *   mod_id: number;
 *   is_available: boolean;
 *   display_name: string;
 *   file_name: string;
 *   release_type: number;
 *   file_status: number;
 *   hashes: CurseForgeFileHash[];
 *   file_date: string;
 *   file_length: number;
 *   download_count: number;
 *   download_url: string;
 *   game_versions: string[];
 *   sortable_game_versions: CurseForgeSortableGameVersion[];
 *   dependencies: CurseForgeModDependency[];
 *   alternate_file_id?: number;
 *   is_server_pack: boolean;
 *   server_pack_file_id?: number;
 *   is_early_access_content: boolean;
 *   early_access_end_date?: string;
 *   file_fingerprint: number;
 *   modules: CurseForgeModuleFingerprint[];
 * }
 * ```
 */
export interface CurseForgeFile {
  id: number;
  game_id: number;
  mod_id: number;
  is_available: boolean;
  display_name: string;
  file_name: string;
  release_type: number;
  file_status: number;
  hashes: CurseForgeFileHash[];
  file_date: string;
  file_length: number;
  download_count: number;
  download_url: string;
  game_versions: string[];
  sortable_game_versions: CurseForgeSortableGameVersion[];
  dependencies: CurseForgeModDependency[];
  alternate_file_id?: number;
  is_server_pack: boolean;
  server_pack_file_id?: number;
  is_early_access_content: boolean;
  early_access_end_date?: string;
  file_fingerprint: number;
  modules: CurseForgeModuleFingerprint[];
}

/** CurseForge file index.
 * ```ts
 * export interface CurseForgeFileIndex {
 *   game_version: string;
 *   file_id: number;
 *   filename: string;
 *   release_type: number;
 *   game_version_type_id?: number;
 *   mod_loader?: number;
 * }
 * ```
 */
export interface CurseForgeFileIndex {
  game_version: string;
  file_id: number;
  filename: string;
  release_type: number;
  game_version_type_id?: number;
  mod_loader?: number;
}

/** CurseForge file hash.
 * ```ts
 * export interface CurseForgeFileHash {
 *   value: string;
 *   algo: number;
 * }
 * ```
 */
export interface CurseForgeFileHash {
  value: string;
  algo: number;
}

/** CurseForge sortable game version.
 * ```ts
 * export interface CurseForgeSortableGameVersion {
 *   game_version_name: string;
 *   game_version_padded: string;
 *   game_version: string;
 *   game_version_release_date: string;
 *   game_version_type_id?: number;
 * }
 * ```
 */
export interface CurseForgeSortableGameVersion {
  game_version_name: string;
  game_version_padded: string;
  game_version: string;
  game_version_release_date: string;
  game_version_type_id?: number;
}

/** CurseForge mod dependency.
 * ```ts
 * export interface CurseForgeModDependency {
 *   mod_id: number;
 *   relation_type: number;
 * }
 * ```
 */
export interface CurseForgeModDependency {
  mod_id: number;
  relation_type: number;
}

/** CurseForge module fingerprint.
 * ```ts
 * export interface CurseForgeModuleFingerprint {
 *   foldername: string;
 *   fingerprints: number[];
 * }
 * ```
 */
export interface CurseForgeModuleFingerprint {
  foldername: string;
  fingerprints: number[];
}

/** CurseForge filter for mod searches.
 * ```ts
 * export interface CurseForgeFilter {
 *   query?: string;
 *   category_id?: number;
 *   game_version?: string;
 *   mod_loader_type?: number;
 *   sort_field?: number;
 *   sort_order?: number;
 * }
 * ```
 */
export interface CurseForgeFilter {
  query?: string;
  category_id?: number;
  game_version?: string;
  mod_loader_type?: number;
  sort_field?: number;
  sort_order?: number;
}

/** Modrinth mod version info.
 * ```ts
 * export interface ModrinthVersion {
 *   id: string;
 *   name: string;
 *   version_number: string;
 *   changelog?: string;
 *   files: ModrinthFile[];
 *   game_versions: string[];
 *   loaders: string[];
 * }
 * ```
 */
export interface ModrinthVersion {
  id: string;
  name: string;
  version_number: string;
  changelog?: string;
  files: ModrinthFile[];
  game_versions: string[];
  loaders: string[];
}

/** File info for a Modrinth mod version.
 * ```ts
 * export interface ModrinthFile {
 *   url: string;
 *   filename: string;
 *   primary: boolean;
 *   hashes: Record<string, string>;
 *   size: number;
 * }
 * ```
 */
export interface ModrinthFile {
  url: string;
  filename: string;
  primary: boolean;
  hashes: Record<string, string>;
  size: number;
}

/** Donation URL for a Modrinth project.
 * ```ts
 * export interface DonationUrl {
 *   id: string;
 *   platform: string;
 *   url: string;
 * }
 * ```
 */
export interface DonationUrl {
  id: string;
  platform: string;
  url: string;
}

/** License info for a Modrinth project.
 * ```ts
 * export interface ModrinthLicense {
 *   id: string;
 *   name: string;
 *   url?: string;
 * }
 * ```
 */
export interface ModrinthLicense {
  id: string;
  name: string;
  url?: string;
}

/** Moderation message for a Modrinth project.
 * ```ts
 * export interface ModerationMessage {
 *   message: string;
 *   body?: string;
 * }
 * ```
 */
export interface ModerationMessage {
  message: string;
  body?: string;
}

/** Extended mod information including jar info and additional metadata.
 * ```ts
 * export interface ExtendedModInfo {
 *   mod_jar_info: ModJarInfo;
 *   page_uri: string | null;
 *   icon_uri: string | null;
 *   description: string | null;
 *   authors: string[];
 * }
 * ```
 */
export interface ExtendedModInfo {
  mod_jar_info: ModJarInfo;
  page_uri: string | null;
  icon_uri: string | null;
  description: string | null;
  authors: string[];
}

//|_____________________________________________________________________________|
//|                            Settings Event Types                             |
//|_____________________________________________________________________________|

/** Event payload for navigation events */
export interface NavigationEventPayload {
  reason: string;
}

/** Event payload for behavior choice requests */
export interface BehaviorChoiceEventPayload {
  options: string[];
  exit_code?: number;
}

/** Event payload for game restart requests */
export interface GameRestartEventPayload {
  exit_code: number;
}

//|_____________________________________________________________________________|
//|                              Skin Types                                     |
//|_____________________________________________________________________________|

/** Skin model types in Minecraft */
export type SkinModelType = "Classic" | "Slim";

/** Configuration for skin upload operations */
export interface SkinUploadConfig {
  model: SkinModelType;
  file_path: string;
}

/** Response from skin upload operation */
export interface SkinUploadResponse {
  success: boolean;
  message: string;
  model_used: SkinModelType;
}

/** Current skin information from Mojang API */
export interface CurrentSkin {
  model: SkinModelType;
  url?: string;
  has_skin: boolean;
}

/** Account skin from Microsoft/Mojang skin history */
export interface AccountSkin {
  id: string;
  name: string;
  url?: string;
  model: SkinModelType;
  is_current: boolean;
  uploaded_date?: number; // Unix timestamp
}
