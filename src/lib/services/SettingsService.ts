import type { CategorizedLauncherSettings } from '../types';
import * as settingsApi from '../api/settings';

/**
 * Settings Service
 * High-level business logic for launcher settings management
 */

export class SettingsService {
  static async getSettings(): Promise<CategorizedLauncherSettings> {
    return await settingsApi.loadSettings();
  }

  static async saveSettings(settings: CategorizedLauncherSettings): Promise<void> {
    return await settingsApi.saveSettings(settings);
  }

  static async getLauncherDir(): Promise<string> {
    return await settingsApi.getLauncherDir();
  }
}
