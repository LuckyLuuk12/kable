import { LogsManager } from "../stores/logs";
import type { Service } from "./app.service";

export class LogsService implements Service {
  async init() {
    // No-op for now. Keep service boundary for log-related wiring.
  }

  emitLauncherEvent(message: string, level: "info" | "warn" | "error" | "debug" = "info", instanceId?: string) {
    LogsManager.addLauncherLog(message, level, instanceId);
  }

  async destroy() {
    // No-op.
  }
}

export const logsService = new LogsService();
