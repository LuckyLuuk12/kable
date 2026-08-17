import { api } from "$lib";
import type { Component } from "svelte";
import { AuthService } from "./auth.svelte";
import { CustomizationService } from "./customization.svelte";
import { DiscordService } from "./discord.svelte";
import { EventsService } from "./events.svelte";
import { LauncherService } from "./launcher.svelte";
import { LogsService } from "./logs.svelte";
import { NotificationService } from "./notification.svelte";
import { ProfilesService } from "./profiles.svelte";
import { ProjectsService } from "./projects.svelte";
import { UpdaterService } from "./updater.svelte";

type ModalInstance = {
  id: string;
  component: Component;
  props: Record<string, unknown>;
  resolve: (value: unknown) => void;
};

export interface Service {
  init(): Promise<void> | void;
  restart?(): Promise<void> | void;
  destroy(): Promise<void> | void;
}

export class AppService {
  notificationService = new NotificationService();
  authService = new AuthService();
  customizationService = new CustomizationService();
  discordService = new DiscordService();
  logsService = new LogsService();
  launcherService = new LauncherService();
  launcherEventsService = new EventsService(this);
  profilesService = new ProfilesService();
  projectsService = new ProjectsService();
  updaterService = new UpdaterService();

  /// ! note that the order of services here matters, as some services depend on others being initialized first
  get services(): Service[] {
    return [
      this.notificationService,
      this.customizationService,
      this.updaterService,
      this.authService,
      this.logsService,
      this.launcherEventsService,
      this.discordService,
      this.launcherService,
      this.profilesService,
      this.projectsService,
    ];
  }

  constructor() {
    console.log("Creating AppService...");
  }

  async initAll() {
    for (const s of this.services) {
      await s.init();
    }
  }

  async destroyAll() {
    for (const s of this.services) {
      await s.destroy();
    }
  }

  async restartAll() {
    for (const s of this.services) {
      if (s.restart) {
        await s.restart();
        continue;
      }
      await s.destroy();
      await s.init();
    }
  }

  // ? We also put some "app generic" api methods here like opening urls, etc. that are not specific to any service
  async openUrl(url: string) {
    try {
      await api.openUrl(url);
    } catch (e) {
      console.error("API call failed: `await api.openUrl(url);`", e);
    }
  }

  async openFile(filePath: string) {
    try {
      await api.openPath(filePath);
    } catch (e) {
      console.error("API call failed: `await api.openPath(filePath);`", e);
    }
  }

  // #region Modal
  public readonly stack = $state<ModalInstance[]>([]);

  public show<TResult = void>(component: Component, props: Record<string, unknown> = {}): Promise<TResult> {
    return new Promise<TResult>((resolve) => {
      this.stack.push({
        id: crypto.randomUUID(),
        component,
        props,
        resolve: resolve as (value: unknown) => void,
      });
    });
  }

  public resolve<TResult>(id: string, value: TResult): void {
    const index = this.stack.findIndex((m) => m.id === id);

    if (index === -1) {
      return;
    }

    this.stack[index].resolve(value);
    this.stack.splice(index, 1);
  }

  public dismiss(id: string): void {
    this.resolve(id, undefined);
  }

  public dismissTop(): void {
    const top = this.stack.at(-1);

    if (top) {
      this.dismiss(top.id);
    }
  }

  public clear(): void {
    while (this.stack.length > 0) {
      this.dismissTop();
    }
  }
  // #endregion Modal
}

export const app = new AppService();
