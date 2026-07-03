// This file is auto-generated with `cd api-typegen && cargo run`. Do not edit directly.

export interface UpdateData {
  version: string;
  date?: string;
  body: string;
  current_version: string;
}

export interface GitHubRelease {
  tag_name: string;
  name: string;
  prerelease: boolean;
  draft: boolean;
  body: string;
}

export interface SymlinkCreateRequest {
  source: string;
  destination: string;
}

export interface Symlink {
  id: string;
  source: unknown;
  destination: unknown;
  is_temporary: boolean;
  enabled: boolean;
  from_launcher: boolean;
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

export type UpdateNotificationStyle = "notification" | "modal";

/**
 * This determines HOW, once checked, to perform the update.
 */
export type UpdateMode = "on_confirm" | "automatic" | "manual";

/**
 * This determines WHEN to check for updates
 */
export type UpdateDetection =
  | { on_startup: "on_startup" }
  | { on_close: "on_close" }
  | { periodically: number }
  | { manual: "manual" };

export type Theme =
  | { light: "light" }
  | { dark: "dark" }
  | { system: "system" }
  | { custom: string };

export interface SoundSettings {
  enabled: boolean;
  music_enabled: boolean;
  master_volume: number;
  sound_volume: number;
  music_volume: number;
  selected_soundpack: string;
}

export type OnGameAction =
  | { ask: "ask" }
  | { exit: "exit" }
  | { minimize: "minimize" }
  | { minimize_to_tray: "minimize_to_tray" }
  | { nothing: "nothing" }
  | { open: string }
  | { restart: "restart" };

export interface NetworkSettings {
  max_download_threads?: number;
  max_download_speed_kbps?: number;
  max_requests_per_second?: number;
}

export interface MiscSettings {
  /**
   * I keep this mostly undocumented because it is open-source and I don't want to spoil the fun features (:
   */
  enable_fun: boolean;
}

export interface LoggingSettings {
  enabled: boolean;
  persistent: boolean;
  compression: boolean;
  retention_days: number;
  max_file_size_mb: number;
  frontend_batch_size: number;
  frontend_batch_interval_ms: number;
  frontend_max_per_second: number;
  max_memory_logs: number;
  dedupe_enabled: boolean;
  dedupe_window_size: number;
}

export type Language = "english";

export type IconTemplate =
  | { default: "default" }
  | { icons: "icons" }
  | { font_awesome: "font_awesome" }
  | { svg: "svg" }
  | { custom: string };

export interface GeneralSettings {
  java_path?: string;
  game_directory?: string;
  on_game_close: OnGameAction;
  on_game_crash: OnGameAction;
  on_game_launch: OnGameAction;
  update_mode: UpdateMode;
  update_detection: UpdateDetection;
  update_notification_style: UpdateNotificationStyle;
}

export interface ContentSettings {
  allow_adult_content: boolean;
  allow_ads: boolean;
  enable_recommendations: boolean;
  enable_notifications: boolean;
}

export interface CategorizedLauncherSettings {
  general: GeneralSettings;
  appearance: AppearanceSettings;
  content: ContentSettings;
  logging: LoggingSettings;
  network: NetworkSettings;
  advanced: AdvancedSettings;
  misc: MiscSettings;
}

export interface AdvancedSettings {
  enable_advanced_features: boolean;
  enable_nightly_updates: boolean;
  developer_mode: boolean;
  extra: Record<string, string>;
}

export interface AppearanceSettings {
  theme: Theme;
  language: Language;
  icon_template: IconTemplate;
  custom_icon_templates: IconTemplate[];
  selected_css_theme?: string;
  sound_settings: SoundSettings;
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

/**
 * The release channel for this version
 */
export type VersionType = "release" | "beta" | "alpha" | "incomplete";

export interface VersionFileHashes {
  sha512?: string;
  sha1?: string;
}

export interface VersionFile {
  hashes: VersionFileHashes;
  /**
   * A direct link to the file
   */
  url: string;
  /**
   * The name of the file
   */
  filename: string;
  /**
   * Whether this file is the primary one for its version. Only a maximum of one file per version will have this set to true. If there are not any primary files, it can be inferred that the first file is the primary one.
   */
  primary: boolean;
  /**
   * The size of the file in bytes
   */
  size: number;
  /**
   * The type of the additional file, used mainly for adding resource packs to datapacks
   */
  file_type?: FileType | null;
}

export type FileType = "required_resource_pack" | "optional_resource_pack";

export interface VersionDependency {
  /**
   * The ID of the version that this version depends on
   */
  version_id?: string | null;
  /**
   * The ID of the project that this version depends on
   */
  project_id?: string | null;
  /**
   * The file name of the dependency, mostly used for showing external dependencies on modpacks
   */
  file_name?: string | null;
  /**
   * The type of dependency that this version has
   */
  dependency_type: DependencyType;
}

export type DependencyType = "required" | "optional" | "incompatible" | "embedded";

export interface UpdateMap {
  kable_project: KableProject;
  update?: Project;
}

export interface Project {
  /**
   * The slug of a project, used for vanity URLs. Regex: ```^[\\w!@$()`.+,"\\-']{3,64}$```
   */
  slug: string;
  /**
   * The title or name of the project
   */
  title: string;
  /**
   * A short description of the project
   */
  description: string;
  /**
   * A list of the categories that the project has
   */
  categories?: string[];
  /**
   * The client side support of the project
   */
  client_side: ClientSide;
  /**
   * The server side support of the project
   */
  server_side: ServerSide;
  /**
   * The project type of the project
   */
  project_type: ProjectType;
  /**
   * The total number of downloads of the project
   */
  downloads: number;
  /**
   * The URL of the project's icon
   */
  icon_url?: string | null;
  /**
   * The RGB color of the project, automatically generated from the project icon
   */
  color?: number | null;
  /**
   * The ID of the moderation thread associated with this project
   */
  thread_id?: string;
  monetization_status?: MonetizationStatus;
  /**
   * The ID of the project
   */
  project_id: string;
  /**
   * The username of the project's author
   */
  author: string;
  /**
   * A list of the categories that the project has which are not secondary
   */
  display_categories?: string[];
  /**
   * A list of the minecraft versions supported by the project
   */
  versions: ProjectVersion[];
  /**
   * The total number of users following the project
   */
  follows: number;
  /**
   * The date the project was added to search
   */
  date_created: string;
  /**
   * The date the project was last modified
   */
  date_modified: string;
  /**
   * The latest version of minecraft that this project supports
   */
  latest_version?: string;
  /**
   * The SPDX license ID of a project
   */
  license: string;
  /**
   * All gallery images attached to the project
   */
  gallery?: string[];
  /**
   * The featured gallery image of the project
   */
  featured_gallery?: string | null;
}

export interface ProjectVersion {
  /**
   * The name of this version
   */
  name: string;
  /**
   * The version number. Ideally will follow semantic versioning
   */
  version_number: string;
  /**
   * The changelog for this version
   */
  changelog?: string | null;
  /**
   * A list of specific versions of projects that this version depends on
   */
  dependencies?: VersionDependency[];
  /**
   * A list of versions of Minecraft that this version supports
   */
  game_versions: string[];
  /**
   * The release channel for this version
   */
  version_type: VersionType;
  /**
   * The mod loaders that this version supports
   */
  loaders: string[];
  /**
   * Whether the version is featured or not
   */
  featured: boolean;
  status?: Status;
  requested_status?: RequestedStatus | null;
  /**
   * The ID of the version, encoded as a base62 string
   */
  id: string;
  /**
   * The ID of the project this version is for
   */
  project_id: string;
  /**
   * The ID of the author who published this version
   */
  author_id: string;
  date_published: string;
  /**
   * The number of times this version has been downloaded
   */
  downloads: number;
  /**
   * A link to the changelog for this version. Always null, only kept for legacy compatibility.
   */
  changelog_url?: string | null;
  /**
   * A list of files available for download for this version
   */
  files: VersionFile[];
}

export type RequestedStatus = "listed" | "archived" | "draft" | "unlisted";

export type Status = "listed" | "archived" | "draft" | "unlisted" | "scheduled" | "unknown";

/**
 * The monetization status of the project
 */
export type MonetizationStatus = "monetized" | "demonetized" | "force_demonetized";

/**
 * The project type of the project
 */
export type ProjectType = "mod" | "modpack" | "resourcepack" | "shader";

/**
 * The server side support of the project
 */
export type ServerSide = "required" | "optional" | "unsupported";

export type ClientSide = "required" | "optional" | "unsupported";

/**
 * Represents a project in a profile's dedicated mods folder, including its metadata and whether it is enabled or disabled
 */
export interface KableProject {
  /**
   * The project itself, containing all of its metadata
   */
  project: Project;
  /**
   * Should match a ProjectVersion.id and indicate what version of the mod is installed in the profile's dedicated mods folder
   */
  version_id: string;
  /**
   * The filename of the mod jar file in the profile's dedicated mods folder, should match a ProjectVersion.files.filename, and is used to locate the mod jar file in the profile's dedicated mods folder
   */
  filename: string;
  /**
   * Whether the mod is enabled or disabled, when disabled the jar should be in the <dedicated_mods_folder>/disabled folder, otherwise it should be in the <dedicated_mods_folder> folder
   */
  enabled: boolean;
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

export type FacetField = "project_type" | "categories" | "version" | "client_side" | "server_side" | "open_source" | "title" | "author" | "follows" | "project_id" | "license" | "downloads" | "color" | "created_timestamp" | "modified_timestamp" | "date_created" | "date_modified";

export interface ModrinthResults {
  /**
   * The list of results
   */
  hits: Project[];
  /**
   * The number of results that were skipped by the query
   */
  offset: number;
  /**
   * The number of results that were returned by the query
   */
  limit: number;
  /**
   * The total number of results that match the query
   */
  total_hits: number;
}

export type Versions = ProfileVersion[];

export interface ProfileVersion {
  /**
   * Raw version ID from the profile, e.g. "1.19.2-forge-43.2.0"
   */
  id: string;
  display_name: string;
  /**
   * Vanilla, Fabric, Forge, NeoForge, Quilt, etc.
   */
  loader: LoaderKind;
  /**
   * Optional Minecraft version, e.g. "1.19.2", note that since 2026 minecraft versioning is <year>.<drop>.<patch> (e.g. 26.2.1)
   */
  minecraft_version?: string;
  /**
   * Optional loader version, e.g. "43.2.0" for forge or "0.14.19" for fabric
   */
  loader_version?: string;
  /**
   * Release, Snapshot, OldBeta, OldAlpha
   */
  version_type?: ProfileVersionType;
  /**
   * Whether this version is marked as stable in the profile, note that this is not necessarily the same as version_type == Release
   */
  stable?: boolean;
  /**
   * Extra metadata that may be present from the version manifest
   */
  release_time?: string;
  updated_time?: string;
  url?: string;
  sha1?: string;
  compliance_level?: number;
  recommended?: boolean;
}

export type ProfileVersionType = "release" | "snapshot" | "old_beta" | "old_alpha";

export type LoaderKind = "vanilla" | "fabric" | "iris_fabric" | "forge" | "neo_forge" | "quilt";

export interface Profile {
  created?: string;
  icon?: string;
  java_args?: string;
  last_used?: string;
  last_version_id?: string;
  name?: string;
  profile_type?: string;
}

export interface OfficialLauncherSettings {
  crash_assistance: boolean;
  enable_advanced: boolean;
  enable_analytics: boolean;
  enable_historical: boolean;
  enable_releases: boolean;
  enable_snapshots: boolean;
  keep_launcher_open: boolean;
  profile_sorting: string;
  show_game_log: boolean;
  show_menu: boolean;
  sound_on: boolean;
}

export interface ModJarInfo {
  file_name: string;
  mod_name?: string;
  mod_version?: string;
  loader?: string;
  disabled: boolean;
}

export interface LauncherProfiles {
  profiles: Record<string, Profile>;
  settings: OfficialLauncherSettings;
  version: number;
}

/**
 * Represents a Kable Minecraft Profile, which contains more information than a standard Minecraft Profile:
 * ```rs
 * pub struct KableProfile {
 *     pub id: String,
 *     pub name: String,
 *     pub icon: Option<String>,
 *     pub version: ProfileVersion,
 *     pub created: String,
 *     pub last_used: String,
 *     pub java_args: Vec<String>,
 *     pub dedicated_mods_folder: Option<String>,
 *     pub dedicated_resource_pack_folder: Option<String>,
 *     pub dedicated_shaders_folder: Option<String>,
 *     pub dedicated_config_folder: Option<String>,
 *     pub favorite: bool,
 *     pub total_time_played_ms: u32,
 *     pub parameters_map: HashMap<String, String>,
 *     pub description: Option<String>,
 *     pub times_launched: u32,
 *     #[serde(default)]
 *     pub enable_pack_merging: bool,
 *     #[serde(default)]
 *     pub pack_order: Vec<String>,
 *     #[serde(default)]
 *     pub merged_packs: Vec<String>,
 * }
 * ```
 */
export interface KableProfile {
  id: string;
  name: string;
  icon?: string;
  version: ProfileVersion;
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

export type LogLevel = "info" | "warn" | "error" | "debug";

export interface LogEntry {
  level: LogLevel;
  message: string;
  timestamp: Timestamp;
  instance_id?: string;
}

export type Timestamp = unknown;

export interface FrontendLogBatch {
  logs: LogEntry[];
  max_logs: number;
}

export interface LaunchResult {
  pid: number;
  runtime_id: string;
  command: string;
}

export interface IconSettings {
  custom_templates: CustomIconTemplate[];
}

export interface CustomIconTemplate {
  id: string;
  name: string;
  description?: string;
  author: string[];
  version?: string;
  fallback_icon: string;
  icons: Record<string, string>;
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
  expires_at: Timestamp;
  /**
   * Raw refresh token from device code flow. Should be encrypted if stored persistently!!
   */
  refresh_token?: string;
}

export interface LauncherAccountsJson {
  accounts: Record<string, LauncherAccount>;
  mojang_client_token: string;
}

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
  license_product_ids: string | null[];
  local_id: string;
  minecraft_profile: MinecraftProfile;
  persistent: boolean;
  remote_id: string;
  account_type: string;
  user_properites: string | null[];
  username: string;
}

export interface KableMinecraftProfile {
  id: string;
  name: string;
  requires_profile_name_change: boolean;
  requires_skin_change: boolean;
}

export interface KableAccountsJson {
  accounts: Record<string, KableAccount>;
  active_account_local_id: string;
  mojang_client_token: string;
}

export interface KableAccount {
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
  /**
   * This is usually the same as the user's UUID but for correctness use the Minecraft Profile's ID!
   */
  local_id: string;
  minecraft_profile: KableMinecraftProfile;
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

