import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import type { MicrosoftAccount } from '../types';
import * as authApi from '../api/auth';
import * as systemApi from '../api/system';

/**
 * Enhanced Authentication Service with proper OAuth2 flow
 * Handles Microsoft account authentication for Minecraft with advanced window management
 */

export class AuthService {
  private static oauthWindow: WebviewWindow | null = null;
  private static pollInterval: number | null = null;

  /**
   * Start Microsoft OAuth2 authentication flow
   * Opens authentication window and handles the complete flow
   */
  static async authenticateWithMicrosoft(): Promise<MicrosoftAccount> {
    return new Promise(async (resolve, reject) => {
      try {
        // Clean up any existing authentication attempt
        await this.cleanup();

        // Step 1: Start the OAuth server and get auth URL
        const authUrl = await authApi.startMicrosoftAuth();
        console.log('🔐 Starting Microsoft authentication...');

        // Step 2: Open authentication window
        this.oauthWindow = new WebviewWindow('microsoft-oauth', {
          url: authUrl,
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

        // Step 3: Poll for authentication result
        this.pollInterval = window.setInterval(async () => {
          try {
            const authCode = await authApi.getOAuthCallbackResult();
            
            if (authCode) {
              console.log('✅ Authentication code received, completing flow...');
              
              // Clear polling
              if (this.pollInterval) {
                clearInterval(this.pollInterval);
                this.pollInterval = null;
              }
              
              // Complete authentication
              const account = await authApi.completeMicrosoftAuth(authCode);
              console.log('✅ Authentication successful!', account.username);
              
              // Clean up and resolve
              await this.cleanup();
              resolve(account);
            }
          } catch (error) {
            console.error('❌ Authentication polling error:', error);
            await this.cleanup();
            reject(error);
          }
        }, 1000);

        // Timeout after 10 minutes
        setTimeout(async () => {
          await this.cleanup();
          reject(new Error('Authentication timeout - please try again'));
        }, 600000);

      } catch (error) {
        console.error('❌ Authentication setup failed:', error);
        await this.cleanup();
        reject(error);
      }
    });
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
   * Browser-based OAuth flow (alternative approach)
   */
  static async authenticateWithMicrosoftBrowser(): Promise<MicrosoftAccount> {
    const authUrl = await authApi.startMicrosoftAuth();
    
    // Open the authentication URL in the system's default browser
    await systemApi.openUrl(authUrl);
    
    return new Promise((resolve, reject) => {
      const checkCallback = async () => {
        try {
          const result = await authApi.getOAuthCallbackResult();
          
          if (result) {
            // Complete the authentication flow
            const account = await authApi.completeMicrosoftAuth(result);
            resolve(account);
          } else {
            // Continue checking
            setTimeout(checkCallback, 1000);
          }
        } catch (error) {
          reject(error);
        }
      };
      
      // Start checking for callback result
      setTimeout(checkCallback, 1000);
    });
  }

  /**
   * Device code authentication flow
   */
  static async startDeviceCodeAuth(): Promise<string> {
    return await authApi.startDeviceCodeAuth();
  }

  static async pollDeviceCodeAuth(): Promise<MicrosoftAccount | null> {
    return await authApi.pollDeviceCodeAuth();
  }

  static async copyToClipboard(text: string): Promise<void> {
    return await systemApi.copyToClipboard(text);
  }

  /**
   * Token validation and refresh logic
   */
  static async refreshAccountToken(accountId: string): Promise<MicrosoftAccount> {
    try {
      console.log('🔄 Refreshing token for account:', accountId);
      const refreshedAccount = await authApi.refreshMinecraftToken(accountId);
      console.log('✅ Token refreshed successfully');
      return refreshedAccount;
    } catch (error) {
      console.error('❌ Token refresh failed:', error);
      throw new Error(`Failed to refresh token: ${error}`);
    }
  }

  static isTokenValid(account: MicrosoftAccount): boolean {
    // First check if we even have a Minecraft access token
    if (!account.minecraft_access_token || account.minecraft_access_token.trim() === '') {
      console.warn('⚠️ No Minecraft access token available for:', account.username);
      return false;
    }
    
    // Then check expiry if we have expiry information
    if (account.minecraft_expires_at) {
      const now = Date.now() / 1000;
      const isValid = account.minecraft_expires_at > now + 300; // 5 minute buffer
      
      if (!isValid) {
        console.warn('⚠️ Token is expired or expiring soon for:', account.username);
      }
      
      return isValid;
    }
    
    // If we have a token but no expiry info, assume it's valid for now
    // The actual validation should happen when launching
    console.log('ℹ️ Token exists but no expiry info available for:', account.username);
    return true;
  }

  static needsRefresh(account: MicrosoftAccount): boolean {
    // First check if we even have a Minecraft access token
    if (!account.minecraft_access_token || account.minecraft_access_token.trim() === '') {
      console.log('🔄 No Minecraft token available, needs refresh for:', account.username);
      return true;
    }
    
    // Then check expiry if we have expiry information
    if (account.minecraft_expires_at) {
      const now = Date.now() / 1000;
      const needsRefresh = account.minecraft_expires_at < now + 600; // 10 minute buffer
      
      if (needsRefresh) {
        console.log('🔄 Token needs refresh due to expiry for:', account.username);
      }
      
      return needsRefresh;
    }
    
    // If we have a token but no expiry info, let's not refresh unless necessary
    console.log('ℹ️ Token exists but no expiry info, will validate when needed for:', account.username);
    return false;
  }

  static getTokenExpiresIn(account: MicrosoftAccount): number {
    const now = Date.now() / 1000;
    return Math.max(0, account.minecraft_expires_at - now);
  }

  static formatTokenExpiry(account: MicrosoftAccount): string {
    const expiresIn = this.getTokenExpiresIn(account);
    
    if (expiresIn <= 0) {
      return 'Expired';
    }
    
    const hours = Math.floor(expiresIn / 3600);
    const minutes = Math.floor((expiresIn % 3600) / 60);
    
    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    } else {
      return `${minutes}m`;
    }
  }

  /**
   * Account management
   */
  static async getActiveLauncherAccount(): Promise<MicrosoftAccount | null> {
    return await authApi.getActiveLauncherAccount();
  }

  static async getAllLauncherAccounts(): Promise<MicrosoftAccount[]> {
    return await authApi.getAllLauncherAccounts();
  }

  static async setActiveLauncherAccount(accountId: string): Promise<void> {
    return await authApi.setActiveLauncherAccount(accountId);
  }

  static async removeLauncherAccount(accountId: string): Promise<void> {
    return await authApi.removeLauncherAccount(accountId);
  }
}
