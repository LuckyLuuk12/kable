import type { InstallationForm, MinecraftInstallation } from '$lib';
import type { MinecraftVersion, ModDetectionResult } from '$lib';
import { InstallationService, ModDetectionService, installations, isLoadingInstallations, installationsError, isLaunching, launchError, selectedInstallation } from '$lib';
import { LaunchService } from '$lib';
import * as minecraftApi from '../api/minecraft';
import { get } from 'svelte/store';
import { AuthManager, SettingsManager } from '$lib';
import * as installationsApi from '../api/installations';

export class InstallationManager {
  static async getInstallations(): Promise<MinecraftInstallation[]> {
    return await InstallationService.getInstallations();
  }

  static async createInstallation(form: any): Promise<MinecraftInstallation> {
    return await InstallationService.createInstallation(
      form.name,
      form.version,
      form.mod_loader,
      form.game_directory,
      form.java_path,
      form.jvm_args,
      form.memory,
      form.description
    );
  }

  static async deleteInstallation(installationId: string): Promise<void> {
    return await InstallationService.deleteInstallation(installationId);
  }

  static async updateInstallation(installationId: string, form: any): Promise<MinecraftInstallation> {
    return await InstallationService.updateInstallation(
      installationId,
      form.name,
      form.version,
      form.mod_loader,
      form.game_directory,
      form.java_path,
      form.jvm_args,
      form.memory,
      form.description
    );
  }

  static async launchInstallation(installation: MinecraftInstallation): Promise<void> {
    // Optionally add validation here
    await InstallationService.launchInstallation(installation.id);
  }

  static async openInstallationFolder(installationId: string): Promise<void> {
    return await InstallationService.openInstallationFolder(installationId);
  }

  static async analyzeInstallation(installation: MinecraftInstallation): Promise<ModDetectionResult> {
    return await ModDetectionService.analyzeInstallation(installation);
  }

  static async getMinecraftVersions(): Promise<MinecraftVersion[]> {
    return await InstallationService.getMinecraftVersions();
  }

  static getFallbackVersions(): MinecraftVersion[] {
    return [
      { id: '1.21.3', type: 'release', releaseTime: '2024-10-23T12:00:00Z', url: '', time: '2024-10-23T12:00:00Z' },
      { id: '1.21.2', type: 'release', releaseTime: '2024-10-22T12:00:00Z', url: '', time: '2024-10-22T12:00:00Z' },
      { id: '1.21.1', type: 'release', releaseTime: '2024-08-08T12:00:00Z', url: '', time: '2024-08-08T12:00:00Z' },
      { id: '1.21', type: 'release', releaseTime: '2024-06-13T12:00:00Z', url: '', time: '2024-06-13T12:00:00Z' },
      { id: '1.20.6', type: 'release', releaseTime: '2024-04-29T12:00:00Z', url: '', time: '2024-04-29T12:00:00Z' },
      { id: '1.20.4', type: 'release', releaseTime: '2023-12-07T12:00:00Z', url: '', time: '2023-12-07T12:00:00Z' },
      { id: '1.19.4', type: 'release', releaseTime: '2023-03-14T12:00:00Z', url: '', time: '2023-03-14T12:00:00Z' },
      { id: '1.18.2', type: 'release', releaseTime: '2022-02-28T12:00:00Z', url: '', time: '2022-02-28T12:00:00Z' },
    ];
  }

  static getEmptyInstallationForm(): InstallationForm {
    return {
      name: '',
      version: '',
      mod_loader: 'vanilla',
      game_directory: '',
      java_path: '',
      jvm_args: '-Xmx2G',
      memory: 2048,
      description: ''
    };
  }

  /**
   * Select installation (sets the selectedInstallation store)
   */
  static selectInstallation(installation: MinecraftInstallation | null): void {
    selectedInstallation.set(installation);
  }

  /**
   * Can launch (overload: uses selectedInstallation if not provided)
   */
  static async canLaunch(installation?: MinecraftInstallation): Promise<{ canLaunch: boolean; reason?: string }> {
    const inst = installation ?? get(selectedInstallation);
    const account = AuthManager.getCurrentAccount?.() ?? null;
    if (!account) {
      return { canLaunch: false, reason: 'Not authenticated' };
    }
    if (!inst) {
      return { canLaunch: false, reason: 'No installation selected' };
    }
    if (!inst.is_valid) {
      return { canLaunch: false, reason: 'Invalid installation' };
    }
    return { canLaunch: true };
  }

