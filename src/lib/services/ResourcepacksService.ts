import { invoke } from "@tauri-apps/api/core";
import { get } from "svelte/store";
import type {
  ResourcePack,
  ResourcePackDownload,
  KableInstallation,
  ResourcePackFilterFacets,
} from "../types";
import { settings } from "../stores/settings";
import {
  resourcepackDownloads,
  resourcepacksLoading,
  resourcepacksError,
  resourcepacksLimit,
  resourcepacksOffset,
  resourcepacksInstallation,
  resourcepacksInstallMode,
} from "../stores/resourcepacks";
import * as ResourcepacksAPI from "../api/resourcepacks";
import { NotificationService } from "./NotificationService";

export interface ResourcePackFilter {
  resolution?: ("16x" | "32x" | "64x" | "128x" | "256x" | "512x" | "1024x")[];
  categories?: string[];
  minecraftVersion?: string;
  searchTerm?: string;
}

export class ResourcepacksService {
  private static minecraftPath: string | null = null;
  private currentFilter: ResourcePackFilterFacets | null = null;
  initialized = false;

  constructor() {
    console.log("[ResourcepacksService] Initializing resourcepack service");
  }

  async initialize() {
    if (this.initialized) return;

    const currentSettings = get(settings);
    ResourcepacksService.minecraftPath =
      currentSettings.general.game_directory || null;

    await this.loadResourcepacks();
    this.initialized = true;
  }

  async loadResourcepacks() {
    resourcepacksLoading.set(true);
    resourcepacksError.set(null);
    const offset = get(resourcepacksOffset);
    const limit = get(resourcepacksLimit);

    console.log(
      `[ResourcepacksService] Loading resourcepacks (offset: ${offset}, limit: ${limit})`,
    );

    try {
      // Search Modrinth for resource packs with current filter
      const packs = this.currentFilter
        ? await ResourcepacksAPI.searchModrinthResourcepacksWithFacets(
            "",
            null,
            this.currentFilter,
            limit,
            offset,
          )
        : await ResourcepacksAPI.searchModrinthResourcepacks(
            "",
            null,
            limit,
            offset,
          );
      console.log(
        `[ResourcepacksService] Successfully loaded ${packs.length} resourcepacks`,
      );
      resourcepackDownloads.set(packs);
    } catch (e: any) {
      console.error(
        "[ResourcepacksService] Failed to load resourcepacks:",
        e.message || "Unknown error",
      );
      resourcepacksError.set(e.message || "Failed to load resourcepacks");
    } finally {
      resourcepacksLoading.set(false);
    }
  }

  async setFilter(filter: ResourcePackFilterFacets | null) {
    this.currentFilter = filter;
    console.log(
      "[ResourcepacksService] Setting resourcepack filter:",
      JSON.stringify(filter, null, 2),
    );
    resourcepacksOffset.set(0); // Reset to first page
    await this.loadResourcepacks();
  }

  async setLimit(limit: number) {
    resourcepacksLimit.set(limit);
    console.log(
      `[ResourcepacksService] Setting resourcepack limit to ${limit}`,
    );
    await this.loadResourcepacks();
  }

  async nextPage() {
    const limit = get(resourcepacksLimit);
    const offset = get(resourcepacksOffset) + limit;

    console.log(
      `[ResourcepacksService] Moving to next page (new offset: ${offset})`,
    );

    resourcepacksOffset.set(offset);
    await this.loadResourcepacks();
  }

  async prevPage() {
    const limit = get(resourcepacksLimit);
    const offset = Math.max(0, get(resourcepacksOffset) - limit);

    console.log(
      `[ResourcepacksService] Moving to previous page (new offset: ${offset})`,
    );

    resourcepacksOffset.set(offset);
    await this.loadResourcepacks();
  }

