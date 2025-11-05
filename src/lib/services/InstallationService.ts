import {
  type InstallationForm,
  type KableInstallation,
  type VersionData,
  installations,
  selectedInstallation,
  isLoadingInstallations,
  installationsError,
  versions,
  isLoadingVersions,
  versionsError,
  type LoaderKind,
  type ModJarInfo,
  type ExtendedModInfo,
  LogsService,
  openPath,
} from "$lib";
import * as installationsApi from "../api/installations";
import { get } from "svelte/store";
import { listen } from "@tauri-apps/api/event";

export class InstallationService {
  // Coalesce concurrent loadInstallations calls. When non-null, callers await this promise.
  private static _inflightLoad: Promise<KableInstallation[]> | null = null;
  private static _versionsListenerUnsubscribe: (() => void) | null = null;
  private static _installationsListenerUnsubscribe: (() => void) | null = null;
  private static _installationUpdatedUnsubscribe: (() => void) | null = null;

  /**
   * Initialize event listeners for progressive loading
   */
  static async initializeProgressiveLoading() {
    // Listen for version chunks
    if (!this._versionsListenerUnsubscribe) {
      const unsubscribe1 = await listen(
        "versions-chunk-loaded",
        (event: any) => {
          const chunk = event.payload;
          console.log(
            `[Versions] Loaded chunk for ${chunk.loader}:`,
            chunk.versions.length,
            "versions",
          );

          // Append new versions to existing ones
          const currentVersions = get(versions);
          const newVersions = [...currentVersions, ...chunk.versions];
          versions.set(newVersions);
        },
      );

      const unsubscribe2 = await listen(
        "versions-loading-complete",
        (event: any) => {
          const complete = event.payload;
          console.log(
            `[Versions] Loading complete! Total:`,
            complete.total_count,
          );
          isLoadingVersions.set(false);
          LogsService.emitLauncherEvent(
            `Loaded ${complete.total_count} versions`,
            "debug",
          );
        },
      );

      this._versionsListenerUnsubscribe = () => {
        unsubscribe1();
        unsubscribe2();
      };
    }

    // Listen for installation chunks
    if (!this._installationsListenerUnsubscribe) {
      const unsubscribe1 = await listen(
        "installations-chunk-loaded",
        (event: any) => {
          const chunk = event.payload;
          console.log(
            `[Installations] Loaded chunk:`,
            chunk.installations.length,
            "installations",
          );

          // Replace installations with the latest chunk (already sorted by last_used)
          installations.set(chunk.installations);
          if (chunk.installations.length > 0 && !get(selectedInstallation)) {
            selectedInstallation.set(chunk.installations[0]);
          }
        },
      );

      const unsubscribe2 = await listen(
        "installations-loading-complete",
        (event: any) => {
          const complete = event.payload;
          console.log(
            `[Installations] Loading complete! Total:`,
            complete.total_count,
          );
          isLoadingInstallations.set(false);
          LogsService.emitLauncherEvent(
            `Loaded ${complete.total_count} installations`,
            "debug",
          );
        },
      );

      this._installationsListenerUnsubscribe = () => {
        unsubscribe1();
        unsubscribe2();
      };
    }

    // Listen for individual installation updates (e.g., playtime updates)
    if (!this._installationUpdatedUnsubscribe) {
      const unsubscribe = await listen("installation-updated", (event: any) => {
        const { installation_id, installation } = event.payload;
        console.log(
          `[InstallationService] Installation updated:`,
          installation_id,
        );

        // Update the specific installation in the store
        installations.update((list) => {
          const index = list.findIndex((i) => i.id === installation_id);
          if (index !== -1) {
            const newList = [...list];
            newList[index] = { ...installation };
            console.log(
              `[InstallationService] Updated installation in store:`,
              installation.name,
              "playtime:",
              installation.total_time_played_ms,
            );
            return newList;
          }
          return list;
        });
      });

      this._installationUpdatedUnsubscribe = unsubscribe;
    }
  }

