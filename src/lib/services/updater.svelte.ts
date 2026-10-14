import { type UpdateData, api, app } from "$lib";
import type { Service } from "./app.svelte";

export class UpdaterService implements Service {
  updateInfo = $state<UpdateData | null>(null);

  checking = $state(false);
  downloading = $state(false);
  installing = $state(false);

  error = $state<string | null>(null);

  async init() {
    try {
      const settings = await api.getSettings();
      const generalSettings = settings?.general as { auto_update_launcher?: boolean } | undefined;
      if (generalSettings?.auto_update_launcher === false) return;

      await this.check(false);
    } catch (error) {
      this.error = String(error);
    }
  }

  /**
   * Check for updates
   */
  async check(includePrerelease: boolean = false) {
    this.checking = true;
    this.error = null;

    try {
      const result = await api.checkLauncherUpdates(includePrerelease);
      this.updateInfo = result;
      return result;
    } catch (e) {
      this.error = String(e);
      app.notificationService.error("Failed to check for launcher updates.");
      return null;
    } finally {
      this.checking = false;
    }
  }

  /**
   * Download update without applying
   */
  async download(includePrerelease: boolean = false) {
    this.downloading = true;
    this.error = null;

    try {
      const path = await api.downloadLauncherUpdate(includePrerelease);

      app.notificationService.success("Launcher update downloaded successfully.");

      return path;
    } catch (e) {
      this.error = String(e);
      app.notificationService.error("Failed to download launcher update.");
      return null;
    } finally {
      this.downloading = false;
    }
  }

  /**
   * Install update immediately (blocking behavior depends on backend)
   */
  async install(includePrerelease: boolean = false) {
    this.installing = true;
    this.error = null;

    try {
      await api.installLauncherUpdate(includePrerelease);

      app.notificationService.success("Launcher update installed successfully.");

      return true;
    } catch (e) {
      this.error = String(e);
      app.notificationService.error("Failed to install launcher update.");
      return false;
    } finally {
      this.installing = false;
    }
  }

  /**
   * Apply already downloaded update (usually restart-triggered)
   */
  async applyDownloaded() {
    this.installing = true;
    this.error = null;

    try {
      await api.applyDownloadedUpdate();

      app.notificationService.success("Downloaded launcher update applied successfully.");

      return true;
    } catch (e) {
      this.error = String(e);
      app.notificationService.error("Failed to apply downloaded launcher update.");
      return false;
    } finally {
      this.installing = false;
    }
  }

  /**
   * Current version (static read)
   */
  async getCurrentVersion() {
    try {
      return await api.getCurrentVersion();
    } catch (e) {
      console.error("API call failed: `return await api.getCurrentVersion();`", e);
    }
  }

  /**
   * Reset state (useful on logout/app restart)
   */
  async destroy() {
    this.updateInfo = null;
    this.checking = false;
    this.downloading = false;
    this.installing = false;
    this.error = null;
  }
}
