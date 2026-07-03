import { AuthService } from "./auth.service";
import { CustomizationService } from "./customization.service";
import { DiscordService } from "./discord.service";
import { LauncherService } from "./launcher.service";
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
  launcherService = new LauncherService();
  profilesService = new ProfilesService();
  projectsService = new ProjectsService();
  updaterService = new UpdaterService();

  /// ! note that the order of services here matters, as some services depend on others being initialized first
  get services(): Service[] {
    return [
      this.customizationService,
      this.updaterService,
      this.authService,
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
}

export const app = new AppService();