  /**
   * Load all installations and update the store.
   * Versions are loaded separately on-demand.
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
      try {
        // Load ONLY installations - versions will be loaded on-demand when needed
        const foundInstallations = await installationsApi.getInstallations();
        console.log(
          "[InstallationService] Setting installations:",
          foundInstallations.length,
          "items",
        );
        installations.set(foundInstallations);
        selectedInstallation.set(foundInstallations[0] || null);
        return get(installations);
      } catch (error) {
        installationsError.set(`Failed to load installations: ${error}`);
        installations.set([]);
        selectedInstallation.set(null);
        return get(installations);
      } finally {
        isLoadingInstallations.set(false);
        // Clear inflight promise
        this._inflightLoad = null;
        console.log("Installations loaded:", get(installations).length);
        LogsService.emitLauncherEvent(
          `Loaded ${get(installations).length} installations`,
          "debug",
        );
      }
    })();

    return this._inflightLoad;
  }

  /**
   * Load all versions and update the store.
   * This is called separately from loadInstallations to avoid blocking.
   * @returns A snapshot of the loaded versions.
   */
  static async loadVersions(): Promise<VersionData[]> {
    isLoadingVersions.set(true);
    versionsError.set(null);
    try {
      const foundVersions = await installationsApi.getAllVersions();
      versions.set(foundVersions);
      console.log("Versions loaded:", get(versions).length);
      LogsService.emitLauncherEvent(
        `Loaded ${get(versions).length} versions`,
        "debug",
      );
      return get(versions);
    } catch (error) {
      versionsError.set(`Failed to load versions: ${error}`);
      versions.set([]);
      return get(versions);
    } finally {
      isLoadingVersions.set(false);
    }
  }

