import { listen, type UnlistenFn } from "@tauri-apps/api/event";
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

export type GameInstanceStatus =
  | "launching"
  | "running"
  | "closed"
  | "crashed"
  | "stopped";

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

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isLogLevel(value: unknown): value is LogLevel {
  return (
    value === "info" ||
    value === "warn" ||
    value === "error" ||
    value === "debug"
  );
}

function parseLogEvent(value: unknown): LogEvent | null {
  if (!isRecord(value)) {
    return null;
  }

  if (
    typeof value.message !== "string" ||
    !isLogLevel(value.level) ||
    typeof value.timestamp !== "number"
  ) {
    return null;
  }

  return {
    level: value.level,
    message: value.message,
    fnName: typeof value.fnName === "string" ? value.fnName : null,
    instanceId:
      typeof value.instanceId === "string"
        ? value.instanceId
        : null,
    context:
      typeof value.context === "string"
        ? value.context
        : null,
    value: value.value ?? null,
    timestamp: value.timestamp,
  };
}

export class LogsService implements Service {
  launcherLogs = $state<LogEvent[]>([]);
  gameLogs = $state<SvelteMap<string, LogEvent[]>>(new SvelteMap());
  gameInstances = $state<SvelteMap<string, GameInstance>>(new SvelteMap());

  private listeners: UnlistenFn[] = [];
  private initialized = false;

  async init(): Promise<void> {
    if (this.initialized) {
      return;
    }

    this.initialized = true;

    await this.load();

    this.listeners.push(
      await listen("game-launched", (event) => {
        this.handleGameLaunched(event.payload);
      })
    );

    this.listeners.push(
      await listen("game-process-event", (event) => {
        this.handleGameProcessEvent(event.payload);
      })
    );

    this.listeners.push(
      await listen("launcher-log", (event) => {
        this.handleLauncherLog(event.payload);
      })
    );

    this.listeners.push(
      await listen("launcher-log-batch", (event) => {
        this.handleLauncherLogBatch(event.payload);
      })
    );
  }

  destroy(): void {
    for (const unlisten of this.listeners) {
      unlisten();
    }

    this.listeners = [];

    this.launcherLogs = [];
    this.gameLogs = new SvelteMap();
    this.gameInstances = new SvelteMap();

    this.initialized = false;
  }

  private async load(): Promise<void> {
    // Reserved for loading persisted launcher/game logs.
  }

  private handleGameLaunched(payload: unknown): void {
    if (!isRecord(payload)) {
      return;
    }

    const runtimeId =
      typeof payload.runtimeId === "string"
        ? payload.runtimeId
        : null;

    const profile = isRecord(payload.profile)
      ? payload.profile
      : null;

    const metadata = isRecord(profile?.metadata)
      ? profile.metadata
      : null;

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
      profileName: metadata.name,
      installationPath:
        typeof profile.installationPath === "string"
          ? profile.installationPath
          : "",
      status: "launching",
      launchedAt: now,
      lastActivity: now,
    };

    this.gameInstances.set(runtimeId, instance);
    this.gameInstances = new SvelteMap(this.gameInstances);

    this.addLauncherLog({
      level: "info",
      message: `Launching ${metadata.name}`,
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

    const runtimeId =
      typeof payload.runtimeId === "string"
        ? payload.runtimeId
        : null;

    const type =
      typeof payload.type === "string"
        ? payload.type
        : null;

    const data = isRecord(payload.data)
      ? payload.data
      : null;

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

      default:
        break;
    }
  }

  private handleProcessStarted(
    runtimeId: string,
    data: Record<string, unknown>
  ): void {
    const pid =
      typeof data.pid === "number"
        ? data.pid
        : undefined;

    const now = Date.now();

    this.updateGameInstance(runtimeId, {
      status: "running",
      processId: pid,
      lastActivity: now,
    });

    this.addLauncherLog({
      level: "info",
      message:
        pid !== undefined
          ? `Game process started (PID: ${pid})`
          : "Game process started",
      fnName: null,
      instanceId: runtimeId,
      context: null,
      value: null,
      timestamp: now,
    });
  }