  /**
   * Launch Minecraft with current settings and authentication
   */
  static async launchGame(): Promise<void> {
    const account = AuthManager.getCurrentAccount?.() ?? null;
    const installation = get(selectedInstallation);
    const settings = await SettingsManager.getSettingsAsync?.();

    if (!account) {
      throw new Error('Please sign in first');
    }
    if (!installation) {
      throw new Error('Please select a Minecraft installation');
    }
    if (!installation.is_valid) {
      throw new Error('Selected installation is not valid');
    }

    try {
      await installationsApi.launchMinecraftInstallation(installation.id);
      // Optionally update last played time for the account here if needed
    } catch (error) {
      throw error;
    }
  }

  /**
   * Load Minecraft installations and update stores
   */
  static async loadInstallations(): Promise<void> {
    try {
      if (typeof installations !== 'undefined' && typeof selectedInstallation !== 'undefined') {
        isLoadingInstallations?.set?.(true);
        installationsError?.set?.(null);
        const foundInstallations = await InstallationService.getInstallations();
        installations.set(foundInstallations);
        const validInstallation = foundInstallations.find((i: MinecraftInstallation) => i.is_valid);
        const toSelect = validInstallation || foundInstallations[0] || null;
        const current = get(selectedInstallation);
        if (!current || !foundInstallations.find(i => i.id === current.id)) {
          selectedInstallation.set(toSelect);
        }
      }
    } catch (error) {
      installationsError?.set?.(`Failed to load installations: ${error}`);
      installations?.set?.([]);
      selectedInstallation?.set?.(null);
    } finally {
      isLoadingInstallations?.set?.(false);
    }
  }

  /**
   * Get current installations from store
   */
  static getInstallationsFromStore(): MinecraftInstallation[] {
    return typeof installations !== 'undefined' ? get(installations) : [];
  }

  /**
   * Get currently selected installation from store
   */
  static getSelectedInstallation(): MinecraftInstallation | null {
    return typeof selectedInstallation !== 'undefined' ? get(selectedInstallation) : null;
  }

  /**
   * Synchronous canLaunch (uses selectedInstallation)
   */
  static canLaunchSync(): { canLaunch: boolean; reason?: string } {
    const account = AuthManager.getCurrentAccount?.() ?? null;
    const installation = typeof selectedInstallation !== 'undefined' ? get(selectedInstallation) : null;
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
    const { canLaunch, reason } = this.canLaunchSync();
    if (!canLaunch) {
      return reason || 'Cannot launch';
    }
    if (typeof isLaunching !== 'undefined' && get(isLaunching)) {
      return 'Launching...';
    }
    if (typeof launchError !== 'undefined') {
      const error = get(launchError);
      if (error) {
        return error;
      }
    }
    return 'Ready to launch';
  }

  /**
   * Check if Minecraft is currently running
   */
  static async isMinecraftRunning(): Promise<boolean> {
    try {
      const launchService = LaunchService.getInstance?.();
      return await launchService?.isMinecraftRunning?.();
    } catch (error) {
      console.error('Failed to check Minecraft status:', error);
      return false;
    }
  }

  /**
   * Launch selected installation
   */
  static async launchSelected(): Promise<void> {
    const installation = typeof selectedInstallation !== 'undefined' ? get(selectedInstallation) : null;
    if (!installation) {
      throw new Error('No installation selected');
    }
    await this.launch(installation.id);
  }

  /**
   * Launch specific installation by id
   */
  static async launch(installationId: string): Promise<void> {
    try {
      isLaunching?.set?.(true);
      launchError?.set?.(null);
      const launchService = LaunchService.getInstance?.();
      const result = await launchService?.launchInstallation?.(installationId);
      if (!result?.success) {
        throw new Error(result?.error || 'Launch failed');
      }
      console.log('✅ Minecraft launched successfully');
    } catch (error) {
      console.error('❌ Launch failed:', error);
      launchError?.set?.(error instanceof Error ? error.message : String(error));
      throw error;
    } finally {
      isLaunching?.set?.(false);
    }
  }

  /**
   * Quick launch by version name
   */
  static async quickLaunch(versionName: string): Promise<void> {
    try {
      isLaunching?.set?.(true);
      launchError?.set?.(null);
      const launchService = LaunchService.getInstance?.();
      const result = await launchService?.quickLaunch?.(versionName);
      if (!result?.success) {
        throw new Error(result?.error || 'Quick launch failed');
      }
      console.log('✅ Minecraft quick launched successfully');
    } catch (error) {
      console.error('❌ Quick launch failed:', error);
      launchError?.set?.(error instanceof Error ? error.message : String(error));
      throw error;
    } finally {
      isLaunching?.set?.(false);
    }
  }

  /**
   * Get default Minecraft directory
   */
  static async getDefaultMinecraftDirectory(): Promise<string> {
    try {
      return await minecraftApi.getDefaultMinecraftDir?.();
    } catch (error) {
      console.error('Failed to get default Minecraft directory:', error);
      throw error;
    }
  }
}
