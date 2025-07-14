import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import type { 
  MinecraftInstallation, 
  LaunchOptions, 
  MicrosoftAccount, 
  LauncherSettings, 
  MinecraftDirectoryInfo,
  LauncherProfiles,
  MinecraftSession
} from './types';

// Installations Management
export async function getMinecraftInstallations(): Promise<MinecraftInstallation[]> {
  return await invoke('get_minecraft_installations');
}

export async function refreshInstallation(id: string): Promise<MinecraftInstallation> {
  return await invoke('refresh_installation', { id });
}

export async function updateInstallationLastPlayed(id: string): Promise<void> {
  return await invoke('update_installation_last_played', { id });
}

// Settings Management
export async function loadSettings(): Promise<LauncherSettings> {
  return await invoke('load_settings');
}

export async function saveSettings(settings: LauncherSettings): Promise<void> {
  return await invoke('save_settings', { settings });
}

export async function getLauncherDir(): Promise<string> {
  return await invoke('get_launcher_dir');
}

// Microsoft Authentication with proper desktop OAuth flow
export async function startMicrosoftAuth(): Promise<string> {
  return await invoke('start_microsoft_auth');
}

export async function completeMicrosoftAuth(authCode: string): Promise<MicrosoftAccount> {
  return await invoke('complete_microsoft_auth', { authCode });
}

// Device Code Flow for public clients
export async function startDeviceCodeAuth(): Promise<string> {
  return await invoke('start_device_code_auth');
}

export async function pollDeviceCodeAuth(): Promise<MicrosoftAccount | null> {
  return await invoke('poll_device_code_auth');
}

export async function copyToClipboard(text: string): Promise<void> {
  return await invoke('copy_to_clipboard', { text });
}

export async function refreshMinecraftToken(accountId: string): Promise<MicrosoftAccount> {
  return await invoke('refresh_minecraft_token', { accountId });
}

export async function getOAuthCallbackResult(): Promise<string | null> {
  return await invoke('get_oauth_callback_result');
}

// Minecraft Management
export async function getCachedUsernames(minecraftPath: string): Promise<string[]> {
  return await invoke('get_cached_usernames', { minecraftPath });
}

export async function launchMinecraft(options: LaunchOptions, minecraftPath: string): Promise<string> {
  return await invoke('launch_minecraft', { options, minecraftPath });
}

export async function checkJavaInstallation(): Promise<string> {
  return await invoke('check_java_installation');
}

export async function getDefaultMinecraftDir(): Promise<string> {
  return await invoke('get_default_minecraft_dir');
}

// Add missing validation function
export async function validateMinecraftDirectory(path: string): Promise<MinecraftDirectoryInfo> {
  return await invoke('validate_minecraft_directory', { path });
}

// Proper OAuth flow for desktop applications using embedded webview
export async function authenticateWithMicrosoft(): Promise<MicrosoftAccount> {
  return new Promise(async (resolve, reject) => {
    try {
      // Step 1: Start the callback server and get the auth URL
      const authUrl = await startMicrosoftAuth();
      
      // Step 2: Create a webview window for authentication
      const authWindow = new WebviewWindow('oauth-window', {
        url: authUrl,
        title: 'Microsoft Authentication',
        width: 500,
        height: 700,
        center: true,
        resizable: false,
        minimizable: false,
        maximizable: false,
        skipTaskbar: true,
        alwaysOnTop: true
      });

      // Handle window close
      authWindow.onCloseRequested(() => {
        reject(new Error('Authentication window was closed'));
      });

      // Step 3: Poll for the callback result
      const pollForResult = async () => {
        try {
          const result = await getOAuthCallbackResult();
          if (result) {
            // Got authorization code, complete the authentication
            const account = await completeMicrosoftAuth(result);
            await authWindow.close();
            resolve(account);
          } else {
            // Keep polling every second
            setTimeout(pollForResult, 1000);
          }
        } catch (error) {
          await authWindow.close();
          reject(error);
        }
      };

      // Start polling for the result
      pollForResult();

      // Timeout after 10 minutes
      setTimeout(async () => {
        try {
          await authWindow.close();
        } catch (e) {
          // Window might already be closed
        }
        reject(new Error('Authentication timeout - please try again'));
      }, 600000);

    } catch (error) {
      reject(error);
    }
  });
}

