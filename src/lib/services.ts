import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import type { 
  MinecraftInstallation, 
  LaunchOptions, 
  MicrosoftAccount, 
  LauncherSettings 
} from './types';

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

export async function refreshMinecraftToken(accountId: string): Promise<MicrosoftAccount> {
  return await invoke('refresh_minecraft_token', { accountId });
}

export async function getOAuthCallbackResult(): Promise<string | null> {
  return await invoke('get_oauth_callback_result');
}

// Minecraft Management
export async function findMinecraftInstallations(): Promise<MinecraftInstallation[]> {
  return await invoke('find_minecraft_installations');
}

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

// Legacy compatibility - keep the existing class structure for now
export class MinecraftService {
  static async findInstallations(): Promise<MinecraftInstallation[]> {
    return findMinecraftInstallations();
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
