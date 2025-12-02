import { checkForUpdates, installUpdate } from "$lib/api/updater";
import { NotificationService } from "./NotificationService";

export class UpdaterService {
  /**
   * Check for updates on app launch
   * Shows a notification if an update is available with click-to-install action
   */
  static async checkForUpdatesOnLaunch(): Promise<void> {
    try {
      console.log("[UpdaterService] Checking for updates on launch...");

      const updateInfo = await checkForUpdates();

      if (updateInfo && updateInfo.version) {
        console.log(
          `[UpdaterService] Update available: ${updateInfo.current_version} -> ${updateInfo.version}`,
        );

        // Show notification with click handler to install
        NotificationService.info(
          `Update available: v${updateInfo.version}. Click to install and restart.`,
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
