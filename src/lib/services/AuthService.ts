import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import * as authApi from '$lib';
import type { LauncherAccount } from '$lib';
import * as systemApi from '$lib';

/**
 * Modern Authentication Service with Authorization Code Flow
 * Auto-authenticates on startup and provides seamless account management
 */

export class AuthService {
  private static oauthWindow: WebviewWindow | null = null;
  private static pollInterval: number | null = null;
  private static isInitialized = false;
  private static currentAccount: LauncherAccount | null = null;

  /**
   * Initialize authentication service and try to auto-authenticate
   * Call this on app startup
   */
  static async initialize(): Promise<LauncherAccount | null> {
    if (this.isInitialized) {
      return this.currentAccount;
    }

    console.log('🔐 Initializing authentication service...');
    this.isInitialized = true;

    try {
      // Try to get existing account with valid token
      const account = await authApi.getLaunchAuthAccount();
      this.currentAccount = account;
      console.log('✅ Auto-authenticated with existing account:', account.username);
      return account;
    } catch (error) {
      console.log('ℹ️ No valid account found, user will need to sign in');
      return null;
    }
  }

  /**
   * Start Microsoft Authorization Code Flow authentication
   * This is the recommended method for desktop apps (no manual code entry)
   */
  static async authenticateWithMicrosoft(): Promise<LauncherAccount> {
    return new Promise(async (resolve, reject) => {
      try {
        // Clean up any existing authentication attempt
        await this.cleanup();

        console.log('🔐 Starting Microsoft Authorization Code Flow...');
        
        // Step 1: Start the authorization code flow
        const authResponse = await authApi.startMicrosoftAuthCode();
        console.log('🌐 Authorization server started on port:', authResponse.local_server_port);

        // Step 2: Open authentication window with the auth URL
        this.oauthWindow = new WebviewWindow('microsoft-oauth', {
          url: authResponse.auth_url,
          title: 'Sign in to Microsoft',
          width: 480,
          height: 640,
          center: true,
          resizable: false,
          minimizable: false,
          maximizable: false,
          skipTaskbar: true,
          alwaysOnTop: true,
          decorations: true
        });

        // Handle window events
        this.oauthWindow.onCloseRequested(() => {
          this.cleanup();
          reject(new Error('Authentication cancelled by user'));
        });

        console.log('✅ Authentication successful! The authorization code flow completed automatically.');
        
        // Step 3: Wait a moment for the callback to complete, then get the account
        setTimeout(async () => {
          try {
            const account = await authApi.getMinecraftAccount('AuthCodeFlow');
            this.currentAccount = account;
            
            // Clean up
            await this.cleanup();
            resolve(account);
          } catch (error) {
            console.error('❌ Failed to get authenticated account:', error);
            await this.cleanup();
            reject(error);
          }
        }, 2000); // Give the callback handler time to save the account

        // Timeout after 5 minutes
        setTimeout(async () => {
          await this.cleanup();
          reject(new Error('Authentication timeout - please try again'));
        }, 300000);

      } catch (error) {
        console.error('❌ Authentication setup failed:', error);
        await this.cleanup();
        reject(error);
      }
    });
  }

  /**
   * Start Device Code Flow authentication (fallback method)
   * Returns device code data for UI display, then polls in background
   */
  static async startDeviceCodeFlow(): Promise<authApi.DeviceCodeResponse> {
    console.log('📱 Starting Device Code Flow...');
    
    const deviceResponse = await authApi.startMicrosoftDeviceAuth();
    console.log('📝 Device code generated:', deviceResponse.user_code);
    
    // Open verification URL automatically
    await systemApi.openUrl(deviceResponse.verification_uri);
    
    return deviceResponse;
  }

  /**
   * Poll for device code completion
   */
  static async pollDeviceCodeCompletion(deviceCode: string): Promise<LauncherAccount> {
    return new Promise(async (resolve, reject) => {
      const pollForCompletion = async () => {
        try {
          const token = await authApi.pollMicrosoftDeviceAuth(deviceCode);
          
          if (token) {
            console.log('✅ Device code authentication successful!');
            // completeMicrosoftAuth returns LauncherAccount, convert to MinecraftAccount
            const account = await authApi.completeMicrosoftAuth(token);
            this.currentAccount = account;
            resolve(account);
          } else {
            // Continue polling
            setTimeout(pollForCompletion, 5000); // Poll every 5 seconds
          }
        } catch (error) {
          console.error('❌ Device code polling failed:', error);
          reject(error);
        }
      };
      
      // Start polling after a short delay
      setTimeout(pollForCompletion, 2000);
    });
  }

