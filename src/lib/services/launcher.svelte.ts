
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

  private versionsRequest: Promise<Versions> | null = null;

  async init() {
    await this.autoDetectJava();
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
        `Launching Minecraft with profile "${profile.metadata.name}".`,
      );

      return result;
    } catch (error) {
      app.notificationService.error(
        `Failed to launch Minecraft with profile "${profile.metadata.name}".`,
      );
      throw error;
    } finally {
      this.launching = false;
      this.launchingProfileId = null;
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
   * Load available Minecraft and loader versions on demand.
   * Concurrent callers share the same request.
   */
  async loadVersions(force = false): Promise<Versions> {
    if (this.versions !== null && !force) {
      return this.versions;
    }

    if (this.versionsRequest !== null && !force) {
      return this.versionsRequest;
    }

    this.loadingVersions = true;

    const request = api.getVersions();
    this.versionsRequest = request;

    try {
      const versions = await request;

      if (this.versionsRequest === request) {
        this.versions = versions;
      }

      return versions;
    } finally {
      if (this.versionsRequest === request) {
        this.versionsRequest = null;
        this.loadingVersions = false;
      }
    }
  }

  /**
   * Refresh versions explicitly.
   */
  async refreshVersions(): Promise<Versions> {
    this.loadingVersions = true;

    const request = api.refreshVersions();
    this.versionsRequest = request;

    try {
      const versions = await request;

      if (this.versionsRequest === request) {
        this.versions = versions;
      }

      return versions;
    } finally {
      if (this.versionsRequest === request) {
        this.versionsRequest = null;
        this.loadingVersions = false;
      }
    }
  }

  /**
   * Release the frontend version cache when it is no longer needed.
   */
  clearVersions(): void {
    this.versions = null;
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
    this.versionsRequest = null;
  }
}
