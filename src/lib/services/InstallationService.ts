import { type InstallationForm, type KableInstallation, type VersionData, installations, selectedInstallation, isLoadingInstallations, installationsError, versions, isLoadingVersions, versionsError, type LoaderKind, type ModJarInfo } from '$lib';
import * as installationsApi from '../api/installations';
import { get } from 'svelte/store';

export class InstallationService {
  /**
   * Load all installations and update the store. 
   * @returns A snapshot of the loaded installations.
   */
  static async loadInstallations(): Promise<KableInstallation[]> {
    isLoadingInstallations.set(true);
    installationsError.set(null);
    isLoadingVersions.set(true);
    versionsError.set(null);
    try {
      // Load installations and versions in parallel
      const [foundInstallations, foundVersions] = await Promise.all([
        installationsApi.get_installations(),
        installationsApi.get_all_versions()
      ]);
      installations.set(foundInstallations);
      versions.set(foundVersions);
      selectedInstallation.set(foundInstallations[0] || null);
    } catch (error) {
      installationsError.set(`Failed to load installations: ${error}`);
      versionsError.set(`Failed to load versions: ${error}`);
      installations.set([]);
      versions.set([]);
      selectedInstallation.set(null);
    } finally {
      isLoadingInstallations.set(false);
      isLoadingVersions.set(false);
    }
    console.log('Installations loaded:', get(installations).length, 'Versions loaded:\n', get(versions));
    return get(installations);
  }

  /**
   * Create a new installation and update the store.
   * @param version_id The ID of the version to create the installation for.
   */
  static createInstallation(version_id: string): void {
    installationsApi.create_installation(version_id)
      .then(() => this.loadInstallations());
  }

  /**
   * Update an existing installation.
   */
  static updateInstallation(id: string, newInstallation: KableInstallation): void {
    // first modify store for reactivity
    installations.update(list => {
      const index = list.findIndex(i => i.id === id);
      if (index !== -1) {
        list[index] = newInstallation;
      }
      return list;
    });

    installationsApi.modify_installation(id, newInstallation)
      .then(async () => console.log('Installation updated:', id));
  }

  /**
   * Delete an installation by ID.
   */
  static deleteInstallation(id: string): void {
    installationsApi.delete_installation(id)
      .then(() => this.loadInstallations());
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
      default:                return 'question-mark';
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
      default:                return '#cccccc'; // Default gray for unknown loaders
    }
  }

  static toggleFavorite(installation: KableInstallation): void {
    const updatedInstallation = { ...installation, favorite: !installation.favorite };
    this.updateInstallation(installation.id, updatedInstallation);
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
    return await installationsApi.get_mod_info(installation) || [];
  }
}