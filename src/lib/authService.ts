import { 
  startMicrosoftAuth, 
  completeMicrosoftAuth, 
  refreshMinecraftToken,
  getOAuthCallbackResult 
} from './services';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import type { MicrosoftAccount } from './types';

/**
 * Enhanced Authentication Service with proper OAuth2 flow
 * Handles Microsoft account authentication for Minecraft
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
        const authUrl = await startMicrosoftAuth();
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

        // Step 3: Poll for OAuth callback result
        let pollAttempts = 0;
        const maxPollAttempts = 600; // 10 minutes (600 seconds)

        const pollForCallback = async () => {
          try {
            pollAttempts++;
            
            // Check if we got a callback result
            const result = await getOAuthCallbackResult();
            
            if (result) {
              console.log('✅ OAuth callback received');
              
              // Complete authentication with the authorization code
              const account = await completeMicrosoftAuth(result);
              console.log(`🎮 Authenticated as: ${account.username}`);
              
              await this.cleanup();
              resolve(account);
              return;
            }

            // Check timeout
            if (pollAttempts >= maxPollAttempts) {
              await this.cleanup();
              reject(new Error('Authentication timeout - please try again'));
              return;
            }

            // Continue polling
            this.pollInterval = window.setTimeout(pollForCallback, 1000);

          } catch (error) {
            console.error('❌ OAuth polling error:', error);
            await this.cleanup();
            reject(error);
          }
        };

        // Start polling
        this.pollInterval = window.setTimeout(pollForCallback, 1000);

      } catch (error) {
        console.error('❌ Authentication failed:', error);
        await this.cleanup();
        reject(error);
      }
    });
  }

  /**
   * Refresh an expired Microsoft account token
   */
  static async refreshAccountToken(accountId: string): Promise<MicrosoftAccount> {
    try {
      console.log('🔄 Refreshing account token...');
      const refreshedAccount = await refreshMinecraftToken(accountId);
      console.log(`✅ Token refreshed for: ${refreshedAccount.username}`);
      return refreshedAccount;
    } catch (error) {
      console.error('❌ Token refresh failed:', error);
      throw new Error(`Failed to refresh token: ${error}`);
    }
  }

  /**
   * Check if an account token is valid or needs refresh
   */
  static isTokenValid(account: MicrosoftAccount): boolean {
    if (!account.expires_at) return false;
    
    const now = Math.floor(Date.now() / 1000);
    const expiresIn = account.expires_at - now;
    
    // Token is valid if it doesn't expire within the next 5 minutes
    return expiresIn > 300;
  }

  /**
   * Check if token needs refresh (expires within 10 minutes)
   */
  static needsRefresh(account: MicrosoftAccount): boolean {
    if (!account.expires_at) return true;
    
    const now = Math.floor(Date.now() / 1000);
    const expiresIn = account.expires_at - now;
    
    // Needs refresh if expires within 10 minutes
    return expiresIn < 600;
  }

  /**
   * Get time until token expires (in seconds)
   */
  static getTokenExpiresIn(account: MicrosoftAccount): number {
    if (!account.expires_at) return 0;
    
    const now = Math.floor(Date.now() / 1000);
    return Math.max(0, account.expires_at - now);
  }

  /**
   * Format token expiry time for display
   */
  static formatTokenExpiry(account: MicrosoftAccount): string {
    const expiresIn = this.getTokenExpiresIn(account);
    
    if (expiresIn <= 0) return 'Expired';
    
    const hours = Math.floor(expiresIn / 3600);
    const minutes = Math.floor((expiresIn % 3600) / 60);
    
    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    } else {
      return `${minutes}m`;
    }
  }

  /**
   * Clean up OAuth resources
   */
  private static async cleanup(): Promise<void> {
    if (this.pollInterval) {
      clearTimeout(this.pollInterval);
      this.pollInterval = null;
    }

    if (this.oauthWindow) {
      try {
        await this.oauthWindow.close();
      } catch (error) {
        // Window might already be closed
      }
      this.oauthWindow = null;
    }
  }

  /**
   * Force cleanup (for emergency cases)
   */
  static async forceCleanup(): Promise<void> {
    await this.cleanup();
  }
}
