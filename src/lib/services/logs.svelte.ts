import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { untrack } from "svelte";
import { SvelteMap } from "svelte/reactivity";
import { app, type Service } from "./app.svelte";

export type LogLevel = "info" | "warn" | "error" | "debug";

export interface LogEvent {
  level: LogLevel;
  message: string;
  fnName: string | null;
  instanceId: string | null;
  context: string | null;
  value: unknown;
  timestamp: number;
}

export type GameInstanceStatus = "launching" | "running" | "closed" | "crashed" | "stopped";

export interface GameInstance {
  id: string;
  profileName: string;
  installationPath: string;
  status: GameInstanceStatus;
  processId?: number;
  exitCode?: number;
  launchedAt: number;
  lastActivity: number;
  completedAt?: number;
}

const MAX_LOG_MESSAGE_CHARS = 16_384;
const MAX_LOG_VALUE_CHARS = 4_096;
const MAX_GAME_INSTANCES = 20;
const MAX_LOGS_PER_GAME_INSTANCE = 2_000;
const MAX_TOTAL_GAME_LOGS = 10_000;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isLogLevel(value: unknown): value is LogLevel {
  return value === "info" || value === "warn" || value === "error" || value === "debug";
}

function truncateText(value: string, maxLength: number): string {
  if (value.length <= maxLength) {
    return value;
  }

  return `${value.slice(0, maxLength)}… [truncated]`;
}

/**
 * Store detached, bounded log values rather than references to arbitrary
 * application objects. This prevents log history from retaining large object
 * graphs through LogEvent.value.
 */
function sanitizeLogValue(value: unknown): unknown {
  if (value === null || value === undefined) {
    return value ?? null;
  }

  if (typeof value === "string") {
    return truncateText(value, MAX_LOG_VALUE_CHARS);
  }

  if (typeof value === "number" || typeof value === "boolean") {
    return value;
  }

  if (typeof value === "bigint") {
    return truncateText(`${value}n`, MAX_LOG_VALUE_CHARS);
  }

  if (typeof value === "symbol") {
    return truncateText(String(value), MAX_LOG_VALUE_CHARS);
  }

  if (typeof value === "function") {
    return truncateText(`[Function ${value.name || "anonymous"}]`, MAX_LOG_VALUE_CHARS);
  }

  if (value instanceof Error) {
    return truncateText(value.stack ?? value.message, MAX_LOG_VALUE_CHARS);
  }

  try {
    const serialized = JSON.stringify(value, (_key, nestedValue: unknown) => {
      if (typeof nestedValue === "bigint") {
        return `${nestedValue}n`;
      }

      if (nestedValue instanceof Error) {
        return {
          name: nestedValue.name,
          message: nestedValue.message,
          stack: nestedValue.stack,
        };
      }

      return nestedValue;
    });

    if (serialized === undefined) {
      return truncateText(String(value), MAX_LOG_VALUE_CHARS);
    }

    if (serialized.length > MAX_LOG_VALUE_CHARS) {
      return truncateText(serialized, MAX_LOG_VALUE_CHARS);
    }

    // JSON.parse creates a detached value, so the log doesn't retain the
    // original object's references or reactive proxies.
    return JSON.parse(serialized) as unknown;
  } catch {
    return truncateText(String(value), MAX_LOG_VALUE_CHARS);
  }
}

function parseLogEvent(value: unknown): LogEvent | null {
  if (!isRecord(value)) {
    return null;
  }

  if (
    typeof value.message !== "string" ||
    !isLogLevel(value.level) ||
    typeof value.timestamp !== "number" ||
    !Number.isFinite(value.timestamp)
  ) {
    return null;
  }

  return {
    level: value.level,
    message: truncateText(value.message, MAX_LOG_MESSAGE_CHARS),
    fnName: typeof value.fnName === "string" ? truncateText(value.fnName, 256) : null,
    instanceId: typeof value.instanceId === "string" ? truncateText(value.instanceId, 256) : null,
    context: typeof value.context === "string" ? truncateText(value.context, 512) : null,
    value: sanitizeLogValue(value.value),
    timestamp: value.timestamp,
  };
}

