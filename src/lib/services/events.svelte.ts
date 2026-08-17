import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { AppService, Service } from "./app.svelte";

type NavigationEventPayload = Record<string, never>;

type BehaviorChoiceEventPayload = {
  exit_code: number;
  options: string[];
};

type GameRestartEventPayload = {
  exit_code: number;
};

export class EventsService implements Service {
  private initialized = false;
  private unlisteners: UnlistenFn[] = [];

  constructor(private app: AppService) {}

  async init() {
    if (this.initialized) return;
    this.initialized = true;

    await this.setupEventListeners();
  }

  private async setupEventListeners() {
    const navigationToLogs = await listen<NavigationEventPayload>("navigate-to-logs", async () => {
      this.app.logsService.emitLauncherEvent("Navigating to logs page due to game settings", "info");

      const { goto } = await import("$app/navigation");
      await goto("/logs");
    });

    const navigationToHome = await listen<NavigationEventPayload>("navigate-to-home", async () => {
      this.app.logsService.emitLauncherEvent("Navigating to home page due to game settings", "info");

      const { goto } = await import("$app/navigation");
      await goto("/");
    });

    const launchBehavior = await listen<BehaviorChoiceEventPayload>("ask-launch-behavior", async (event) => {
      const choice = await this.showBehaviorDialog("Launch Behavior", "What should happen when the game launches?", event.payload.options);
      if (choice) await this.handleUserChoice("on_game_launch", choice);
    });

    const closeBehavior = await listen<BehaviorChoiceEventPayload>("ask-close-behavior", async (event) => {
      const choice = await this.showBehaviorDialog("Close Behavior", `What should happen now? (Game exited with code ${event.payload.exit_code})`, event.payload.options);
      if (choice) await this.handleUserChoice("on_game_close", choice);
    });

    const crashBehavior = await listen<BehaviorChoiceEventPayload>("ask-crash-behavior", async (event) => {
      const choice = await this.showBehaviorDialog("Game Crashed", `The game crashed (exit code ${event.payload.exit_code}). What should we do?`, event.payload.options);
      if (choice) await this.handleUserChoice("on_game_crash", choice);
    });

    const restartRequested = await listen<GameRestartEventPayload>("game-restart-requested", (event) => {
      this.app.logsService.emitLauncherEvent(`Game restart requested due to crash (exit code: ${event.payload.exit_code})`, "warn");
      alert("Game restart feature is not implemented yet. Please launch manually.");
    });

    const gameStarted = await listen<{ pid: number; installation_id: string }>("game-started", (event) => {
      this.app.logsService.emitLauncherEvent(`Game started (PID: ${event.payload.pid})`, "info");

      this.app.launcherService.launching = false;
      this.app.launcherService.lastLaunch = null;
    });

    this.unlisteners.push(navigationToLogs, navigationToHome, launchBehavior, closeBehavior, crashBehavior, restartRequested, gameStarted);
  }

  private async showBehaviorDialog(title: string, message: string, options: string[]): Promise<string | null> {
    const optionLabels: Record<string, string> = {
      keep_open: "Keep Launcher Open",
      exit: "Close Launcher",
      minimize: "Minimize Launcher",
      open_logs: "Open Logs Page",
      open_home: "Go to Home Page",
      restart: "Restart Game",
      close: "Close Launcher",
      ask: "Ask Me Each Time",
    };

    const buttons = options.map((opt) => optionLabels[opt] || opt);

    if (options.length === 2) {
      const result = confirm(`${title}\n\n${message}\n\nClick OK for "${buttons[0]}" or Cancel for "${buttons[1]}"`);
      return result ? options[0] : options[1];
    }

    let promptMessage = `${title}\n\n${message}\n\nOptions:\n`;
    buttons.forEach((label, index) => {
      promptMessage += `${index + 1}. ${label}\n`;
    });
    promptMessage += "\nEnter the number of your choice:";

    const choice = prompt(promptMessage);
    const choiceIndex = parseInt(choice || "0") - 1;

    if (choiceIndex >= 0 && choiceIndex < options.length) {
      return options[choiceIndex];
    }

    return null;
  }

  private async handleUserChoice(settingType: string, choice: string) {
    this.app.logsService.emitLauncherEvent(`User chose "${choice}" for ${settingType}`, "info");

    const window = getCurrentWindow();

    switch (choice) {
      case "exit":
      case "close":
        await window.close();
        break;
      case "minimize":
        await window.minimize();
        break;
      case "open_logs":
        await (await import("$app/navigation")).goto("/logs");
        break;
      case "open_home":
        await (await import("$app/navigation")).goto("/");
        break;
      case "restart":
        this.app.logsService.emitLauncherEvent("Game restart requested by user", "info");
        alert("Game restart feature is not implemented yet. Please launch manually.");
        break;
      case "keep_open":
        this.app.logsService.emitLauncherEvent("Keeping launcher open as requested", "info");
        break;
      default:
        console.warn(`Unknown choice: ${choice}`);
    }
  }

  async destroy() {
    for (const unlisten of this.unlisteners) {
      unlisten();
    }
    this.unlisteners = [];
    this.initialized = false;
  }
}
