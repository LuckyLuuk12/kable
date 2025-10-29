import { type InstallationForm, type KableInstallation, type VersionData, installations, selectedInstallation, isLoadingInstallations, installationsError, versions, isLoadingVersions, versionsError, type LoaderKind, type ModJarInfo, type ExtendedModInfo, LogsService, openPath } from '$lib';
import * as installationsApi from '../api/installations';
import { get } from 'svelte/store';

export class InstallationService {
  // Coalesce concurrent loadInstallations calls. When non-null, callers await this promise.
  private static _inflightLoad: Promise<KableInstallation[]> | null = null;

  /**
   * Load all installations and update the store. 
   * @returns A snapshot of the loaded installations.
   */
  static async loadInstallations(): Promise<KableInstallation[]> {
    // If a load is already in-flight, return that promise so we don't trigger duplicate backend calls
    if (this._inflightLoad) {
      return this._inflightLoad;
    }

    // Start the in-flight promise
    this._inflightLoad = (async () => {
      isLoadingInstallations.set(true);
      installationsError.set(null);
      isLoadingVersions.set(true);
      versionsError.set(null);
      try {
        // Load installations and versions in parallel
        const [foundInstallations, foundVersions] = await Promise.all([
          installationsApi.getInstallations(),
          installationsApi.getAllVersions()
        ]);
        console.log('[InstallationService] Setting installations:', foundInstallations.length, 'items');
        installations.set(foundInstallations);
        versions.set(foundVersions);
        selectedInstallation.set(foundInstallations[0] || null);
        return get(installations);
      } catch (error) {
        installationsError.set(`Failed to load installations: ${error}`);
        versionsError.set(`Failed to load versions: ${error}`);
        installations.set([]);
        versions.set([]);
        selectedInstallation.set(null);
        return get(installations);
      } finally {
        isLoadingInstallations.set(false);
        isLoadingVersions.set(false);
        // Clear inflight promise
        this._inflightLoad = null;
        console.log('Installations loaded:', get(installations).length, 'Versions loaded:\n', get(versions).length);
        LogsService.emitLauncherEvent(`Loaded ${get(installations).length} installations and ${get(versions).length} versions`, 'debug');
      }
    })();

    return this._inflightLoad;
  }

  /**
   * Create a new installation and update the store.
   * @param version_id The ID of the version to create the installation for.
   */
  static async createInstallation(version_id: string): Promise<void> {
    await installationsApi.createInstallation(version_id);
    await this.loadInstallations();
  }

  /**
   * Create a new installation by copying from an existing one.
   * Optionally copies mods (with version updates), resource packs, and shaders.
   * @param version_id The ID of the version to create the new installation for.
   * @param sourceInstallation The source installation to copy from.
   * @param options Copy options for mods, resource packs, and shaders.
   */
  static async createInstallationFromExisting(
    version_id: string,
    sourceInstallation: KableInstallation,
    options: {
      copyMods: boolean;
      copyResourcePacks: boolean;
      copyShaders: boolean;
    }
  ): Promise<void> {
    await installationsApi.createInstallationFromExisting(
      version_id,
      sourceInstallation.id,
      options
    );
    await this.loadInstallations();
  }

  /**
   * Update an existing installation.
   */
  static async updateInstallation(id: string, newInstallation: KableInstallation): Promise<void> {
    console.log('[Service] updateInstallation called with:', { id, newInstallation });
    
    // Update store first for immediate UI feedback
    installations.update(list => {
      const index = list.findIndex(i => i.id === id);
      if (index !== -1) {
        // Create a new array with the updated installation for reactivity
        const newList = [...list];
        newList[index] = { ...newInstallation }; // Ensure new object reference
        console.log('[Service] Updated installation in store:', newList[index]);
        return newList;
      }
      return list;
    });

    // Then persist to backend
    await installationsApi.modifyInstallation(id, newInstallation);
    console.log('Installation updated in backend:', id);
  }

  /**
   * Delete an installation by ID.
   */
  static async deleteInstallation(id: string): Promise<void> {
    await installationsApi.deleteInstallation(id);
    await this.loadInstallations();
  }

  /**
   * Select an installation.
   */
  static selectInstallation(installation: KableInstallation | null): void {
    selectedInstallation.set(installation);
  }

