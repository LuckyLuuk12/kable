import { get } from 'svelte/store';
import { AuthService } from '../services/AuthService';
import { 
  currentAccount, 
  isAuthenticated, 
  isAuthenticating, 
  availableAccounts 
} from '../stores/auth';
import type { LauncherAccount } from '$lib';

/**
 * Authentication Manager
 * Coordinates authentication state between stores, services, and API
 * Now uses modern Authorization Code Flow with auto-authentication
 */

export class AuthManager {
  private static isInitialized = false;

  /**
   * Get the current active account
   */
  static getCurrentAccount(): LauncherAccount | null {
    return get(currentAccount);
  }

  /**
   * Initialize authentication with auto-authentication
   */
  static async initialize(): Promise<void> {
    if (this.isInitialized) {
      console.log('ℹ️ AuthManager: Already initialized, skipping...');
      return;
    }

    console.log('🔐 AuthManager: Initializing with auto-authentication...');
    this.isInitialized = true;

    try {
      // Initialize the AuthService and try auto-authentication
      const account = await AuthService.initialize();
      
      if (account) {
        console.log('✅ AuthManager: Auto-authenticated as', account.username);
        currentAccount.set(account);
        
        // Load all available accounts
        await this.refreshAvailableAccounts();
      } else {
        console.log('ℹ️ AuthManager: No valid account found, user needs to sign in');
      }
    } catch (error) {
      console.error('❌ AuthManager: Initialization error:', error);
    }
  }

  /**
   * Sign in with Microsoft account using Authorization Code Flow
   */
  static async signIn(): Promise<LauncherAccount> {
    try {
      isAuthenticating.set(true);
      console.log('🔐 Starting Microsoft Authorization Code Flow...');
      
      const account = await AuthService.authenticateWithMicrosoft();
      
      // Update state
      currentAccount.set(account);
      
      
      // Refresh available accounts list
      await this.refreshAvailableAccounts();
      
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
   * Sign in with Device Code Flow (fallback method)
   */
  static async signInWithDeviceCode(): Promise<LauncherAccount> {
    try {
      isAuthenticating.set(true);
      console.log('📱 Starting Device Code Flow...');
      
      const account = await AuthService.authenticateWithDeviceCode();
      
      // Update state
      currentAccount.set(account);
      
      
      // Refresh available accounts list
      await this.refreshAvailableAccounts();
      
      console.log('✅ Successfully authenticated with device code:', account.username);
      return account;
    } catch (error) {
      console.error('❌ Device code authentication failed:', error);
      throw error;
    } finally {
      isAuthenticating.set(false);
    }
  }

  /**
   * Refresh current account token
   */
  static async refreshCurrentAccount(): Promise<LauncherAccount> {
    try {
      const account = await AuthService.refreshCurrentAccount();
      currentAccount.set(account);
      console.log('✅ Token refreshed for:', account.username);
      return account;
    } catch (error) {
      console.error('❌ Token refresh failed:', error);
      // If refresh fails, clear current account
      currentAccount.set(null);
      
      throw error;
    }
  }

  /**
   * Switch to a different account
   */
  static async switchAccount(accountId: string): Promise<LauncherAccount> {
    try {
      const account = await AuthService.switchAccount(accountId);
      currentAccount.set(account);
      console.log('✅ Switched to account:', account.username);
      return account;
    } catch (error) {
      console.error('❌ Failed to switch account:', error);
      throw error;
    }
  }

  /**
   * Remove an account
   */
  static async removeAccount(accountId: string): Promise<void> {
    try {
      await AuthService.removeAccount(accountId);
      
      // If we removed the current account, clear it
      const current = get(currentAccount);
      if (current && current.local_id === accountId) {
        currentAccount.set(null);
      }
      
      // Refresh available accounts
      await this.refreshAvailableAccounts();
      
      console.log('✅ Account removed successfully');
    } catch (error) {
      console.error('❌ Failed to remove account:', error);
      throw error;
    }
  }

  /**
   * Sign out current account
   */
  static async signOut(): Promise<void> {
    try {
      await AuthService.signOut();
      currentAccount.set(null);
      
      console.log('✅ Signed out successfully');
    } catch (error) {
      console.error('❌ Sign out failed:', error);
      throw error;
    }
  }

  /**
   * Check if current account is valid
   */
  static isCurrentAccountValid(): boolean {
    return AuthService.isCurrentAccountValid();
  }

  /**
   * Get all available accounts
   */
  static async getAllAccounts(): Promise<LauncherAccount[]> {
    return await AuthService.getAllAccounts();
  }

  /**
   * Refresh the available accounts list
   */
  static async refreshAvailableAccounts(): Promise<void> {
    try {
      const accounts = await this.getAllAccounts();
      availableAccounts.set(accounts);
      console.log('🔄 Refreshed available accounts list');
    } catch (error) {
      console.error('❌ Failed to refresh accounts list:', error);
      availableAccounts.set([]);
    }
  }

  /**
   * Format token expiry for display
   */
  static formatTokenExpiry(account: LauncherAccount): string {
    return AuthService.formatTokenExpiry(account);
  }

  /**
   * Get authentication status
   */
  static getAuthStatus() {
    return {
      isAuthenticated: get(isAuthenticated),
      isAuthenticating: get(isAuthenticating),
      currentAccount: get(currentAccount),
      availableAccounts: get(availableAccounts)
    };
  }

  /**
   * Utility functions
   */
  static async copyToClipboard(text: string): Promise<void> {
    return await AuthService.copyToClipboard(text);
  }
}
