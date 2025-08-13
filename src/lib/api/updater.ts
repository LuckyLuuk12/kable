/**
 * Updater API
 * Handles auto-update functionality
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * Check for available updates
 * @returns Promise with update info or null if no update available
 */
export async function checkForUpdates(): Promise<any> {
    return await invoke('check_for_updates');
}

/**
 * Download and install available update
 * @returns Promise that resolves when update is installed
 */
export async function installUpdate(): Promise<void> {
    return await invoke('install_update');
}

/**
 * Get current app version
 * @returns Promise with current version string
 */
export async function getCurrentVersion(): Promise<string> {
    return await invoke('get_current_version');
}
