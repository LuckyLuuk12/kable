import { type KableProfile, type LaunchResult, type Versions, api, app } from "$lib";
import type { Service } from "./app.svelte";

export class LauncherService implements Service {
  launching = $state(false);
  launchingProfileId = $state<string | null>(null);
  lastLaunch = $state<LaunchResult | null>(null);

  javaPath = $state<string | null>(null);
  versions = $state<Versions | null>(null);

  loadingVersions = $state(false);
  loadingJava = $state(false);

  async init() {
    await Promise.all([this.loadVersions(), this.autoDetectJava()]);
  }

  /**
   * Launch a Minecraft profile.
   */
  async launch(profile: KableProfile): Promise<LaunchResult> {
    this.launching = true;
    this.launchingProfileId = profile.id;

    try {
      const result = await api.launchGame(profile);
      this.lastLaunch = result;

      app.notificationService.success(
        `Launching Minecraft with profile "${profile.metadata.name}".`
      );

      return result;
    } catch (error) {
      app.notificationService.error(
        `Failed to launch Minecraft with profile "${profile.metadata.name}".`
      );
      throw error;
    } finally {
      this.launching = false;
      this.launchingProfileId = null;
      // TODO: execute the on_game_launch action if set in settings
    }
  }

  async launchLatest() {
    try {
      const profiles = await api.getProfiles(true);

      if (!profiles || profiles.length === 0) {
        throw new Error("No profiles found to launch.");
      }

      return await this.launch(profiles[0]);
    } catch (error) {
      if (error instanceof Error && error.message === "No profiles found to launch.") {
        app.notificationService.warning(error.message);
      }

      throw error;
    }
  }

  /**
   * Auto-detect Java installation path.
   */
  async autoDetectJava() {
    this.loadingJava = true;

    try {
      const path = await api.autoDetectJava();
      this.javaPath = path;
      return path;
    } finally {
      this.loadingJava = false;
    }
  }

  /**
   * Resolve or validate a Java path.
   */
  async getJavaPath(javaPath: string | null) {
    try {
      return await api.getJavaPath(javaPath);
    } catch (error) {
      console.error("API call failed: `return await api.getJavaPath(javaPath);`", error);
      return null;
    }
  }

  /**
   * Load available Minecraft and loader versions.
   */
  async loadVersions(force: boolean = false) {
    if (this.versions && !force) return this.versions;

    this.loadingVersions = true;

    try {
      const versions = await api.getVersions();
      this.versions = versions;
      return versions;
    } finally {
      this.loadingVersions = false;
    }
  }

  /**
   * Refresh versions explicitly.
   */
  async refreshVersions() {
    this.loadingVersions = true;

    try {
      this.versions = await api.refreshVersions();
      return this.versions;
    } finally {
      this.loadingVersions = false;
    }
  }

  /**
   * Clear runtime state, for example on profile switch or app reset.
   */
  async destroy() {
    this.launching = false;
    this.launchingProfileId = null;
    this.lastLaunch = null;
    this.javaPath = null;
    this.versions = null;
    this.loadingVersions = false;
    this.loadingJava = false;
  }
}