export class LogsService implements Service {
  launcherLogs = $state<LogEvent[]>([]);
  gameLogs = $state<SvelteMap<string, LogEvent[]>>(new SvelteMap());
  gameInstances = $state<SvelteMap<string, GameInstance>>(new SvelteMap());

  private listeners: UnlistenFn[] = [];
  private initialized = false;
  private totalGameLogCount = 0;

  private get maxMemoryLogs(): number {
    const configured = app.customizationService.settings?.logging?.max_memory_logs;

    return typeof configured === "number" && Number.isFinite(configured)
      ? Math.max(0, Math.floor(configured))
      : 10_000;
  }

  async init(): Promise<void> {
    if (this.initialized) {
      return;
    }

    // Prevent concurrent calls from registering duplicate listeners while
    // init() is awaiting the event subscriptions.
    this.initialized = true;

    try {
      this.enforceMemoryLimit();
      this.recalculateGameLogCount();

      const unlisteners = await Promise.all([
        listen("game-launched", (event) => {
          this.handleGameLaunched(event.payload);
        }),
        listen("game-process-event", (event) => {
          this.handleGameProcessEvent(event.payload);
        }),
        listen("launcher-log", (event) => {
          this.handleLauncherLog(event.payload);
        }),
        listen("launcher-log-batch", (event) => {
          this.handleLauncherLogBatch(event.payload);
        }),
      ]);

      this.listeners.push(...unlisteners);
    } catch (cause) {
      for (const unlisten of this.listeners) {
        unlisten();
      }

      this.listeners = [];
      this.initialized = false;
      throw cause;
    }
  }

  destroy(): void {
    for (const unlisten of this.listeners) {
      unlisten();
    }

    this.listeners = [];
    this.launcherLogs = [];
    this.gameLogs = new SvelteMap();
    this.gameInstances = new SvelteMap();
    this.totalGameLogCount = 0;
    this.initialized = false;
  }

  private trimArray<T>(items: T[], limit: number): void {
    if (limit <= 0) {
      items.splice(0, items.length);
      return;
    }

    const excess = items.length - limit;

    if (excess > 0) {
      items.splice(0, excess);
    }
  }

  private get maxGameLogsPerInstance(): number {
    return Math.min(this.maxMemoryLogs, MAX_LOGS_PER_GAME_INSTANCE);
  }

  private enforceMemoryLimit(): void {
    this.trimArray(this.launcherLogs, this.maxMemoryLogs);

    for (const [instanceId, logs] of this.gameLogs) {
      this.trimArray(logs, this.maxGameLogsPerInstance);

      if (logs.length === 0) {
        this.gameLogs.delete(instanceId);
      }
    }

    this.enforceTotalGameLogLimit();
    this.pruneGameInstances();
  }

  private recalculateGameLogCount(): void {
    let count = 0;

    for (const logs of this.gameLogs.values()) {
      count += logs.length;
    }

    this.totalGameLogCount = count;
  }

  private enforceTotalGameLogLimit(): void {
    while (this.totalGameLogCount > MAX_TOTAL_GAME_LOGS) {
      let oldestInstanceId: string | null = null;
      let oldestTimestamp = Number.POSITIVE_INFINITY;

      for (const [instanceId, logs] of this.gameLogs) {
        const firstLog = logs[0];

        if (firstLog && firstLog.timestamp < oldestTimestamp) {
          oldestTimestamp = firstLog.timestamp;
          oldestInstanceId = instanceId;
        }
      }

      if (oldestInstanceId === null) {
        this.totalGameLogCount = 0;
        break;
      }

      const logs = this.gameLogs.get(oldestInstanceId);

      if (!logs || logs.length === 0) {
        this.gameLogs.delete(oldestInstanceId);
        continue;
      }

      logs.shift();
      this.totalGameLogCount--;

      if (logs.length === 0) {
        this.gameLogs.delete(oldestInstanceId);
      }
    }
  }

