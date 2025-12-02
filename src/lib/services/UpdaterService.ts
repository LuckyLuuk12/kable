import { checkForUpdates, checkForNightlyUpdates, installUpdate } from "$lib/api/updater";
import { NotificationService } from "./NotificationService";
import { settings } from "$lib/stores";
import { get } from "svelte/store";

export class UpdaterService {
  /**
   * Check for updates on app launch
   * Shows a notification if an update is available with click-to-install action
   * Respects user's nightly update preference
   */
  static async checkForUpdatesOnLaunch(): Promise<void> {
    try {
      const settingsValue = get(settings);
      const checkNightly = settingsValue?.advanced?.check_nightly_updates ?? false;
      
      console.log(`[UpdaterService] Checking for ${checkNightly ? "nightly" : "stable"} updates on launch...`);

      const updateInfo = checkNightly 
        ? await checkForNightlyUpdates()
        : await checkForUpdates();

      if (updateInfo && updateInfo.version) {
        const updateType = checkNightly ? "Nightly update" : "Update";
        console.log(
          `[UpdaterService] Update available: ${updateInfo.current_version} -> ${updateInfo.version}`,
        );

        // Show notification with click handler to install
        NotificationService.info(
          `${updateType} available: v${updateInfo.version}. Click to install and restart.`,
          30, // 30 seconds
          false,
          async () => {
            await this.installUpdateWithNotifications();
          },
        );
      } else {
        console.log("[UpdaterService] No updates available");
      }
    } catch (error) {
      console.error("[UpdaterService] Failed to check for updates:", error);
      // Don't show error notification on launch - silent fail
    }
  }

  /**
   * Install update with user-friendly notifications
   * Used by the notification click handler
   */
  static async installUpdateWithNotifications(): Promise<void> {
    try {
      console.log("[UpdaterService] Starting update installation...");

      NotificationService.info("Downloading update...", 0); // 0 = no auto-dismiss

      // Download and install the update (backend handles relaunch)
      await installUpdate();

      console.log("[UpdaterService] Update installed, app will restart...");
      NotificationService.success("Update installed! Restarting...", 2);
    } catch (error) {
      console.error("[UpdaterService] Failed to install update:", error);
      NotificationService.error(`Failed to install update: ${error}`);
      throw error;
    }
  }
}
