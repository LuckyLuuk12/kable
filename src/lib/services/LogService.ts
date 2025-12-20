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
  // Reduced buffer size to prevent excessive memory usage (100 lines is enough for crash detection)
  private crashBufferSize: number = 100;
  private static instance: LogsService;
  private isInitialized: boolean = false;
  private cleanupInterval: number | null = null;
  private isPaused: boolean = false;
  private pausedLogs: Array<{ type: "launcher" | "game"; data: any }> = [];

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

            // Clean up crash buffer for this instance to prevent memory leaks
            this.crashBuffers.delete(instanceId);

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

    // Listen for batched launcher log events (new optimized system)
    const launcherLogBatchListener = await listen(
      "launcher-log-batch",
      (event) => {
        if (this.isPaused) {
          // Queue logs while paused
          this.pausedLogs.push({ type: "launcher", data: event.payload });
          return;
        }

        try {
          // Backend now sends { logs: [...], maxLogs: number }
          const payload = event.payload as {
            logs: Array<{
              level: LogEntry["level"];
              message: string;
              instanceId?: string;
              timestamp: string;
            }>;
            maxLogs: number;
          };

          // Process batch of logs - backend already handled deduplication
          for (const log of payload.logs) {
            LogsManager.addLauncherLog(log.message, log.level, log.instanceId);
          }
        } catch (error) {
          console.error("Error handling launcher log batch event:", error);
          LogsManager.addLauncherLog(
            "Error processing launcher log batch event",
            "error",
          );
        }
      },
    );

    // Also listen for single launcher log events (backward compatibility)
    const launcherLogListener = await listen("launcher-log", (event) => {
      if (this.isPaused) {
        // Queue logs while paused
        this.pausedLogs.push({ type: "launcher", data: event.payload });
        return;
      }

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
    this.listeners.set("launcher-log-batch", launcherLogBatchListener);
    this.listeners.set("launcher-log", launcherLogListener);
    this.listeners.set("show-logs-page", showLogsListener);

    // Mark as initialized
    this.isInitialized = true;

    // Setup periodic cleanup of crash buffers for closed instances
    this.cleanupInterval = window.setInterval(() => {
      this.cleanupCrashBuffers();
      // Also cleanup old game instances from store
      const removed = LogsManager.cleanupOldInstances(30 * 60 * 1000); // 30 minutes
      if (removed > 0) {
        console.log(`[LogsService] Cleaned up ${removed} old game instances`);
      }
    }, 60000); // Every minute

    console.log("Logs service initialization complete");
  }

  private cleanupCrashBuffers() {
    const instances = get(gameInstances);
    const toDelete: string[] = [];

    // Remove buffers for instances that no longer exist or are closed
    for (const instanceId of this.crashBuffers.keys()) {
      const instance = instances.get(instanceId);
      if (
        !instance ||
        instance.status === "closed" ||
        instance.status === "crashed" ||
        instance.status === "stopped"
      ) {
        const lastActivity = instance?.lastActivity;
        if (!lastActivity) {
          toDelete.push(instanceId);
        } else {
          const activityTime =
            lastActivity instanceof Date
              ? lastActivity.getTime()
              : new Date(lastActivity).getTime();
          // Remove buffers older than 10 minutes
          if (Date.now() - activityTime > 10 * 60 * 1000) {
            toDelete.push(instanceId);
          }
        }
      }
    }

    toDelete.forEach((id) => this.crashBuffers.delete(id));
  }

  private parseGameOutput(instanceId: string, line: string) {
    // Skip if paused
    if (this.isPaused) {
      this.pausedLogs.push({ type: "game", data: { instanceId, line } });
      return;
    }

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

  /**
   * Pause log ingestion temporarily (for operations like copying all logs)
   */
  pause(): void {
    this.isPaused = true;
    this.pausedLogs = [];
  }

  /**
   * Resume log ingestion and flush any queued logs
   */
  resume(): void {
    this.isPaused = false;

    // Process any queued logs
    const queued = [...this.pausedLogs];
    this.pausedLogs = [];

    for (const item of queued) {
      if (item.type === "launcher") {
        // Handle both new payload format { logs: [...], maxLogs: number } and old formats
        const data = item.data;
        if (data.logs && Array.isArray(data.logs)) {
          // New batch format from backend
          for (const log of data.logs) {
            LogsManager.addLauncherLog(log.message, log.level, log.instanceId);
          }
        } else if (Array.isArray(data)) {
          // Old batch format (array of logs)
          for (const log of data) {
            LogsManager.addLauncherLog(log.message, log.level, log.instanceId);
          }
        } else if (data.level && data.message) {
          // Single log format
          LogsManager.addLauncherLog(data.message, data.level, data.instanceId);
        }
      } else if (item.type === "game") {
        const { instanceId, line } = item.data;
        this.parseGameOutput(instanceId, line);
      }
    }
  }

  destroy() {
    for (const [, unlisten] of this.listeners) {
      unlisten();
    }
    this.listeners.clear();

    // Clear cleanup interval
    if (this.cleanupInterval !== null) {
      clearInterval(this.cleanupInterval);
      this.cleanupInterval = null;
    }

    // Clear crash buffers
    this.crashBuffers.clear();

    this.isInitialized = false;
    console.log("Logs service destroyed");
  }
}

// Export singleton instance
export const logsService = LogsService.getInstance();
