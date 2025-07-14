import { writable, get } from 'svelte/store';
import { 
  loadSettings, 
  saveSettings as saveSettingsToBackend,
  getDefaultMinecraftDir,
  validateMinecraftDirectory
} from './services';
import type { LauncherSettings, MinecraftDirectoryInfo } from './types';

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

// Settings store
export const settings = writable<LauncherSettings>(defaultSettings);
export const isSettingsLoading = writable(false);
export const settingsError = writable<string | null>(null);
export const isSettingsInitialized = writable(false);

// Minecraft directory info store
export const minecraftDirectoryInfo = writable<MinecraftDirectoryInfo | null>(null);
export const isMinecraftFound = writable(false);

export class SettingsManager {
  /**
   * Initialize settings - load from backend and detect Minecraft directory
   */
  static async initialize(): Promise<void> {
    if (get(isSettingsInitialized)) {
      return; // Already initialized
    }

    isSettingsLoading.set(true);
    settingsError.set(null);

    try {
      // Load settings from backend
      const loadedSettings = await loadSettings();
      
      // Merge with defaults to ensure all properties exist
      const mergedSettings = { ...defaultSettings, ...loadedSettings };
      
      // Auto-detect Minecraft directory if not set
      if (!mergedSettings.minecraft_path) {
        try {
          const defaultMinecraftPath = await getDefaultMinecraftDir();
          mergedSettings.minecraft_path = defaultMinecraftPath;
        } catch (error) {
          console.warn('Could not auto-detect Minecraft directory:', error);
        }
      }

      // Validate and update Minecraft directory info
      await this.updateMinecraftDirectoryInfo(mergedSettings.minecraft_path);
      
      settings.set(mergedSettings);
      isSettingsInitialized.set(true);

      // Save updated settings back to backend if we auto-detected the path
      if (mergedSettings.minecraft_path && !loadedSettings.minecraft_path) {
        await this.save();
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
      settingsError.set(`Failed to load settings: ${error}`);
      
      // Use default settings if loading fails
      settings.set(defaultSettings);
      isSettingsInitialized.set(true);
    } finally {
      isSettingsLoading.set(false);
    }
  }

  /**
   * Validate and update Minecraft directory information
   */
  static async updateMinecraftDirectoryInfo(minecraftPath?: string): Promise<void> {
    if (!minecraftPath) {
      minecraftDirectoryInfo.set(null);
      isMinecraftFound.set(false);
      return;
    }

    try {
      const directoryInfo = await validateMinecraftDirectory(minecraftPath);
      minecraftDirectoryInfo.set(directoryInfo);
      isMinecraftFound.set(directoryInfo.is_valid);
    } catch (error) {
      console.error('Failed to validate Minecraft directory:', error);
      minecraftDirectoryInfo.set(null);
      isMinecraftFound.set(false);
    }
  }

  /**
   * Get current Minecraft path, with fallback
   */
  static getMinecraftPath(): string | null {
    const currentSettings = get(settings);
    const isFound = get(isMinecraftFound);
    
    return isFound ? currentSettings.minecraft_path || null : null;
  }

  /**
   * Save current settings to backend
   */
  static async save(): Promise<void> {
    const currentSettings = get(settings);
    
    try {
      await saveSettingsToBackend(currentSettings);
      settingsError.set(null);
    } catch (error) {
      console.error('Failed to save settings:', error);
      settingsError.set(`Failed to save settings: ${error}`);
      throw error;
    }
  }

  /**
   * Update a specific setting
   */
  static async updateSetting<K extends keyof LauncherSettings>(
    key: K, 
    value: LauncherSettings[K]
  ): Promise<void> {
    settings.update(current => ({
      ...current,
      [key]: value
    }));

    // If updating minecraft_path, also update directory info
    if (key === 'minecraft_path') {
      await this.updateMinecraftDirectoryInfo(value as string);
    }

    await this.save();
  }

  /**
   * Get current settings
   */
  static getSettings(): LauncherSettings {
    return get(settings);
  }

  /**
   * Get a specific setting value
   */
  static getSetting<K extends keyof LauncherSettings>(key: K): LauncherSettings[K] {
    return get(settings)[key];
  }

  /**
   * Reset settings to defaults
   */
  static async resetToDefaults(): Promise<void> {
    settings.set(defaultSettings);
    await this.save();
    await this.updateMinecraftDirectoryInfo(defaultSettings.minecraft_path);
  }

  /**
   * Check if Minecraft directory is valid
   */
  static isMinecraftDirectoryValid(): boolean {
    return get(isMinecraftFound);
  }

  /**
   * Get Minecraft directory info
   */
  static getMinecraftDirectoryInfo(): MinecraftDirectoryInfo | null {
    return get(minecraftDirectoryInfo);
  }
}
