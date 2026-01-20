/**
 * Updater API
 * Handles auto-update functionality
 */

import { invoke } from "@tauri-apps/api/core";

/**
 * Check for available updates
 * @param includePrerelease - Include prerelease/nightly versions
 * @returns Promise with update info or null if no update available
 */
export async function checkForUpdates(
  includePrerelease: boolean = false,
): Promise<any> {
  return await invoke("check_for_updates", { includePrerelease });
}

/**
 * Check specifically for nightly updates (prereleases)
 * @returns Promise with update info or null if no update available
 */
export async function checkForNightlyUpdates(): Promise<any> {
  return await checkForUpdates(true);
}

/**
 * Download and install available update
 * @param includePrerelease - Include prerelease/nightly versions (should match what was used in checkForUpdates)
 * @returns Promise that resolves when update is installed
 */
export async function installUpdate(
  includePrerelease: boolean = false,
): Promise<void> {
  return await invoke("install_update", { includePrerelease });
}

/**
 * Download available update but do not install it.
 * Returns the path to the downloaded installer.
 */
export async function downloadUpdate(
  includePrerelease: boolean = false,
): Promise<string> {
  return await invoke("download_update", { includePrerelease });
}

/**
 * Apply a previously downloaded update (runs installer and exits).
 */
export async function applyDownloadedUpdate(): Promise<void> {
  return await invoke("apply_downloaded_update");
}

/**
 * Get current app version
 * @returns Promise with current version string
 */
export async function getCurrentVersion(): Promise<string> {
  return await invoke("get_current_version");
}
