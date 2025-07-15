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
  accessToken?: string;
  accessTokenExpiresAt?: string; // ISO date string
  avatar?: string; // base64 encoded image
  eligibleForFreeTrials?: boolean;
  eligibleForMigration?: boolean;
  franchiseInventoryId?: string;
  hasMultipleProfiles?: boolean;
  inForcedMigration?: boolean;
  legacy?: boolean;
  licenseProductIds?: string[];
  localId: string; // Required for identification
  minecraftProfile?: {
    id?: string;
    name?: string;
    requiresProfileNameChange?: boolean;
    requiresSkinChange?: boolean;
  };
  persistent?: boolean;
  remoteId?: string;
  type?: string; // Usually "Xbox"
  // Support both correct and typo versions
  userProperties?: any[];
  userProperites?: any[];
  username?: string;
}

export interface LauncherAccountsJson {
  accounts?: Record<string, LauncherAccount>;
  activeAccountLocalId?: string;
  mojangClientToken?: string;
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

export interface LauncherSettings {
  java_path?: string;
  jvm_args: string;
  memory: number;
  game_directory?: string;
  keep_launcher_open: boolean;
  window_width?: number;
  window_height?: number;
  fullscreen?: boolean;
  
  // Additional properties used in settings
  theme?: string;
  language?: string;
  minecraft_path?: string;
  default_memory?: number;
  max_memory?: number;
  close_launcher_on_game_start?: boolean;
  auto_update_launcher?: boolean;
  show_logs_on_launch?: boolean;
  enable_experimental_features?: boolean;
  parallel_downloads?: number;
  connection_timeout?: number;
  animation_speed?: string;
  card_spacing?: number;
  sidebar_width?: number;
  auto_backup_worlds?: boolean;
  max_world_backups?: number;
  shader_quality_preset?: string;
  enable_shader_caching?: boolean;
  
  // Icon settings
  selected_icon_template?: string;
  icon_settings?: IconSettings;
  
  custom?: any; // For custom user settings
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

// Icon template definitions
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

export interface IconSettings {
  selectedTemplate: string; // The currently selected template name
  customTemplates: CustomIconTemplate[]; // User-uploaded templates
  builtinTemplates: string[]; // Available built-in templates (emoji, fontawesome)
}