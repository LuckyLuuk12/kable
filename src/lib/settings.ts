import { writable, get } from 'svelte/store';
import { loadSettings, saveSettings as saveSettingsToBackend } from './services';
import type { LauncherSettings } from './types';

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
  window_width: 1200,
  window_height: 800,
  sidebar_width: 250,
  card_spacing: 16,
  animation_speed: 'normal',
  parallel_downloads: 3,
  connection_timeout: 30,
  enable_experimental_features: false,
  custom: {}
};

// Settings store
export const settings = writable<LauncherSettings>(defaultSettings);
export const isSettingsLoading = writable(false);
export const settingsError = writable<string | null>(null);

export class SettingsManager {
  /**
   * Initialize settings - load from backend
   */
  static async initialize(): Promise<void> {
    isSettingsLoading.set(true);
    settingsError.set(null);

    try {
      const loadedSettings = await loadSettings();
      
      // Merge with defaults to ensure all properties exist
      const mergedSettings = { ...defaultSettings, ...loadedSettings };
      
      settings.set(mergedSettings);
    } catch (error) {
      console.error('Failed to load settings:', error);
      settingsError.set(`Failed to load settings: ${error}`);
      
      // Use default settings if loading fails
      settings.set(defaultSettings);
    } finally {
      isSettingsLoading.set(false);
    }
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

    // Auto-save after update
    await this.save();
  }

  /**
   * Update multiple settings at once
   */
  static async updateSettings(updates: Partial<LauncherSettings>): Promise<void> {
    settings.update(current => ({
      ...current,
      ...updates
    }));

    // Auto-save after update
    await this.save();
  }

  /**
   * Reset settings to defaults
   */
  static async reset(): Promise<void> {
    settings.set(defaultSettings);
    await this.save();
  }

  /**
   * Get current settings value
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
   * Subscribe to settings changes
   */
  static subscribe(callback: (settings: LauncherSettings) => void) {
    return settings.subscribe(callback);
  }

  /**
   * Validate memory settings
   */
  static validateMemorySettings(defaultMemory: number, maxMemory: number): { isValid: boolean; error?: string } {
    if (defaultMemory < 512) {
      return { isValid: false, error: 'Default memory must be at least 512 MB' };
    }
    
    if (defaultMemory > maxMemory) {
      return { isValid: false, error: 'Default memory cannot be higher than max memory' };
    }
    
    if (maxMemory > 32768) { // 32 GB limit
      return { isValid: false, error: 'Max memory cannot exceed 32 GB' };
    }

    return { isValid: true };
  }

  /**
   * Get recommended memory settings based on system RAM
   */
  static getRecommendedMemory(): { defaultMemory: number; maxMemory: number } {
    // This is a basic calculation - in a real app you'd get system RAM
    const systemRAM = 8192; // Assume 8GB for now
    
    return {
      defaultMemory: Math.min(2048, Math.floor(systemRAM * 0.25)),
      maxMemory: Math.min(8192, Math.floor(systemRAM * 0.75))
    };
  }
}
