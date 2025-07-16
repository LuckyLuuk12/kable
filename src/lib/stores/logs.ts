import { writable, derived } from 'svelte/store';
import type { GameInstance, LogEntry, GameInstanceLogs } from '../types';

// Active game instances
export const gameInstances = writable<Map<string, GameInstance>>(new Map());

// All logs data
export const logsData = writable<Map<string, GameInstanceLogs>>(new Map());

// Global launcher logs (not tied to specific instances)
export const globalLauncherLogs = writable<LogEntry[]>([]);

// Recent launcher messages for duplicate detection (last 5 minutes)
const recentLauncherMessages = new Map<string, number>();

// Currently selected instance/tab
export const selectedInstanceId = writable<string | 'global'>('global');

// Derived store for current logs based on selection
export const currentLogs = derived(
  [logsData, globalLauncherLogs, selectedInstanceId],
  ([$logsData, $globalLogs, $selectedId]) => {
    try {
      if ($selectedId === 'global') {
        return {
          launcherLogs: $globalLogs || [],
          gameLogs: [] as LogEntry[]
        };
      }
      
      const instanceLogs = $logsData.get($selectedId);
      return instanceLogs ? {
        launcherLogs: instanceLogs.launcherLogs || [],
        gameLogs: instanceLogs.gameLogs || []
      } : {
        launcherLogs: [] as LogEntry[],
        gameLogs: [] as LogEntry[]
      };
    } catch (error) {
      console.error('Error in currentLogs derived store:', error);
      return {
        launcherLogs: [] as LogEntry[],
        gameLogs: [] as LogEntry[]
      };
    }
  }
);

// Helper functions
export const LogsManager = {
  addGameInstance(instance: GameInstance) {
    gameInstances.update(instances => {
      const newInstances = new Map(instances);
      newInstances.set(instance.id, instance);
      return newInstances;
    });
    
    logsData.update(logs => {
      const newLogs = new Map(logs);
      newLogs.set(instance.id, {
        instanceId: instance.id,
        launcherLogs: [],
        gameLogs: []
      });
      return newLogs;
    });
  },

  updateGameInstance(instanceId: string, updates: Partial<GameInstance>) {
    gameInstances.update(instances => {
      const newInstances = new Map(instances);
      const instance = newInstances.get(instanceId);
      if (instance) {
        newInstances.set(instanceId, { ...instance, ...updates });
      }
      return newInstances;
    });
  },

  removeGameInstance(instanceId: string) {
    gameInstances.update(instances => {
      const newInstances = new Map(instances);
      newInstances.delete(instanceId);
      return newInstances;
    });
    
    // Keep logs for historical reference but mark as closed
    // Could add cleanup logic here if desired
  },

  addLauncherLog(message: string, level: LogEntry['level'] = 'info', instanceId?: string) {
    // Simple duplicate detection for launcher messages (5 minute window)
    if (!instanceId) {
      const now = Date.now();
      const messageKey = `${level}:${message}`;
      const lastSeen = recentLauncherMessages.get(messageKey);
      
      if (lastSeen && (now - lastSeen) < 5 * 60 * 1000) {
        // Skip duplicate message within 5 minutes
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
    }

    const logEntry: LogEntry = {
      timestamp: new Date(),
      level,
      source: 'launcher',
      instanceId,
      message,
      raw: message
    };

    if (instanceId) {
      logsData.update(logs => {
        const newLogs = new Map(logs);
        const instanceLogs = newLogs.get(instanceId);
        if (instanceLogs) {
          instanceLogs.launcherLogs.push(logEntry);
          newLogs.set(instanceId, instanceLogs);
        }
        return newLogs;
      });
    } else {
      globalLauncherLogs.update(logs => [...logs, logEntry]);
    }
  },

  addGameLog(instanceId: string, message: string, level: LogEntry['level'] = 'info') {
    const logEntry: LogEntry = {
      timestamp: new Date(),
      level,
      source: 'game',
      instanceId,
      message,
      raw: message
    };

    logsData.update(logs => {
      const newLogs = new Map(logs);
      const instanceLogs = newLogs.get(instanceId);
      if (instanceLogs) {
        instanceLogs.gameLogs.push(logEntry);
        newLogs.set(instanceId, instanceLogs);
      }
      return newLogs;
    });
  },

  clearLogs(instanceId?: string) {
    if (instanceId === 'global' || !instanceId) {
      globalLauncherLogs.set([]);
    } else {
      logsData.update(logs => {
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
  emitLauncherEvent(message: string, level: 'info' | 'warn' | 'error' = 'info') {
    LogsManager.addLauncherLog(message, level);
  },
};
