import { api } from "$lib";
import { AuthService } from "./auth.service";
import { CustomizationService } from "./customization.service";
import { DiscordService } from "./discord.service";
import { EventsService } from "./events.service";
import { LauncherService } from "./launcher.service";
import { LogsService } from "./logs.service";
import { ProfilesService } from "./profiles.service";
import { ProjectsService } from "./projects.service";
import { UpdaterService } from "./updater.service";

export interface Service {
  init(): Promise<void> | void;
  restart?(): Promise<void> | void;
  destroy(): Promise<void> | void;
}

export class AppService {
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
}

export const app = new AppService();
