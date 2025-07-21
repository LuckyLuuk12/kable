import { writable } from 'svelte/store';
import type { CategorizedLauncherSettings, GeneralSettings, AppearanceSettings, LoggingSettings, NetworkSettings, ContentSettings, AdvancedSettings, MiscSettings, MinecraftDirectoryInfo } from '../types';

/**
 * Settings Stores
 * Svelte stores for launcher settings state management
 */

export const defaultCategorizedSettings: CategorizedLauncherSettings = {
  general: defaultGeneralSettings(),
  appearance: defaultAppearanceSettings(),
  logging: defaultLoggingSettings(),
  network: defaultNetworkSettings(),
  content: defaultContentSettings(),
  advanced: defaultAdvancedSettings(),
  misc: defaultMiscSettings(),
};


// Settings state
export const settings = writable<CategorizedLauncherSettings>(defaultCategorizedSettings);
export const isSettingsLoading = writable(false);
export const settingsError = writable<string | null>(null);
export const isSettingsInitialized = writable(false);

// Minecraft directory info
export const minecraftDirectoryInfo = writable<MinecraftDirectoryInfo | null>(null);
export const isMinecraftFound = writable(false);


export function defaultGeneralSettings(): GeneralSettings {
  return {
    java_path: undefined,
    game_directory: undefined,
    on_game_close: 'exit',
    on_game_crash: 'restart',
    on_game_launch: 'keep_open',
    auto_update_launcher: true,
    show_ads: true,
  };
}
export function defaultAppearanceSettings(): AppearanceSettings {
  return {
    theme: 'system',
    language: 'en',
    extra_spacing: 16,
    sidebar_width: 250,
    selected_icon_template: 'default',
    icon_settings: {
      selected_template: 'default',
      custom_templates: [],
      builtin_templates: ['emoji', 'fontawesome', 'svg'],
    },
  };
}

export function defaultLoggingSettings(): LoggingSettings {
  return {
    show_logs_page_in_nav: true,
    enable_persistent_logging: true,
    enable_log_compression: true,
    log_file_size_limit_mb: 10,
    log_retention_days: 30,
    merge_log_tabs: true,
    default_log_levels: ['info', 'warn', 'error'],
  };
}

export function defaultNetworkSettings(): NetworkSettings {
  return {
    parallel_downloads: 3,
    connection_timeout: 30,
    download_speed_limit: 'unlimited',
  };
}

export function defaultContentSettings(): ContentSettings {
  return {
    max_world_backups: 5,
    auto_backup_worlds: true,
    use_per_installation_mods_folder: true,
    use_per_installation_resource_packs: true,
  };
}

export function defaultAdvancedSettings(): AdvancedSettings {
  return {
    enable_experimental_features: false,
    default_memory: 2048,
    separate_logs_window: false,
    auto_save_interval: 10,
  };
}

export function defaultMiscSettings(): MiscSettings {
  return {
    use_titlebar: true,
    auth_preference: 'code',
  };
}
