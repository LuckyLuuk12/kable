import {
  type DeviceCodeResponse,
  type KableAccount,
  type MicrosoftToken,
  api,
} from "$lib";

class AuthService {
  accounts = $state<KableAccount[]>([]);
  activeAccount = $state<KableAccount | null>(null);

  loading = $state(false);
  authenticating = $state(false);

  loaded = $state(false);

  /**
   * Initial load of accounts + active account
   */
  async load() {
    if (this.loaded) return;

    this.loading = true;
    try {
      const [accounts, active] = await Promise.all([
        api.listAccounts(),
        api.getActiveAccount(),
      ]);

      this.accounts = accounts;
      this.activeAccount = active;
      this.loaded = true;
    } finally {
      this.loading = false;
    }
  }

  /**
   * Start Microsoft device code authentication flow
   */
  async startAuth(): Promise<DeviceCodeResponse> {
    this.authenticating = true;
    try {
      return await api.startAuthentication();
    } finally {
      this.authenticating = false;
    }
  }

  /**
   * Poll authentication until token is available
   */
  async pollAuth(deviceCode: string): Promise<MicrosoftToken> {
    return await api.pollAuthentication(deviceCode);
  }

  /**
   * Add account to local store
   */
  async addAccount(account: KableAccount) {
    await api.addAccount(account);
    this.accounts = [...this.accounts, account];
  }

  /**
   * Remove account
   */
  async removeAccount(account: KableAccount) {
    const updated = await api.removeAccount(account);
    this.accounts = updated;
  }

  /**
   * Set active account (also affects backend state)
   */
  async setActive(account: KableAccount) {
    await api.setActiveAccount(account);

    this.activeAccount = account;
  }

  /**
   * Refresh accounts from backend
   */
  async refresh() {
    this.loaded = false;
    await this.load();
  }

  /**
   * Derived helpers
   */
  get hasAccounts() {
    return this.accounts.length > 0;
  }

  get isLoggedIn() {
    return this.activeAccount !== null;
  }

  get activeMinecraftProfile() {
    return this.activeAccount?.minecraft_profile ?? null;
  }
}

export const authService = new AuthService();