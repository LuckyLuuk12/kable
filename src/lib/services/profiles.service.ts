import { type KableAccount, type KableProfile, api } from "$lib";

class ProfilesService {
  profiles = $state<KableProfile[]>([]);
  activeProfileId = $state<string | null>(null);

  loaded = $state(false);
  loading = $state(false);

  async load() {
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
    return this.profiles.find(p => p.id === this.activeProfileId) ?? null;
  }

  async setActive(profile: KableProfile) {
    await api.setActiveAccount(profile as unknown as KableAccount);

    this.activeProfileId = profile.id;
  }

  async add(profile: KableProfile) {
    await api.addAccount(profile as unknown as KableAccount);

    this.profiles = [...this.profiles, profile];
  }

  async modify(oldProfile: KableProfile, newProfile: KableProfile) {
    const updated = await api.modifyProfile(oldProfile, newProfile);

    this.profiles = this.profiles.map(p =>
      p.id === updated.id ? updated : p
    );
  }

  async remove(id: string) {
    await api.deleteProfile(id);

    this.profiles = this.profiles.filter(p => p.id !== id);

    if (this.activeProfileId === id) {
      this.activeProfileId = null;
    }
  }

  async refresh() {
    this.loaded = false;
    await this.load();
  }
}

export const profilesService = new ProfilesService();