import { type KableProfile, api } from "$lib";
import type { Service } from "./app.service";

export class DiscordService implements Service {
  enabled = $state(false);

  initialized = $state(false);
  connected = $state(false);

  currentState = $state<"idle" | "playing" | "browsing" | "disabled">("idle");

  lastSection = $state<string | null>(null);
  lastProfile = $state<KableProfile | null>(null);

  error = $state<string | null>(null);

  /**
   * Initialize Discord RPC connection (must be called once at startup)
   */
  async init() {
    this.error = null;

    try {
      await api.initializeDiscordRpc();
      this.initialized = true;
      this.connected = true;
      this.enabled = true;
      this.currentState = "idle";
    } catch (e) {
      this.error = String(e);
      this.connected = false;
    }
  }

  /**
   * Enable or disable Discord integration globally
   */
  async setEnabled(enabled: boolean) {
    this.error = null;

    try {
      await api.setDiscordEnabled(enabled);
      this.enabled = enabled;

      if (!enabled) {
        this.currentState = "disabled";
        await api.clearDiscordPresence();
      } else {
        this.currentState = "idle";
      }
    } catch (e) {
      this.error = String(e);
    }
  }

  /**
   * Set "playing Minecraft" state for a profile
   */
  async setPlaying(profile: KableProfile) {
    if (!this.enabled) return;

    this.error = null;

    try {
      await api.setDiscordPlaying(profile);
      this.currentState = "playing";
      this.lastProfile = profile;
    } catch (e) {
      this.error = String(e);
    }
  }

  /**
   * Set "browsing launcher" state
   */
  async setBrowsing(section: string) {
    if (!this.enabled) return;

    this.error = null;

    try {
      await api.setDiscordBrowsing(section);
      this.currentState = "browsing";
      this.lastSection = section;
    } catch (e) {
      this.error = String(e);
    }
  }

  /**
   * Clear only the "playing" state (fallback to idle)
   */
  async clearPlaying() {
    if (!this.enabled) return;

    this.error = null;

    try {
      await api.clearDiscordPlaying();
      this.currentState = "idle";
      this.lastProfile = null;
    } catch (e) {
      this.error = String(e);
    }
  }

  /**
   * Hard reset all presence
   */
  async clearAll() {
    this.error = null;

    try {
      await api.clearDiscordPresence();
      this.currentState = "idle";
      this.lastProfile = null;
      this.lastSection = null;
    } catch (e) {
      this.error = String(e);
    }
  }

  /**
   * Fully disconnect from Discord IPC
   */
  async disconnect() {
    this.error = null;

    try {
      await api.disconnectDiscord();
      this.connected = false;
      this.initialized = false;
      this.enabled = false;
      this.currentState = "disabled";
    } catch (e) {
      this.error = String(e);
    }
  }

  /**
   * Convenience: auto-update based on app state
   */
  async syncFromApp(state: {
    profile?: KableProfile | null;
    section?: string;
    mode: "idle" | "browse" | "play";
  }) {
    if (!this.enabled) return;

    switch (state.mode) {
      case "play":
        if (state.profile) {
          await this.setPlaying(state.profile);
        }
        break;

      case "browse":
        if (state.section) {
          await this.setBrowsing(state.section);
        }
        break;

      case "idle":
      default:
        await this.clearPlaying();
        break;
    }
  }

  /**
   * Cleanup
   */
  async destroy() {
    this.initialized = false;
    this.connected = false;
    this.enabled = false;

    this.currentState = "idle";
    this.lastProfile = null;
    this.lastSection = null;
    this.error = null;
  }
}