  /**
   * Get current installations from store.
   * @return A snapshot of the currently loaded installations.
   */
  static getInstallations(): KableInstallation[] {
    return get(installations);
  }

  /**
   * Get currently selected installation from store.
   * @return The currently selected installation, or null if none is selected.
   */
  static getSelectedInstallation(): KableInstallation | null {
    return get(selectedInstallation);
  }

  static getEmptyInstallationForm(): InstallationForm {
    const allVersions = get(versions);
    return {
      name: '',
      icon: '',
      version_id: allVersions?.[0]?.version_id || '',
      java_args: [
        "-Xmx2048M",
      ],
      dedicated_resource_pack_folder: null,
      dedicated_shaders_folder: null,
    };
  }

  static fromInstallationForm(form: InstallationForm): KableInstallation {
    return {
      id: crypto.randomUUID(),
      name: form.name,
      icon: form.icon,
      version_id: form.version_id,
      created: new Date().toISOString(),
      last_used: new Date().toISOString(),
      java_args: form.java_args || [],
      dedicated_resource_pack_folder: form.dedicated_resource_pack_folder || null,
      dedicated_shaders_folder: form.dedicated_shaders_folder || null,
      favorite: false,
      total_time_played_ms: 0,
      description: form.description || '',
      parameters_map: {},
      times_launched: 0,
    };
  }

  static getFallbackVersions(): VersionData[] {
    return [
      {
        display_name: '1.21.8',
        version_id: '1.21.8',
        loader: "Vanilla",
        is_stable: true,
        extra: {},
      },
      {
        display_name: '1.19.4',
        version_id: '1.19.4',
        loader: "Vanilla",
        is_stable: true,
        extra: {},
      },
      {
        display_name: '1.8.9',
        version_id: '1.8.9',
        loader: "Vanilla",
        is_stable: true,
        extra: {},
      },
    ];
  }

  static getLoaderIcon(loader: LoaderKind): string {
    switch (loader) {
      case "Vanilla":    return 'cube';
      case "Fabric":     return 'fabric';
      case "Forge":      return 'forge';
      case "Quilt":      return 'quilt';
      case "NeoForge":   return 'neoforge';
      case "IrisFabric": return 'iris';
      default:           return 'question-mark';
    }
  }
  static getLoaderColor(loader: LoaderKind): string {
    switch (loader) {
      case "Vanilla":    return '#11833c'; // Vanilla's green/grass color
      case "Fabric":     return '#dbb866'; // Fabric's golden color
      case "Forge":      return '#466381'; // Forge's dark color
      case "Quilt":      return '#9c5aa0'; // Quilt's purple color
      case "NeoForge":   return '#f16436'; // NeoForge's orange color
      case "IrisFabric": return '#4c8cff'; // Iris Fabric's blue color
      default:           return '#cccccc'; // Default gray for unknown loaders
    }
  }

  static async toggleFavorite(installation: KableInstallation): Promise<void> {
    const updatedInstallation = { ...installation, favorite: !installation.favorite };
    await this.updateInstallation(installation.id, updatedInstallation);
  }

  static getVersionData(installation: KableInstallation): VersionData {
    const version = get(versions).find(v => v.version_id === installation.version_id);
    return version || {
      version_id: installation.version_id,
      loader: "Vanilla",
      display_name: installation.name || 'Unknown Version',
      is_stable: true,
      extra: {},
    };
  }

  static async getModInfo(installation: KableInstallation): Promise<ModJarInfo[]> {
    return await installationsApi.getModInfo(installation) || [];
  }

  static async exportInstallation(installation: KableInstallation) {
    console.log('Exporting installation:', installation);
    const path = await installationsApi.exportInstallation(installation);
    // Open the file location in the system file explorer
    if (path) {
      console.log('Exported installation to:', path);
      LogsService.emitLauncherEvent(`Exported installation ${installation.name} to ${path}`, 'info');
      await openPath(path);
    } else {
      LogsService.emitLauncherEvent(`Failed to export installation ${installation.name}`, 'error');
    }
  }

