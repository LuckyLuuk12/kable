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

export type VersionType = "release" | "snapshot" | "old_beta" | "old_alpha";

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
  version_type?: VersionType;
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

export type LoaderKind = "vanilla" | "fabric" | "iris_fabric" | "forge" | "neo_forge" | "quilt";

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

export type Status = "listed" | "archived" | "draft" | "unlisted" | "scheduled" | "unknown";

/**
 * The server side support of the project
 */
export type ServerSide = "required" | "optional" | "unsupported";

export type SearchIndex = "relevance" | "downloads" | "follows" | "newest" | "updated";

export type RequestedStatus = "listed" | "archived" | "draft" | "unlisted";

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

/**
 * The project type of the project
 */
export type ProjectType = "mod" | "modpack" | "resourcepack" | "shader";

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

/**
 * The monetization status of the project
 */
export type MonetizationStatus = "monetized" | "demonetized" | "force_demonetized";

export type ClientSide = "required" | "optional" | "unsupported";

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

