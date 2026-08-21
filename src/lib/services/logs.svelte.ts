import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { SvelteMap } from "svelte/reactivity";
import type { Service } from "./app.svelte";

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

interface GameLaunchedEvent {
  instanceId: string;
  profile: {
    name: string;
  };
  installation: {
    path: string;
    mod_loader?: string;
  };
}

interface GameProcessEvent {
  instanceId: string;
  type: "started" | "output" | "error" | "exit";
  data: Record<string, unknown>;
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
      typeof value.instanceId === "string" ? value.instanceId : null,
    context:
      typeof value.context === "string" ? value.context : null,
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
    this.gameLogs.clear();
    this.gameInstances.clear();

    this.initialized = false;
  }

  private async load(): Promise<void> {
    // Initial loading belongs here.
    //
    // If logs are persisted by the backend, load them here through the
    // appropriate Tauri command. Runtime events are handled separately below.
  }

  private handleGameLaunched(payload: unknown): void {
    if (!isRecord(payload)) {
      return;
    }

    const instanceId =
      typeof payload.instanceId === "string"
        ? payload.instanceId
        : null;

    const profile = isRecord(payload.profile)
      ? payload.profile
      : null;

    const installation = isRecord(payload.installation)
      ? payload.installation
      : null;

    if (
      !instanceId ||
      !profile ||
      typeof profile.name !== "string" ||
      !installation ||
      typeof installation.path !== "string"
    ) {
      return;
    }

    const now = Date.now();

    const instance: GameInstance = {
      id: instanceId,
      profileName: profile.name,
      installationPath: installation.path,
      status: "launching",
      launchedAt: now,
      lastActivity: now,
    };

    this.gameInstances.set(instanceId, instance);

    this.addLauncherLog({
      level: "info",
      message: `Launching ${profile.name} (${typeof installation.mod_loader === "string" ? installation.mod_loader : "vanilla"})`,
      fnName: null,
      instanceId,
      context: null,
      value: null,
      timestamp: now,
    });
  }

  private handleGameProcessEvent(payload: unknown): void {
    if (!isRecord(payload)) {
      return;
    }

    const instanceId =
      typeof payload.instanceId === "string"
        ? payload.instanceId
        : null;

    const type =
      typeof payload.type === "string"
        ? payload.type
        : null;

    const data = isRecord(payload.data)
      ? payload.data
      : null;

    if (!instanceId || !type || !data) {
      return;
    }

    switch (type) {
      case "started":
        this.handleProcessStarted(instanceId, data);
        break;

      case "output":
        this.handleProcessOutput(instanceId, data);
        break;

      case "error":
        this.handleProcessError(instanceId, data);
        break;

      case "exit":
        this.handleProcessExit(instanceId, data);
        break;
    }
  }

  private handleProcessStarted(
    instanceId: string,
    data: Record<string, unknown>
  ): void {
    const pid =
      typeof data.pid === "number"
        ? data.pid
        : undefined;

    this.updateGameInstance(instanceId, {
      status: "running",
      processId: pid,
      lastActivity: Date.now(),
    });

    this.addLauncherLog({
      level: "info",
      message: pid !== undefined
        ? `Game process started (PID: ${pid})`
        : "Game process started",
      fnName: null,
      instanceId,
      context: null,
      value: null,
      timestamp: Date.now(),
    });
  }

  private handleProcessOutput(
    instanceId: string,
    data: Record<string, unknown>
  ): void {
    const log = parseLogEvent(data);

    if (!log) {
      if (typeof data.line === "string") {
        this.addGameLog(instanceId, {
          level: "info",
          message: data.line,
          fnName: null,
          instanceId,
          context: null,
          value: null,
          timestamp: Date.now(),
        });
      }

      this.updateGameInstance(instanceId, {
        lastActivity: Date.now(),
      });

      return;
    }

    this.addGameLog(instanceId, {
      ...log,
      instanceId: log.instanceId ?? instanceId,
    });

    this.updateGameInstance(instanceId, {
      lastActivity: log.timestamp,
    });
  }

  private handleProcessError(
    instanceId: string,
    data: Record<string, unknown>
  ): void {
    const log = parseLogEvent(data);

    if (log) {
      this.addGameLog(instanceId, {
        ...log,
        level: "error",
        instanceId: log.instanceId ?? instanceId,
      });
    } else if (typeof data.line === "string") {
      this.addGameLog(instanceId, {
        level: "error",
        message: data.line,
        fnName: null,
        instanceId,
        context: null,
        value: null,
        timestamp: Date.now(),
      });
    }

    this.updateGameInstance(instanceId, {
      lastActivity: Date.now(),
    });
  }

  private handleProcessExit(
    instanceId: string,
    data: Record<string, unknown>
  ): void {
    const exitCode =
      typeof data.code === "number"
        ? data.code
        : undefined;

    const status = this.getExitStatus(exitCode);

    const now = Date.now();

    this.updateGameInstance(instanceId, {
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
      message: exitCode === undefined
        ? `Game process ${statusMessage}`
        : `Game process ${statusMessage} (exit code: ${exitCode})`,
      fnName: null,
      instanceId,
      context: null,
      value: null,
      timestamp: now,
    });
  }

  private handleLauncherLog(payload: unknown): void {
    const log = parseLogEvent(payload);

    if (!log) {
      return;
    }

    this.addLauncherLog(log);
  }

  private handleLauncherLogBatch(payload: unknown): void {
    if (!isRecord(payload) || !Array.isArray(payload.logs)) {
      return;
    }

    for (const value of payload.logs) {
      const log = parseLogEvent(value);

      if (log) {
        this.addLauncherLog(log);
      }
    }
  }

  private addLauncherLog(log: LogEvent): void {
    this.launcherLogs.push(log);
  }

  private addGameLog(instanceId: string, log: LogEvent): void {
    const logs = this.gameLogs.get(instanceId);

    if (logs) {
      logs.push(log);
    } else {
      this.gameLogs.set(instanceId, [log]);
    }

    this.gameLogs = new SvelteMap(this.gameLogs);
  }

  private updateGameInstance(
    instanceId: string,
    update: Partial<GameInstance>
  ): void {
    const instance = this.gameInstances.get(instanceId);

    if (!instance) {
      return;
    }

    this.gameInstances.set(instanceId, {
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
    if (instanceId) {
      this.gameLogs.delete(instanceId);
      this.gameLogs = new SvelteMap(this.gameLogs);
      return;
    }

    this.launcherLogs = [];
  }

  getLogs(instanceId?: string): LogEvent[] {
    if (instanceId) {
      return this.gameLogs.get(instanceId) ?? [];
    }

    return this.launcherLogs;
  }

  getInstance(instanceId: string): GameInstance | undefined {
    return this.gameInstances.get(instanceId);
  }
}