  private handleGameLaunched(payload: unknown): void {
    if (!isRecord(payload)) {
      return;
    }

    const runtimeId = typeof payload.runtimeId === "string" ? payload.runtimeId : null;
    const profile = isRecord(payload.profile) ? payload.profile : null;
    const metadata = isRecord(profile?.metadata) ? profile.metadata : null;

    if (
      runtimeId === null ||
      profile === null ||
      metadata === null ||
      typeof metadata.name !== "string"
    ) {
      return;
    }

    const now = Date.now();

    const instance: GameInstance = {
      id: runtimeId,
      profileName: truncateText(metadata.name, 512),
      installationPath:
        typeof profile.installationPath === "string"
          ? truncateText(profile.installationPath, 2_048)
          : "",
      status: "launching",
      launchedAt: now,
      lastActivity: now,
    };

    this.gameInstances.set(runtimeId, instance);
    this.pruneGameInstances();

    this.addLauncherLog({
      level: "info",
      message: `Launching ${instance.profileName}`,
      fnName: null,
      instanceId: runtimeId,
      context: null,
      value: null,
      timestamp: now,
    });
  }

  private handleGameProcessEvent(payload: unknown): void {
    if (!isRecord(payload)) {
      return;
    }

    const runtimeId = typeof payload.runtimeId === "string" ? payload.runtimeId : null;
    const type = typeof payload.type === "string" ? payload.type : null;
    const data = isRecord(payload.data) ? payload.data : null;

    if (runtimeId === null || type === null || data === null) {
      return;
    }

    switch (type) {
      case "started":
        this.handleProcessStarted(runtimeId, data);
        break;

      case "log":
        this.handleProcessLog(runtimeId, data);
        break;

      case "exit":
        this.handleProcessExit(runtimeId, data);
        break;
    }
  }

  private handleProcessStarted(runtimeId: string, data: Record<string, unknown>): void {
    const pid = typeof data.pid === "number" ? data.pid : undefined;
    const now = Date.now();

    this.updateGameInstance(runtimeId, {
      status: "running",
      processId: pid,
      lastActivity: now,
    });

    this.addLauncherLog({
      level: "info",
      message: pid !== undefined ? `Game process started (PID: ${pid})` : "Game process started",
      fnName: null,
      instanceId: runtimeId,
      context: null,
      value: null,
      timestamp: now,
    });
  }

  private handleProcessLog(runtimeId: string, data: Record<string, unknown>): void {
    const log = parseLogEvent(data);

    if (log === null) {
      return;
    }

    this.addGameLog(runtimeId, {
      ...log,
      instanceId: runtimeId,
    });

    this.updateGameInstance(runtimeId, {
      lastActivity: log.timestamp,
    });
  }

  private handleProcessExit(runtimeId: string, data: Record<string, unknown>): void {
    const exitCode = typeof data.exitCode === "number" ? data.exitCode : undefined;
    const status = this.getExitStatus(exitCode);
    const now = Date.now();

    this.updateGameInstance(runtimeId, {
      status,
      exitCode,
      completedAt: now,
      lastActivity: now,
    });

    const statusMessage =
      status === "closed"
        ? "completed normally"
        : status === "stopped"
          ? "was stopped by user"
          : "crashed";

    this.addLauncherLog({
      level: status === "crashed" ? "error" : "info",
      message:
        exitCode === undefined
          ? `Game process ${statusMessage}`
          : `Game process ${statusMessage} (exit code: ${exitCode})`,
      fnName: null,
      instanceId: runtimeId,
      context: null,
      value: null,
      timestamp: now,
    });

    this.pruneGameInstances();
  }

