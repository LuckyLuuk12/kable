import {
  checkForUpdates,
  checkForNightlyUpdates,
  installUpdate,
  downloadUpdate,
} from "$lib/api/updater";
import { NotificationService } from "./NotificationService";
import { settings } from "$lib/stores";
import { get } from "svelte/store";

// UpdateModal state - we'll use a separate store
import { writable } from "svelte/store";

export const updateModalOpen = writable(false);
export const updateModalInfo = writable<any>(null);

export class UpdaterService {
  /**
   * Check for updates on app launch
   * Handles instant updates, downloads, or shows notification/modal based on settings
   * Respects user's nightly update preference
   */
  static async checkForUpdatesOnLaunch(): Promise<void> {
    try {
      const settingsValue = get(settings);
      const checkNightly =
        settingsValue?.advanced?.check_nightly_updates ?? false;
      const updateMode = settingsValue?.general?.update_mode ?? "on_confirm";
      const notificationStyle =
        settingsValue?.general?.update_notification_style ?? "notification";

      console.log(
        `[UpdaterService] Checking for ${checkNightly ? "nightly" : "stable"} updates on launch...`,
      );
      console.log(`[UpdaterService] Update mode: ${updateMode}`);
      console.log(`[UpdaterService] Notification style: ${notificationStyle}`);

      const updateInfo = checkNightly
        ? await checkForNightlyUpdates()
        : await checkForUpdates();

      if (!updateInfo || !updateInfo.version) {
        console.log("[UpdaterService] No updates available");
        return;
      }

      const updateType = checkNightly ? "Nightly update" : "Update";
      console.log(
        `[UpdaterService] Update available: ${updateInfo.current_version} -> ${updateInfo.version}`,
      );

      // Handle different update modes
      switch (updateMode) {
        case "instant":
          // Install immediately without prompting
          console.log("[UpdaterService] Instant mode: Installing update now");
          NotificationService.info("Installing update...", 0);
          try {
            await installUpdate(checkNightly);
            console.log(
              "[UpdaterService] Update installed, app will restart...",
            );
          } catch (error) {
            console.error("[UpdaterService] Failed to install update:", error);
            NotificationService.error(`Failed to install update: ${error}`);
          }
          break;

        case "on_restart":
          // Download in background, install on next restart
          console.log(
            "[UpdaterService] On restart mode: Downloading update for next restart",
          );
          try {
            await downloadUpdate(checkNightly);
            if (notificationStyle === "modal") {
              updateModalInfo.set(updateInfo);
              updateModalOpen.set(true);
            } else {
              NotificationService.success(
                `${updateType} v${updateInfo.version} downloaded. Will install on next restart.`,
                10,
              );
            }
          } catch (error) {
            console.error("[UpdaterService] Failed to download update:", error);
            NotificationService.error(`Failed to download update: ${error}`);
          }
          break;

        case "on_confirm":
        default:
          // Show modal or notification asking user
          if (notificationStyle === "modal") {
            // Open the update modal
            updateModalInfo.set(updateInfo);
            updateModalOpen.set(true);
          } else {
            // Show notification with click handler to install
            NotificationService.info(
              `${updateType} available: v${updateInfo.version}. Click to install and restart.`,
              30, // 30 seconds
              false,
              async () => {
                await this.installUpdateWithNotifications();
              },
            );
          }
          break;
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
      // Read the checkNightly setting from the settings store
      const settingsValue = get(settings);
      const checkNightly =
        settingsValue?.advanced?.check_nightly_updates ?? false;

      await installUpdate(checkNightly);

      console.log("[UpdaterService] Update installed, app will restart...");
      NotificationService.success("Update installed! Restarting...", 2);
    } catch (error) {
      console.error("[UpdaterService] Failed to install update:", error);
      NotificationService.error(`Failed to install update: ${error}`);
      throw error;
    }
  }

  /**
   * Download update for later installation
   */
  static async downloadUpdateForLater(): Promise<void> {
    try {
      const settingsValue = get(settings);
      const checkNightly =
        settingsValue?.advanced?.check_nightly_updates ?? false;

      NotificationService.info("Downloading update...", 0);
      await downloadUpdate(checkNightly);
      NotificationService.success(
        "Update downloaded! Will install on next restart.",
        5,
      );
    } catch (error) {
      console.error("[UpdaterService] Failed to download update:", error);
      NotificationService.error(`Failed to download update: ${error}`);
      throw error;
    }
  }
}