  static async importInstallation(path: string): Promise<void> {
    try {
      const newInstallation = await installationsApi.importInstallation(path);
      console.log('Imported installation from:', path, newInstallation);
      LogsService.emitLauncherEvent(`Imported installation ${newInstallation.name} from ${path}`, 'info');
      await this.loadInstallations();
    } catch (error) {
      console.error('Failed to import installation:', error);
      LogsService.emitLauncherEvent(`Failed to import installation from ${path}`, 'error');
      throw error;
    }
  }

  static async importFromMinecraftFolder(path: string): Promise<void> {
    try {
      const newInstallations = await installationsApi.importFromMinecraftFolder(path);
      console.log('Imported installations from .minecraft folder:', path, newInstallations);
      LogsService.emitLauncherEvent(`Imported ${newInstallations.length} installation(s) from ${path}`, 'info');
      await this.loadInstallations();
    } catch (error) {
      console.error('Failed to import from .minecraft folder:', error);
      LogsService.emitLauncherEvent(`Failed to import from .minecraft folder: ${path}`, 'error');
      throw error;
    }
  }

  static async duplicateInstallation(installation: KableInstallation): Promise<void> {
    try {
      await installationsApi.duplicateInstallation(installation);
      console.log('Duplicated installation:', installation);
      LogsService.emitLauncherEvent(`Duplicated installation ${installation.name}`, 'info');
      await this.loadInstallations();
    } catch (error) {
      console.error('Failed to duplicate installation:', error);
      LogsService.emitLauncherEvent(`Failed to duplicate installation ${installation.name}`, 'error');
    }
  }

  static async totalTimePlayed() {
    // return installations.reduce((total: number, installation: KableInstallation) => total + installation.total_time_played_ms, 0);
    let total = 0;
    for (const inst of get(installations)) {
      total += inst.total_time_played_ms;
    }
    return total;
  }

  /**
   * Get statistics about all installations
   */
  static getStatistics() {
    const allInstallations = get(installations);
    
    // Total playtime in milliseconds
    const totalPlaytimeMs = allInstallations.reduce((sum, inst) => sum + inst.total_time_played_ms, 0);
    
    // Last played installation
    const lastPlayedInstallation = allInstallations
      .filter(inst => inst.last_used)
      .sort((a, b) => new Date(b.last_used).getTime() - new Date(a.last_used).getTime())[0];
    
    // Most played installation
    const mostPlayedInstallation = allInstallations
      .filter(inst => inst.total_time_played_ms > 0)
      .sort((a, b) => b.total_time_played_ms - a.total_time_played_ms)[0];
    
    // Total launches across all installations
    const totalLaunches = allInstallations.reduce((sum, inst) => sum + inst.times_launched, 0);
    
    // Count installations by loader type
    const versionsList = get(versions);
    const loaderCounts: Record<LoaderKind, number> = {
      Vanilla: 0,
      Fabric: 0,
      Forge: 0,
      Quilt: 0,
      NeoForge: 0,
      IrisFabric: 0,
    };
    
    allInstallations.forEach(inst => {
      const version = versionsList.find(v => v.version_id === inst.version_id);
      if (version && version.loader in loaderCounts) {
        loaderCounts[version.loader]++;
      }
    });
    
    // Find most used loader
    const mostUsedLoader = Object.entries(loaderCounts)
      .sort(([, a], [, b]) => b - a)
      .filter(([, count]) => count > 0)[0]?.[0] as LoaderKind | undefined;
    
    return {
      totalInstallations: allInstallations.length,
      totalPlaytimeMs,
      totalPlaytimeHours: Math.floor(totalPlaytimeMs / 3600000),
      totalPlaytimeMinutes: Math.floor((totalPlaytimeMs % 3600000) / 60000),
      lastPlayedInstallation,
      lastPlayedDate: lastPlayedInstallation?.last_used || null,
      mostPlayedInstallation,
      totalLaunches,
      averageLaunchesPerInstallation: allInstallations.length > 0 
        ? Math.round(totalLaunches / allInstallations.length) 
        : 0,
      favoriteCount: allInstallations.filter(inst => inst.favorite).length,
      loaderCounts,
      mostUsedLoader,
    };
  }

  /**
   * Create a desktop shortcut for an installation.
   * @param installation The installation to create a shortcut for.
   * @returns The path to the created shortcut.
   */
  static async createShortcut(installation: KableInstallation): Promise<string> {
    return await installationsApi.createShortcut(installation);
  }
}