  private handleLauncherLog(payload: unknown): void {
    const log = parseLogEvent(payload);

    if (log === null) {
      return;
    }

    this.addLauncherLog(log);
  }

  private handleLauncherLogBatch(payload: unknown): void {
    if (!isRecord(payload) || !Array.isArray(payload.logs)) {
      return;
    }

    const limit = this.maxMemoryLogs;

    if (limit === 0) {
      this.launcherLogs = [];
      return;
    }

    for (const value of payload.logs) {
      const log = parseLogEvent(value);

      if (log !== null) {
        this.launcherLogs.push(log);
      }
    }

    this.trimArray(this.launcherLogs, limit);
  }

  addLauncherLog(log: LogEvent): void {
    const limit = this.maxMemoryLogs;

    if (limit === 0) {
      this.launcherLogs = [];
      return;
    }

    const safeLog = this.normalizeLog(log);
    this.launcherLogs.push(safeLog);
    this.trimArray(this.launcherLogs, limit);
  }

  addGameLog(runtimeId: string, log: LogEvent): void {
    const perInstanceLimit = this.maxGameLogsPerInstance;

    if (perInstanceLimit === 0) {
      this.removeGameLogs(runtimeId);
      return;
    }

    const safeLog: LogEvent = {
      ...this.normalizeLog(log),
      instanceId: runtimeId,
    };

    let logs = this.gameLogs.get(runtimeId);

    if (!logs) {
      logs = [];
      this.gameLogs.set(runtimeId, logs);
    }

    const wasAtLimit = logs.length >= perInstanceLimit;

    logs.push(safeLog);

    if (wasAtLimit) {
      logs.shift();
    } else {
      this.totalGameLogCount++;
    }

    this.enforceTotalGameLogLimit();
  }

  private normalizeLog(log: LogEvent): LogEvent {
    return {
      level: isLogLevel(log.level) ? log.level : "info",
      message: truncateText(
        typeof log.message === "string" ? log.message : String(log.message),
        MAX_LOG_MESSAGE_CHARS,
      ),
      fnName: typeof log.fnName === "string" ? truncateText(log.fnName, 256) : null,
      instanceId:
        typeof log.instanceId === "string" ? truncateText(log.instanceId, 256) : null,
      context: typeof log.context === "string" ? truncateText(log.context, 512) : null,
      value: sanitizeLogValue(log.value),
      timestamp:
        typeof log.timestamp === "number" && Number.isFinite(log.timestamp)
          ? log.timestamp
          : Date.now(),
    };
  }

  private updateGameInstance(runtimeId: string, update: Partial<GameInstance>): void {
    const instance = this.gameInstances.get(runtimeId);

    if (instance === undefined) {
      return;
    }

    this.gameInstances.set(runtimeId, {
      ...instance,
      ...update,
    });
  }

  private removeGameLogs(runtimeId: string): void {
    const logs = this.gameLogs.get(runtimeId);

    if (logs) {
      this.totalGameLogCount = Math.max(0, this.totalGameLogCount - logs.length);
      this.gameLogs.delete(runtimeId);
    }
  }

  private removeGameInstance(runtimeId: string): void {
    this.removeGameLogs(runtimeId);
    this.gameInstances.delete(runtimeId);
  }

  private pruneGameInstances(): void {
    if (this.gameInstances.size <= MAX_GAME_INSTANCES) {
      return;
    }

    const completed = [...this.gameInstances.values()]
      .filter((instance) => instance.status !== "launching" && instance.status !== "running")
      .sort((a, b) => (a.completedAt ?? a.lastActivity) - (b.completedAt ?? b.lastActivity));

    while (
      this.gameInstances.size > MAX_GAME_INSTANCES &&
      completed.length > 0
    ) {
      const oldest = completed.shift();

      if (oldest) {
        this.removeGameInstance(oldest.id);
      }
    }
  }

