import type {
  UpdateData,
  UpdateDetection,
  UpdateMode,
  UpdateNotificationStyle,
} from "$lib";
import { api, app, UpdateModal } from "$lib";
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
  private developmentBuild: boolean | null = null;

  async init() {
    this.destroyed = false;

    try {
      const [settings, currentVersion] = await Promise.all([
        api.getSettings(),
        api.getCurrentVersion(),
      ]);

      this.updateDetection =
        settings?.general?.update_detection ?? "on_startup";

      this.updateMode =
        settings?.general?.update_mode ?? "on_confirm";

      this.notificationStyle =
        settings?.general?.update_notification_style ?? "notification";

      this.includePrerelease =
        settings?.advanced?.enable_nightly_updates ?? false;

      this.developmentBuild = this.isDevelopmentVersion(currentVersion);

      if (this.developmentBuild) {
        console.info(
          `[Updater] Development build detected (${currentVersion}). Update checks are enabled; downloads and installation are disabled.`,
        );
      }

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
      console.error("[Updater] Failed to initialize updater service:", error);
    }
  }

  /**
   * Development versions are predicted nightly versions returned by the
   * backend, such as 0.1.9-34. Stable packaged versions remain 0.1.9.
   *
   * This heuristic is only used for UI behavior. Backend guards remain
   * authoritative for download and installation operations.
   */
  private isDevelopmentVersion(version: string): boolean {
    const match = version.match(/^(\d+\.\d+\.\d+)-(\d+)$/);

    if (!match) return false;

    const buildNumber = Number(match[2]);

    return Number.isSafeInteger(buildNumber) && buildNumber > 0;
  }

  private isDevelopmentBuild(): boolean {
    return this.developmentBuild === true;
  }

  private schedulePeriodicCheck(seconds: number) {
    this.clearPeriodicTimer();

    if (this.destroyed) return;

    const interval = Math.max(1, seconds) * 1000;

    this.periodicTimer = setTimeout(async () => {
      this.periodicTimer = null;

      if (this.destroyed) return;

      try {
        await this.runDetectionCheck();
      } catch (error) {
        console.error("[Updater] Periodic update check failed:", error);
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
        if (this.isDevelopmentBuild()) {
          this.presentUpdate(result);
          app.notificationService.info(
            "Automatic launcher updates are disabled in development builds. The available update is shown for testing only.",
          );
          return result;
        }

        await this.install(this.includePrerelease);
        return result;
    }
  }

  /**
   * Show the update modal, with actions adapted to the current build.
   */
  private presentUpdate(update: UpdateData) {
    if (this.destroyed) return;

    if (this.lastPresentedVersion === update.version) return;

    this.lastPresentedVersion = update.version;
    this.updateInfo = update;

    const developmentBuild = this.isDevelopmentBuild();

    const openModal = () => {
      if (this.destroyed) return;

      app.show(UpdateModal, {
        updateInfo: update,
        open: true,

        onclose: () => {
          this.updateInfo = null;
        },

        oninstallnow: async () => {
          if (developmentBuild) {
            this.notifyDevelopmentRestriction("install");
            return;
          }

          const success = await this.install(this.includePrerelease);

          if (success) {
            this.updateInfo = null;
          }
        },

        ondownload: async () => {
          if (developmentBuild) {
            this.notifyDevelopmentRestriction("download");
            return;
          }

          await this.download(this.includePrerelease);
        },
      });
    };

    if (developmentBuild) {
      app.notificationService.info(
        `Development build: Kable ${update.version} is available for update-flow testing. Downloading and installing are disabled.`,
        10,
        false,
        openModal,
      );

      return;
    }

    if (this.notificationStyle === "notification") {
      app.notificationService.info(
        `Kable ${update.version} is available. Click here to view the update.`,
        10,
        false,
        openModal,
      );

      return;
    }

    openModal();
  }

  private notifyDevelopmentRestriction(action: "download" | "install" | "apply") {
    const actionText = {
      download: "Downloading updates",
      install: "Installing updates",
      apply: "Applying downloaded updates",
    }[action];

    const message =
      `${actionText} is disabled in development builds. ` +
      "The update notification is shown for testing only.";

    this.error = message;
    app.notificationService.info(message);
  }

  /**
   * Check for launcher updates.
   */
  async check(includePrerelease: boolean = this.includePrerelease) {
    if (this.checking || this.destroyed) return null;

    this.checking = true;
    this.error = null;

    try {
      const result = await api.checkLauncherUpdates(includePrerelease);

      this.updateInfo = result;
      return result;
    } catch (error) {
      const message = String(error);

      if (
        message.includes("No update available") ||
        message.includes("No updates available")
      ) {
        this.updateInfo = null;
        return null;
      }

      this.error = message;

      app.notificationService.error(
        (includePrerelease
          ? "Failed to check for nightly launcher updates: "
          : "Failed to check for launcher updates: ") + message,
      );

      return null;
    } finally {
      this.checking = false;
    }
  }

  /**
   * Download an update without applying it.
   */
  async download(includePrerelease: boolean = this.includePrerelease) {
    if (this.isDevelopmentBuild()) {
      this.notifyDevelopmentRestriction("download");
      return null;
    }

    if (this.downloading || this.destroyed) return null;

    this.downloading = true;
    this.error = null;

    try {
      const path = await api.downloadLauncherUpdate(includePrerelease);

      app.notificationService.success(
        "Launcher update downloaded successfully.",
      );

      return path;
    } catch (error) {
      const message = String(error);
      this.error = message;

      app.notificationService.error(
        `Failed to download launcher update: ${message}`,
      );

      return null;
    } finally {
      this.downloading = false;
    }
  }

  /**
   * Install an update immediately.
   */
  async install(includePrerelease: boolean = this.includePrerelease) {
    if (this.isDevelopmentBuild()) {
      this.notifyDevelopmentRestriction("install");
      return false;
    }

    if (this.installing || this.destroyed) return false;

    this.installing = true;
    this.error = null;

    try {
      await api.installLauncherUpdate(includePrerelease);

      app.notificationService.success(
        "Launcher update installed successfully.",
      );

      this.updateInfo = null;
      return true;
    } catch (error) {
      const message = String(error);
      this.error = message;

      app.notificationService.error(
        `Failed to install launcher update: ${message}`,
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
    if (this.isDevelopmentBuild()) {
      this.notifyDevelopmentRestriction("apply");
      return false;
    }

    if (this.installing || this.destroyed) return false;

    this.installing = true;
    this.error = null;

    try {
      await api.applyDownloadedUpdate();

      app.notificationService.success(
        "Downloaded launcher update applied successfully.",
      );

      this.updateInfo = null;
      return true;
    } catch (error) {
      const message = String(error);
      this.error = message;

      app.notificationService.error(
        `Failed to apply downloaded launcher update: ${message}`,
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
    } catch (error) {
      console.error(
        "API call failed: `api.getCurrentVersion()`",
        error,
      );
      return undefined;
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
