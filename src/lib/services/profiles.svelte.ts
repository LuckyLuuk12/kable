import { type KableAccount, type KableProfile, type LoaderKind, api } from "$lib";
import type { Service } from "./app.svelte";

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

  async init() {
    if (this.loaded) return;

    this.loading = true;
    try {
      const [profiles] = await Promise.all([api.getProfiles()]);

      this.profiles = profiles;

      this.loaded = true;
    } finally {
      this.loading = false;
    }
  }

  async refreshProfiles() {
    this.loading = true;
    try {
      const profiles = await api.getProfiles();
      this.profiles = profiles;
    } catch (e) {
      console.error("Failed to refresh profiles", e);
    } finally {
      this.loading = false;
    }
  }

  /**
   * Creates a deep clone of the given profile object in memory. This is useful for creating a temporary copy of a profile for editing purposes without affecting the original profile until changes are saved.
   * @param profile The profile to clone.
   * @returns A deep clone of the given profile.
   */
  memoryClone(profile: KableProfile): KableProfile {
    return JSON.parse(JSON.stringify(profile));
  }

  async add(profile: KableProfile) {
    try {
      await api.addAccount(profile as unknown as KableAccount);

      this.profiles = [...this.profiles, profile];
    } catch (e) {
      console.error("Failed to add profile", e);
    }
  }

  async modify(oldProfile: KableProfile, newProfile: KableProfile) {
    try {
      const updated = await api.modifyProfile(oldProfile, newProfile);

      this.profiles = this.profiles.map((p) => (p.id === updated.id ? updated : p));
    } catch (e) {
      console.error("Failed to modify profile", e);
    }
  }

  /// Modifies a profile's favorite status, and then updates with the this.modify method to ensure the change is persisted and reflected in the profiles list.
  async toggleFavorite(profile: KableProfile) {
    try {
      await api.toggleFavorite(profile);
    } catch (e) {
      console.error("Failed to toggle favorite status for profile", e);
    }
  }

  getLoaderColor(loader: LoaderKind): string {
    switch (loader) {
      case "vanilla":
        return "#11833c"; // Vanilla's green/grass color
      case "fabric":
        return "#dbb866"; // Fabric's golden color
      case "forge":
        return "#1e2d43"; // Forge's dark color
      case "quilt":
        return "#9c5aa0"; // Quilt's purple color
      case "neo_forge":
        return "#f16436"; // NeoForge's orange color
      case "iris_fabric":
        return "#4c8cff"; // Iris Fabric's blue color
      default:
        return "#cccccc"; // Default gray for unknown loaders
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
        return "unknown"; // Default icon for unknown loaders
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
        return "unknown"; // Default image for unknown loaders
    }
  }

  async remove(id: string) {
    try {
      await api.deleteProfile(id);
    } catch (e) {
      console.error("Failed to delete profile", e);
    }

    this.profiles = this.profiles.filter((p) => p.id !== id);
  }

  async createProfile(versionId: string, baseProfile: KableProfile | null = null, exportedZip: string | null = null, mrpack: string | null = null) {
    try {
      // TODO: change to versionId, baseProfile, exportedZip, mrpack, instead of 3 nulls
      const newProfile = await api.createProfile(versionId, baseProfile, exportedZip, mrpack);
      console.log("[Profiles] Created new profile:", newProfile?.metadata.name);
      this.profiles = [...this.profiles, newProfile];
    } catch (e) {
      console.error("Failed to create profile", e);
    }
  }

  async createShortcut(profile: KableProfile) {
    try {
      // await api.createShortcut(profile);
    } catch (e) {
      console.error("Failed to create shortcut for profile", e);
    }
  }

  async exportProfile(profile: KableProfile) {
    try {
      // await api.exportProfile(profile);
    } catch (e) {
      console.error("Failed to export profile", e);
    }
  }

  public getStatistics(): AccountStatistics {
    const profiles = this.profiles;

    const totalProfiles = profiles.length;

    const totalPlaytimeMs = profiles.reduce((total, profile) => total + profile.metadata.total_time_played_ms, 0);

    const totalLaunches = profiles.reduce((total, profile) => total + profile.metadata.times_launched, 0);

    const favoriteCount = profiles.filter((profile) => profile.metadata.favorite).length;

    const lastPlayedDate = profiles.reduce<string | null>((latest, profile) => {
      if (!latest) {
        return profile.metadata.last_used;
      }

      return new Date(profile.metadata.last_used) > new Date(latest) ? profile.metadata.last_used : latest;
    }, null);

    const mostPlayedProfile = profiles.reduce<KableProfile | null>((mostPlayed, profile) => {
      if (!mostPlayed || profile.metadata.total_time_played_ms > mostPlayed.metadata.total_time_played_ms) {
        return profile;
      }

      return mostPlayed;
    }, null);

    const loaderCounts: Record<string, number> = {};

    for (const profile of profiles) {
      const loader = profile.version.loader;

      if (!loader) {
        continue;
      }

      loaderCounts[loader] = (loaderCounts[loader] ?? 0) + 1;
    }

    const mostUsedLoader = Object.entries(loaderCounts).reduce<string | null>((mostUsed, [loader, count]) => {
      if (!mostUsed || count > loaderCounts[mostUsed]) {
        return loader;
      }

      return mostUsed;
    }, null);

    return {
      profiles: {
        totalPlaytimeMs,
        totalProfiles,
        lastPlayedDate,
        totalLaunches,
        favoriteCount,
        averageLaunchesPerProfile: totalProfiles > 0 ? totalLaunches / totalProfiles : 0,
        mostPlayedProfile,
        mostUsedLoader,
        loaderCounts,
      },
    };
  }

  async destroy() {
    this.profiles = [];
    this.loaded = false;
  }
}
