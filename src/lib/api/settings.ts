import { invoke } from '@tauri-apps/api/core';
import type { CategorizedLauncherSettings } from '../types';

/**
 * Settings API
 * Pure Tauri invoke calls for launcher settings management
 */

export async function loadSettings(): Promise<CategorizedLauncherSettings> {
  return await invoke('load_settings');
}

export async function saveSettings(settings: CategorizedLauncherSettings): Promise<void> {
  return await invoke('save_settings', { settings });
}

export async function getLauncherDir(): Promise<string> {
  return await invoke('get_launcher_dir');
}
