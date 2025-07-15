import { get } from 'svelte/store';
import { SettingsService } from '../services/SettingsService';
import * as minecraftApi from '../api/minecraft';
import { 
  settings, 
  isSettingsLoading, 
  settingsError, 
  isSettingsInitialized, 
  minecraftDirectoryInfo, 
  isMinecraftFound 
} from '../stores/settings';
import type { LauncherSettings, MinecraftDirectoryInfo } from '../types';

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
      const loadedSettings = await SettingsService.getSettings();
      
      // Auto-detect Minecraft directory if not set
      if (!loadedSettings.minecraft_path) {
        try {
          const defaultMinecraftPath = await minecraftApi.getDefaultMinecraftDir();
          loadedSettings.minecraft_path = defaultMinecraftPath;
        } catch (error) {
          console.warn('Could not auto-detect Minecraft directory:', error);
        }
      }

      // Validate and update Minecraft directory info
      await this.updateMinecraftDirectoryInfo(loadedSettings.minecraft_path);
      
      settings.set(loadedSettings);
      isSettingsInitialized.set(true);

      // Save updated settings back to backend if we auto-detected the path
      if (loadedSettings.minecraft_path) {
        await this.save();
      }
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
  static async save(): Promise<void> {
    try {
      const currentSettings = get(settings);
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
  static async updateSetting<K extends keyof LauncherSettings>(
    key: K, 
    value: LauncherSettings[K]
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
      
      const defaultSettings: LauncherSettings = {
        theme: 'dark',
        language: 'en',
        minecraft_path: defaultPath,
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
  static getSettings(): LauncherSettings {
    return get(settings);
  }

  /**
   * Get current settings (async version for compatibility)
   */
  static async getSettingsAsync(): Promise<LauncherSettings> {
    return get(settings);
  }
}
