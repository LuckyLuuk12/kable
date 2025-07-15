import { writable } from 'svelte/store';
import type { LauncherSettings, MinecraftDirectoryInfo } from '../types';

/**
 * Settings Stores
 * Svelte stores for launcher settings state management
 */

// Default settings
const defaultSettings: LauncherSettings = {
  theme: 'dark',
  language: 'en',
  minecraft_path: undefined,
  default_memory: 2048,
  max_memory: 8192,
  java_path: undefined,
  keep_launcher_open: true,
  show_logs_on_launch: false,
  auto_update_launcher: true,
  close_launcher_on_game_start: false,
  window_width: 1080,
  window_height: 720,
  sidebar_width: 250,
  card_spacing: 16,
  animation_speed: 'normal',
  parallel_downloads: 3,
  connection_timeout: 30,
  enable_experimental_features: false,
  auto_backup_worlds: false,
  max_world_backups: 5,
  shader_quality_preset: 'medium',
  enable_shader_caching: true,
  custom: {},
  jvm_args: '',
  memory: 2048,
};

// Settings state
export const settings = writable<LauncherSettings>(defaultSettings);
export const isSettingsLoading = writable(false);
export const settingsError = writable<string | null>(null);
export const isSettingsInitialized = writable(false);

// Minecraft directory info
export const minecraftDirectoryInfo = writable<MinecraftDirectoryInfo | null>(null);
export const isMinecraftFound = writable(false);