  /**
   * Force refresh version manifests from the network, ignoring cache.
   * This is useful for getting the latest snapshots and new versions.
   * @returns A snapshot of the refreshed versions.
   */
  static async refreshVersionManifests(): Promise<VersionData[]> {
    isLoadingVersions.set(true);
    versionsError.set(null);
    try {
      // Clear existing versions to show we're refreshing
      versions.set([]);
      const foundVersions = await installationsApi.refreshVersionManifests();
      versions.set(foundVersions);
      console.log("Version manifests refreshed:", get(versions).length);
      LogsService.emitLauncherEvent(
        `Refreshed ${get(versions).length} versions from network`,
        "info",
      );
      return get(versions);
    } catch (error) {
      versionsError.set(`Failed to refresh version manifests: ${error}`);
      versions.set([]);
      return get(versions);
    } finally {
      isLoadingVersions.set(false);
    }
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
    },
  ): Promise<void> {
    await installationsApi.createInstallationFromExisting(
      version_id,
      sourceInstallation.id,
      options,
    );
    await this.loadInstallations();
  }

  /**
   * Update an existing installation.
   */
  static async updateInstallation(
    id: string,
    newInstallation: KableInstallation,
  ): Promise<void> {
    console.log("[Service] updateInstallation called with:", {
      id,
      newInstallation,
    });

    // Update store first for immediate UI feedback
    installations.update((list) => {
      const index = list.findIndex((i) => i.id === id);
      if (index !== -1) {
        // Create a new array with the updated installation for reactivity
        const newList = [...list];
        newList[index] = { ...newInstallation }; // Ensure new object reference
        console.log("[Service] Updated installation in store:", newList[index]);
        return newList;
      }
      return list;
    });

    // Then persist to backend
    await installationsApi.modifyInstallation(id, newInstallation);
    console.log("Installation updated in backend:", id);
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
      name: "",
      icon: "",
      version_id: allVersions?.[0]?.version_id || "",
      java_args: ["-Xmx2048M"],
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
      dedicated_resource_pack_folder:
        form.dedicated_resource_pack_folder || null,
      dedicated_shaders_folder: form.dedicated_shaders_folder || null,
      favorite: false,
      total_time_played_ms: 0,
      description: form.description || "",
      parameters_map: {},
      times_launched: 0,
    };
  }

  static getFallbackVersions(): VersionData[] {
    return [
      {
        display_name: "1.21.8",
        version_id: "1.21.8",
        loader: "Vanilla",
        is_stable: true,
        extra: {},
      },
      {
        display_name: "1.19.4",
        version_id: "1.19.4",
        loader: "Vanilla",
        is_stable: true,
        extra: {},
      },
      {
        display_name: "1.8.9",
        version_id: "1.8.9",
        loader: "Vanilla",
        is_stable: true,
        extra: {},
      },
    ];
  }

  static getLoaderIcon(loader: LoaderKind): string {
    switch (loader) {
      case "Vanilla":
        return "cube";
      case "Fabric":
        return "fabric";
      case "Forge":
        return "forge";
      case "Quilt":
        return "quilt";
      case "NeoForge":
        return "neoforge";
      case "IrisFabric":
        return "iris";
      default:
        return "question-mark";
    }
  }
  static getLoaderColor(loader: LoaderKind): string {
    switch (loader) {
      case "Vanilla":
        return "#11833c"; // Vanilla's green/grass color
      case "Fabric":
        return "#dbb866"; // Fabric's golden color
      case "Forge":
        return "#466381"; // Forge's dark color
      case "Quilt":
        return "#9c5aa0"; // Quilt's purple color
      case "NeoForge":
        return "#f16436"; // NeoForge's orange color
      case "IrisFabric":
        return "#4c8cff"; // Iris Fabric's blue color
      default:
        return "#cccccc"; // Default gray for unknown loaders
    }
  }

  static async toggleFavorite(installation: KableInstallation): Promise<void> {
    const updatedInstallation = {
      ...installation,
      favorite: !installation.favorite,
    };
    await this.updateInstallation(installation.id, updatedInstallation);
  }

  static getVersionData(installation: KableInstallation): VersionData {
    const version = get(versions).find(
      (v) => v.version_id === installation.version_id,
    );
    return (
      version || {
        version_id: installation.version_id,
        loader: "Vanilla",
        display_name: installation.name || "Unknown Version",
        is_stable: true,
        extra: {},
      }
    );
  }

  static async getModInfo(
    installation: KableInstallation,
  ): Promise<ModJarInfo[]> {
    return (await installationsApi.getModInfo(installation)) || [];
  }

  static async exportInstallation(installation: KableInstallation) {
    console.log("Exporting installation:", installation);
    const path = await installationsApi.exportInstallation(installation);
    // Open the file location in the system file explorer
    if (path) {
      console.log("Exported installation to:", path);
      LogsService.emitLauncherEvent(
        `Exported installation ${installation.name} to ${path}`,
        "info",
      );
      await openPath(path);
    } else {
      LogsService.emitLauncherEvent(
        `Failed to export installation ${installation.name}`,
        "error",
      );
    }
  }

  static async importInstallation(path: string): Promise<void> {
    try {
      console.log("[InstallationService] Starting import from:", path);
      const newInstallation = await installationsApi.importInstallation(path);
      console.log(
        "[InstallationService] Successfully imported installation:",
        newInstallation.name,
      );
      LogsService.emitLauncherEvent(
        `✓ Successfully imported installation "${newInstallation.name}" from ${path}`,
        "info",
      );

      // Reload installations to show the new one
      await this.loadInstallations();
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      console.error(
        "[InstallationService] Failed to import installation:",
        errorMsg,
      );
      LogsService.emitLauncherEvent(
        `✗ Failed to import installation from ${path}: ${errorMsg}`,
        "error",
      );
      throw error;
    }
  }

  static async importFromMinecraftFolder(path: string): Promise<void> {
    try {
      console.log(
        "[InstallationService] Starting import from .minecraft folder:",
        path,
      );
      const newInstallations =
        await installationsApi.importFromMinecraftFolder(path);
      const count = newInstallations.length;
      console.log(
        `[InstallationService] Successfully imported ${count} installation(s) from .minecraft folder`,
      );
      LogsService.emitLauncherEvent(
        `✓ Successfully imported ${count} installation(s) from ${path}`,
        "info",
      );

      // Reload installations to show the new ones
      await this.loadInstallations();
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      console.error(
        "[InstallationService] Failed to import from .minecraft folder:",
        errorMsg,
      );
      LogsService.emitLauncherEvent(
        `✗ Failed to import from .minecraft folder ${path}: ${errorMsg}`,
        "error",
      );
      throw error;
    }
  }

  static async duplicateInstallation(
    installation: KableInstallation,
  ): Promise<void> {
    try {
      await installationsApi.duplicateInstallation(installation);
      console.log("Duplicated installation:", installation);
      LogsService.emitLauncherEvent(
        `Duplicated installation ${installation.name}`,
        "info",
      );
      await this.loadInstallations();
    } catch (error) {
      console.error("Failed to duplicate installation:", error);
      LogsService.emitLauncherEvent(
        `Failed to duplicate installation ${installation.name}`,
        "error",
      );
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
    const totalPlaytimeMs = allInstallations.reduce(
      (sum, inst) => sum + inst.total_time_played_ms,
      0,
    );

    // Last played installation
    const lastPlayedInstallation = allInstallations
      .filter((inst) => inst.last_used)
      .sort(
        (a, b) =>
          new Date(b.last_used).getTime() - new Date(a.last_used).getTime(),
      )[0];

    // Most played installation
    const mostPlayedInstallation = allInstallations
      .filter((inst) => inst.total_time_played_ms > 0)
      .sort((a, b) => b.total_time_played_ms - a.total_time_played_ms)[0];

    // Total launches across all installations
    const totalLaunches = allInstallations.reduce(
      (sum, inst) => sum + inst.times_launched,
      0,
    );

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

    allInstallations.forEach((inst) => {
      const version = versionsList.find(
        (v) => v.version_id === inst.version_id,
      );
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
      averageLaunchesPerInstallation:
        allInstallations.length > 0
          ? Math.round(totalLaunches / allInstallations.length)
          : 0,
      favoriteCount: allInstallations.filter((inst) => inst.favorite).length,
      loaderCounts,
      mostUsedLoader,
    };
  }

  /**
   * Create a desktop shortcut for an installation.
   * @param installation The installation to create a shortcut for.
   * @returns The path to the created shortcut.
   */
  static async createShortcut(
    installation: KableInstallation,
  ): Promise<string> {
    return await installationsApi.createShortcut(installation);
  }
}