  async searchResourcepacks(query: string) {
    resourcepacksLoading.set(true);
    resourcepacksError.set(null);
    const limit = get(resourcepacksLimit);

    console.log(`[ResourcepacksService] Searching resourcepacks: "${query}"`);

    try {
      const packs = this.currentFilter
        ? await ResourcepacksAPI.searchModrinthResourcepacksWithFacets(
            query,
            null,
            this.currentFilter,
            limit,
            0,
          )
        : await ResourcepacksAPI.searchModrinthResourcepacks(
            query,
            null,
            limit,
            0,
          );
      console.log(
        `[ResourcepacksService] Found ${packs.length} resourcepacks matching "${query}"`,
      );
      resourcepackDownloads.set(packs);
      resourcepacksOffset.set(0); // Reset to first page on new search
    } catch (e: any) {
      console.error(
        "[ResourcepacksService] Failed to search resourcepacks:",
        e.message || "Unknown error",
      );
      resourcepacksError.set(e.message || "Failed to search resourcepacks");
    } finally {
      resourcepacksLoading.set(false);
    }
  }

  async downloadResourcepack(
    pack: ResourcePackDownload,
    installation: KableInstallation | null,
  ) {
    resourcepacksLoading.set(true);
    resourcepacksError.set(null);
    const mode = get(resourcepacksInstallMode);

    const isGlobal =
      !installation || installation.id === "__global__" || mode === "global";

    console.log(
      `[ResourcepacksService] Downloading resourcepack "${pack.name}" to ${isGlobal ? "global" : installation?.name || "unknown"}`,
    );

    try {
      if (isGlobal) {
        await ResourcepacksService.downloadResourcepackGlobal(pack);
      } else if (installation) {
        await ResourcepacksService.downloadResourcepackToDedicated(
          pack,
          installation,
        );
      } else {
        throw new Error(
          "No installation selected for dedicated resourcepack installation",
        );
      }
      console.log(
        `[ResourcepacksService] Successfully downloaded resourcepack "${pack.name}"`,
      );
      NotificationService.success(`Resource pack "${pack.name}" downloaded`);
    } catch (e: any) {
      console.error(
        `[ResourcepacksService] Failed to download resourcepack "${pack.name}":`,
        e.message || "Unknown error",
      );
      const errorMsg = e.message || "Failed to download resourcepack";
      resourcepacksError.set(errorMsg);
      NotificationService.error(
        `Failed to download resource pack: ${errorMsg}`,
      );
      throw e;
    } finally {
      resourcepacksLoading.set(false);
    }
  }

  // Helpers for UI
  getResourcepacks() {
    return get(resourcepackDownloads);
  }

  isLoading() {
    return get(resourcepacksLoading);
  }

  getError() {
    return get(resourcepacksError);
  }

  static async ensureInitialized() {
    if (!this.minecraftPath) {
      const currentSettings = get(settings);
      this.minecraftPath = currentSettings.general.game_directory || null;
    }
  }

  static async getInstalledResourcepacks(): Promise<ResourcePack[]> {
    await this.ensureInitialized();
    if (!this.minecraftPath) {
      throw new Error("Minecraft directory not configured");
    }
    return ResourcepacksAPI.getInstalledResourcepacks(this.minecraftPath);
  }

  static async searchResourcepacks(
    query: string,
    minecraftVersion: string | null,
    limit: number,
    offset: number,
  ): Promise<ResourcePackDownload[]> {
    return ResourcepacksAPI.searchModrinthResourcepacks(
      query,
      minecraftVersion,
      limit,
      offset,
    );
  }

  static async downloadResourcepackGlobal(
    pack: ResourcePackDownload,
  ): Promise<string> {
    await this.ensureInitialized();
    if (!this.minecraftPath) {
      throw new Error("Minecraft directory not configured");
    }

    // Extract filename from download URL
    const filename = pack.download_url.split("/").pop() || `${pack.name}.zip`;

    return ResourcepacksAPI.downloadAndInstallResourcepack(
      this.minecraftPath,
      pack.download_url,
      filename,
    );
  }