// Enhanced Authentication Service with Session Management
export class AuthService {
  static async authenticateWithMicrosoft(): Promise<MicrosoftAccount> {
    const authUrl = await invoke<string>('start_microsoft_auth');
    
    // Open the authentication URL in the system's default browser
    await invoke('open_url', { url: authUrl });
    
    return new Promise((resolve, reject) => {
      const checkCallback = async () => {
        try {
          const result = await invoke<string | null>('get_oauth_callback_result');
          
          if (result) {
            // Complete the authentication flow
            const account = await invoke<MicrosoftAccount>('complete_microsoft_auth', { 
              authCode: result 
            });
            
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

  static async startDeviceCodeAuth(): Promise<string> {
    return await invoke<string>('start_device_code_auth');
  }

  static async pollDeviceCodeAuth(): Promise<MicrosoftAccount | null> {
    return await invoke<MicrosoftAccount | null>('poll_device_code_auth');
  }

  static async copyToClipboard(text: string): Promise<void> {
    return await invoke('copy_to_clipboard', { text });
  }

  static async refreshAccountToken(accountId: string): Promise<MicrosoftAccount> {
    return await invoke<MicrosoftAccount>('refresh_minecraft_token', { accountId });
  }

  static isTokenValid(account: MicrosoftAccount): boolean {
    const now = Date.now() / 1000;
    return account.minecraft_expires_at > now + 300; // 5 minute buffer
  }

  static needsRefresh(account: MicrosoftAccount): boolean {
    const now = Date.now() / 1000;
    return account.minecraft_expires_at < now + 600; // 10 minute buffer
  }

  // Session Management Functions
  static async getMinecraftSessionPath(): Promise<string> {
    return await invoke<string>('get_minecraft_session_path');
  }

  static async readMinecraftSessions(): Promise<LauncherProfiles> {
    return await invoke<LauncherProfiles>('read_minecraft_sessions');
  }

  static async writeMinecraftSession(account: MicrosoftAccount): Promise<void> {
    return await invoke<void>('write_minecraft_session', { account });
  }

  static async getMinecraftLaunchArgs(account: MicrosoftAccount): Promise<string[]> {
    return await invoke<string[]>('get_minecraft_launch_args', { account });
  }

  static async validateMinecraftToken(accessToken: string): Promise<boolean> {
    return await invoke<boolean>('validate_minecraft_token', { accessToken });
  }

  static async refreshIfNeeded(account: MicrosoftAccount): Promise<MicrosoftAccount> {
    if (this.needsRefresh(account)) {
      return await this.refreshAccountToken(account.id);
    }
    return account;
  }
}

// Legacy compatibility - keep the existing class structure for now
export class MinecraftService {
  static async findInstallations(): Promise<MinecraftInstallation[]> {
    return getMinecraftInstallations();
  }

  static async launchMinecraft(options: LaunchOptions, minecraftPath: string): Promise<string> {
    return launchMinecraft(options, minecraftPath);
  }

  static async checkJavaInstallation(): Promise<string> {
    return checkJavaInstallation();
  }

  static async getDefaultMinecraftDir(): Promise<string> {
    return getDefaultMinecraftDir();
  }

  static async validateMinecraftDirectory(path: string): Promise<MinecraftDirectoryInfo> {
    return validateMinecraftDirectory(path);
  }

  static async startMicrosoftAuth(): Promise<string> {
    return startMicrosoftAuth();
  }

  static async authenticateWithMicrosoft(): Promise<MicrosoftAccount> {
    return authenticateWithMicrosoft();
  }
}

export class SettingsService {
  static async getSettings(): Promise<LauncherSettings> {
    return loadSettings();
  }

  static async saveSettings(settings: LauncherSettings): Promise<void> {
    return saveSettings(settings);
  }

  static async getLauncherDir(): Promise<string> {
    return getLauncherDir();
  }
}
