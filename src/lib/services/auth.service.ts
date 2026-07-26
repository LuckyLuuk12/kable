import {
  type DeviceCodeResponse,
  type KableAccount,
  type MicrosoftToken,
  api,
} from "$lib";
import type { Service } from "./app.service";

export class AuthService implements Service {
  accounts = $state<KableAccount[]>([]);
  activeAccount = $state<KableAccount | null>(null);

  loading = $state(false);
  authenticating = $state(false);

  loaded = $state(false);

  /**
   * Initial load of accounts + active account
   */
  async init() {
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
    try {
      return await api.pollAuthentication(deviceCode);
    } catch (error) {
      console.error("Failed to poll authentication:", error);
      throw error;
    }
  }

  /**
   * Add account to local store
   */
  async addAccount(account: KableAccount) {
    try {
      await api.addAccount(account);
      this.accounts = [...this.accounts, account];
    } catch (error) {
      console.error("Failed to add account:", error);
      throw error;
    }
  }

  /**
   * Remove account
   */
  async removeAccount(account: KableAccount) {
    const updated = await api.removeAccount(account);
    this.accounts = updated;

    if (this.activeAccount?.local_id === account.local_id) {
      this.activeAccount = null;
    }
  }

  /**
   * Remove active account
   */
  async removeActiveAccount() {
    if (!this.activeAccount) return;

    await this.removeAccount(this.activeAccount);
  }

  /**
   * Set active account (also affects backend state)
   */
  async setActive(account: KableAccount) {
    try {
      await api.setActiveAccount(account);

      this.activeAccount = account;
    } catch (error) {
      console.error("Failed to set active account:", error);
      throw error;
    }
  }

  /**
   * Account display name
   */
  getAccountDisplayName(account: KableAccount | null): string {
    return account?.minecraft_profile?.name ?? account?.username ?? "Unknown User";
  }

  /**
   * Account UUID display
   */
  getAccountUuid(account: KableAccount | null): string {
    return account?.minecraft_profile?.id ?? "Unknown UUID";
  }

  /**
   * Avatar title
   */
  getAccountAvatarTitle(account: KableAccount | null): string {
    return `${this.getAccountDisplayName(account)}'s avatar`;
  }

  /**
   * Determine account status
   */
  getAccountStatus(
    account: KableAccount | null,
  ): "online" | "offline" | "expired" {
    if (!account?.access_token) return "offline";

    if (account.access_token_expires_at) {
      const expiryDate = new Date(account.access_token_expires_at);
      if (expiryDate <= new Date()) return "expired";
    }

    return "online";
  }

  /**
   * Format token expiry for display
   */
  formatTokenExpiry(
    account: KableAccount | null,
    now: number = Date.now(),
  ): string {
    if (!account || !account.access_token_expires_at) return "Never expires";

    const expiryDate = new Date(account.access_token_expires_at);
    const diff = expiryDate.getTime() - now;

    if (diff <= 0) return "Expired";

    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));

    if (days > 0) return `Expires in ${days} day${days > 1 ? "s" : ""}`;
    if (hours > 0) return `Expires in ${hours} hour${hours > 1 ? "s" : ""}`;
    if (minutes > 0) {
      return `Expires in ${minutes} minute${minutes > 1 ? "s" : ""}`;
    }

    return "Expires soon";
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

  /**
   * Cleanup
   */
  async destroy() {
    this.accounts = [];
    this.activeAccount = null;
    this.loaded = false;
  }
}
