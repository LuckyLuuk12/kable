import { get } from "svelte/store";
import {
  currentAccount,
  installations,
  settings,
  launchedInstallations,
  currentLaunchingInstallation,
  isLaunching,
  launchError,
  launchTimeoutHandle,
} from "$lib";
import * as launcherApi from "$lib/api/launcher";
import type { KableInstallation, LaunchResult } from "$lib";

export class Launcher {
  /**
   * Launch latest Minecraft version
   */
  static async launchLatest(): Promise<LaunchResult> {
    try {
      let latest = get(installations).sort(
        (a: KableInstallation, b: KableInstallation) =>
          b.last_used.localeCompare(a.last_used),
      )[0];
      return await Launcher.launchInstallation(latest);
    } catch (error) {
      console.error("Launch latest failed:", error);
      return {
        pid: -1,
        success: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Launch specific installation
   */
  static async launchInstallation(
    installation: KableInstallation,
  ): Promise<LaunchResult> {
    try {
      const account = get(currentAccount);
      if (!account) throw new Error("No account selected");

      // Signal UI that a launch is starting
      isLaunching.set(true);
      currentLaunchingInstallation.set(installation);
      launchError.set(null);

      // Start a 30s fallback timer to clear the launching UI in case backend/events fail
      try {
        // Clear any existing timeout first
        const prev = get(launchTimeoutHandle);
        if (prev) clearTimeout(prev);
      } catch (e) {}
      const handle = setTimeout(() => {
        // If still launching after 30s, clear UI and set an error
        isLaunching.set(false);
        currentLaunchingInstallation.set(null);
        launchError.set("Launch timed out (no confirmation from backend).");
        // clear stored handle
        launchTimeoutHandle.set(null);
      }, 30_000) as unknown as number;
      launchTimeoutHandle.set(handle);

      const result = await launcherApi.launchInstallation(
        installation,
        get(settings),
        account,
      );

      // If launch succeeded, record launched installation (keep recent-first)
      if (result && result.success) {
        launchedInstallations.update((list) => {
          const filtered = list.filter((i) => i.id !== installation.id);
          return [installation, ...filtered];
        });
      } else if (result && !result.success) {
        launchError.set(result.error || "Unknown launch error");
      }

      // On success we keep the launching indicators active until the backend emits
      // a 'game-started' event which the frontend listens for. This avoids race
      // conditions where the command returns before the OS shows the process.
      if (!result || !result.success) {
        // If there was no success, clear launching indicators immediately
        isLaunching.set(false);
        currentLaunchingInstallation.set(null);
        // clear timeout
        try {
          const prev = get(launchTimeoutHandle);
          if (prev) clearTimeout(prev);
        } catch (e) {}
        launchTimeoutHandle.set(null);
      } else {
        // On success we keep the launching indicators active until the backend emits
        // a 'game-started' event which the frontend listens for. The timer will be
        // cleared by the event listener when it arrives.
      }

      return result;
    } catch (error) {
      console.error(
        "Launching " +
          installation.name +
          " failed with account " +
          JSON.stringify(get(currentAccount)),
        error,
      );
      // Ensure UI state is cleared
      isLaunching.set(false);
      currentLaunchingInstallation.set(null);
      try {
        const prev = get(launchTimeoutHandle);
        if (prev) clearTimeout(prev);
      } catch (e) {}
      launchTimeoutHandle.set(null);
      launchError.set(error instanceof Error ? error.message : String(error));
      return {
        pid: -1,
        success: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Check if Minecraft is currently running
   */
  static async isMinecraftRunning(): Promise<boolean> {
    try {
      return await launcherApi.isMinecraftRunning();
    } catch (error) {
      console.error("Failed to check Minecraft status:", error);
      return false;
    }
  }

  /**
   * Get all running Minecraft processes
   */
  static async getRunningProcesses(): Promise<number[]> {
    try {
      return await launcherApi.getRunningMinecraftProcesses();
    } catch (error) {
      console.error("Failed to get running processes:", error);
      return [];
    }
  }

  /**
   * Kill a specific Minecraft process
   */
  static async killProcess(processId: number): Promise<LaunchResult> {
    try {
      await launcherApi.killMinecraftProcess(processId);
      return { success: true, pid: processId };
    } catch (error) {
      console.error("Failed to kill process:", error);
      return {
        pid: processId,
        success: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Wait for a Minecraft process to exit
   */
  static async waitForExit(processId: number): Promise<void> {
    try {
      await launcherApi.waitForMinecraftExit(processId);
    } catch (error) {
      console.error("Failed to wait for process exit:", error);
    }
  }
}
