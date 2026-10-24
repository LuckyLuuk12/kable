import { type DeviceCodeResponse, type KableAccount, type MicrosoftToken, api } from "$lib";
import { SvelteDate } from "svelte/reactivity";
import type { Service } from "./app.svelte";

export class AuthService implements Service {
  accounts = $state<KableAccount[]>([]);
  activeAccount = $state<KableAccount | null>(null);

  loading = $state(false);
  authenticating = $state(false);

  loaded = $state(false);

  async init() {
    if (this.loaded) return;

    this.loading = true;

    try {
      const [accounts, active] = await Promise.all([api.listAccounts(), api.getActiveAccount()]);

      this.accounts = accounts;
      this.activeAccount = active;
      this.loaded = true;
    } finally {
      this.loading = false;
    }
  }

  async startAuth(): Promise<DeviceCodeResponse> {
    this.authenticating = true;

    try {
      return await api.startAuthentication();
    } finally {
      this.authenticating = false;
    }
  }

  async pollAuth(deviceCode: string): Promise<MicrosoftToken> {
    this.authenticating = true;

    try {
      return await api.pollAuthentication(deviceCode);
    } catch (error) {
      console.error("Failed to poll authentication:", error);
      throw error;
    } finally {
      this.authenticating = false;
    }
  }

  async authenticateAccount(token: MicrosoftToken): Promise<KableAccount> {
    this.authenticating = true;

    try {
      const account = await api.authenticate(token);

      const existingIndex = this.accounts.findIndex((existing) => existing.local_id === account.local_id);

      if (existingIndex >= 0) {
        this.accounts = this.accounts.map((existing, index) => (index === existingIndex ? account : existing));
      } else {
        this.accounts = [...this.accounts, account];
      }

      if (!this.activeAccount) {
        this.activeAccount = account;
      }

      return account;
    } catch (error) {
      console.error("Failed to authenticate account:", error);
      throw error;
    } finally {
      this.authenticating = false;
    }
  }

  async removeAccount(account: KableAccount) {
    const wasActive = this.activeAccount?.local_id === account.local_id;

    const updated = await api.removeAccount(account);
    this.accounts = updated;

    if (wasActive) {
      const nextAccount = updated[0] ?? null;

      if (nextAccount) {
        await this.setActive(nextAccount);
      } else {
        this.activeAccount = null;
      }
    }
  }

  async removeActiveAccount() {
    if (!this.activeAccount) return;

    await this.removeAccount(this.activeAccount);
  }

  async setActive(account: KableAccount) {
    try {
      await api.setActiveAccount(account);
      this.activeAccount = account;
    } catch (error) {
      console.error("Failed to set active account:", error);
      throw error;
    }
  }

  getAccountDisplayName(account: KableAccount | null): string {
    return account?.minecraft_profile?.name ?? account?.username ?? "Unknown User";
  }

  getAccountUuid(account: KableAccount | null): string {
    return account?.minecraft_profile?.id ?? "Unknown UUID";
  }

  getAccountAvatarTitle(account: KableAccount | null): string {
    return `${this.getAccountDisplayName(account)}'s avatar`;
  }

  getAccountStatus(account: KableAccount | null): "online" | "offline" | "expired" {
    if (!account?.access_token) return "offline";

    if (account.access_token_expires_at) {
      const expiryDate = new SvelteDate(account.access_token_expires_at);

      if (expiryDate <= new SvelteDate()) {
        return "expired";
      }
    }

    return "online";
  }

  formatTokenExpiry(account: KableAccount | null, now: number = Date.now()): string {
    if (!account || !account.access_token_expires_at) {
      return "Never expires";
    }

    const expiryDate = new SvelteDate(account.access_token_expires_at);
    const diff = expiryDate.getTime() - now;

    if (diff <= 0) return "Expired";

    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));

    if (days > 0) {
      return `Expires in ${days} day${days > 1 ? "s" : ""}`;
    }

    if (hours > 0) {
      return `Expires in ${hours} hour${hours > 1 ? "s" : ""}`;
    }

    if (minutes > 0) {
      return `Expires in ${minutes} minute${minutes > 1 ? "s" : ""}`;
    }

    return "Expires soon";
  }

  get hasAccounts() {
    return this.accounts.length > 0;
  }

  get isLoggedIn() {
    return this.activeAccount !== null;
  }

  get activeMinecraftProfile() {
    return this.activeAccount?.minecraft_profile ?? null;
  }

  async destroy() {
    this.accounts = [];
    this.activeAccount = null;
    this.loaded = false;
  }
}
