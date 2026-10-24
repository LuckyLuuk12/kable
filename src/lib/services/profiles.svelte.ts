
import { type CreateProfileRequest, type KableProfile, type LoaderKind, api, error, log } from "$lib";
import { SvelteDate } from "svelte/reactivity";
import type { Service } from "./app.svelte";
import { app } from "./app.svelte";

export type ProfileStatistics = {
  totalPlaytimeMs: number;
  totalProfiles: number;
  lastPlayedDate: string | null;
  totalLaunches: number;
  favoriteCount: number;
  averageLaunchesPerProfile: number;
  mostPlayedProfile: KableProfile | null;
  mostUsedLoader: string | null;
  loaderCounts: Record<string, number>;
};

export type AccountStatistics = {
  profiles: ProfileStatistics;
};

export class ProfilesService implements Service {
  profiles = $state<KableProfile[]>([]);

  loaded = $state(false);
  loading = $state(false);

  private loadGeneration = 0;
  private refreshRequest = 0;
  private refreshPromise: Promise<void> | null = null;
  private pendingMutations = 0;

  async init() {
    if (this.loaded) return;
    await this.refreshProfiles();
  }

  async refreshProfiles(force = false): Promise<void> {
    const request = ++this.refreshRequest;
    const generation = this.loadGeneration;

    this.loading = true;

    try {
      const profiles = await api.getProfiles(force);

      // Do not allow an older response to overwrite a newer refresh or
      // a state reset that happened while this request was running.
      if (request !== this.refreshRequest || generation !== this.loadGeneration) {
        return;
      }

      if (JSON.stringify(profiles) !== JSON.stringify(this.profiles)) {
        this.profiles = profiles;
      }

      this.loaded = true;
    } catch (err) {
      error("[ProfilesService] Failed to refresh profiles", err);
      throw err;
    } finally {
      if (request === this.refreshRequest && generation === this.loadGeneration) {
        this.loading = false;
      }
    }
  }

  /**
   * Refresh the profile list without allowing concurrent refreshes to
   * overwrite newer state.
   */
  private async refreshAfterMutation(force = true): Promise<void> {
    this.pendingMutations++;

    try {
      await this.refreshProfiles(force);
    } finally {
      this.pendingMutations--;
    }
  }

  /**
   * Creates a deep clone of the given profile object in memory.
   * Changes to the clone do not affect the original object.
   */
  memoryClone(profile: KableProfile): KableProfile {
    const res = JSON.parse(JSON.stringify(profile)) as KableProfile;
    log("[ProfilesService] Cloned profile", profile.id, "to memory clone", res.id);
    return res;
  }

  async modify(oldProfile: KableProfile, newProfile: KableProfile): Promise<void> {
    try {
      await api.modifyProfile(oldProfile, newProfile);
      await this.refreshAfterMutation();

      app.notificationService.success("Profile updated successfully.");
    } catch (err) {
      error("[ProfilesService] Failed to modify profile", err);
      app.notificationService.error("Failed to update profile.");
      throw err;
    }
  }

  async toggleFavorite(profile: KableProfile): Promise<void> {
    const previousProfiles = this.profiles;
    const target = previousProfiles.find((item) => item.id === profile.id);

    if (!target) {
      await this.refreshProfiles(true);
      return;
    }

    const previousFavorite = target.metadata.favorite;
    const optimisticProfiles = previousProfiles.map((item) =>
      item.id === profile.id
        ? {
          ...item,
          metadata: {
            ...item.metadata,
            favorite: !previousFavorite,
          },
        }
        : item,
    );

    this.profiles = optimisticProfiles;

    try {
      await api.toggleFavorite(target);
      await this.refreshAfterMutation();
    } catch (err) {
      // Restore only if no later state update has replaced our optimistic
      // state. Otherwise refresh from the backend to avoid clobbering it.
      error("[ProfilesService] Failed to toggle favorite status for profile", err);

      try {
        await this.refreshProfiles(true);
      } catch (refreshError) {
        error("[ProfilesService] Failed to recover profiles after favorite update", refreshError);

        if (this.profiles === optimisticProfiles) {
          this.profiles = previousProfiles;
        }
      }

      app.notificationService.error("Failed to update profile favorite status.");
      throw err;
    }
  }

  getLoaderColor(loader: LoaderKind): string {
    switch (loader) {
      case "vanilla":
        return "#11833c";
      case "fabric":
        return "#dbb866";
      case "forge":
        return "#1e2d43";
      case "quilt":
        return "#9c5aa0";
      case "neo_forge":
        return "#f16436";
      case "iris_fabric":
        return "#4c8cff";
      default:
        return "#cccccc";
    }
  }

