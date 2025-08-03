import { invoke } from '@tauri-apps/api/core';
import { writeText, readText } from '@tauri-apps/plugin-clipboard-manager';
/**
 * System API
 * Pure Tauri invoke calls for system-level operations
 */

export async function copyToClipboard(text: string): Promise<void> {
  await writeText(text);
}

export async function openUrl(url: string): Promise<void> {
  return await invoke('open_url', { url });
}
