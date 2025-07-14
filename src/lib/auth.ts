import { writable, derived, get } from 'svelte/store';
import { AuthService } from './authService';
import type { MicrosoftAccount } from './types';

// Authentication state store
export const currentAccount = writable<MicrosoftAccount | null>(null);
export const isAuthenticated = derived(currentAccount, $account => $account !== null);
export const isAuthenticating = writable(false);

// Derived store for checking if signed in
export const isSignedIn = isAuthenticated;

// Persist account to localStorage
export function saveAccountToStorage(account: MicrosoftAccount) {
  localStorage.setItem('kable_account', JSON.stringify(account));
}

export function loadAccountFromStorage(): MicrosoftAccount | null {
  try {
    const stored = localStorage.getItem('kable_account');
    if (stored) {
      const account = JSON.parse(stored) as MicrosoftAccount;
      // Use AuthService to check if token is valid
      if (AuthService.isTokenValid(account)) {
        return account;
      }
    }
  } catch (error) {
    console.error('Failed to load account from storage:', error);
  }
  return null;
}

export function clearAccountFromStorage() {
  localStorage.removeItem('kable_account');
}

// Authentication methods
export class AuthManager {
  /**
   * Initialize authentication - load from storage if available
   */
  static async initialize(): Promise<void> {
    const stored = loadAccountFromStorage();
    if (stored) {
      // Try to refresh token if it's close to expiry
      if (AuthService.needsRefresh(stored)) {
        try {
          const refreshed = await this.refreshToken(stored.id);
          currentAccount.set(refreshed);
          saveAccountToStorage(refreshed);
        } catch (error) {
          console.warn('Failed to refresh token on init:', error);
          currentAccount.set(stored); // Use stored account anyway
        }
      } else {
        currentAccount.set(stored);
      }
    }
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
      saveAccountToStorage(account);
      
      return account;
    } catch (error) {
      console.error('Authentication failed:', error);
      throw error;
    } finally {
      isAuthenticating.set(false);
    }
  }

  /**
   * Sign out current user
   */
  static async signOut(): Promise<void> {
    currentAccount.set(null);
    clearAccountFromStorage();
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
      saveAccountToStorage(refreshed);
      
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