  private handleProcessLog(
    runtimeId: string,
    data: Record<string, unknown>
  ): void {
    const log = parseLogEvent(data);

    if (log === null) {
      return;
    }

    const normalizedLog: LogEvent = {
      ...log,
      instanceId: runtimeId,
    };

    this.addGameLog(runtimeId, normalizedLog);

    this.updateGameInstance(runtimeId, {
      lastActivity: normalizedLog.timestamp,
    });
  }

  private handleProcessExit(
    runtimeId: string,
    data: Record<string, unknown>
  ): void {
    const exitCode =
      typeof data.exitCode === "number"
        ? data.exitCode
        : undefined;

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

    const logs: LogEvent[] = [];

    for (const value of payload.logs) {
      const log = parseLogEvent(value);

      if (log !== null) {
        logs.push(log);
      }
    }

    if (logs.length === 0) {
      return;
    }

    this.launcherLogs = [
      ...this.launcherLogs,
      ...logs,
    ];
  }

  addLauncherLog(log: LogEvent): void {
    this.launcherLogs = [
      ...this.launcherLogs,
      log,
    ];
  }

  addGameLog(
    runtimeId: string,
    log: LogEvent
  ): void {
    const existing = this.gameLogs.get(runtimeId) ?? [];

    this.gameLogs.set(runtimeId, [
      ...existing,
      {
        ...log,
        instanceId: runtimeId,
      },
    ]);

    this.gameLogs = new SvelteMap(this.gameLogs);
  }

  private updateGameInstance(
    runtimeId: string,
    update: Partial<GameInstance>
  ): void {
    const instance = this.gameInstances.get(runtimeId);

    if (instance === undefined) {
      return;
    }

    this.gameInstances.set(runtimeId, {
      ...instance,
      ...update,
    });

    this.gameInstances = new SvelteMap(this.gameInstances);
  }

  private getExitStatus(
    exitCode: number | undefined
  ): GameInstanceStatus {
    if (exitCode === 0) {
      return "closed";
    }

    if (
      exitCode === 130 ||
      exitCode === 143 ||
      exitCode === -1073741510
    ) {
      return "stopped";
    }

    if (
      exitCode === undefined ||
      exitCode < 0 ||
      exitCode > 128
    ) {
      return "crashed";
    }

    return "stopped";
  }

  clearLogs(instanceId?: string): void {
    if (instanceId !== undefined) {
      this.gameLogs.delete(instanceId);
      this.gameLogs = new SvelteMap(this.gameLogs);
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

  getInstance(
    runtimeId: string
  ): GameInstance | undefined {
    return this.gameInstances.get(runtimeId);
  }
}

// #region static api

type ConsoleLogMethod =
  | "log"
  | "info"
  | "warn"
  | "error"
  | "debug";

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
    return JSON.stringify(value, (_, nestedValue) =>
      typeof nestedValue === "bigint"
        ? `${nestedValue}n`
        : nestedValue
    );
  } catch {
    return String(value);
  }
}

function formatConsoleMessage(args: unknown[]): string {
  return args
    .map(formatConsoleArgument)
    .join(" ");
}

function writeConsoleLog(
  method: ConsoleLogMethod,
  args: unknown[],
  instanceId: string | null = null
): void {
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
    value:
      args.length === 1
        ? args[0] ?? null
        : args,
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

export function instanceLog(
  instanceId: string,
  ...args: unknown[]
): void {
  writeConsoleLog("log", args, instanceId);
}

export function instanceInfo(
  instanceId: string,
  ...args: unknown[]
): void {
  writeConsoleLog("info", args, instanceId);
}

export function instanceWarn(
  instanceId: string,
  ...args: unknown[]
): void {
  writeConsoleLog("warn", args, instanceId);
}

export function instanceError(
  instanceId: string,
  ...args: unknown[]
): void {
  writeConsoleLog("error", args, instanceId);
}

export function instanceDebug(
  instanceId: string,
  ...args: unknown[]
): void {
  writeConsoleLog("debug", args, instanceId);
}

// #endregion