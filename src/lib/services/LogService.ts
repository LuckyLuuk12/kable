import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { get } from "svelte/store";
import { LogsManager, gameInstances } from "../stores/logs";
import type { GameInstance, LogEntry } from "../types";

export class LogsService {
  /**
   * Static helper for emitting launcher log events (compat with old manager)
   */
  static emitLauncherEvent(
    message: string,
    level: LogEntry["level"] = "info",
    instanceId?: string,
  ) {
    LogsManager.addLauncherLog(message, level, instanceId);
  }
  private listeners: Map<string, UnlistenFn> = new Map();
  // Small sliding window of recent log lines per instance to make crash detection
  // less sensitive to single 'error' or stack-trace lines coming from mods/plugins.
  private crashBuffers: Map<string, string[]> = new Map();
  // Increase buffer to capture long stack traces and more context for crash summaries
  private crashBufferSize: number = 1000;
  private static instance: LogsService;
  private isInitialized: boolean = false;

  static getInstance(): LogsService {
    if (!LogsService.instance) {
      LogsService.instance = new LogsService();
    }
    return LogsService.instance;
  }

  async initialize() {
    // Prevent multiple initializations
    if (this.isInitialized) {
      console.log("Logs service already initialized, skipping...");
      return;
    }

    console.log("Initializing logs service...");

    // Initialize global launcher logs
    LogsManager.addLauncherLog("Logs service initialized", "info");

    // Listen for game launch events
    const launchListener = await listen("game-launched", (event) => {
      console.log(
        "[LogsService] Received game-launched event:",
        JSON.stringify(event.payload, null, 2),
      );
      try {
        const { instanceId, profile, installation } = event.payload as {
          instanceId: string;
          profile: any;
          installation: any;
        };

        console.log("[LogsService] Extracted data:", {
          instanceId,
          profileName: profile?.name,
          installationPath: installation?.path,
        });

        const gameInstance: GameInstance = {
          id: instanceId,
          profileName: profile.name,
          installationPath: installation.path,
          status: "launching",
          launchedAt: new Date(),
          lastActivity: new Date(),
        };

        LogsManager.addGameInstance(gameInstance);
        LogsManager.addLauncherLog(
          `Launching ${profile.name} (${installation.mod_loader || "vanilla"})`,
          "info",
          instanceId,
        );
        console.log("[LogsService] Created game instance:", gameInstance);
      } catch (error) {
        console.error("Error handling game launch event:", error);
        LogsManager.addLauncherLog(
          "Error processing game launch event",
          "error",
        );
      }
    });

    // Listen for game process events
    const processListener = await listen("game-process-event", (event) => {
      try {
        const { instanceId, type, data } = event.payload as {
          instanceId: string;
          type: "started" | "output" | "error" | "exit";
          data: any;
        };

        switch (type) {
          case "started":
            LogsManager.updateGameInstance(instanceId, {
              status: "running",
              processId: data.pid,
            });
            LogsManager.addLauncherLog(
              `Game process started (PID: ${data.pid})`,
              "info",
              instanceId,
            );
            break;

          case "output":
            this.parseGameOutput(instanceId, data.line);
            break;

          case "error":
            LogsManager.addGameLog(instanceId, data.line, "error");
            LogsManager.updateGameInstance(instanceId, {
              lastActivity: new Date(),
            });
            break;

          case "exit":
            console.log(
              "[LogsService] Processing exit event for instanceId:",
              instanceId,
            );
            // Better exit code interpretation
            const exitCode = data.code;
            let status: "closed" | "crashed" | "stopped";

            if (exitCode === 0) {
              status = "closed"; // Normal exit
            } else if (
              exitCode === 130 ||
              exitCode === 143 ||
              exitCode === -1073741510
            ) {
              status = "stopped"; // User terminated (Ctrl+C, SIGTERM, or Windows close)
            } else if (exitCode < 0 || exitCode > 128) {
              status = "crashed"; // Abnormal exit or system termination
            } else {
              status = "stopped"; // Other controlled exits
            }

            console.log("[LogsService] Updating game instance:", {
              instanceId,
              status,
              exitCode,
            });
            LogsManager.updateGameInstance(instanceId, {
              status,
              exitCode,
              completedAt: new Date(),
            });

            const statusMessage =
              status === "closed"
                ? "completed normally"
                : status === "stopped"
                  ? "was stopped by user"
                  : "crashed";

            LogsManager.addLauncherLog(
              `Game process ${statusMessage} (exit code: ${exitCode})`,
              status === "crashed" ? "error" : "info",
              instanceId,
            );
            console.log("[LogsService] Exit event processed successfully");
            break;
        }
      } catch (error) {
        console.error("Error handling game process event:", error);
        LogsManager.addLauncherLog(
          "Error processing game process event",
          "error",
        );
      }
    });

    // Listen for launcher log events
    const launcherLogListener = await listen("launcher-log", (event) => {
      try {
        const { level, message, instanceId } = event.payload as {
          level: LogEntry["level"];
          message: string;
          instanceId?: string;
        };

        LogsManager.addLauncherLog(message, level, instanceId);
      } catch (error) {
        console.error("Error handling launcher log event:", error);
        LogsManager.addLauncherLog(
          "Error processing launcher log event",
          "error",
        );
      }
    });

    // Listen for show logs page events
    const showLogsListener = await listen("show-logs-page", (event) => {
      console.log("Received show logs page event:", event);
      try {
        const { instanceId, installationId, reason } = event.payload as {
          instanceId: string;
          installationId: string;
          reason: string;
        };

        console.log(
          `Show logs request: ${reason} for ${installationId} (${instanceId})`,
        );
        LogsManager.addLauncherLog(
          `Navigating to logs page (${reason})`,
          "info",
          instanceId,
        );

        // Navigate to logs page and set the active instance
        this.navigateToLogs(instanceId);
      } catch (error) {
        console.error("Error handling show-logs-page event:", error);
        LogsManager.addLauncherLog("Error processing show logs event", "error");
      }
    });

    this.listeners.set("game-launched", launchListener);
    this.listeners.set("game-process-event", processListener);
    this.listeners.set("launcher-log", launcherLogListener);
    this.listeners.set("show-logs-page", showLogsListener);

    // Mark as initialized
    this.isInitialized = true;
    console.log("Logs service initialization complete");
  }

