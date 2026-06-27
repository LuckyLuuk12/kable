import {
  type KableProfile,
  type LaunchResult,
  type Versions,
  api,
} from "$lib";

class LauncherService {
  launching = $state(false);
  lastLaunch = $state<LaunchResult | null>(null);

  javaPath = $state<string | null>(null);
  versions = $state<Versions | null>(null);

  loadingVersions = $state(false);
  loadingJava = $state(false);

  /**
   * Launch a Minecraft profile
   */
  async launch(profile: KableProfile): Promise<LaunchResult> {
    this.launching = true;

    try {
      const result = await api.launchGame(profile);
      this.lastLaunch = result;
      return result;
    } finally {
      this.launching = false;
      // TODO: execute the on_game_launch action if set in settings
    }
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
    this.versions = null;
    return await this.loadVersions(true);
  }

  /**
   * Reset runtime state (useful on profile switch or app reset)
   */
  reset() {
    this.launching = false;
    this.lastLaunch = null;
    this.javaPath = null;
    this.versions = null;
    this.loadingVersions = false;
    this.loadingJava = false;
  }
}

export const launcherService = new LauncherService();