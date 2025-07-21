import { get } from 'svelte/store';
import { SettingsService } from '../services/SettingsService';
import * as minecraftApi from '../api/minecraft';
import { 
  settings, 
  isSettingsLoading, 
  settingsError, 
  isSettingsInitialized, 
  minecraftDirectoryInfo, 
  isMinecraftFound, 
  defaultCategorizedSettings
} from '../stores/settings';
import type { CategorizedLauncherSettings } from '../types';

/**
 * Settings Manager
 * Coordinates settings state between stores, services, and API
 */

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
      let loadedSettings = await SettingsService.getSettings();

      // Deep-merge missing fields from defaults
      function mergeDefaults<T>(defaults: T, actual: any): T {
        if (typeof defaults !== 'object' || defaults === null) return actual ?? defaults;
        const result: any = Array.isArray(defaults) ? [] : {};
        for (const key in defaults) {
          if (Object.prototype.hasOwnProperty.call(defaults, key)) {
            result[key] = mergeDefaults(defaults[key], actual?.[key]);
          }
        }
        // Copy any extra keys from actual
        if (actual && typeof actual === 'object') {
          for (const key in actual) {
            if (!(key in result)) {
              result[key] = actual[key];
            }
          }
        }
        return result;
      }

      loadedSettings = mergeDefaults(defaultCategorizedSettings, loadedSettings);

      // Auto-detect Minecraft directory if not set
      if (!loadedSettings.general.game_directory) {
        console.log('🔍 Auto-detecting Minecraft directory...');
        try {
          const defaultMinecraftPath = await minecraftApi.getDefaultMinecraftDir();
          loadedSettings.general.game_directory = defaultMinecraftPath;
        } catch (error) {
          console.warn('Could not auto-detect Minecraft directory:', error);
        }
      }

      // Validate and update Minecraft directory info
      await this.updateMinecraftDirectoryInfo(loadedSettings.general.game_directory);

      settings.set(loadedSettings);
      isSettingsInitialized.set(true);

      // Save updated settings back to backend if we auto-detected the path or filled missing fields
      await this.save(loadedSettings);
    } catch (error) {
      console.error('Failed to load settings:', error);
      settingsError.set(`Failed to load settings: ${error}`);
      isSettingsInitialized.set(true);
    } finally {
      isSettingsLoading.set(false);
    }
  }

  /**
   * Save current settings to backend
   */
  static async save(newSettings: CategorizedLauncherSettings | null = null): Promise<void> {
    try {
      const currentSettings = newSettings || get(settings);
      await SettingsService.saveSettings(currentSettings);
      console.log('✅ Settings saved successfully');
    } catch (error) {
      console.error('❌ Failed to save settings:', error);
      settingsError.set(`Failed to save settings: ${error}`);
      throw error;
    }
  }

  /**
   * Update a specific setting value
   */
  static async updateSetting<K extends keyof CategorizedLauncherSettings>(
    key: K, 
    value: CategorizedLauncherSettings[K]
  ): Promise<void> {
    try {
      const currentSettings = get(settings);
      const updatedSettings = { ...currentSettings, [key]: value };
      
      settings.set(updatedSettings);
      await this.save();
    } catch (error) {
      console.error(`Failed to update setting ${String(key)}:`, error);
      throw error;
    }
  }

  static async update<K extends keyof CategorizedLauncherSettings>(
    key: K, 
    value: CategorizedLauncherSettings[K]
  ): Promise<void> {
    await this.updateSetting(key, value);
  }


  /**
   * Validate and update Minecraft directory information
   */
  static async updateMinecraftDirectoryInfo(path?: string): Promise<void> {
    if (!path) {
      minecraftDirectoryInfo.set(null);
      isMinecraftFound.set(false);
      return;
    }

    try {
      const info = await minecraftApi.validateMinecraftDirectory(path);
      minecraftDirectoryInfo.set(info);
      isMinecraftFound.set(info.is_valid);
      
      if (!info.is_valid) {
        console.warn('Minecraft directory validation failed for path:', path);
      }
    } catch (error) {
      console.error('Failed to validate Minecraft directory:', error);
      minecraftDirectoryInfo.set(null);
      isMinecraftFound.set(false);
    }
  }

  /**
   * Reset settings to defaults
   */
  static async resetToDefaults(): Promise<void> {
    try {
      // Get default settings by clearing and reloading
      const defaultPath = await minecraftApi.getDefaultMinecraftDir();
      
      const defaultSettings = defaultCategorizedSettings;
      
      settings.set(defaultSettings);
      await this.save();
      await this.updateMinecraftDirectoryInfo(defaultPath);
      
      console.log('✅ Settings reset to defaults');
    } catch (error) {
      console.error('❌ Failed to reset settings:', error);
      throw error;
    }
  }

  /**
   * Get launcher directory path
   */
  static async getLauncherDirectory(): Promise<string> {
    try {
      return await SettingsService.getLauncherDir();
    } catch (error) {
      console.error('Failed to get launcher directory:', error);
      throw error;
    }
  }

  /**
   * Get current settings (synchronously from store)
   */
  static getSettings(): CategorizedLauncherSettings {
    return get(settings);
  }

  /**
   * Get current settings (async version for compatibility)
   */
  static async getSettingsAsync(): Promise<CategorizedLauncherSettings> {
    return get(settings);
  }
}
