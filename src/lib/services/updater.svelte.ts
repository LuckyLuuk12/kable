import type {
  UpdateDetection,
  UpdateMode,
  UpdateNotificationStyle,
} from "$lib";
import { type UpdateData, api, app, UpdateModal } from "$lib";
import type { Service } from "./app.svelte";

export class UpdaterService implements Service {
  updateInfo = $state<UpdateData | null>(null);

  checking = $state(false);
  downloading = $state(false);
  installing = $state(false);

  error = $state<string | null>(null);

  updateMode = $state<UpdateMode>("on_confirm");
  notificationStyle = $state<UpdateNotificationStyle>("notification");

  private updateDetection: UpdateDetection = "on_startup";
  private includePrerelease = false;

  private periodicTimer: ReturnType<typeof setTimeout> | null = null;
  private destroyed = false;
  private lastPresentedVersion: string | null = null;

  async init() {
    this.destroyed = false;

    try {
      const settings = await api.getSettings();

      this.updateDetection =
        settings?.general?.update_detection ?? "on_startup";

      this.updateMode =
        settings?.general?.update_mode ?? "on_confirm";

      this.notificationStyle =
        settings?.general?.update_notification_style ?? "notification";

      this.includePrerelease =
        settings?.advanced?.enable_nightly_updates ?? false;

      switch (this.updateDetection) {
        case "on_startup":
          await this.runDetectionCheck();
          break;

        case "on_close":
        case "manual":
          break;

        default:
          this.schedulePeriodicCheck(this.updateDetection.periodically);
          break;
      }
    } catch (error) {
      this.error = String(error);
    }
  }

  /**
   * Schedule recurring checks without overlapping timers.
   * The interval is expressed in seconds.
   */
  private schedulePeriodicCheck(seconds: number) {
    this.clearPeriodicTimer();

    if (this.destroyed) return;

    const interval = Math.max(1, seconds) * 1000;

    this.periodicTimer = setTimeout(async () => {
      this.periodicTimer = null;

      if (this.destroyed) return;

      try {
        await this.runDetectionCheck();
      } finally {
        if (!this.destroyed) {
          this.schedulePeriodicCheck(seconds);
        }
      }
    }, interval);
  }

  private clearPeriodicTimer() {
    if (this.periodicTimer !== null) {
      clearTimeout(this.periodicTimer);
      this.periodicTimer = null;
    }
  }

  /**
   * Check for updates and apply the configured update policy.
   */
  private async runDetectionCheck() {
    const result = await this.check(this.includePrerelease);

    if (!result || this.destroyed) return result;

    switch (this.updateMode) {
      case "manual":
        return result;

      case "on_confirm":
        this.presentUpdate(result);
        return result;

      case "automatic":
        await this.install(this.includePrerelease);
        return result;
    }
  }

  /**
   * Present an available update using the configured UI.
   */
  private presentUpdate(update: UpdateData) {
    if (this.destroyed) return;

    // Do not repeatedly present the same version.
    if (this.lastPresentedVersion === update.version) return;

    this.lastPresentedVersion = update.version;
    this.updateInfo = update;

    if (this.notificationStyle === "notification") {
      app.notificationService.info(
        `Kable ${update.version} is available. Click here to view the update.`,
        10,
        false,
        () => {
          app.show(UpdateModal, {
            updateInfo: update,
            open: true,
            onclose: () => {
              this.updateInfo = null;
            },
            oninstallnow: async () => {
              await this.install(this.includePrerelease);
            },
            ondownload: async () => {
              const path = await this.download(this.includePrerelease);

              if (path === null) {
                throw new Error("Failed to download launcher update.");
              }
            },
          });
        },
      );

      return;
    }

    app.show(UpdateModal, {
      updateInfo: update,
      open: true,

      onclose: () => {
        this.updateInfo = null;
      },

      oninstallnow: async () => {
        const success = await this.install(this.includePrerelease);

        if (success) {
          this.updateInfo = null;
        }
      },

      ondownload: async () => {
        await this.download(this.includePrerelease);
      },
    });
  }

  /**
   * Check for launcher updates.
   */
  async check(includePrerelease: boolean = false) {
    if (this.checking) return null;

    this.checking = true;
    this.error = null;

    try {
      const result = await api.checkLauncherUpdates(includePrerelease);
      this.updateInfo = result;
      return result;
    } catch (e) {
      const message = String(e);

      if (
        message.includes("No updates available") ||
        message.includes("Updates are disabled in development mode")
      ) {
        this.updateInfo = null;
        return null;
      }

      this.error = message;
      app.notificationService.error(
        "Failed to check for launcher updates.",
      );

      return null;
    } finally {
      this.checking = false;
    }
  }

  /**
   * Download an update without applying it.
   */
  async download(includePrerelease: boolean = false) {
    if (this.downloading) return null;

    this.downloading = true;
    this.error = null;

    try {
      const path = await api.downloadLauncherUpdate(includePrerelease);

      app.notificationService.success(
        "Launcher update downloaded successfully.",
      );

      return path;
    } catch (e) {
      this.error = String(e);
      app.notificationService.error(
        "Failed to download launcher update.",
      );

      throw e;
    } finally {
      this.downloading = false;
    }
  }

  /**
   * Install an update immediately.
   */
  async install(includePrerelease: boolean = false) {
    if (this.installing) return false;

    this.installing = true;
    this.error = null;

    try {
      await api.installLauncherUpdate(includePrerelease);

      app.notificationService.success(
        "Launcher update installed successfully.",
      );

      this.updateInfo = null;
      return true;
    } catch (e) {
      this.error = String(e);
      app.notificationService.error(
        "Failed to install launcher update.",
      );

      return false;
    } finally {
      this.installing = false;
    }
  }

  /**
   * Apply an update that has already been downloaded.
   */
  async applyDownloaded() {
    if (this.installing) return false;

    this.installing = true;
    this.error = null;

    try {
      await api.applyDownloadedUpdate();

      app.notificationService.success(
        "Downloaded launcher update applied successfully.",
      );

      this.updateInfo = null;
      return true;
    } catch (e) {
      this.error = String(e);
      app.notificationService.error(
        "Failed to apply downloaded update.",
      );

      return false;
    } finally {
      this.installing = false;
    }
  }

  /**
   * Explicitly check for updates on user request.
   */
  async checkManually() {
    return this.runDetectionCheck();
  }

  /**
   * Call when the game closes.
   */
  async onGameClose() {
    if (this.updateDetection !== "on_close" || this.destroyed) return;

    await this.runDetectionCheck();
  }

  /**
   * Current launcher version.
   */
  async getCurrentVersion() {
    try {
      return await api.getCurrentVersion();
    } catch (e) {
      console.error(
        "API call failed: `api.getCurrentVersion()`",
        e,
      );
    }
  }

  /**
   * Clean up timers and transient state.
   */
  async destroy() {
    if (this.destroyed) return;

    if (this.updateDetection === "on_close") {
      await this.runDetectionCheck();
    }

    this.destroyed = true;
    this.clearPeriodicTimer();

    this.updateInfo = null;
    this.checking = false;
    this.downloading = false;
    this.installing = false;
    this.error = null;
    this.lastPresentedVersion = null;
  }
}