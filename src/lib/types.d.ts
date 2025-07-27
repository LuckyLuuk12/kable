export interface MinecraftInstallation {
  id: string;
  name: string;
  path: string;
  version: string;
  is_valid: boolean;
  mod_loader: 'vanilla' | 'fabric' | 'forge' | 'quilt' | 'neoforge'; // Backend uses mod_loader
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
  type: 'vanilla' | 'fabric' | 'forge' | 'quilt' | 'neoforge';
  modLoader: 'vanilla' | 'fabric' | 'forge' | 'quilt' | 'neoforge';
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
  user_properites: any[]; // Note: backend keeps the typo from official launcher
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
  modLoaderType: 'vanilla' | 'fabric' | 'forge' | 'quilt' | 'neoforge';
  loaderVersion?: string;
  modsList?: Array<{
    name: string;
    fileName: string;
    enabled: boolean;
  }>;
}

// Window State Manager Types
export interface WindowState {
  width: number;
  height: number;
  x: number;
  y: number;
  maximized: boolean;
  fullscreen: boolean;
  monitor_name?: string;
  monitor_position?: [number, number];
  monitor_size?: [number, number];
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
  type: 'release' | 'snapshot' | 'old_beta' | 'old_alpha';
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
  path: string;
  last_played?: number;  // Changed to number (timestamp)
  size?: number;
  size_mb: number;  // Size in MB for compatibility
  version?: string;
  game_mode?: string;
  hardcore?: boolean;
  difficulty?: string;  // Added difficulty for world stats
  folder_name: string;  // Folder name for searching
  created: number;  // Creation timestamp
  backup_count: number;  // Number of backups for this world
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
  path: string;
  version?: string;
  description?: string;
  compatible_versions?: string[];
  // Additional properties for shader management
  file_size: number;  // File size in bytes
  enabled: boolean;  // Whether shader is enabled
  shader_loader: string;  // Shader loader type (OptiFine, Iris, etc.)
  installed_date: number;  // Installation timestamp
  author: string;  // Author name
  file_name: string;  // File name for searching
}

export interface ShaderDownload {
  id: string;
  name: string;
  url: string;
  description?: string;
  compatible_versions?: string[];
}

export interface ShaderSettings {
  enabled: boolean;
  current_pack?: string;
  quality_preset?: string;
  enable_caching?: boolean;
}

export interface MinecraftSkin {
  id: string;
  name: string;
  url?: string;
  type: 'steve' | 'alex';
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
  type: 'steve' | 'alex';
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
  source: 'Modrinth' | 'CurseForge' | 'Local';
  source_id: string;
  file_path: string;
  minecraft_version: string;
  mod_loader: 'Fabric' | 'Forge' | 'Quilt' | 'NeoForge';
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
  client_side: 'Required' | 'Optional' | 'Unsupported';
  server_side: 'Required' | 'Optional' | 'Unsupported';
  downloads: number;
  icon_url?: string;
  source: 'Modrinth' | 'CurseForge' | 'Other';
}

export interface ModVersion {
  id: string;
  version_number: string;
  version_type: 'Release' | 'Beta' | 'Alpha';
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
  status: 'launching' | 'running' | 'crashed' | 'stopped' | 'closed';
  exitCode?: number;
  processId?: number;
  restartAttempts?: number;
}

export interface LogEntry {
  timestamp: Date;
  level: 'debug' | 'info' | 'warn' | 'error';
  source: 'launcher' | 'game';
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
  on_game_close: 'open_logs' | 'open_home' | 'exit' | 'minimize' | 'ask';
  /** What to do when the game crashes */
  on_game_crash: 'restart' | 'close' | 'ask';
  /** Whether to keep the launcher open after launching the game */
  on_game_launch: 'keep_open' | 'close_launcher' | 'open_logs' | 'ask';
  /** Whether to automatically check for updates on startup */
  auto_update_launcher: boolean;
  /** Whether to show ads; I am a nice guy, no paid subscription needed to disable ads */
  show_ads: boolean;
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
  iconType: 'emoji' | 'fontawesome' | 'css' | 'svg' | 'image';
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
 * }
 * ```
 */
export interface AppearanceSettings {
  /** The theme to use for the launcher */
  theme: 'light' | 'dark' | 'system';
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
  log_file_size_limit_mb: number | 'disabled';
  /** The number of days to keep log files */
  log_retention_days: number | 'disabled';
  /** Whether to try to "merge" log tabs into one tab if they are from the same game instance (but a different launch) */
  merge_log_tabs: boolean;
  /** Which log levels are shown by default */
  default_log_levels: ('debug' | 'info' | 'warn' | 'error')[];
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
  download_speed_limit: number | 'unlimited';
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
  max_world_backups: number | 'disabled';
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
  auto_save_interval: number | 'disabled'; // in seconds, 'disabled' means no auto save
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
  auth_preference: 'code' | 'device_code';
}

// _____________________________________________________________________________
//|                                                                             |
//|                            Installation Types                               |
//|_____________________________________________________________________________|


/** LoaderKind enum (matches backend LoaderKind) */
export type LoaderKind =
  | 'Vanilla'
  | 'Fabric'
  | 'IrisFabric'
  | 'Forge'
  | 'NeoForge'
  | 'Quilt';

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
  dedicated_resource_pack_folder?: string | null;
  dedicated_shaders_folder?: string | null;
  favorite: boolean;
  total_time_played_ms: number;
  parameters_map: Record<string, string>;
  description?: string | null;
  times_launched: number;
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