  getLoaderIcon(loader: LoaderKind): string {
    switch (loader) {
      case "vanilla":
        return "vanilla";
      case "fabric":
        return "fabric";
      case "forge":
        return "forge";
      case "quilt":
        return "quilt";
      case "neo_forge":
        return "neoforge";
      case "iris_fabric":
        return "iris_fabric";
      default:
        return "unknown";
    }
  }

  getLoaderImage(loader: LoaderKind): string {
    switch (loader) {
      case "vanilla":
        return "vanilla";
      case "fabric":
        return "fabric";
      case "forge":
        return "forge";
      case "quilt":
        return "quilt";
      case "neo_forge":
        return "neo_forge";
      case "iris_fabric":
        return "iris_fabric";
      default:
        return "unknown";
    }
  }

  async remove(id: string): Promise<void> {
    try {
      await api.deleteProfile(id);
      await this.refreshAfterMutation();
      log("[ProfilesService] Profile deleted successfully.");

      app.notificationService.success("Profile deleted successfully.");
    } catch (err) {
      error("[ProfilesService] Failed to delete profile", err);
      app.notificationService.error("Failed to delete profile.");
      throw err;
    }
  }

  async createProfile(request: CreateProfileRequest): Promise<void> {
    try {
      await api.createProfile(request);
      await this.refreshAfterMutation();
      log("[ProfilesService] Profile created successfully.");

      app.notificationService.success("Profile created successfully.");
    } catch (err) {
      error("[ProfilesService] Failed to create profile", err);
      app.notificationService.error("Failed to create profile.");
      throw err;
    }
  }

  async createShortcut(profile: KableProfile): Promise<void> {
    try {
      await api.createDesktopShortcut(profile);

      app.notificationService.success("Desktop shortcut created successfully.");
    } catch (err) {
      error("[ProfilesService] Failed to create shortcut for profile", err);
      app.notificationService.error("Failed to create desktop shortcut.");
      throw err;
    }
  }

  async exportProfile(profile: KableProfile, destination: string): Promise<void> {
    try {
      await api.exportProfile(profile, destination);

      app.notificationService.success("Profile exported successfully.");
    } catch (err) {
      error("[ProfilesService] Failed to export profile", err);
      app.notificationService.error("Failed to export profile.");
      throw err;
    }
  }

  public getStatistics(): AccountStatistics {
    const profiles = this.profiles;

    const totalProfiles = profiles.length;

    const totalPlaytimeMs = profiles.reduce(
      (total, profile) => total + profile.metadata.total_time_played_ms,
      0,
    );

    const totalLaunches = profiles.reduce(
      (total, profile) => total + profile.metadata.times_launched,
      0,
    );

    const favoriteCount = profiles.filter((profile) => profile.metadata.favorite).length;

    const lastPlayedDate = profiles.reduce<string | null>((latest, profile) => {
      const candidate = profile.metadata.last_used;

      if (!candidate) return latest;
      if (!latest) return candidate;

      const candidateDate = new SvelteDate(candidate);
      const latestDate = new SvelteDate(latest);

      if (
        Number.isNaN(candidateDate.getTime()) ||
        Number.isNaN(latestDate.getTime())
      ) {
        return latest;
      }

      return candidateDate.getTime() > latestDate.getTime() ? candidate : latest;
    }, null);

    const mostPlayedProfile = profiles.reduce<KableProfile | null>(
      (mostPlayed, profile) => {
        if (
          !mostPlayed ||
          profile.metadata.total_time_played_ms >
          mostPlayed.metadata.total_time_played_ms
        ) {
          return profile;
        }

        return mostPlayed;
      },
      null,
    );

    const loaderCounts: Record<string, number> = {};

    for (const profile of profiles) {
      const loader = profile.version.loader;

      if (!loader) continue;

      loaderCounts[loader] = (loaderCounts[loader] ?? 0) + 1;
    }

    const mostUsedLoader = Object.entries(loaderCounts).reduce<string | null>(
      (mostUsed, [loader, count]) => {
        if (!mostUsed || count > (loaderCounts[mostUsed] ?? 0)) {
          return loader;
        }

        return mostUsed;
      },
      null,
    );

    return {
      profiles: {
        totalPlaytimeMs,
        totalProfiles,
        lastPlayedDate,
        totalLaunches,
        favoriteCount,
        averageLaunchesPerProfile:
          totalProfiles > 0 ? totalLaunches / totalProfiles : 0,
        mostPlayedProfile,
        mostUsedLoader,
        loaderCounts,
      },
    };
  }

  async destroy(): Promise<void> {
    this.loadGeneration++;
    this.refreshRequest++;
    this.profiles = [];
    this.loaded = false;
    this.loading = false;
    this.refreshPromise = null;
    this.pendingMutations = 0;
  }
}
