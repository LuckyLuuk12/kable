export interface MinecraftInstallation {
  path: string;
  version: string;
  is_valid: boolean;
  type: 'vanilla' | 'fabric' | 'forge' | 'quilt' | 'neoforge';
  loader_version?: string;
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

// Microsoft Account Management
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
}

// Version Management
export interface MinecraftVersion {
  id: string;
  type: 'release' | 'snapshot' | 'old_beta' | 'old_alpha';
  url: string;
  time: string;
  releaseTime: string;
  sha1?: string;
  complianceLevel?: number;
}

export interface ModLoader {
  name: 'fabric' | 'forge' | 'quilt' | 'neoforge';
  version: string;
  minecraft_version: string;
  stable: boolean;
}

// Mod/Modpack Management
export interface ModrinthProject {
  id: string;
  slug: string;
  title: string;
  description: string;
  categories: string[];
  client_side: 'required' | 'optional' | 'unsupported';
  server_side: 'required' | 'optional' | 'unsupported';
  downloads: number;
  icon_url?: string;
  versions: ModrinthVersion[]; // Changed from string[] to full version objects
}

export interface ModrinthVersion {
  id: string;
  version_number: string;
  version_type: 'release' | 'beta' | 'alpha';
  minecraft_versions: string[];
  mod_loaders: string[];
  date_published: string;
  downloads: number;
  changelog?: string;
  files: {
    url: string;
    filename: string;
    size: number;
    sha1: string;
  }[];
}

export interface CurseForgeProject {
  id: number;
  name: string;
  summary: string;
  downloadCount: number;
  iconUrl?: string;
  gameVersions: string[];
  categories: { id: number; name: string }[];
  versions: CurseForgeVersion[]; // Added version support
}

export interface CurseForgeVersion {
  id: number;
  fileId: number;
  fileName: string;
  displayName: string;
  releaseType: 'release' | 'beta' | 'alpha';
  gameVersions: string[];
  fileDate: string;
  downloadUrl: string;
  fileLength: number;
  changelog?: string;
}

export interface ModpackFile {
  id: string;
  name: string;
  current_version: string; // Renamed for clarity
  source: 'modrinth' | 'curseforge' | 'local';
  source_id: string; // Project ID from the source
  file_path: string;
  minecraft_version: string;
  mod_loader: ModLoader['name'];
  enabled: boolean;
  dependencies?: string[];
  
  // Version management
  available_versions?: ModVersion[]; // Cached available versions
  can_update: boolean;
  latest_version?: string;
  auto_update: boolean;
  version_type_preference: 'release' | 'beta' | 'alpha'; // User preference for update type
}

export interface ModVersion {
  version: string;
  version_type: 'release' | 'beta' | 'alpha';
  minecraft_versions: string[];
  mod_loaders: string[];
  date_published: string;
  downloads: number;
  changelog?: string;
  file_size: number;
  compatible: boolean; // Whether it's compatible with current profile
}

// Version Update System
export interface ModUpdateCheck {
  mod_id: string;
  current_version: string;
  latest_version?: string;
  available_versions: ModVersion[];
  has_update: boolean;
  update_type?: 'major' | 'minor' | 'patch';
  breaking_changes?: boolean;
}

export interface ModUpdateTask {
  id: string;
  mod_id: string;
  from_version: string;
  to_version: string;
  status: 'pending' | 'downloading' | 'installing' | 'completed' | 'failed';
  progress: number;
  error?: string;
}

export interface Modpack {
  id: string;
  name: string;
  version: string;
  minecraft_version: string;
  mod_loader: ModLoader['name'];
  loader_version: string;
  mods: ModpackFile[];
  created_at: number;
  last_played?: number;
  icon?: string;
  description?: string;
}

// Shader Management
export interface ShaderPack {
  id: string;
  name: string;
  version: string;
  author: string;
  description?: string;
  file_path: string;
  compatible_versions: string[];
  enabled: boolean;
  source_url?: string;
  thumbnail?: string;
}

// World Management
export interface MinecraftWorld {
  id: string;
  name: string;
  folder_name: string;
  game_mode: 'survival' | 'creative' | 'adventure' | 'spectator';
  difficulty: 'peaceful' | 'easy' | 'normal' | 'hard';
  version: string;
  size_mb: number;
  last_played: number;
  created: number;
  seed?: string;
  icon?: string;
  backup_count: number;
}

export interface WorldDownload {
  id: string;
  title: string;
  description: string;
  author: string;
  download_url: string;
  thumbnail?: string;
  tags: string[];
  minecraft_version: string;
  size_mb: number;
  rating: number;
  downloads: number;
  source: 'planetminecraft' | 'other';
}

// Settings System
export interface LauncherSettings {
  // General
  theme: 'light' | 'dark' | 'auto';
  language: string;
  
  // Minecraft
  minecraft_path?: string;
  default_memory: number;
  max_memory: number;
  java_path?: string;
  
  // Launcher Behavior
  keep_launcher_open: boolean;
  show_logs_on_launch: boolean;
  auto_update_launcher: boolean;
  close_launcher_on_game_start: boolean;
  
  // UI/UX
  window_width: number;
  window_height: number;
  sidebar_width: number;
  card_spacing: number;
  animation_speed: 'slow' | 'normal' | 'fast' | 'disabled';
  
  // Advanced
  parallel_downloads: number;
  connection_timeout: number;
  enable_experimental_features: boolean;
  
  // Custom settings (for future extensibility)
  custom: Record<string, any>;
}

// Profile System
export interface GameProfile {
  id: string;
  name: string;
  minecraft_version: string;
  mod_loader?: ModLoader['name'];
  loader_version?: string;
  modpack_id?: string;
  java_path?: string;
  jvm_args: string[];
  memory: number;
  resolution?: { width: number; height: number };
  game_directory?: string;
  icon?: string;
  last_played?: number;
  play_time: number; // in minutes
}

// Launch System
export interface LaunchInstance {
  id: string;
  profile_id: string;
  account_id: string;
  pid?: number;
  status: 'preparing' | 'downloading' | 'launching' | 'running' | 'crashed' | 'stopped';
  started_at: number;
  logs: string[];
  crash_report?: string;
}

// Download System
export interface DownloadTask {
  id: string;
  type: 'minecraft' | 'mod' | 'modpack' | 'shader' | 'world' | 'java';
  name: string;
  progress: number; // 0-100
  status: 'pending' | 'downloading' | 'completed' | 'failed' | 'cancelled';
  total_size?: number;
  downloaded_size: number;
  speed?: number; // bytes per second
  eta?: number; // seconds
  error?: string;
}

// Future Mod Settings (for feature #8)
export interface ModConfiguration {
  mod_id: string;
  mod_name: string;
  config_file_path: string;
  settings: ModConfigSection[];
}

export interface ModConfigSection {
  name: string;
  description?: string;
  settings: ModConfigOption[];
}

export interface ModConfigOption {
  key: string;
  name: string;
  description?: string;
  type: 'boolean' | 'number' | 'string' | 'enum' | 'range';
  value: any;
  default_value: any;
  options?: string[]; // for enum type
  min?: number; // for number/range type
  max?: number; // for number/range type
  step?: number; // for range type
  required?: boolean;
}

export interface AppSettings {
  minecraft_path?: string;
  default_memory: number;
  java_path?: string;
  keep_launcher_open: boolean;
}