  static async downloadResourcepackToDedicated(
    pack: ResourcePackDownload,
    installation: KableInstallation,
  ): Promise<string> {
    await this.ensureInitialized();
    if (!this.minecraftPath) {
      throw new Error("Minecraft directory not configured");
    }

    const dedicatedFolder =
      installation.dedicated_resource_pack_folder || installation.id;
    const filename = pack.download_url.split("/").pop() || `${pack.name}.zip`;

    // Download to dedicated folder
    // Note: Symlinks are managed dynamically by symlink_manager when launching the game
    await ResourcepacksAPI.downloadAndInstallResourcepackToDedicated(
      this.minecraftPath,
      dedicatedFolder,
      pack.download_url,
      filename,
    );

    return filename;
  }

  static async deleteResourcepack(packFile: string): Promise<void> {
    await this.ensureInitialized();
    if (!this.minecraftPath) {
      throw new Error("Minecraft directory not configured");
    }
    try {
      await ResourcepacksAPI.deleteResourcepack(this.minecraftPath, packFile);
      NotificationService.success(`Resource pack deleted`);
    } catch (error) {
      NotificationService.error(`Failed to delete resource pack: ${error}`);
      throw error;
    }
  }

  static async deleteResourcepackFromDedicated(
    packFile: string,
    installation: KableInstallation,
  ): Promise<void> {
    await this.ensureInitialized();
    if (!this.minecraftPath) {
      throw new Error("Minecraft directory not configured");
    }

    const dedicatedFolder =
      installation.dedicated_resource_pack_folder || installation.id;
    const symlinkName = installation.id;

    try {
      await ResourcepacksAPI.deleteResourcepackFromDedicated(
        this.minecraftPath,
        dedicatedFolder,
        packFile,
        symlinkName,
      );
      NotificationService.success(`Resource pack deleted`);
    } catch (error) {
      NotificationService.error(`Failed to delete resource pack: ${error}`);
      throw error;
    }
  }

  static filterResourcepacks(
    packs: ResourcePack[],
    filters: {
      enabled?: boolean;
      minSize?: number;
      maxSize?: number;
      searchTerm?: string;
    },
  ): ResourcePack[] {
    return packs.filter((pack) => {
      if (filters.enabled !== undefined && pack.enabled !== filters.enabled) {
        return false;
      }
      if (filters.minSize && pack.file_size < filters.minSize) {
        return false;
      }
      if (filters.maxSize && pack.file_size > filters.maxSize) {
        return false;
      }
      if (filters.searchTerm) {
        const term = filters.searchTerm.toLowerCase();
        if (
          !pack.name.toLowerCase().includes(term) &&
          !pack.author.toLowerCase().includes(term) &&
          !pack.file_name.toLowerCase().includes(term)
        ) {
          return false;
        }
      }
      return true;
    });
  }

  static filterResourcepackDownloads(
    packs: ResourcePackDownload[],
    filters: ResourcePackFilter,
  ): ResourcePackDownload[] {
    return packs.filter((pack) => {
      // Resolution filter
      if (filters.resolution && filters.resolution.length > 0) {
        if (
          !pack.resolution ||
          !filters.resolution.includes(pack.resolution as any)
        ) {
          return false;
        }
      }

      // Categories filter (based on tags)
      if (filters.categories && filters.categories.length > 0) {
        const hasCategory = filters.categories.some((category) =>
          pack.tags.some((tag) => tag.toLowerCase() === category.toLowerCase()),
        );
        if (!hasCategory) return false;
      }

      // Minecraft version filter
      if (filters.minecraftVersion) {
        if (!pack.minecraft_versions.includes(filters.minecraftVersion)) {
          return false;
        }
      }

      // Search term filter
      if (filters.searchTerm) {
        const term = filters.searchTerm.toLowerCase();
        if (
          !pack.name.toLowerCase().includes(term) &&
          !pack.author.toLowerCase().includes(term) &&
          !pack.description.toLowerCase().includes(term)
        ) {
          return false;
        }
      }

      return true;
    });
  }
}
