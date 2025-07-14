import { writable, derived, get } from 'svelte/store';
import { AuthService } from './authService';
import { 
  getActiveLauncherAccount, 
  getAllLauncherAccounts, 
  writeLauncherAccount, 
  removeLauncherAccount, 
  setActiveLauncherAccount 
} from './services';
import type { MicrosoftAccount } from './types';

// Authentication state store
export const currentAccount = writable<MicrosoftAccount | null>(null);
export const isAuthenticated = derived(currentAccount, $account => $account !== null);
export const isAuthenticating = writable(false);
export const availableAccounts = writable<MicrosoftAccount[]>([]);

// Derived store for checking if signed in
export const isSignedIn = isAuthenticated;

// Persist account to launcher_accounts.json
export async function saveAccountToStorage(account: MicrosoftAccount): Promise<void> {
  try {
    await writeLauncherAccount(account);
    await refreshAccountsList();
  } catch (error) {
    console.error('Failed to save account to launcher_accounts.json:', error);
    throw error;
  }
}

export async function loadAccountFromStorage(): Promise<MicrosoftAccount | null> {
  try {
    const account = await getActiveLauncherAccount();
    if (account && AuthService.isTokenValid(account)) {
      return account;
    }
  } catch (error) {
    console.error('Failed to load account from launcher_accounts.json:', error);
  }
  return null;
}

export async function clearAccountFromStorage(accountId?: string): Promise<void> {
  try {
    if (accountId) {
      await removeLauncherAccount(accountId);
    } else {
      // Clear current account - remove the active one
      const currentAcc = get(currentAccount);
      if (currentAcc) {
        await removeLauncherAccount(currentAcc.uuid);
      }
    }
    await refreshAccountsList();
  } catch (error) {
    console.error('Failed to clear account from launcher_accounts.json:', error);
  }
}

export async function switchAccount(accountId: string): Promise<void> {
  try {
    await setActiveLauncherAccount(accountId);
    const account = await getActiveLauncherAccount();
    currentAccount.set(account);
  } catch (error) {
    console.error('Failed to switch account:', error);
    throw error;
  }
}

export async function refreshAccountsList(): Promise<void> {
  try {
    const accounts = await getAllLauncherAccounts();
    availableAccounts.set(accounts);
  } catch (error) {
    console.error('Failed to refresh accounts list:', error);
  }
}

// Authentication methods
export class AuthManager {
  /**
   * Initialize authentication - load from storage if available
   */
  static async initialize(): Promise<void> {
    const stored = await loadAccountFromStorage();
    if (stored) {
      // Try to refresh token if it's close to expiry
      if (AuthService.needsRefresh(stored)) {
        try {
          const refreshed = await this.refreshToken(stored.id);
          currentAccount.set(refreshed);
          await saveAccountToStorage(refreshed);
        } catch (error) {
          console.warn('Failed to refresh token on init:', error);
          currentAccount.set(stored); // Use stored account anyway
        }
      } else {
        currentAccount.set(stored);
      }
    }
    
    // Also refresh the accounts list
    await refreshAccountsList();
  }

  /**
   * Start Microsoft OAuth flow
   */
  static async signIn(): Promise<MicrosoftAccount> {
    isAuthenticating.set(true);
    
    try {
      const account = await AuthService.authenticateWithMicrosoft();
      
      // Update stores
      currentAccount.set(account);
      await saveAccountToStorage(account);
      
      return account;
    } catch (error) {
      console.error('Authentication failed:', error);
      throw error;
    } finally {
      isAuthenticating.set(false);
    }
  }

  /**
   * Refresh the current account's token
   */
  static async refreshToken(accountId?: string): Promise<MicrosoftAccount> {
    const account = get(currentAccount);
    const id = accountId || account?.id;
    
    if (!id) {
      throw new Error('No account to refresh');
    }

    try {
      const refreshed = await AuthService.refreshAccountToken(id);
      
      // Update stores
      currentAccount.set(refreshed);
      await saveAccountToStorage(refreshed);
      
      return refreshed;
    } catch (error) {
      console.error('Token refresh failed:', error);
      // If refresh fails, sign out the user
      await this.signOut();
      throw error;
    }
  }

  /**
   * Check if current token needs refresh and do it automatically
   */
  static async ensureValidToken(): Promise<MicrosoftAccount | null> {
    const account = get(currentAccount);
    if (!account) return null;

    // Use AuthService to check if token needs refresh
    if (AuthService.needsRefresh(account)) {
      try {
        return await this.refreshToken(account.id);
      } catch (error) {
        console.error('Auto token refresh failed:', error);
        return null;
      }
    }

    return account;
  }

  /**
   * Get current account information
   */
  static getCurrentAccount(): MicrosoftAccount | null {
    return get(currentAccount);
  }

  /**
   * Check if user is currently authenticated
   */
  static isSignedIn(): boolean {
    return get(isAuthenticated);
  }

  /**
   * Sign out current user
   */
  static async signOut(): Promise<void> {
    const currentAcc = get(currentAccount);
    currentAccount.set(null);
    if (currentAcc) {
      await clearAccountFromStorage(currentAcc.uuid);
    }
  }

  /**
   * Sign out specific user by account ID
   */
  static async signOutAccount(accountId: string): Promise<void> {
    await clearAccountFromStorage(accountId);
    // If this was the current account, clear it
    const currentAcc = get(currentAccount);
    if (currentAcc && currentAcc.uuid === accountId) {
      currentAccount.set(null);
    }
  }

  /**
   * Switch to a different account
   */
  static async switchToAccount(accountId: string): Promise<void> {
    await switchAccount(accountId);
  }

  /**
   * Get all available accounts
   */
  static getAvailableAccounts(): MicrosoftAccount[] {
    return get(availableAccounts);
  }
}

// Auto-refresh token periodically (every 30 minutes)
if (typeof window !== 'undefined') {
  setInterval(async () => {
    try {
      await AuthManager.ensureValidToken();
    } catch (error) {
      console.error('Periodic token refresh failed:', error);
    }
  }, 30 * 60 * 1000); // 30 minutes
}
