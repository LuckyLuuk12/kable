import { writable, get } from 'svelte/store';
import { 
  getInstallations, 
  launchMinecraft, 
  checkJavaInstallation, 
  getDefaultMinecraftDir 
} from './services';
import { AuthManager, saveAccountToStorage } from './auth';
import { SettingsManager } from './settings';
import type { MinecraftInstallation, LaunchOptions } from './types';

// Game state stores
export const installations = writable<MinecraftInstallation[]>([]);
export const selectedInstallation = writable<MinecraftInstallation | null>(null);
export const isLoadingInstallations = writable(false);
export const installationsError = writable<string | null>(null);
export const javaStatus = writable<string>('Checking...');
export const isLaunching = writable(false);
export const launchError = writable<string | null>(null);

export class GameManager {
  /**
   * Initialize game manager - check Java and load installations
   */
  static async initialize(): Promise<void> {
    await Promise.all([
      this.checkJava(),
      this.loadInstallations()
    ]);
  }

  /**
   * Check Java installation status
   */
  static async checkJava(): Promise<void> {
    try {
      const status = await checkJavaInstallation();
      javaStatus.set(status);
    } catch (error) {
      console.error('Java check failed:', error);
      javaStatus.set(`Error: ${error}`);
    }
  }

  /**
   * Load Minecraft installations
   */
  static async loadInstallations(): Promise<void> {
    isLoadingInstallations.set(true);
    installationsError.set(null);

    try {
      const foundInstallations = await getInstallations();
      installations.set(foundInstallations);

      // Auto-select the first valid installation
      const validInstallation = foundInstallations.find((i: MinecraftInstallation) => i.is_valid);
      const toSelect = validInstallation || foundInstallations[0] || null;
      selectedInstallation.set(toSelect);

    } catch (error) {
      console.error('Failed to load installations:', error);
      installationsError.set(`Failed to load installations: ${error}`);
      installations.set([]);
      selectedInstallation.set(null);
    } finally {
      isLoadingInstallations.set(false);
    }
  }

  /**
   * Get default Minecraft directory
   */
  static async getDefaultMinecraftDirectory(): Promise<string> {
    return await getDefaultMinecraftDir();
  }

  /**
   * Launch Minecraft with current settings
   */
  static async launchGame(): Promise<void> {
    const account = AuthManager.getCurrentAccount();
    const installation = get(selectedInstallation);
    const settings = SettingsManager.getSettings();

    // Validation
    if (!account) {
      throw new Error('Please sign in first');
    }

    if (!installation) {
      throw new Error('Please select a Minecraft installation');
    }

    if (!installation.is_valid) {
      throw new Error('Selected installation is not valid');
    }

    // Ensure token is valid
    const validAccount = await AuthManager.ensureValidToken();
    if (!validAccount) {
      throw new Error('Authentication expired. Please sign in again.');
    }

    isLaunching.set(true);
    launchError.set(null);

    try {
      const launchOptions: LaunchOptions = {
        version: installation.version,
        username: validAccount.username,
        uuid: validAccount.uuid,
        access_token: validAccount.access_token,
        memory: settings.memory,
        java_path: settings.java_path,
        window_width: settings.window_width,
        window_height: settings.window_height,
        jvm_args: [],
        game_args: []
      };

      const result = await launchMinecraft(launchOptions, installation.path);
      console.log('Launch result:', result);

      // Update last played time for the account
      validAccount.last_used = Math.floor(Date.now() / 1000);
      saveAccountToStorage(validAccount);

    } catch (error) {
      console.error('Launch failed:', error);
      launchError.set(`Launch failed: ${error}`);
      throw error;
    } finally {
      isLaunching.set(false);
    }
  }

  /**
   * Select a specific installation
   */
  static selectInstallation(installation: MinecraftInstallation): void {
    selectedInstallation.set(installation);
  }

  /**
   * Get current installations
   */
  static getInstallations(): MinecraftInstallation[] {
    return get(installations);
  }

  /**
   * Get currently selected installation
   */
  static getSelectedInstallation(): MinecraftInstallation | null {
    return get(selectedInstallation);
  }

  /**
   * Check if ready to launch
   */
  static canLaunch(): { canLaunch: boolean; reason?: string } {
    const account = AuthManager.getCurrentAccount();
    const installation = get(selectedInstallation);

    if (!account) {
      return { canLaunch: false, reason: 'Not authenticated' };
    }

    if (!installation) {
      return { canLaunch: false, reason: 'No installation selected' };
    }

    if (!installation.is_valid) {
      return { canLaunch: false, reason: 'Invalid installation' };
    }

    return { canLaunch: true };
  }

  /**
   * Get launch status message
   */
  static getLaunchStatus(): string {
    const { canLaunch, reason } = this.canLaunch();
    
    if (!canLaunch) {
      return reason || 'Cannot launch';
    }

    if (get(isLaunching)) {
      return 'Launching...';
    }

    const error = get(launchError);
    if (error) {
      return error;
    }

    return 'Ready to launch';
  }
}
