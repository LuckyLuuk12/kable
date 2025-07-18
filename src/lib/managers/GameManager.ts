import { get } from 'svelte/store';
import { InstallationService, LaunchService, AuthManager, SettingsManager, installations, selectedInstallation, isLoadingInstallations, installationsError, javaStatus, isLaunching, launchError } from '$lib';
import * as installationsApi from '../api/installations';
import * as minecraftApi from '../api/minecraft';
import type { MinecraftInstallation } from '$lib';

/**
 * Game Manager
 * Coordinates game state between stores, services, and API
 */

export class GameManager {
  private static isInitialized = false;

  /**
   * Initialize game manager - check Java and load installations
   */
  static async initialize(): Promise<void> {
    if (this.isInitialized) {
      return;
    }

    await Promise.all([
      this.checkJava(),
      this.loadInstallations()
    ]);
    
    this.isInitialized = true;
  }

  /**
   * Check Java installation status
   */
  static async checkJava(): Promise<void> {
    try {
      const status = await minecraftApi.checkJavaInstallation();
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
      const foundInstallations = await InstallationService.getInstallations();
      installations.set(foundInstallations);
      
      // Auto-select the first valid installation
      const validInstallation = foundInstallations.find((i: MinecraftInstallation) => i.is_valid);
      const toSelect = validInstallation || foundInstallations[0] || null;
      
      // Only set if no current selection or current selection is no longer valid
      const current = get(selectedInstallation);
      if (!current || !foundInstallations.find(i => i.id === current.id)) {
        selectedInstallation.set(toSelect);
      }
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
   * Launch selected installation
   */
  static async launchSelected(): Promise<void> {
    const installation = get(selectedInstallation);
    if (!installation) {
      throw new Error('No installation selected');
    }
    
    await this.launch(installation.id);
  }

  /**
   * Launch specific installation
   */
  static async launch(installationId: string): Promise<void> {
    try {
      isLaunching.set(true);
      launchError.set(null);
      
      const launchService = LaunchService.getInstance();
      const result = await launchService.launchInstallation(installationId);
      
      if (!result.success) {
        throw new Error(result.error || 'Launch failed');
      }
      
      console.log('✅ Minecraft launched successfully');
    } catch (error) {
      console.error('❌ Launch failed:', error);
      launchError.set(error instanceof Error ? error.message : String(error));
      throw error;
    } finally {
      isLaunching.set(false);
    }
  }

  /**
   * Quick launch by version name
   */
  static async quickLaunch(versionName: string): Promise<void> {
    try {
      isLaunching.set(true);
      launchError.set(null);
      
      const launchService = LaunchService.getInstance();
      const result = await launchService.quickLaunch(versionName);
      
      if (!result.success) {
        throw new Error(result.error || 'Quick launch failed');
      }
      
      console.log('✅ Minecraft quick launched successfully');
    } catch (error) {
      console.error('❌ Quick launch failed:', error);
      launchError.set(error instanceof Error ? error.message : String(error));
      throw error;
    } finally {
      isLaunching.set(false);
    }
  }

  /**
   * Create new installation
   */
  static async createInstallation(
    name: string,
    version: string,
    modLoader: string,
    options?: {
      gameDirectory?: string;
      javaPath?: string;
      jvmArgs?: string;
      memory?: number;
      description?: string;
    }
  ): Promise<MinecraftInstallation> {
    try {
      const installation = await InstallationService.createInstallation(
        name,
        version,
        modLoader,
        options?.gameDirectory,
        options?.javaPath,
        options?.jvmArgs,
        options?.memory,
        options?.description
      );
      
      // Reload installations to include the new one
      await this.loadInstallations();
      
      console.log('✅ Installation created:', installation.name);
      return installation;
    } catch (error) {
      console.error('❌ Failed to create installation:', error);
      throw error;
    }
  }

  /**
   * Delete installation
   */
  static async deleteInstallation(installationId: string): Promise<void> {
    try {
      await InstallationService.deleteInstallation(installationId);
      
      // Clear selection if this was the selected installation
      const current = get(selectedInstallation);
      if (current && current.id === installationId) {
        selectedInstallation.set(null);
      }
      
      // Reload installations
      await this.loadInstallations();
      
      console.log('✅ Installation deleted');
    } catch (error) {
      console.error('❌ Failed to delete installation:', error);
      throw error;
    }
  }

  /**
   * Select installation
   */
  static selectInstallation(installation: MinecraftInstallation | null): void {
    selectedInstallation.set(installation);
  }

  /**
   * Get default Minecraft directory
   */
  static async getDefaultMinecraftDirectory(): Promise<string> {
    try {
      return await minecraftApi.getDefaultMinecraftDir();
    } catch (error) {
      console.error('Failed to get default Minecraft directory:', error);
      throw error;
    }
  }

  /**
   * Launch Minecraft with current settings and authentication
   * Enhanced version that handles authentication and settings integration
   */
  static async launchGame(): Promise<void> {
    const account = AuthManager.getCurrentAccount();
    const installation = get(selectedInstallation);
    const settings = await SettingsManager.getSettingsAsync();

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

    isLaunching.set(true);
    launchError.set(null);

    try {
      // Use the installation-specific launch function that handles everything internally
      await installationsApi.launchMinecraftInstallation(installation.id);
      console.log('Launch initiated for installation:', installation.name);
      // Optionally update last played time for the account here if needed
    } catch (error) {
      console.error('Launch failed:', error);
      launchError.set(`Launch failed: ${error}`);
      throw error;
    } finally {
      isLaunching.set(false);
    }
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

  /**
   * Check if Minecraft is currently running
   */
  static async isMinecraftRunning(): Promise<boolean> {
    try {
      const launchService = LaunchService.getInstance();
      return await launchService.isMinecraftRunning();
    } catch (error) {
      console.error('Failed to check Minecraft status:', error);
      return false;
    }
  }
}
