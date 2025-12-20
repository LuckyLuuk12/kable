import { writable, derived, get } from "svelte/store";
import type { GameInstance, LogEntry, GameInstanceLogs } from "../types";
import { settings } from "./settings";

// Configuration for log memory management - backend handles optimization
// Frontend just stores and displays what backend sends
const LOG_CONFIG = {
  maxLogsPerInstance: 10000, // High limit - backend manages actual limit
  maxGlobalLogs: 10000, // High limit - backend manages actual limit
};

// Subscribe to settings to update log limits dynamically
settings.subscribe(($settings) => {
  if ($settings?.logging?.max_memory_logs) {
    const limit = $settings.logging.max_memory_logs;
    // If limit is 0 or negative, use very high number
    if (limit > 0) {
      LOG_CONFIG.maxLogsPerInstance = limit;
      LOG_CONFIG.maxGlobalLogs = limit;
    } else {
      LOG_CONFIG.maxLogsPerInstance = Number.MAX_SAFE_INTEGER;
      LOG_CONFIG.maxGlobalLogs = Number.MAX_SAFE_INTEGER;
    }
  }
});

// Active game instances
export const gameInstances = writable<Map<string, GameInstance>>(new Map());

// All logs data
export const logsData = writable<Map<string, GameInstanceLogs>>(new Map());

// Global launcher logs (not tied to specific instances)
export const globalLauncherLogs = writable<LogEntry[]>([]);

// Currently selected instance/tab
export const selectedInstanceId = writable<string | "global">("global");

// Derived store for current logs based on selection
export const currentLogs = derived(
  [logsData, globalLauncherLogs, selectedInstanceId],
  ([$logsData, $globalLogs, $selectedId]) => {
    try {
      if ($selectedId === "global") {
        return {
          launcherLogs: $globalLogs || [],
          gameLogs: [] as LogEntry[],
        };
      }

      const instanceLogs = $logsData.get($selectedId);
      return instanceLogs
        ? {
            launcherLogs: instanceLogs.launcherLogs || [],
            gameLogs: instanceLogs.gameLogs || [],
          }
        : {
            launcherLogs: [] as LogEntry[],
            gameLogs: [] as LogEntry[],
          };
    } catch (error) {
      console.error("Error in currentLogs derived store:", error);
      return {
        launcherLogs: [] as LogEntry[],
        gameLogs: [] as LogEntry[],
      };
    }
  },
);

// Helper function to trim log arrays to max size (circular buffer behavior)
// Backend handles this, but keep as safety net
function trimLogsToMaxSize(logs: LogEntry[], maxSize: number): LogEntry[] {
  if (logs.length <= maxSize) return logs;
  // Keep the most recent logs
  return logs.slice(logs.length - maxSize);
}

