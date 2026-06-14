// This file is auto-generated with `cd api-typegen && cargo run`. Do not edit directly.

export interface SymlinkInfo {
  source_path: string;
  target_path: string;
  enabled: boolean;
}

export interface CustomSymlinksConfig {
  symlinks: CustomSymlink[];
}

export interface CustomSymlink {
  id: string;
  source_path: string;
  target_path: string;
  enabled: boolean;
}

export interface SoundpackMetadata {
  id: string;
  name: string;
  description?: string;
  author?: string;
  version?: string;
  file_path?: string;
}

export interface SkinUploadResponse {
  success: boolean;
  message: string;
  model_used: SkinModel;
}

export type SkinModel = "classic" | "slim";

export interface SkinUploadConfig {
  model: SkinModel;
  file_path: string;
}

export interface PlayerProfile {
  id: string;
  name: string;
  skins: AccountSkin[];
  capes: AccountCape[];
}

export interface AccountCape {
  id: string;
  state: string;
  url?: string;
  alias?: string;
}

export interface AccountSkin {
  id: string;
  name: string;
  url?: string;
  model: SkinModel;
  is_current: boolean;
  uploaded_date?: number;
}

export interface CustomSkinsRoot {
  custom_skins: Record<string, CustomSkinEntry>;
  version?: number;
}

export interface CustomSkinEntry {
  cape_id: string;
  created: string;
  id: string;
  model_image: string;
  name: string;
  skin_image: string;
  slim: boolean;
  texture_id: string;
  updated: string;
}

export interface CurrentSkin {
  model: SkinModel;
  url?: string;
  has_skin: boolean;
}

export type ShaderSource =
  | { modrinth: "modrinth" }
  | { curse_forge: "curse_forge" }
  | { other: string };

export interface ShaderSettings {
  quality: ShaderQuality;
  shadows: boolean;
  shadow_resolution: number;
  anti_aliasing: boolean;
  bloom: boolean;
  motion_blur: boolean;
  custom_settings: Record<string, string>;
}

export type ShaderQuality = "low" | "medium" | "high" | "ultra" | "custom";

export interface ShaderPack {
  id: string;
  name: string;
  version: string;
  author: string;
  description?: string;
  file_path: string;
  file_name: string;
  file_size: number;
  compatible_versions: string[];
  enabled: boolean;
  source_url?: string;
  thumbnail?: string;
  shader_loader: ShaderLoader;
  installed_date: number;
  last_used?: number;
}

export type ShaderLoader = "canvas" | "iris" | "opti_fine" | "vanilla";

export interface ShaderFilterFacets {
  query?: string;
  loaders?: [string, string][];
  categories?: [string, string][];
  game_versions?: string[];
}

export interface ShaderDownload {
  id: string;
  name: string;
  author: string;
  description: string;
  download_url: string;
  thumbnail?: string;
  gallery?: string[];
  featured_gallery?: string;
  tags: string[];
  minecraft_versions: string[];
  shader_loader: ShaderLoader;
  rating: number;
  downloads: number;
  size_mb: number;
  source: ShaderSource;
}

export interface SoundSettings {
  enabled: boolean;
  music_enabled: boolean;
  master_volume: number;
  sound_volume: number;
  music_volume: number;
  selected_soundpack: string;
}

export interface NetworkSettings {
  use_proxy: boolean;
}

export interface MiscSettings {
  check_for_updates_on_start: boolean;
}

export interface MinecraftDirectoryInfo {
  path: string;
  exists: boolean;
}

export interface LoggingSettings {
  enabled: boolean;
}

export interface GeneralSettings {
  java_path?: string;
  game_directory?: string;
  on_game_close: string;
  on_game_crash: string;
  on_game_launch: string;
  auto_update_launcher: boolean;
  show_ads: boolean;
  update_mode: string;
  update_notification_style: string;
}

export interface ContentSettings {
  allow_adult_content: boolean;
}

export interface CategorizedLauncherSettings {
  general: GeneralSettings;
  appearance: AppearanceSettings;
  logging: LoggingSettings;
  network: NetworkSettings;
  content: ContentSettings;
  advanced: AdvancedSettings;
  misc: MiscSettings;
}

export interface AdvancedSettings {
  developer_mode: boolean;
}

export interface AppearanceSettings {
  selected_css_theme: string;
}

export type ResourcePackSource =
  | { modrinth: "modrinth" }
  | { curse_forge: "curse_forge" }
  | { other: string };

export interface ResourcePackFilterFacets {
  query?: string;
  categories?: [string, string][];
  game_versions?: string[];
}

export interface ResourcePackDownload {
  id: string;
  name: string;
  author: string;
  description: string;
  download_url: string;
  thumbnail?: string;
  gallery?: string[];
  featured_gallery?: string;
  tags: string[];
  minecraft_versions: string[];
  resolution?: string;
  rating: number;
  downloads: number;
  size_mb: number;
  source: ResourcePackSource;
}

export interface ResourcePack {
  id: string;
  name: string;
  version: string;
  author: string;
  description?: string;
  file_path: string;
  file_name: string;
  file_size: number;
  compatible_versions: string[];
  pack_format: number;
  enabled: boolean;
  source_url?: string;
  thumbnail?: string;
  installed_date: number;
  last_used?: number;
}

export type Versions = VersionData[];

export interface VersionData {
  version_id: string;
  loader: LoaderKind;
  display_name: string;
  is_stable: boolean;
  extra?: Record<string, string>;
}

