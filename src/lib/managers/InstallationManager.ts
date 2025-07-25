import { Loader, type InstallationForm, type KableInstallation, type VersionData, installations, selectedInstallation, isLoadingInstallations, installationsError } from '$lib';
import * as installationsApi from '../api/installations';
import { get } from 'svelte/store';

export class InstallationManager {
  /**
   * Load all installations and update the store. 
   * @returns A snapshot of the loaded installations.
   */
  static async loadInstallations(): Promise<KableInstallation[]> {
    try {
      isLoadingInstallations.set(true);
      installationsError.set(null);
      const foundInstallations = await installationsApi.get_installations();
      installations.set(foundInstallations);
      // Optionally select the first installation
      selectedInstallation.set(foundInstallations[0] || null);
    } catch (error) {
      installationsError.set(`Failed to load installations: ${error}`);
      installations.set([]);
      selectedInstallation.set(null);
    } finally {
      isLoadingInstallations.set(false);
    }
    return get(installations);
  }

  /**
   * Create a new installation and update the store.
   * @param version_id The ID of the version to create the installation for.
   */
  static async createInstallation(version_id: string): Promise<KableInstallation> {
    let installation = await installationsApi.create_installation(version_id);
    await this.loadInstallations();
    return installation;
  }

  /**
   * Update an existing installation.
   */
  static async updateInstallation(id: string, newInstallation: KableInstallation): Promise<void> {
    await installationsApi.modify_kable_installation(id, newInstallation);
    await this.loadInstallations();
  }

  /**
   * Delete an installation by ID.
   */
  static async deleteInstallation(id: string): Promise<void> {
    await installationsApi.delete_installation(id);
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
    return {
      name: '',
      icon: '',
      version: {
        id: '',
        loader: Loader.Vanilla,
        stable: true,
      },
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
      version: form.version,
      created: new Date().toISOString(),
      last_used: new Date().toISOString(),
      java_args: form.java_args || [],
      dedicated_resource_pack_folder: form.dedicated_resource_pack_folder || null,
      dedicated_shaders_folder: form.dedicated_shaders_folder || null,
    };
  }

  static getFallbackVersions(): VersionData[] {
    return [
      {
        id: '1.21.8',
        loader: Loader.Vanilla,
        stable: true,
      },
      {
        id: '1.19.4',
        loader: Loader.Vanilla,
        stable: true,
      },
      {
        id: '1.8.9',
        loader: Loader.Vanilla,
        stable: true,
      },
    ];
  }

  static getLoaderIcon(loader: Loader): string {
    switch (loader) {
      case Loader.Vanilla:    return 'cube';
      case Loader.Fabric:     return 'fabric';
      case Loader.Forge:      return 'forge';
      case Loader.Quilt:      return 'quilt';
      case Loader.NeoForge:   return 'neoforge';
      case Loader.IrisFabric: return 'iris';
      default:                return 'question-mark';
    }
  }
  static getLoaderColor(loader: Loader): string {
    switch (loader) {
      case Loader.Vanilla:    return '#11833c'; // Vanilla's green/grass color
      case Loader.Fabric:     return '#dbb866'; // Fabric's golden color
      case Loader.Forge:      return '#1e2328'; // Forge's dark color
      case Loader.Quilt:      return '#9c5aa0'; // Quilt's purple color
      case Loader.NeoForge:   return '#f16436'; // NeoForge's orange color
      case Loader.IrisFabric: return '#4c8cff'; // Iris Fabric's blue color
      default:                return '#cccccc'; // Default gray for unknown loaders
    }
  }

  static toggleFavorite(installation: KableInstallation): void {
    const updatedInstallation = { ...installation, favorite: !installation.favorite };
    this.updateInstallation(installation.id, updatedInstallation);
  }
}