import { get } from 'svelte/store';
import { AuthService } from '../services/AuthService';
import * as authApi from '../api/auth';
import { 
  currentAccount, 
  isAuthenticated, 
  isAuthenticating, 
  availableAccounts 
} from '../stores/auth';
import type { MicrosoftAccount } from '../types';

/**
 * Authentication Manager
 * Coordinates authentication state between stores, services, and API
 */

export class AuthManager {
  /**
   * Get the current active account
   */
  static getCurrentAccount(): MicrosoftAccount | null {
    return get(currentAccount);
  }

  /**
   * Initialize authentication - load from storage if available
   */
  static async initialize(): Promise<void> {
    try {
      const stored = await authApi.getActiveLauncherAccount();
      if (stored) {
        // Try to refresh token if it's close to expiry
        if (AuthService.needsRefresh(stored)) {
          try {
            const refreshed = await this.refreshToken(stored.id);
            currentAccount.set(refreshed);
            await this.saveAccountToStorage(refreshed);
          } catch (error) {
            console.warn('Failed to refresh token on init:', error);
            currentAccount.set(stored); // Use stored account anyway
          }
        } else {
          currentAccount.set(stored);
        }
      }
      
      // Load all available accounts
      await this.refreshAccountsList();
    } catch (error) {
      console.error('Auth initialization failed:', error);
    }
  }

  /**
   * Sign in with Microsoft account
   */
  static async signIn(): Promise<MicrosoftAccount> {
    try {
      isAuthenticating.set(true);
      console.log('🔐 Starting Microsoft authentication...');
      
      const account = await AuthService.authenticateWithMicrosoft();
      
      // Save to storage and update state
      await this.saveAccountToStorage(account);
      await authApi.setActiveLauncherAccount(account.id);
      currentAccount.set(account);
      
      console.log('✅ Successfully authenticated:', account.username);
      return account;
    } catch (error) {
      console.error('❌ Authentication failed:', error);
      throw error;
    } finally {
      isAuthenticating.set(false);
    }
  }

  /**
   * Refresh account token
   */
  static async refreshToken(accountId?: string): Promise<MicrosoftAccount> {
    const targetAccountId = accountId || get(currentAccount)?.id;
    if (!targetAccountId) {
      throw new Error('No account to refresh');
    }

    try {
      console.log('🔄 Refreshing token for account:', targetAccountId);
      const refreshedAccount = await AuthService.refreshAccountToken(targetAccountId);
      
      // Update storage and state if this is the current account
      const current = get(currentAccount);
      if (current && current.id === targetAccountId) {
        await this.saveAccountToStorage(refreshedAccount);
        currentAccount.set(refreshedAccount);
      }
      
      await this.refreshAccountsList();
      console.log('✅ Token refreshed successfully');
      return refreshedAccount;
    } catch (error) {
      console.error('❌ Token refresh failed:', error);
      throw error;
    }
  }

  /**
   * Ensure current account has valid token
   */
  static async ensureValidToken(): Promise<MicrosoftAccount | null> {
    const account = get(currentAccount);
    if (!account) {
      return null;
    }

    try {
      if (AuthService.needsRefresh(account)) {
        console.log('🔄 Token needs refresh, refreshing...');
        return await this.refreshToken(account.id);
      } else if (!AuthService.isTokenValid(account)) {
        console.warn('⚠️ Token is invalid, signing out...');
        await this.signOut();
        return null;
      }
      
      return account;
    } catch (error) {
      console.error('❌ Token validation failed:', error);
      await this.signOut();
      return null;
    }
  }

  /**
   * Sign out current account
   */
  static async signOut(): Promise<void> {
    try {
      const account = get(currentAccount);
      if (account) {
        await authApi.removeLauncherAccount(account.id);
      }
      
      currentAccount.set(null);
      await this.refreshAccountsList();
      console.log('👋 Signed out successfully');
    } catch (error) {
      console.error('❌ Sign out failed:', error);
    }
  }

  /**
   * Sign out specific account
   */
  static async signOutAccount(accountId: string): Promise<void> {
    try {
      await authApi.removeLauncherAccount(accountId);
      
      // If this was the current account, clear it
      const current = get(currentAccount);
      if (current && current.id === accountId) {
        currentAccount.set(null);
      }
      
      await this.refreshAccountsList();
      console.log('👋 Account removed:', accountId);
    } catch (error) {
      console.error('❌ Account removal failed:', error);
    }
  }

  /**
   * Switch to different account
   */
  static async switchToAccount(accountId: string): Promise<void> {
    try {
      await authApi.setActiveLauncherAccount(accountId);
      const account = await authApi.getActiveLauncherAccount();
      
      if (account) {
        // Ensure token is valid
        if (AuthService.needsRefresh(account)) {
          const refreshed = await this.refreshToken(account.id);
          currentAccount.set(refreshed);
        } else {
          currentAccount.set(account);
        }
      }
      
      console.log('🔄 Switched to account:', accountId);
    } catch (error) {
      console.error('❌ Account switch failed:', error);
      throw error;
    }
  }

  /**
   * Save account to storage
   */
  /**
   * Save account to storage (made public for external use)
   */
  static async saveAccountToStorage(account: MicrosoftAccount): Promise<void> {
    try {
      await authApi.writeLauncherAccount(account);
      await this.refreshAccountsList();
    } catch (error) {
      console.error('Failed to save account to storage:', error);
      throw error;
    }
  }

  /**
   * Refresh list of available accounts
   */
  private static async refreshAccountsList(): Promise<void> {
    try {
      const accounts = await authApi.getAllLauncherAccounts();
      availableAccounts.set(accounts);
    } catch (error) {
      console.error('Failed to refresh accounts list:', error);
    }
  }
}
