import type { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import * as authApi from "$lib";
import {
  currentAccount,
  availableAccounts,
  isAuthenticating,
} from "../stores/auth";
import { get } from "svelte/store";
import type { LauncherAccount } from "../types";
import * as systemApi from "$lib";

/**
 * Modern Authentication Service with Authorization Code Flow
 * Auto-authenticates on startup and provides seamless account management
 */

// TODO:
// TODO: The normal Code Flow does not really seem to work / do anything
// TODO:

export class AuthService {
  /**
   * Refresh the available accounts list (for compatibility with old AuthManager usage)
   */
  static async refreshAvailableAccounts(): Promise<void> {
    try {
      const accounts = await this.getAllAccounts();
      availableAccounts.set(accounts);
      console.log("🔄 Refreshed available accounts list");
    } catch (error) {
      console.error("❌ Failed to refresh accounts list:", error);
      availableAccounts.set([]);
    }
  }
  // These are assigned after the class for Svelte compatibility
  static signIn = async () => await this.authenticateWithDeviceCode();
  static signInWithDeviceCode = async () =>
    await this.authenticateWithDeviceCode();
  private static oauthWindow: WebviewWindow | null = null;
  private static refreshTimer: ReturnType<typeof setInterval> | null = null;
  private static isInitialized = false;
  // Removed: private static currentAccount

  /**
   * Initialize authentication service and try to auto-authenticate
   * Call this on app startup
   */
  /**
   * Initialize authentication service and background refresh
   */
  static async initialize(): Promise<LauncherAccount | null> {
    if (this.isInitialized) {
      return get(currentAccount);
    }
    console.log("🔐 Initializing authentication service...");
    this.isInitialized = true;
    try {
      // Try to get existing account with valid token
      const account = await authApi.getLaunchAuthAccount();
      currentAccount.set(account);
      // Load all available accounts
      await this.refreshAvailableAccounts();
      // Start background refresh
      this.initializeBackgroundRefresh();
      console.log(
        "✅ Auto-authenticated with existing account:",
        account?.username,
      );
      return account;
    } catch (error) {
      currentAccount.set(null);
      this.initializeBackgroundRefresh();
      console.log("ℹ️ No valid account found, user will need to sign in");
      return null;
    }
  }

  /**
   * Start background refresh timer (checks every 5 minutes)
   */
  private static initializeBackgroundRefresh(): void {
    if (this.refreshTimer) {
      clearInterval(this.refreshTimer);
      this.refreshTimer = null;
    }
    // Check every 5 minutes
    this.refreshTimer = setInterval(
      () => {
        this.startBackgroundRefresh();
      },
      5 * 60 * 1000,
    );
  }

  /**
   * Refresh token if expiring soon (no sign out or prompt)
   */
  private static async startBackgroundRefresh(): Promise<void> {
    const account = get(currentAccount);
    if (!account) return;
    if (!account.access_token_expires_at) return;
    // Check for encrypted_refresh_token before attempting refresh
    if (!account.encrypted_refresh_token) {
      console.warn(
        "⚠️ Cannot auto-refresh token: encrypted_refresh_token is missing",
      );
      return;
    }
    const expiresAt = new Date(account.access_token_expires_at);
    const now = new Date();
    // If token expires in less than 10 minutes, refresh
    if (expiresAt.getTime() - now.getTime() < 10 * 60 * 1000) {
      try {
        const refreshed = await authApi.refreshMicrosoftToken(account.local_id);
        currentAccount.set(refreshed);
        await this.refreshAvailableAccounts();
        console.log("🔄 Token auto-refreshed in background");
      } catch (error) {
        console.error("❌ Background token refresh failed:", error);
      }
    }
  }

  /**
   * Refresh tokens for all available accounts if they're expired or expiring soon
   * This is called during initialization to ensure all accounts are ready to use
   */
  static async refreshAllAccountTokens(): Promise<void> {
    try {
      const accounts = await this.getAllAccounts();
      if (!accounts || accounts.length === 0) return;

      console.log(`🔄 Checking tokens for ${accounts.length} account(s)...`);

      // Use Promise.allSettled to handle partial failures
      const results = await Promise.allSettled(
        accounts.map(async (account) => {
          // Skip if no expiry time
          if (!account.access_token_expires_at) return;

          // Skip if no refresh token available
          if (!account.encrypted_refresh_token) {
            console.warn(
              `⚠️ Cannot refresh ${account.username}: no refresh token`,
            );
            return;
          }

          const expiresAt = new Date(account.access_token_expires_at);
          const now = new Date();
          const timeUntilExpiry = expiresAt.getTime() - now.getTime();

          // If token is expired or will expire soon (within 10 minutes), refresh it
          if (timeUntilExpiry < 10 * 60 * 1000) {
            console.log(`🔄 Refreshing token for ${account.username}...`);
            try {
              await authApi.refreshMicrosoftToken(account.local_id);
              console.log(`✅ Token refreshed for ${account.username}`);
            } catch (error) {
              console.error(
                `❌ Failed to refresh token for ${account.username}:`,
                error,
              );
              throw error;
            }
          }
        }),
      );

      // Log summary
      const successful = results.filter((r) => r.status === "fulfilled").length;
      const failed = results.filter((r) => r.status === "rejected").length;

      if (failed > 0) {
        console.warn(
          `⚠️ Token refresh completed: ${successful} successful, ${failed} failed`,
        );
      } else if (successful > 0) {
        console.log(
          `✅ All tokens refreshed successfully (${successful} account(s))`,
        );
      }

      // Only refresh the available accounts list if we actually refreshed any tokens
      if (successful > 0 || failed > 0) {
        await this.refreshAvailableAccounts();
      }
    } catch (error) {
      console.error("❌ Error during bulk token refresh:", error);
    }
  }

  /**
   * Start Device Code Flow authentication (fallback method)
   * Returns device code data for UI display, then polls in background
   */
  static async startDeviceCodeFlow(): Promise<authApi.DeviceCodeResponse> {
    console.log("📱 Starting Device Code Flow...");

    const deviceResponse = await authApi.startMicrosoftDeviceAuth();
    console.log("📝 Device code generated:", deviceResponse.user_code);

    // Don't auto-open - let UI show instructions first
    // await systemApi.openUrl(deviceResponse.verification_uri);

    return deviceResponse;
  }

  /**
   * Poll for device code completion
   */
  static async pollDeviceCodeCompletion(
    deviceCode: string,
  ): Promise<LauncherAccount> {
    return new Promise(async (resolve, reject) => {
      const pollForCompletion = async () => {
        try {
          const token = await authApi.pollMicrosoftDeviceAuth(deviceCode);
          if (token) {
            console.log("✅ Device code authentication successful!");
            const account = await authApi.completeMicrosoftAuth(token);
            currentAccount.set(account);
            await this.refreshAvailableAccounts();
            resolve(account);
          } else {
            setTimeout(pollForCompletion, 5000);
          }
        } catch (error) {
          console.error("❌ Device code polling failed:", error);
          currentAccount.set(null);
          reject(error);
        }
      };
      setTimeout(pollForCompletion, 2000);
    });
  }

  /**
   * Legacy method - Start Device Code Flow authentication (fallback method)
   * This requires users to manually enter a code
   */
  static async authenticateWithDeviceCode(): Promise<LauncherAccount> {
    isAuthenticating.set(true);
    return new Promise(async (resolve, reject) => {
      try {
        console.log("📱 Starting Device Code Flow...");
        const deviceResponse = await authApi.startMicrosoftDeviceAuth();
        console.log("📝 Device code generated:", deviceResponse.user_code);
        await systemApi.openUrl(deviceResponse.verification_uri);
        const pollForCompletion = async () => {
          try {
            const token = await authApi.pollMicrosoftDeviceAuth(
              deviceResponse.device_code,
            );
            if (token) {
              console.log("✅ Device code authentication successful!");
              const account = await authApi.completeMicrosoftAuth(token);
              currentAccount.set(account);
              await this.refreshAvailableAccounts();
              isAuthenticating.set(false);
              resolve(account);
            } else {
              setTimeout(pollForCompletion, deviceResponse.interval * 1000);
            }
          } catch (error) {
            console.error("❌ Device code polling failed:", error);
            isAuthenticating.set(false);
            currentAccount.set(null);
            reject(error);
          }
        };
        setTimeout(pollForCompletion, deviceResponse.interval * 1000);
        return {
          userCode: deviceResponse.user_code,
          verificationUri: deviceResponse.verification_uri,
          expiresIn: deviceResponse.expires_in,
        } as any;
      } catch (error) {
        console.error("❌ Device code authentication failed:", error);
        isAuthenticating.set(false);
        currentAccount.set(null);
        reject(error);
      }
    });
  }

  /**
   * Refresh current account token
   */
  /**
   * Manual refresh for current account (used by AccountManager refresh button)
   */
  static async refreshCurrentAccount(): Promise<LauncherAccount | null> {
    const account = get(currentAccount);
    if (!account) {
      console.warn("⚠️ No account to refresh");
      return null;
    }
    if (!account.encrypted_refresh_token) {
      console.warn(
        "⚠️ Cannot refresh token: encrypted_refresh_token is missing",
      );
      return null;
    }
    try {
      console.log("🔄 Manually refreshing current account token...");
      const refreshed = await authApi.refreshMicrosoftToken(account.local_id);
      currentAccount.set(refreshed);
      // Note: refreshAvailableAccounts() will be called by refreshAllAccountTokens() in NavBar
      console.log("✅ Token manually refreshed");
      return refreshed;
    } catch (error) {
      console.error("❌ Manual token refresh failed:", error);
      return null;
    }
  }

  /**
   * Get current authenticated account
   */
  static getCurrentAccount(): LauncherAccount | null {
    return get(currentAccount);
  }

  /**
   * Check if current account has a valid token
   */
  static isCurrentAccountValid(): boolean {
    const account = get(currentAccount);
    if (!account) {
      return false;
    }
    // Check if we have an access token
    if (!account.access_token) {
      return false;
    }
    // Check expiry if available
    if (account.access_token_expires_at) {
      const expiresAt = new Date(account.access_token_expires_at);
      const now = new Date();
      const fiveMinutesFromNow = new Date(now.getTime() + 5 * 60 * 1000);
      return expiresAt > fiveMinutesFromNow;
    }
    // If no expiry info, assume valid (will be validated when actually used)
    return true;
  }

  /**
   * Sign out current account
   */
  static async signOut(): Promise<void> {
    currentAccount.set(null);
    await this.refreshAvailableAccounts();
    console.log("✅ Signed out successfully");
  }

  /**
   * Clean up authentication resources
   */
  private static async cleanup(): Promise<void> {
    if (this.refreshTimer) {
      clearInterval(this.refreshTimer);
      this.refreshTimer = null;
    }

    if (this.oauthWindow) {
      try {
        await this.oauthWindow.close();
      } catch (error) {
        console.warn("Failed to close OAuth window:", error);
      }
      this.oauthWindow = null;
    }
  }

  /**
   * Force cleanup of authentication resources
   */
  static async forceCleanup(): Promise<void> {
    await this.cleanup();
  }

  /**
   * Account Management Functions
   */
  static async getAllAccounts(): Promise<LauncherAccount[]> {
    try {
      const allAccounts = await authApi.getAllLauncherAccounts();
      console.log("� Final accounts:", allAccounts.length);
      return allAccounts;
    } catch (error) {
      console.error("❌ Failed to get all accounts:", error);
      return [];
    }
  }

  static async switchAccount(accountId: string): Promise<LauncherAccount> {
    try {
      // First, get the account to check if token needs refresh
      const accounts = await this.getAllAccounts();
      const targetAccount = accounts.find((acc) => acc.local_id === accountId);

      if (!targetAccount) {
        throw new Error("Account not found");
      }

      // Check if token is expired or will expire soon (within 10 minutes)
      if (targetAccount.access_token_expires_at) {
        const expiresAt = new Date(targetAccount.access_token_expires_at);
        const now = new Date();
        const timeUntilExpiry = expiresAt.getTime() - now.getTime();

        // If token is expired or will expire soon, refresh it first
        if (timeUntilExpiry < 10 * 60 * 1000) {
          console.log(
            "🔄 Token expired or expiring soon, refreshing before switch...",
          );
          try {
            await authApi.refreshMicrosoftToken(accountId);
            console.log("✅ Token refreshed successfully");
          } catch (error) {
            console.error("❌ Failed to refresh token before switch:", error);
            // Continue with switch anyway - backend will handle it
          }
        }
      }

      await authApi.setActiveLauncherAccount(accountId);
      const account = await authApi.getMinecraftAccount();
      currentAccount.set(account);
      await this.refreshAvailableAccounts();
      console.log("✅ Switched to account:", account.username);
      return account;
    } catch (error) {
      console.error("❌ Failed to switch account:", error);
      currentAccount.set(null);
      throw error;
    }
  }

  static async removeAccount(accountId: string): Promise<void> {
    try {
      await authApi.removeLauncherAccount(accountId);
      const account = get(currentAccount);
      if (account && account.local_id === accountId) {
        currentAccount.set(null);
      }
      await this.refreshAvailableAccounts();
      console.log("✅ Account removed successfully");
    } catch (error) {
      console.error("❌ Failed to remove account:", error);
      throw error;
    }
  }

  static formatTokenExpiry(account: LauncherAccount): string {
    if (!account.access_token_expires_at) {
      return "Unknown";
    }
    const expiresAt = new Date(account.access_token_expires_at);
    const now = new Date();
    const diffMs = expiresAt.getTime() - now.getTime();
    if (diffMs <= 0) {
      return "Expired";
    }
    const hours = Math.floor(diffMs / (1000 * 60 * 60));
    const minutes = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60));
    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    } else {
      return `${minutes}m`;
    }
  }
}

//  Aliases for compatibility with old AuthManager usage in Svelte components
// Should be removed or put into static methods instead!
export const signIn = AuthService.authenticateWithDeviceCode;
export const signInWithDeviceCode = AuthService.authenticateWithDeviceCode;
export const refreshAvailableAccounts = AuthService.getAllAccounts;
export const getCurrentAccount = AuthService.getCurrentAccount;
export const isCurrentAccountValid = AuthService.isCurrentAccountValid;
export const signOut = AuthService.signOut;
export const getAllAccounts = AuthService.getAllAccounts;
export const switchAccount = AuthService.switchAccount;
export const removeAccount = AuthService.removeAccount;
export const formatTokenExpiry = AuthService.formatTokenExpiry;
export const refreshCurrentAccount = AuthService.refreshCurrentAccount;
export const startDeviceCodeFlow = AuthService.startDeviceCodeFlow;
export const pollDeviceCodeCompletion = AuthService.pollDeviceCodeCompletion;
