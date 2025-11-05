import type { CategorizedLauncherSettings } from "../types";
import * as settingsApi from "../api/settings";
import { get } from "svelte/store";
import * as minecraftApi from "../api/minecraft";
import {
  settings,
  isSettingsLoading,
  settingsError,
  isSettingsInitialized,
  minecraftDirectoryInfo,
  isMinecraftFound,
  defaultCategorizedSettings,
} from "../stores/settings";

/**
 * SettingsService merges all logic from the old SettingsManager and SettingsService.
 * Coordinates settings state between stores, services, and API.
 */
export class SettingsService {
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
      let loadedSettings = await settingsApi.loadSettings();

      // Deep-merge missing fields from defaults
      function mergeDefaults<T>(defaults: T, actual: any): T {
        if (typeof defaults !== "object" || defaults === null)
          return actual ?? defaults;
        const result: any = Array.isArray(defaults) ? [] : {};
        for (const key in defaults) {
          if (Object.prototype.hasOwnProperty.call(defaults, key)) {
            result[key] = mergeDefaults(defaults[key], actual?.[key]);
          }
        }
        // Copy any extra keys from actual
        if (actual && typeof actual === "object") {
          for (const key in actual) {
            if (!(key in result)) {
              result[key] = actual[key];
            }
          }
        }
        return result;
      }

      loadedSettings = mergeDefaults(
        defaultCategorizedSettings,
        loadedSettings,
      );

      // Auto-detect Minecraft directory if not set
      if (!loadedSettings.general.game_directory) {
        console.log("🔍 Auto-detecting Minecraft directory...");
        try {
          const defaultMinecraftPath =
            await minecraftApi.getDefaultMinecraftDir();
          loadedSettings.general.game_directory = defaultMinecraftPath;
        } catch (error) {
          console.warn("Could not auto-detect Minecraft directory:", error);
        }
      }

      // Validate and update Minecraft directory info
      await this.updateMinecraftDirectoryInfo(
        loadedSettings.general.game_directory,
      );

      settings.set(loadedSettings);
      isSettingsInitialized.set(true);

      // Save updated settings back to backend if we auto-detected the path or filled missing fields
      await this.save(loadedSettings);

      // Initialize custom CSS after settings are loaded
      await this.initializeCustomCss();
    } catch (error) {
      console.error("Failed to load settings:", error);
      settingsError.set(`Failed to load settings: ${error}`);
      isSettingsInitialized.set(true);
    } finally {
      isSettingsLoading.set(false);
    }
  }

  /**
   * Save current settings to backend
   */
  static async save(
    newSettings: CategorizedLauncherSettings | null = null,
  ): Promise<void> {
    try {
      const currentSettings = newSettings || get(settings);
      await settingsApi.saveSettings(currentSettings);
      console.log("✅ Settings saved successfully");
    } catch (error) {
      console.error("❌ Failed to save settings:", error);
      settingsError.set(`Failed to save settings: ${error}`);
      throw error;
    }
  }

  /**
   * Update a specific setting value
   */
  static async updateSetting<K extends keyof CategorizedLauncherSettings>(
    key: K,
    value: CategorizedLauncherSettings[K],
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
    value: CategorizedLauncherSettings[K],
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
        console.warn("Minecraft directory validation failed for path:", path);
      }
    } catch (error) {
      console.error("Failed to validate Minecraft directory:", error);
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
      console.log("✅ Settings reset to defaults");
    } catch (error) {
      console.error("❌ Failed to reset settings:", error);
      throw error;
    }
  }

  /**
   * Get current settings (synchronously from store)
   */
  static async getSettings(): Promise<CategorizedLauncherSettings> {
    return await settingsApi.loadSettings();
  }

  static async saveSettings(
    settings: CategorizedLauncherSettings,
  ): Promise<void> {
    return await settingsApi.saveSettings(settings);
  }

  /**
   * Load custom CSS content from a theme name
   */
  static async loadCustomCss(themeName: string): Promise<string> {
    return await settingsApi.loadCustomCss(themeName);
  }

  /**
   * Set the selected CSS theme in settings and apply it
   */
  static async setSelectedCssTheme(themeName: string): Promise<void> {
    await settingsApi.setSelectedCssTheme(themeName);

    // Reload settings to update the store
    await this.initialize();

    // Apply the CSS theme
    await this.applyCustomCss(themeName);
  }

  /**
   * Get the current selected CSS theme from settings
   */
  static async getSelectedCssTheme(): Promise<string> {
    return await settingsApi.getSelectedCssTheme();
  }

  /**
   * Get all available CSS themes
   */
  static async getCssThemes(): Promise<string[]> {
    return await settingsApi.getCssThemes();
  }

  /**
   * Save a CSS theme with content
   */
  static async saveCssTheme(
    themeName: string,
    cssContent: string,
  ): Promise<string> {
    return await settingsApi.saveCssTheme(themeName, cssContent);
  }

  /**
   * Delete a CSS theme
   */
  static async deleteCssTheme(themeName: string): Promise<void> {
    return await settingsApi.deleteCssTheme(themeName);
  }

  /**
   * Open CSS themes directory
   */
  static async openCssThemesDirectory(): Promise<void> {
    return await settingsApi.openCssThemesDirectory();
  }

  /**
   * Apply custom CSS by theme name
   */
  static async applyCustomCss(themeName: string): Promise<void> {
    try {
      // Remove any existing custom CSS
      this.removeCustomCss();

      if (themeName === "default") {
        console.log("✅ Default theme applied (no custom CSS)");
        return;
      }

      const cssContent = await this.loadCustomCss(themeName);

      // Create and inject new style element
      const styleElement = document.createElement("style");
      styleElement.id = "kable-custom-css";
      styleElement.textContent = cssContent;
      document.head.appendChild(styleElement);

      console.log("✅ Custom CSS theme applied:", themeName);
    } catch (error) {
      console.error("❌ Failed to apply custom CSS theme:", error);
      throw error;
    }
  }

  /**
   * Remove custom CSS from the document
   */
  static removeCustomCss(): void {
    const existingStyle = document.getElementById("kable-custom-css");
    if (existingStyle) {
      existingStyle.remove();
      console.log("✅ Custom CSS removed");
    }
  }

  /**
   * Initialize custom CSS on app startup
   */
  static async initializeCustomCss(): Promise<void> {
    const themeName = await this.getSelectedCssTheme();
    if (themeName && themeName !== "default") {
      try {
        await this.applyCustomCss(themeName);
      } catch (error) {
        console.warn("⚠️ Failed to load custom CSS theme on startup:", error);
        // Reset to default theme if loading fails
        await this.setSelectedCssTheme("default");
      }
    }
  }
}
