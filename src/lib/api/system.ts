import { invoke } from '@tauri-apps/api/core';

/**
 * System API
 * Pure Tauri invoke calls for system-level operations
 */

export async function copyToClipboard(text: string): Promise<void> {
  return await invoke('copy_to_clipboard', { text });
}

export async function openUrl(url: string): Promise<void> {
  return await invoke('open_url', { url });
}
