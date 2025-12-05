import { writable, derived, get } from "svelte/store";
import type { GameInstance, LogEntry, GameInstanceLogs } from "../types";
import { settings } from "./settings";

// Configuration for log memory management
const LOG_CONFIG = {
  maxLogsPerInstance: 5000, // Maximum logs to keep per instance (overridden by settings)
  maxGlobalLogs: 5000, // Maximum global launcher logs (overridden by settings)
  dedupeWindowSize: 50, // Check last N messages for duplicates
  enableDedupe: true, // Enable deduplication
  dedupeTimeWindow: 10000, // Time window for duplicate detection (ms)
};

// Subscribe to settings to update log limits dynamically
settings.subscribe(($settings) => {
  if ($settings?.logging?.max_memory_logs) {
    const limit = $settings.logging.max_memory_logs;
    // If limit is 0 or negative, disable limit (keep all logs)
    if (limit > 0) {
      LOG_CONFIG.maxLogsPerInstance = limit;
      LOG_CONFIG.maxGlobalLogs = limit;
    } else {
      // No limit - use a very high number
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

// Recent launcher messages for duplicate detection (last 5 minutes)
const recentLauncherMessages = new Map<string, number>();

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
function trimLogsToMaxSize(logs: LogEntry[], maxSize: number): LogEntry[] {
  if (logs.length <= maxSize) return logs;
  // Keep the most recent logs
  return logs.slice(logs.length - maxSize);
}

// Helper function to normalize message for deduplication (strip timestamp, trim whitespace)
function normalizeMessage(message: string): string {
  // Remove common timestamp patterns: [HH:MM:SS], [YYYY-MM-DD HH:MM:SS], etc.
  return message
    .replace(/\[\d{2}:\d{2}:\d{2}\]/g, "")
    .replace(/\[\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}\]/g, "")
    .replace(/\d{2}:\d{2}:\d{2}/g, "")
    .trim();
}

// Helper function to check if message is duplicate within recent logs
function isDuplicateMessage(
  newMessage: string,
  recentLogs: LogEntry[],
  windowSize: number,
): boolean {
  if (!LOG_CONFIG.enableDedupe || recentLogs.length === 0) return false;

  const normalizedNew = normalizeMessage(newMessage);
  if (!normalizedNew) return false;

  // Check last N messages
  const checkWindow = recentLogs.slice(-windowSize);

  for (const log of checkWindow) {
    const normalizedExisting = normalizeMessage(log.message);
    if (normalizedNew === normalizedExisting) {
      return true;
    }
  }

  return false;
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

    // Keep logs for historical reference but mark as closed
    // Could add cleanup logic here if desired
  },

  addLauncherLog(
    message: string,
    level: LogEntry["level"] = "info",
    instanceId?: string,
  ) {
    // Enhanced duplicate detection for launcher messages (time-based window)
    const now = Date.now();
    const messageKey = instanceId
      ? `${instanceId}:${level}:${message}`
      : `global:${level}:${message}`;
    const lastSeen = recentLauncherMessages.get(messageKey);

    if (lastSeen && now - lastSeen < LOG_CONFIG.dedupeTimeWindow) {
      // Skip duplicate message within time window
      return;
    }

    recentLauncherMessages.set(messageKey, now);

    // Clean up old entries (older than 5 minutes)
    const cutoff = now - 5 * 60 * 1000;
    for (const [key, timestamp] of recentLauncherMessages.entries()) {
      if (timestamp < cutoff) {
        recentLauncherMessages.delete(key);
      }
    }

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
          // Check for duplicates in recent logs
          if (
            !isDuplicateMessage(
              message,
              instanceLogs.launcherLogs,
              LOG_CONFIG.dedupeWindowSize,
            )
          ) {
            // Add new log and trim to max size
            const updatedLogs = trimLogsToMaxSize(
              [...instanceLogs.launcherLogs, logEntry],
              LOG_CONFIG.maxLogsPerInstance,
            );
            instanceLogs.launcherLogs = updatedLogs;
            newLogs.set(instanceId, { ...instanceLogs });
          }
        }
        return newLogs;
      });
    } else {
      globalLauncherLogs.update((logs) => {
        // Check for duplicates in recent global logs
        if (!isDuplicateMessage(message, logs, LOG_CONFIG.dedupeWindowSize)) {
          // Add new log and trim to max size
          return trimLogsToMaxSize(
            [...logs, logEntry],
            LOG_CONFIG.maxGlobalLogs,
          );
        }
        return logs;
      });
    }
  },

  addGameLog(
    instanceId: string,
    message: string,
    level: LogEntry["level"] = "info",
  ) {
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
        // Check for duplicates in recent game logs
        if (
          !isDuplicateMessage(
            message,
            instanceLogs.gameLogs,
            LOG_CONFIG.dedupeWindowSize,
          )
        ) {
          // Add new log and trim to max size
          const updatedLogs = trimLogsToMaxSize(
            [...instanceLogs.gameLogs, logEntry],
            LOG_CONFIG.maxLogsPerInstance,
          );
          instanceLogs.gameLogs = updatedLogs;
          newLogs.set(instanceId, { ...instanceLogs });
        }
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

  // Update log configuration (called from settings)
  updateLogConfig(config: Partial<typeof LOG_CONFIG>) {
    Object.assign(LOG_CONFIG, config);
  },

  // Get current log config
  getLogConfig() {
    return { ...LOG_CONFIG };
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