  /**
   * Legacy method - Start Device Code Flow authentication (fallback method)
   * This requires users to manually enter a code
   */
  static async authenticateWithDeviceCode(): Promise<LauncherAccount> {
    return new Promise(async (resolve, reject) => {
      try {
        console.log('📱 Starting Device Code Flow...');
        
        const deviceResponse = await authApi.startMicrosoftDeviceAuth();
        console.log('📝 Device code generated:', deviceResponse.user_code);
        
        // Open verification URL
        await systemApi.openUrl(deviceResponse.verification_uri);
        
        // Start polling for completion
        const pollForCompletion = async () => {
          try {
            const token = await authApi.pollMicrosoftDeviceAuth(deviceResponse.device_code);
            
            if (token) {
              console.log('✅ Device code authentication successful!');
              // completeMicrosoftAuth returns LauncherAccount, convert to MinecraftAccount
              const account = await authApi.completeMicrosoftAuth(token);
              this.currentAccount = account;
              resolve(account);
            } else {
              // Continue polling
              setTimeout(pollForCompletion, deviceResponse.interval * 1000);
            }
          } catch (error) {
            console.error('❌ Device code polling failed:', error);
            reject(error);
          }
        };
        
        // Start polling
        setTimeout(pollForCompletion, deviceResponse.interval * 1000);
        
        // Return device code info for UI display
        return {
          userCode: deviceResponse.user_code,
          verificationUri: deviceResponse.verification_uri,
          expiresIn: deviceResponse.expires_in
        } as any; // This will be replaced by the actual account when polling completes
        
      } catch (error) {
        console.error('❌ Device code authentication failed:', error);
        reject(error);
      }
    });
  }

  /**
   * Refresh current account token
   */
  static async refreshCurrentAccount(): Promise<LauncherAccount> {
    try {
      console.log('🔄 Refreshing current account token...');
      const refreshedAccount = await authApi.refreshMinecraftAccount();
      this.currentAccount = refreshedAccount;
      console.log('✅ Token refreshed successfully');
      return refreshedAccount;
    } catch (error) {
      console.error('❌ Token refresh failed:', error);
      // If refresh fails, user needs to re-authenticate
      this.currentAccount = null;
      throw new Error(`Failed to refresh token: ${error}`);
    }
  }

  /**
   * Get current authenticated account
   */
  static getCurrentAccount(): LauncherAccount | null {
    return this.currentAccount;
  }

  /**
   * Check if current account has a valid token
   */
  static isCurrentAccountValid(): boolean {
    if (!this.currentAccount) {
      return false;
    }
    // Check if we have an access token
    if (!this.currentAccount.access_token) {
      return false;
    }
    // Check expiry if available
    if (this.currentAccount.access_token_expires_at) {
      const expiresAt = new Date(this.currentAccount.access_token_expires_at);
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
    this.currentAccount = null;
    console.log('✅ Signed out successfully');
  }

  /**
   * Clean up authentication resources
   */
  private static async cleanup(): Promise<void> {
    if (this.pollInterval) {
      clearInterval(this.pollInterval);
      this.pollInterval = null;
    }

    if (this.oauthWindow) {
      try {
        await this.oauthWindow.close();
      } catch (error) {
        console.warn('Failed to close OAuth window:', error);
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
      console.log('� Final accounts:', allAccounts.length);
      return allAccounts;
    } catch (error) {
      console.error('❌ Failed to get all accounts:', error);
      return [];
    }
  }

  static async switchAccount(accountId: string): Promise<LauncherAccount> {
    try {
      await authApi.setActiveLauncherAccount(accountId);
      const account = await authApi.getMinecraftAccount();
      this.currentAccount = account;
      console.log('✅ Switched to account:', account.username);
      return account;
    } catch (error) {
      console.error('❌ Failed to switch account:', error);
      throw error;
    }
  }

  static async removeAccount(accountId: string): Promise<void> {
    try {
      await authApi.removeLauncherAccount(accountId);
      // If we removed the current account, clear it
      if (this.currentAccount && this.currentAccount.local_id === accountId) {
        this.currentAccount = null;
      }
      console.log('✅ Account removed successfully');
    } catch (error) {
      console.error('❌ Failed to remove account:', error);
      throw error;
    }
  }

  /**
   * Utility functions
   */
  static async copyToClipboard(text: string): Promise<void> {
    return await systemApi.copyToClipboard(text);
  }

  static formatTokenExpiry(account: LauncherAccount): string {
    if (!account.access_token_expires_at) {
      return 'Unknown';
    }
    const expiresAt = new Date(account.access_token_expires_at);
    const now = new Date();
    const diffMs = expiresAt.getTime() - now.getTime();
    if (diffMs <= 0) {
      return 'Expired';
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