export type LoaderKind = "vanilla" | "fabric" | "iris_fabric" | "forge" | "neo_forge" | "quilt";

export interface ShaderPackInfo {
  file_name: string;
  name?: string;
  description?: string;
  disabled: boolean;
}

export interface ResourcePackInfo {
  file_name: string;
  name?: string;
  description?: string;
  disabled: boolean;
}

export interface PackFileInfo {
  path: string;
  file_size: number;
  hashes: Record<string, string>;
  downloads: string[];
  env?: MrpackEnv;
  already_installed: boolean;
  overwrite: boolean;
}

export interface MrpackEnv {
  client?: string;
  server?: string;
}

export interface PackFileGroups {
  mods: PackFileInfo[];
  resourcepacks: PackFileInfo[];
  shaderpacks: PackFileInfo[];
  others: PackFileInfo[];
}

export interface PackFileDetailedGroup {
  disabled: PackFileInfo[];
  optional: PackFileInfo[];
  to_be_installed: PackFileInfo[];
}

export interface MrpackIndex {
  name: string;
  version_id: string;
  format_version: number;
  files: MrpackFile[];
}

export interface MrpackFile {
  path: string;
  file_size: number;
  hashes: Record<string, string>;
  downloads: string[];
  env?: MrpackEnv;
}

export interface MrPackDetailed {
  mods: PackFileDetailedGroup;
  resourcepacks: PackFileDetailedGroup;
  shaderpacks: PackFileDetailedGroup;
}

export interface ModJarInfo {
  file_name: string;
  mod_name?: string;
  mod_version?: string;
  loader?: string;
  disabled: boolean;
}

export interface KableProfile {
  id: string;
  name: string;
  icon?: string;
  version_id: string;
  created: string;
  last_used: string;
  java_args: string[];
  dedicated_mods_folder?: string;
  dedicated_resource_pack_folder?: string;
  dedicated_shaders_folder?: string;
  dedicated_config_folder?: string;
  favorite: boolean;
  total_time_played_ms: number;
  parameters_map: Record<string, string>;
  description?: string;
  times_launched: number;
  enable_pack_merging: boolean;
  pack_order: string[];
  merged_packs: string[];
}

export interface SkinData {
  id: string;
  state: string;
  url: string;
  variant: string;
  alias?: string;
}

export interface CapeData {
  id: string;
  state: string;
  url: string;
  alias?: string;
}

export type SearchIndex = "relevance" | "downloads" | "follows" | "newest" | "updated";

export interface ProjectSearch {
  query?: string;
  facets: FacetGroup[];
  index?: SearchIndex;
  offset?: number;
  limit?: number;
}

export interface FacetGroup {
  facets: Facet[];
}

export interface Facet {
  field: FacetField;
  operator: FacetOperator;
  value: string;
}

export type FacetOperator = "eq" | "not_eq" | "greater" | "greater_eq" | "less" | "less_eq";

export type FacetField = "project_type" | "category" | "version" | "client_side" | "server_side" | "open_source" | "title" | "author" | "follows" | "project_id" | "license" | "downloads" | "color" | "created_timestamp" | "modified_timestamp" | "date_created" | "date_modified";

export type WorldSource =
  | { local: "local" }
  | { remote: "remote" }
  | { other: string };

export interface WorldDownload {
  name: string;
  url: string;
  version?: string;
}

export interface LocalWorld {
  name: string;
  path: string;
  source: WorldSource;
}

export type GameMode = "survival" | "creative" | "adventure" | "spectator";

export type Difficulty = "peaceful" | "easy" | "normal" | "hard";

export interface LaunchResult {
  pid: number;
  command: string;
}

export interface IconSettings {
  custom_templates: CustomIconTemplate[];
}

export interface CustomIconTemplate {
  id: string;
  name: string;
  description?: string;
  svg_data: string;
  preview_svg?: string;
  created_at?: number;
  updated_at?: number;
}

export interface PresenceState {
  state: string;
  details: string;
  priority: ActivityPriority;
  large_image?: string;
  large_text?: string;
  small_image?: string;
  small_text?: string;
  start_timestamp?: number;
}

export type ActivityPriority = "idle" | "browsing" | "managing" | "playing";

export interface MinecraftProfile {
  id: string;
  name: string;
  requires_profile_name_change: boolean;
  requires_skin_change: boolean;
}

export interface MicrosoftToken {
  access_token: string;
  expires_at: unknown;
  /**
   * Raw refresh token from device code flow. Should be encrypted if stored persistently!!
   */
  refresh_token?: string;
}

export interface LauncherAccountsJson {
  accounts: Record<string, LauncherAccount>;
  active_account_local_id: string;
  mojang_client_token: string;
}

export interface LauncherAccount {
  access_token: string;
  access_token_expires_at: string;
  /**
   * AES-encrypted refresh token for "persistent" accounts.
   */
  encrypted_refresh_token?: string;
  avatar: string;
  eligible_for_free_trials: boolean;
  eligible_for_migration: boolean;
  franchise_inventory_id: string;
  has_multiple_profiles: boolean;
  in_forced_migration: boolean;
  legacy: boolean;
  license_product_ids: string[];
  local_id: string;
  minecraft_profile: MinecraftProfile;
  persistent: boolean;
  remote_id: string;
  account_type: string;
  user_properties: unknown[];
  username: string;
}

export interface DeviceCodeResponse {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
}