  private parseGameOutput(instanceId: string, line: string) {
    // Update last activity
    LogsManager.updateGameInstance(instanceId, { lastActivity: new Date() });

    // Maintain small per-instance buffer of recent lines for more accurate crash detection
    const buf = this.crashBuffers.get(instanceId) || [];
    buf.push(line);
    if (buf.length > this.crashBufferSize) buf.shift();
    this.crashBuffers.set(instanceId, buf);

    // Determine log level based on content
    let level: LogEntry["level"] = "info";
    const lowerLine = line.toLowerCase();

    if (
      lowerLine.includes("error") ||
      lowerLine.includes("exception") ||
      lowerLine.includes("failed")
    ) {
      level = "error";
    } else if (lowerLine.includes("warn") || lowerLine.includes("warning")) {
      level = "warn";
    } else if (lowerLine.includes("debug") || lowerLine.includes("trace")) {
      level = "debug";
    }

    LogsManager.addGameLog(instanceId, line, level);

    // Only check for actual crashes, not just any error. Use buffered context.
    if (this.isCrashIndicator(instanceId, line)) {
      // Only update to crashed if not already in a final state
      const instances = get(gameInstances);
      const instance = instances.get(instanceId);
      if (instance && instance.status === "running") {
        LogsManager.updateGameInstance(instanceId, { status: "crashed" });
        // Include a short summary of recent log lines to explain why we marked this as crashed
        const crashSummary = this.getCrashSummary(instanceId);
        LogsManager.addLauncherLog(
          `Game crash detected from output:\n${crashSummary}`,
          "error",
          instanceId,
        );
      }
    }
  }
  private isCrashIndicator(instanceId: string, line: string): boolean {
    // Gather recent context
    const buf = this.crashBuffers.get(instanceId) || [];
    const context = [...buf].join("\n").toLowerCase();

    // Strong indicators that should immediately mark a crash
    const strongPatterns: RegExp[] = [
      /---- minecraft crash report ----/i,
      /a fatal error has been detected by the java runtime environment/i,
      /fatal error/i,
      /segmentation fault/i,
      /access violation/i,
      /exception_access_violation/i,
      /sig(segv|ill|abrt|bus)/i,
      /the game crashed!/i,
      /crash report saved to/i,
      /out of memory/i,
      /java\.lang\.outofmemoryerror/i,
    ];

    for (const p of strongPatterns) {
      if (p.test(context)) return true;
    }
    return false;
  }

  /**
   * Return a short, safe summary of recent lines for crash reporting.
   * We keep this concise to avoid dumping huge logs into the launcher log.
   */
  private getCrashSummary(instanceId: string): string {
    const buf = this.crashBuffers.get(instanceId) || [];
    if (buf.length === 0) return "No recent log lines available.";

    // Use the last N lines (increased so developer has more context) and keep the total
    // length under a larger sensible limit to allow deeper inspection.
    const lastLines = buf.slice(-200);
    const joined = lastLines.join("\n");
    const maxLen = 8000;
    if (joined.length > maxLen) {
      // keep the tail (most relevant recent lines)
      return "... (truncated)\n" + joined.slice(joined.length - maxLen);
    }
    return joined;
  }

  async exportLogs(instanceId?: string): Promise<void> {
    try {
      await invoke("export_logs", { instanceId });
      LogsManager.addLauncherLog("Logs exported successfully", "info");
    } catch (error) {
      LogsManager.addLauncherLog(`Failed to export logs: ${error}`, "error");
    }
  }

  async clearLogs(instanceId?: string): Promise<void> {
    LogsManager.clearLogs(instanceId);
    LogsManager.addLauncherLog(
      instanceId
        ? `Cleared logs for instance ${instanceId}`
        : "Cleared global logs",
      "info",
    );
  }

  /**
   * Navigate to logs page and optionally set active instance
   */
  private navigateToLogs(instanceId?: string): void {
    try {
      // Build URL
      let url = "/logs";
      if (instanceId) {
        // Add instance ID as a query parameter
        url += `?instance=${encodeURIComponent(instanceId)}`;
      }

      // Use SvelteKit's goto to navigate
      import("$app/navigation")
        .then(({ goto }) => {
          goto(url);
        })
        .catch((error) => {
          console.error("Failed to navigate to logs page:", error);
          // Fallback to window location
          window.location.href = url;
        });
    } catch (error) {
      console.error("Error navigating to logs page:", error);
    }
  }

  isReady(): boolean {
    return this.isInitialized;
  }

  destroy() {
    for (const [, unlisten] of this.listeners) {
      unlisten();
    }
    this.listeners.clear();
    this.isInitialized = false;
    console.log("Logs service destroyed");
  }
}

// Export singleton instance
export const logsService = LogsService.getInstance();