// Helper functions
export const LogsManager = {
  addGameInstance(instance: GameInstance) {
    gameInstances.update((instances) => {
      const newInstances = new Map(instances);
      newInstances.set(instance.id, instance);
      return newInstances;
    });

    logsData.update((logs) => {
      const newLogs = new Map(logs);
      newLogs.set(instance.id, {
        instanceId: instance.id,
        launcherLogs: [],
        gameLogs: [],
      });
      return newLogs;
    });
  },

  updateGameInstance(instanceId: string, updates: Partial<GameInstance>) {
    gameInstances.update((instances) => {
      const newInstances = new Map(instances);
      const instance = newInstances.get(instanceId);
      if (instance) {
        newInstances.set(instanceId, { ...instance, ...updates });
      }
      return newInstances;
    });
  },

  removeGameInstance(instanceId: string) {
    gameInstances.update((instances) => {
      const newInstances = new Map(instances);
      newInstances.delete(instanceId);
      return newInstances;
    });

    // Also remove logs to free memory (only when explicitly removing the instance)
    logsData.update((logs) => {
      const newLogs = new Map(logs);
      newLogs.delete(instanceId);
      return newLogs;
    });
  },

  // Cleanup old closed instances after a delay to free memory
  // This is called manually from LogService, not automatically by the store
  cleanupOldInstances(maxAge: number = 30 * 60 * 1000) {
    // Default 30 minutes
    const now = Date.now();
    const instances = get(gameInstances);
    const toRemove: string[] = [];

    instances.forEach((instance, id) => {
      if (
        instance.status === "closed" ||
        instance.status === "crashed" ||
        instance.status === "stopped"
      ) {
        const lastActivity =
          instance.lastActivity instanceof Date
            ? instance.lastActivity.getTime()
            : new Date(instance.lastActivity).getTime();

        if (now - lastActivity > maxAge) {
          toRemove.push(id);
        }
      }
    });

    toRemove.forEach((id) => this.removeGameInstance(id));
    return toRemove.length;
  },

  addLauncherLog(
    message: string,
    level: LogEntry["level"] = "info",
    instanceId?: string,
  ) {
    // Backend handles deduplication - just add the log
    const logEntry: LogEntry = {
      timestamp: new Date(),
      level,
      source: "launcher",
      instanceId,
      message,
      raw: message,
    };

    if (instanceId) {
      logsData.update((logs) => {
        const newLogs = new Map(logs);
        const instanceLogs = newLogs.get(instanceId);
        if (instanceLogs) {
          // Add new log and trim to max size (safety net)
          const updatedLogs = trimLogsToMaxSize(
            [...instanceLogs.launcherLogs, logEntry],
            LOG_CONFIG.maxLogsPerInstance,
          );
          instanceLogs.launcherLogs = updatedLogs;
          newLogs.set(instanceId, { ...instanceLogs });
        }
        return newLogs;
      });
    } else {
      globalLauncherLogs.update((logs) => {
        // Add new log and trim to max size (safety net)
        return trimLogsToMaxSize([...logs, logEntry], LOG_CONFIG.maxGlobalLogs);
      });
    }
  },

  addGameLog(
    instanceId: string,
    message: string,
    level: LogEntry["level"] = "info",
  ) {
    // Backend handles deduplication - just add the log
    const logEntry: LogEntry = {
      timestamp: new Date(),
      level,
      source: "game",
      instanceId,
      message,
      raw: message,
    };

    logsData.update((logs) => {
      const newLogs = new Map(logs);
      const instanceLogs = newLogs.get(instanceId);
      if (instanceLogs) {
        // Add new log and trim to max size (safety net)
        const updatedLogs = trimLogsToMaxSize(
          [...instanceLogs.gameLogs, logEntry],
          LOG_CONFIG.maxLogsPerInstance,
        );
        instanceLogs.gameLogs = updatedLogs;
        newLogs.set(instanceId, { ...instanceLogs });
      }
      return newLogs;
    });
  },

  clearLogs(instanceId?: string) {
    if (instanceId === "global" || !instanceId) {
      globalLauncherLogs.set([]);
    } else {
      logsData.update((logs) => {
        const newLogs = new Map(logs);
        const instanceLogs = newLogs.get(instanceId);
        if (instanceLogs) {
          instanceLogs.launcherLogs = [];
          instanceLogs.gameLogs = [];
          newLogs.set(instanceId, instanceLogs);
        }
        return newLogs;
      });
    }
  },

  // Emit global launcher events for better logging
  emitLauncherEvent(
    message: string,
    level: "info" | "warn" | "error" = "info",
  ) {
    LogsManager.addLauncherLog(message, level);
  },

  // Get memory usage statistics
  getMemoryStats() {
    let totalLogs = 0;
    let instanceCount = 0;

    // Use get() instead of subscribe() to avoid memory leaks
    const logs = get(logsData);
    logs.forEach((instanceLogs) => {
      totalLogs +=
        instanceLogs.launcherLogs.length + instanceLogs.gameLogs.length;
      instanceCount++;
    });

    const globalLogs = get(globalLauncherLogs);
    totalLogs += globalLogs.length;

    return {
      totalLogs,
      instanceCount,
      maxLogsPerInstance: LOG_CONFIG.maxLogsPerInstance,
      maxGlobalLogs: LOG_CONFIG.maxGlobalLogs,
    };
  },
};

// Export config for external access
export const logConfig = LOG_CONFIG;
