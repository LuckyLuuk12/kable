import { type KableAccount, type KableProfile, type LoaderKind, api } from "$lib";
import type { Service } from "./app.service";

export class ProfilesService implements Service {
  profiles = $state<KableProfile[]>([]);
  activeProfileId = $state<string | null>(null);

  loaded = $state(false);
  loading = $state(false);

  async init() {
    if (this.loaded) return;

    this.loading = true;
    try {
      const [profiles, activeAccount] = await Promise.all([
        api.getProfiles(),
        api.getActiveAccount(),
      ]);

      this.profiles = profiles;
      this.activeProfileId = activeAccount?.minecraft_profile.id ?? null;

      this.loaded = true;
    } finally {
      this.loading = false;
    }
  }

  get activeProfile(): KableProfile | null {
    return this.profiles.find((p) => p.id === this.activeProfileId) ?? null;
  }

  async setActive(profile: KableProfile) {
    try {
      await api.setActiveAccount(profile as unknown as KableAccount);

      this.activeProfileId = profile.id;
    } catch (e) {
      console.error("Failed to set active profile", e);
    }
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

      this.profiles = this.profiles.map((p) =>
        p.id === updated.id ? updated : p,
      );
    } catch (e) {
      console.error("Failed to modify profile", e);
    }
  }

  getLoaderColor(loader: LoaderKind): string {
    switch (loader) {
      case "vanilla":
        return "#11833c"; // Vanilla's green/grass color
      case "fabric":
        return "#dbb866"; // Fabric's golden color
      case "forge":
        return "#466381"; // Forge's dark color
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

  async remove(id: string) {
    try {
      await api.deleteProfile(id);
    } catch (e) {
      console.error("Failed to delete profile", e);
    }

    this.profiles = this.profiles.filter((p) => p.id !== id);

    if (this.activeProfileId === id) {
      this.activeProfileId = null;
    }
  }

  async destroy() {
    this.profiles = [];
    this.activeProfileId = null;
    this.loaded = false;
  }
}
