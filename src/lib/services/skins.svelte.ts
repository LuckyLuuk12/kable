
import type {
  AccountCape,
  AccountSkin,
  CurrentSkin,
  KableAccount,
  PlayerProfile,
  SkinModel,
  SkinUploadConfig,
  SkinUploadResponse,
} from "#lib";
import { api, app } from "#lib";
import type { Service } from "./app.svelte";

const SKIN_PROPAGATION_DELAYS_MS = [
  1_000,
  2_000,
  4_000,
  8_000,
  10_000,
  10_000,
  10_000,
];

export class SkinsService implements Service {
  private skinApplyOperation = 0;
  initialized = $state(false);
  loading = $state(false);
  uploading = $state(false);
  modifying = $state(false);

  error = $state<string | null>(null);

  get account(): KableAccount | null {
    return app.authService.activeAccount;
  }

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
   * Initialize the service for the currently active account.
   */
  async init() {
    if (!this.account) {
      this.clear();
      return;
    }

    await this.refresh();
  }

  /**
   * Check whether an account is still the active account.
   */
  private isActiveAccount(account: KableAccount): boolean {
    return this.account?.local_id === account.local_id;
  }

  /**
   * Refresh all skin-related data.
   */
  async refresh() {
    const account = this.account;

    if (!account) {
      this.clear();
      return;
    }

    this.loading = true;
    this.error = null;

    try {
      const [profile, currentSkin, localSkins] = await Promise.all([
        api.getPlayerProfile(account),
        api.getCurrentSkinInfo(account),
        api.getLocalSkins(),
      ]);

      if (!this.isActiveAccount(account)) return;

      this.profile = profile;
      this.currentSkin = currentSkin;
      this.onlineSkins = profile.skins;
      this.localSkins = localSkins;
      this.activeCape =
        profile.capes.find((cape) => cape.state.toUpperCase() === "ACTIVE") ?? null;

      this.initialized = true;
    } catch (e) {
      if (this.isActiveAccount(account)) {
        this.error = String(e);
      }
    } finally {
      if (this.isActiveAccount(account)) {
        this.loading = false;
      }
    }
  }

  /**
   * Refresh only the online profile and current skin.
   */
  async refreshOnline() {
    const account = this.account;

    if (!account) {
      this.clear();
      return;
    }

    this.error = null;

    try {
      const [profile, currentSkin] = await Promise.all([
        api.getPlayerProfile(account),
        api.getCurrentSkinInfo(account),
      ]);

      if (!this.isActiveAccount(account)) return;

      this.profile = profile;
      this.currentSkin = currentSkin;
      this.onlineSkins = profile.skins;
      this.activeCape =
        profile.capes.find((cape) => cape.state.toUpperCase() === "ACTIVE") ?? null;
    } catch (e) {
      if (this.isActiveAccount(account)) {
        this.error = String(e);
      }
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
  async addLocalSkin(
    config: SkinUploadConfig,
  ): Promise<SkinUploadResponse | null> {
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
 * Apply a locally saved skin to the currently active Minecraft account.
 *
 * Waits for Mojang's session profile and texture CDN to expose the
 * uploaded skin before updating the frontend's current skin state.
 */
  async applySkin(skinId: string): Promise<boolean> {
    const account = this.account;

    if (!account) {
      this.error = "No Minecraft account selected.";
      return false;
    }

    const operation = ++this.skinApplyOperation;
    const previousSkin = this.currentSkin
      ? { ...this.currentSkin }
      : null;

    this.uploading = true;
    this.error = null;

    const isCurrentOperation = () =>
      operation === this.skinApplyOperation &&
      this.isActiveAccount(account);

    try {
      await api.applyAccountSkin(skinId);

      if (!isCurrentOperation()) return false;

      let updatedSkin: CurrentSkin | null = null;
      let lastError: unknown = null;

      for (
        let attempt = 0;
        attempt <= SKIN_PROPAGATION_DELAYS_MS.length;
        attempt++
      ) {
        if (!isCurrentOperation()) return false;

        try {
          const candidate = await api.getCurrentSkinInfo(account);

          if (!isCurrentOperation()) return false;

          const skinChanged =
            candidate.url !== previousSkin?.url ||
            candidate.model !== previousSkin?.model;

          if (candidate.url && skinChanged) {
            const texture = await api.downloadSkinFromUrl(candidate.url);

            if (!isCurrentOperation()) return false;

            if (texture.length > 0) {
              updatedSkin = candidate;
              break;
            }
          }
        } catch (e) {
          lastError = e;
        }

        if (attempt < SKIN_PROPAGATION_DELAYS_MS.length) {
          await new Promise((resolve) =>
            setTimeout(resolve, SKIN_PROPAGATION_DELAYS_MS[attempt]),
          );
        }
      }

      if (!isCurrentOperation()) return false;

      if (!updatedSkin) {
        this.error = lastError
          ? `Mojang accepted the upload, but the updated skin could not be retrieved: ${String(lastError)}`
          : "Mojang accepted the upload, but the updated skin has not propagated yet. Try refreshing later.";

        return false;
      }

      // The new texture is available. Update it immediately rather than
      // waiting for the separate profile request.
      this.currentSkin = updatedSkin;

      try {
        const profile = await api.getPlayerProfile(account);

        if (!isCurrentOperation()) return false;

        this.profile = profile;
        this.onlineSkins = profile.skins;
        this.activeCape =
          profile.capes.find(
            (cape) => cape.state.toUpperCase() === "ACTIVE",
          ) ?? null;
      } catch (e) {
        // The skin itself was successfully updated. A profile refresh
        // failure should not turn a successful skin upload into a failure.
        this.error = `Skin updated, but refreshing the player profile failed: ${String(e)}`;
      }

      return true;
    } catch (e) {
      if (isCurrentOperation()) {
        this.error = String(e);
      }

      return false;
    } finally {
      if (operation === this.skinApplyOperation) {
        this.uploading = false;
      }
    }
  }

  /**
   * Change the model of the currently active skin.
   */
  async changeModel(model: SkinModel): Promise<boolean> {
    const account = this.account;

    if (!account) {
      this.error = "No Minecraft account selected.";
      return false;
    }

    this.uploading = true;
    this.error = null;

    try {
      await api.changeSkinModel(model);

      if (!this.isActiveAccount(account)) return false;

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
    const account = this.account;

    if (!account) {
      this.error = "No Minecraft account selected.";
      return false;
    }

    this.modifying = true;
    this.error = null;

    try {
      await api.applyCape(capeId);

      if (!this.isActiveAccount(account)) return false;

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
