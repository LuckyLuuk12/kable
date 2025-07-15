import type { LauncherSettings } from '../types';
import * as settingsApi from '../api/settings';

/**
 * Settings Service
 * High-level business logic for launcher settings management
 */

export class SettingsService {
  static async getSettings(): Promise<LauncherSettings> {
    return await settingsApi.loadSettings();
  }

  static async saveSettings(settings: LauncherSettings): Promise<void> {
    return await settingsApi.saveSettings(settings);
  }

  static async getLauncherDir(): Promise<string> {
    return await settingsApi.getLauncherDir();
  }
}