  private getExitStatus(exitCode: number | undefined): GameInstanceStatus {
    if (exitCode === 0) {
      return "closed";
    }

    if (exitCode === 130 || exitCode === 143 || exitCode === -1073741510) {
      return "stopped";
    }

    if (exitCode === undefined || exitCode < 0 || exitCode > 128) {
      return "crashed";
    }

    return "stopped";
  }

  clearLogs(instanceId?: string): void {
    if (instanceId !== undefined) {
      this.removeGameLogs(instanceId);
      return;
    }

    this.launcherLogs = [];
  }

  getLogs(instanceId?: string): LogEvent[] {
    if (instanceId !== undefined) {
      return this.gameLogs.get(instanceId) ?? [];
    }

    return this.launcherLogs;
  }

  getInstance(runtimeId: string): GameInstance | undefined {
    return this.gameInstances.get(runtimeId);
  }
}

// #region static api

type ConsoleLogMethod = "log" | "info" | "warn" | "error" | "debug";

function formatConsoleArgument(value: unknown): string {
  if (typeof value === "string") {
    return value;
  }

  if (value instanceof Error) {
    return value.stack ?? value.message;
  }

  if (typeof value === "bigint") {
    return `${value}n`;
  }

  if (typeof value === "undefined") {
    return "undefined";
  }

  if (value === null) {
    return "null";
  }

  try {
    const serialized = JSON.stringify(value, (_key, nestedValue: unknown) =>
      typeof nestedValue === "bigint" ? `${nestedValue}n` : nestedValue,
    );

    return serialized === undefined ? String(value) : serialized;
  } catch {
    return String(value);
  }
}

function formatConsoleMessage(args: unknown[]): string {
  return truncateText(
    args.map(formatConsoleArgument).join(" "),
    MAX_LOG_MESSAGE_CHARS,
  );
}

function writeConsoleLog(
  method: ConsoleLogMethod,
  args: unknown[],
  instanceId: string | null = null,
): void {
  untrack(() => {
    const message = formatConsoleMessage(args);

    const log: LogEvent = {
      level:
        method === "error"
          ? "error"
          : method === "warn"
            ? "warn"
            : method === "debug"
              ? "debug"
              : "info",
      message,
      fnName: null,
      instanceId,
      context: "frontend",
      value: sanitizeLogValue(args.length === 1 ? args[0] : args),
      timestamp: Date.now(),
    };

    const logsService = app.logsService;

    if (logsService !== undefined) {
      if (instanceId !== null) {
        logsService.addGameLog(instanceId, log);
      } else {
        logsService.addLauncherLog(log);
      }
    }

    switch (method) {
      case "log":
        console.log(...args);
        break;

      case "info":
        console.info(...args);
        break;

      case "warn":
        console.warn(...args);
        break;

      case "error":
        console.error(...args);
        break;

      case "debug":
        console.debug(...args);
        break;
    }
  });
}

export function log(...args: unknown[]): void {
  writeConsoleLog("log", args);
}

export function info(...args: unknown[]): void {
  writeConsoleLog("info", args);
}

export function warn(...args: unknown[]): void {
  writeConsoleLog("warn", args);
}

export function error(...args: unknown[]): void {
  writeConsoleLog("error", args);
}

export function debug(...args: unknown[]): void {
  writeConsoleLog("debug", args);
}

export function instanceLog(instanceId: string, ...args: unknown[]): void {
  writeConsoleLog("log", args, instanceId);
}

export function instanceInfo(instanceId: string, ...args: unknown[]): void {
  writeConsoleLog("info", args, instanceId);
}

export function instanceWarn(instanceId: string, ...args: unknown[]): void {
  writeConsoleLog("warn", args, instanceId);
}

export function instanceError(instanceId: string, ...args: unknown[]): void {
  writeConsoleLog("error", args, instanceId);
}

export function instanceDebug(instanceId: string, ...args: unknown[]): void {
  writeConsoleLog("debug", args, instanceId);
}

// #endregion