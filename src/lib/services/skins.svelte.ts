import type {
  AccountCape,
  AccountSkin,
  CurrentSkin,
  KableAccount,
  PlayerProfile,
  SkinModel,
  SkinUploadConfig,
  SkinUploadResponse,
} from "$lib";
import { api, app } from "$lib";
import type { Service } from "./app.svelte";

export class SkinsService implements Service {
  initialized = $state(false);
  loading = $state(false);
  uploading = $state(false);
  modifying = $state(false);

  error = $state<string | null>(null);

  account = $state<KableAccount | null>(null);
  profile = $state<PlayerProfile | null>(null);
  currentSkin = $state<CurrentSkin | null>(null);

  onlineSkins = $state<AccountSkin[]>([]);
  localSkins = $state<AccountSkin[]>([]);
  activeCape = $state<AccountCape | null>(null);

  get allSkins(): AccountSkin[] {
    return [...this.onlineSkins, ...this.localSkins];
  }

  get currentSkinModel(): SkinModel | null {
    return this.currentSkin?.model ?? null;
  }

  get hasSkin(): boolean {
    return this.currentSkin?.has_skin ?? false;
  }

  get capes(): AccountCape[] {
    return this.profile?.capes ?? [];
  }

  /**
   * Initialize the service for an account.
   * Loads the account's online profile, current skin, and local skins.
   */
  async init() {
    const account = app.authService.activeAccount;

    if (!account) {
      this.clear();
      return;
    }

    this.account = account;
    await this.refresh();
    this.initialized = true;
  }

  /**
   * Refresh all skin-related data.
   */
  async refresh() {
    if (!this.account) {
      this.error = "No Minecraft account selected.";
      return;
    }

    this.loading = true;
    this.error = null;

    try {
      const [profile, currentSkin, localSkins] = await Promise.all([
        api.getPlayerProfile(this.account),
        api.getCurrentSkinInfo(this.account),
        api.getLocalSkins(),
      ]);

      this.profile = profile;
      this.currentSkin = currentSkin;
      this.onlineSkins = profile.skins;
      this.localSkins = localSkins;
      this.activeCape =
        profile.capes.find((cape) => cape.state.toUpperCase() === "ACTIVE") ??
        null;
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  /**
   * Change the account and reload its skin data.
   */
  async setAccount(account: KableAccount | null) {
    this.account = account;

    if (!account) {
      this.clear();
      return;
    }

    await this.refresh();
    this.initialized = true;
  }

  /**
   * Refresh only the online profile and current skin.
   */
  async refreshOnline() {
    if (!this.account) {
      this.error = "No Minecraft account selected.";
      return;
    }

    this.error = null;

    try {
      const [profile, currentSkin] = await Promise.all([
        api.getPlayerProfile(this.account),
        api.getCurrentSkinInfo(this.account),
      ]);

      this.profile = profile;
      this.currentSkin = currentSkin;
      this.onlineSkins = profile.skins;
      this.activeCape =
        profile.capes.find((cape) => cape.state.toUpperCase() === "ACTIVE") ??
        null;
    } catch (e) {
      this.error = String(e);
    }
  }

  /**
   * Refresh only locally saved skins.
   */
  async refreshLocal() {
    this.error = null;

    try {
      this.localSkins = await api.getLocalSkins();
    } catch (e) {
      this.error = String(e);
    }
  }

  /**
   * Save a skin to local custom-skin storage.
   * This does not upload the skin to Mojang.
   */
  async addLocalSkin(config: SkinUploadConfig): Promise<SkinUploadResponse | null> {
    this.uploading = true;
    this.error = null;

    try {
      const response = await api.uploadSkinToAccount(config);
      await this.refreshLocal();
      return response;
    } catch (e) {
      this.error = String(e);
      return null;
    } finally {
      this.uploading = false;
    }
  }

  /**
   * Update local skin metadata.
   */
  async modifySkin(
    skinId: string,
    options: {
      name?: string;
      capeId?: string;
      slim?: boolean;
    },
  ): Promise<boolean> {
    this.modifying = true;
    this.error = null;

    try {
      await api.modifySkinById(
        skinId,
        options.name ?? null,
        options.capeId ?? null,
        options.slim ?? null,
      );

      await this.refreshLocal();
      return true;
    } catch (e) {
      this.error = String(e);
      return false;
    } finally {
      this.modifying = false;
    }
  }

  /**
   * Remove a locally saved skin.
   */
  async removeLocalSkin(skinId: string): Promise<boolean> {
    this.modifying = true;
    this.error = null;

    try {
      await api.removeSkinById(skinId);
      this.localSkins = this.localSkins.filter((skin) => skin.id !== skinId);
      return true;
    } catch (e) {
      this.error = String(e);
      return false;
    } finally {
      this.modifying = false;
    }
  }

  /**
   * Apply a locally saved skin to the current Minecraft account.
   */
  async applySkin(skinId: string): Promise<boolean> {
    this.uploading = true;
    this.error = null;

    try {
      await api.applyAccountSkin(skinId);
      await this.refreshOnline();
      return true;
    } catch (e) {
      this.error = String(e);
      return false;
    } finally {
      this.uploading = false;
    }
  }

  /**
   * Change the model of the currently active skin.
   */
  async changeModel(model: SkinModel): Promise<boolean> {
    this.uploading = true;
    this.error = null;

    try {
      await api.changeSkinModel(model);
      await this.refreshOnline();
      return true;
    } catch (e) {
      this.error = String(e);
      return false;
    } finally {
      this.uploading = false;
    }
  }

  /**
   * Activate a cape, or pass null to hide the active cape.
   */
  async applyCape(capeId: string | null): Promise<boolean> {
    this.modifying = true;
    this.error = null;

    try {
      await api.applyCape(capeId);
      await this.refreshOnline();
      return true;
    } catch (e) {
      this.error = String(e);
      return false;
    } finally {
      this.modifying = false;
    }
  }

  /**
 * Download skin image data from a URL.
 */
  async downloadSkin(url: string): Promise<Uint8Array | null> {
    this.error = null;

    try {
      return new Uint8Array(await api.downloadSkinFromUrl(url));
    } catch (e) {
      this.error = String(e);
      return null;
    }
  }

  /**
   * Clear account-specific state.
   */
  clear() {
    this.account = null;
    this.profile = null;
    this.currentSkin = null;
    this.onlineSkins = [];
    this.localSkins = [];
    this.activeCape = null;
    this.initialized = false;
    this.error = null;
    this.loading = false;
    this.uploading = false;
    this.modifying = false;
  }

  /**
   * Cleanup.
   */
  async destroy() {
    this.clear();
  }
}
