import { invoke } from '@tauri-apps/api/core';

/**
 * Discord Rich Presence API
 */

/**
 * Set Discord Rich Presence to show browsing activity
 * @param section - The section being browsed (mods, shaders, resourcepacks, maps, etc.)
 */
export async function setBrowsing(section: string): Promise<void> {
  return await invoke('discord_set_browsing', { section });
}

/**
 * Enable or disable Discord Rich Presence
 * @param enabled - Whether to enable Discord RPC
 */
export async function setEnabled(enabled: boolean): Promise<void> {
  return await invoke('discord_set_enabled', { enabled });
}

/**
 * Clear Discord Rich Presence
 */
export async function clear(): Promise<void> {
  return await invoke('discord_clear');
}
