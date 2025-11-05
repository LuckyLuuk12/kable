import * as discordApi from "$lib/api/discord";

/**
 * Discord Rich Presence Service
 * Manages Discord RPC updates based on user activity
 */
export class DiscordService {
  private static enabled = true;

  /**
   * Initialize Discord Rich Presence
   */
  static async initialize(): Promise<void> {
    // Discord is initialized on backend startup
    // This is just a placeholder for future frontend state management
  }

  /**
   * Enable or disable Discord Rich Presence
   */
  static async setEnabled(enabled: boolean): Promise<void> {
    this.enabled = enabled;
    try {
      await discordApi.setEnabled(enabled);
    } catch (error) {
      console.error("Failed to set Discord enabled state:", error);
    }
  }

  /**
   * Update Discord status based on current route/activity
   */
  static async updateBrowsing(route: string): Promise<void> {
    if (!this.enabled) return;

    try {
      // Extract section from route path
      const section = route.split("/")[1] || "home";
      await discordApi.setBrowsing(section);
    } catch (error) {
      console.error("Failed to update Discord browsing status:", error);
    }
  }

  /**
   * Clear Discord Rich Presence
   */
  static async clear(): Promise<void> {
    try {
      await discordApi.clear();
    } catch (error) {
      console.error("Failed to clear Discord status:", error);
    }
  }
}
