import { type KableProfile, type LaunchResult, type Versions, api } from "$lib";
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
    // Load versions, javapath, etc. on startup
    await Promise.all([this.loadVersions(), this.autoDetectJava()]);
  }

  /**
   * Launch a Minecraft profile
   */
  async launch(profile: KableProfile): Promise<LaunchResult> {
    this.launching = true;
    this.launchingProfileId = profile.id;

    try {
      const result = await api.launchGame(profile);
      this.lastLaunch = result;
      return result;
    } finally {
      this.launching = false;
      this.launchingProfileId = null;
      // TODO: execute the on_game_launch action if set in settings
    }
  }

  async launchLatest() {
    const profiles = await api.getProfiles();
    if (!profiles || profiles.length === 0) {
      throw new Error("No profiles found to launch");
    }
    const latestProfile = profiles[0];
    return await this.launch(latestProfile);
  }

  /**
   * Auto-detect Java installation path
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
   * Resolve / validate a Java path
   */
  async getJavaPath(javaPath: string | null) {
    return await api.getJavaPath(javaPath);
  }

  /**
   * Load available Minecraft / loader versions
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
   * Refresh versions explicitly
   */
  async refreshVersions() {
    this.versions = await api.refreshVersions();
    return this.versions;
  }

  /**
   * Clears runtime state (useful on profile switch or app reset)
   */
  async destroy() {
    this.launching = false;
    this.lastLaunch = null;
    this.javaPath = null;
    this.versions = null;
    this.loadingVersions = false;
    this.loadingJava = false;
  }
}
