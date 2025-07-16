import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { LogsManager } from '../stores/logs';
import type { GameInstance, LogEntry } from '../types';

export class LogsService {
  private listeners: Map<string, UnlistenFn> = new Map();
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
      console.log('Logs service already initialized, skipping...');
      return;
    }

    console.log('Initializing logs service...');
    
    // Initialize global launcher logs
    LogsManager.addLauncherLog('Logs service initialized', 'info');
    
    // Listen for game launch events
    const launchListener = await listen('game-launched', (event) => {
      try {
        const { instanceId, profile, installation } = event.payload as {
          instanceId: string;
          profile: any;
          installation: any;
        };

        const gameInstance: GameInstance = {
          id: instanceId,
          profileName: profile.name,
          installationPath: installation.path,
          status: 'launching',
          launchedAt: new Date(),
          lastActivity: new Date()
        };

        LogsManager.addGameInstance(gameInstance);
        LogsManager.addLauncherLog(
          `Launching ${profile.name} (${installation.mod_loader || 'vanilla'})`,
          'info',
          instanceId
        );
      } catch (error) {
        console.error('Error handling game launch event:', error);
        LogsManager.addLauncherLog('Error processing game launch event', 'error');
      }
    });

    // Listen for game process events
    const processListener = await listen('game-process-event', (event) => {
      try {
        const { instanceId, type, data } = event.payload as {
          instanceId: string;
          type: 'started' | 'output' | 'error' | 'exit';
          data: any;
        };

        switch (type) {
          case 'started':
            LogsManager.updateGameInstance(instanceId, { 
              status: 'running',
              processId: data.pid 
            });
            LogsManager.addLauncherLog(`Game process started (PID: ${data.pid})`, 'info', instanceId);
            break;

          case 'output':
            this.parseGameOutput(instanceId, data.line);
            break;

          case 'error':
            LogsManager.addGameLog(instanceId, data.line, 'error');
            LogsManager.updateGameInstance(instanceId, { lastActivity: new Date() });
            break;

          case 'exit':
            LogsManager.updateGameInstance(instanceId, { 
              status: data.code === 0 ? 'completed' : 'crashed',
              exitCode: data.code,
              completedAt: new Date()
            });
            LogsManager.addLauncherLog(
              `Game process exited with code ${data.code}`,
              data.code === 0 ? 'info' : 'error',
              instanceId
            );
            break;
        }
      } catch (error) {
        console.error('Error handling game process event:', error);
        LogsManager.addLauncherLog('Error processing game process event', 'error');
      }
    });

    // Listen for launcher log events
    const launcherLogListener = await listen('launcher-log', (event) => {
      try {
        const { level, message, instanceId } = event.payload as {
          level: LogEntry['level'];
          message: string;
          instanceId?: string;
        };

        LogsManager.addLauncherLog(message, level, instanceId);
      } catch (error) {
        console.error('Error handling launcher log event:', error);
        LogsManager.addLauncherLog('Error processing launcher log event', 'error');
      }
    });

    this.listeners.set('game-launched', launchListener);
    this.listeners.set('game-process-event', processListener);
    this.listeners.set('launcher-log', launcherLogListener);
    
    // Mark as initialized
    this.isInitialized = true;
    console.log('Logs service initialization complete');
  }

  private parseGameOutput(instanceId: string, line: string) {
    // Update last activity
    LogsManager.updateGameInstance(instanceId, { lastActivity: new Date() });

    // Determine log level based on content
    let level: LogEntry['level'] = 'info';
    const lowerLine = line.toLowerCase();

    if (lowerLine.includes('error') || lowerLine.includes('exception') || lowerLine.includes('failed')) {
      level = 'error';
    } else if (lowerLine.includes('warn') || lowerLine.includes('warning')) {
      level = 'warn';
    } else if (lowerLine.includes('debug') || lowerLine.includes('trace')) {
      level = 'debug';
    }

    LogsManager.addGameLog(instanceId, line, level);

    // Check for crash indicators
    if (this.isCrashIndicator(line)) {
      LogsManager.updateGameInstance(instanceId, { status: 'crashed' });
      LogsManager.addLauncherLog('Game crash detected', 'error', instanceId);
    }
  }

  private isCrashIndicator(line: string): boolean {
    const crashPatterns = [
      /crash/i,
      /fatal/i,
      /segmentation fault/i,
      /access violation/i,
      /out of memory/i,
      /java\.lang\.OutOfMemoryError/i,
      /unexpected error/i
    ];

    return crashPatterns.some(pattern => pattern.test(line));
  }

  async exportLogs(instanceId?: string): Promise<void> {
    try {
      await invoke('export_logs', { instanceId });
      LogsManager.addLauncherLog('Logs exported successfully', 'info');
    } catch (error) {
      LogsManager.addLauncherLog(`Failed to export logs: ${error}`, 'error');
    }
  }

  async clearLogs(instanceId?: string): Promise<void> {
    LogsManager.clearLogs(instanceId);
    LogsManager.addLauncherLog(
      instanceId ? `Cleared logs for instance ${instanceId}` : 'Cleared global logs',
      'info'
    );
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
    console.log('Logs service destroyed');
  }
}

// Export singleton instance
export const logsService = LogsService.getInstance();
