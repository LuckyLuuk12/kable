import { invoke } from '@tauri-apps/api/core';
import type { LauncherSettings } from '../types';

/**
 * Settings API
 * Pure Tauri invoke calls for launcher settings management
 */

export async function loadSettings(): Promise<LauncherSettings> {
  return await invoke('load_settings');
}

export async function saveSettings(settings: LauncherSettings): Promise<void> {
  return await invoke('save_settings', { settings });
}

export async function getLauncherDir(): Promise<string> {
  return await invoke('get_launcher_dir');
}
