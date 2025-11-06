import * as modsApi from "../api/mods";
import { get } from "svelte/store";
import {
  modsByProvider,
  modsLoading,
  modsError,
  modsLimit,
  modsOffset,
  modsFilter,
  modsInstallation,
  modsProvider,
  extendedModInfo,
  NotificationService,
} from "$lib";
import type {
  ProviderKind,
  ModInfoKind,
  KableInstallation,
  ModFilter,
  ModJarInfo,
  ExtendedModInfo,
} from "$lib";

export class ModsService {
  initialized = false;

  constructor(provider: ProviderKind) {
    console.log(`[ModsService] Initializing with ${provider} provider`);
    modsProvider.set(provider);
  }

  async initialize() {
    if (this.initialized) return;
    await this.loadMods();
    this.initialized = true;
  }

  async loadMods() {
    modsLoading.set(true);
    modsError.set(null);
    const provider = get(modsProvider);
    const offset = get(modsOffset);
    const filter = get(modsFilter);
    const installation = get(modsInstallation);

    if (!provider) {
      modsError.set("No provider selected");
      modsLoading.set(false);
      return;
    }

    // Log the request details
    console.log(
      `[ModsService] Loading mods from ${provider} provider (offset: ${offset}, filter: ${filter ? JSON.stringify(filter) : "none"}, installation: ${installation?.name || "none"})`,
    );

    try {
      const mods = await modsApi.getMods(provider, offset);
      console.log(
        `[ModsService] Successfully loaded ${mods.length} mods from ${provider} provider`,
      );
      modsByProvider.update((map) => ({ ...map, [provider]: mods }));
    } catch (e: any) {
      console.error(
        `[ModsService] Failed to load mods from ${provider} provider:`,
        e.message || "Unknown error",
      );
      modsError.set(e.message || "Failed to load mods");
    } finally {
      modsLoading.set(false);
    }
  }

  async setLimit(limit: number) {
    modsLimit.set(limit);
    const provider = get(modsProvider);
    if (!provider) {
      modsError.set("No provider selected");
      return;
    }

    console.log(
      `[ModsService] Setting mod limit to ${limit} for ${provider} provider`,
    );

    try {
      await modsApi.setProviderLimit(provider, limit);
      await this.loadMods();
    } catch (e: any) {
      console.error(
        `[ModsService] Failed to set limit for ${provider} provider:`,
        e.message || "Unknown error",
      );
      throw e;
    }
  }

  async setFilter(
    filter: ModFilter | null,
    installation: KableInstallation | null,
  ) {
    modsFilter.set(filter);
    modsInstallation.set(installation);
    const provider = get(modsProvider);
    if (!provider) {
      modsError.set("No provider selected");
      return;
    }

    console.log(
      `[ModsService] Setting filter for ${provider} provider: ${filter ? JSON.stringify(filter) : "none"} (installation: ${installation?.name || "none"})`,
    );

    try {
      await modsApi.setProviderFilter(provider, installation, filter);
      await this.loadMods();
    } catch (e: any) {
      console.error(
        `[ModsService] Failed to set filter for ${provider} provider:`,
        e.message || "Unknown error",
      );
      throw e;
    }
  }

  async nextPage() {
    const limit = get(modsLimit);
    const offset = get(modsOffset) + limit;
    const provider = get(modsProvider);

    console.log(
      `[ModsService] Moving to next page for ${provider} provider (new offset: ${offset})`,
    );

    modsOffset.set(offset);
    await this.loadMods();
  }

  async prevPage() {
    const limit = get(modsLimit);
    const offset = Math.max(0, get(modsOffset) - limit);
    const provider = get(modsProvider);

    console.log(
      `[ModsService] Moving to previous page for ${provider} provider (new offset: ${offset})`,
    );

    modsOffset.set(offset);
    await this.loadMods();
  }

  async downloadMod(
    modId: string,
    versionId: string | null,
    installation: KableInstallation,
  ) {
    modsLoading.set(true);
    modsError.set(null);
    const provider = get(modsProvider);
    if (!provider) {
      modsError.set("No provider selected");
      modsLoading.set(false);
      return;
    }

    console.log(
      `[ModsService] Downloading mod ${modId}${versionId ? ` (version: ${versionId})` : ""} from ${provider} provider to installation "${installation.name}"`,
    );

    try {
      await modsApi.downloadMod(provider, modId, versionId, installation);
      console.log(
        `[ModsService] Successfully downloaded mod ${modId} from ${provider} provider`,
      );
      NotificationService.success(`Mod downloaded successfully`);
    } catch (e: any) {
      console.error(
        `[ModsService] Failed to download mod ${modId} from ${provider} provider to ${installation.dedicated_mods_folder}:`,
        e.message || e || "Unknown error",
      );
      const errorMsg = e.message || "Failed to download mod";
      modsError.set(errorMsg);
      NotificationService.error(`Failed to download mod: ${errorMsg}`);
    } finally {
      modsLoading.set(false);
    }
  }

  async clearProviderCache() {
    const provider = get(modsProvider);
    if (!provider) {
      modsError.set("No provider selected");
      return;
    }

    console.log(`[ModsService] Clearing cache for ${provider} provider`);

    try {
      await modsApi.clearProviderCache(provider);
      await this.loadMods();
      console.log(
        `[ModsService] Successfully cleared cache for ${provider} provider`,
      );
      NotificationService.success(`Cache cleared for ${provider}`);
    } catch (e: any) {
      console.error(
        `[ModsService] Failed to clear cache for ${provider} provider:`,
        e.message || "Unknown error",
      );
      NotificationService.error(
        `Failed to clear cache: ${e.message || "Unknown error"}`,
      );
      throw e;
    }
  }

  async purgeStaleProviderCache() {
    const provider = get(modsProvider);
    if (!provider) {
      modsError.set("No provider selected");
      return;
    }

    console.log(`[ModsService] Purging stale cache for ${provider} provider`);

    try {
      await modsApi.purgeStaleProviderCache(provider);
      await this.loadMods();
      console.log(
        `[ModsService] Successfully purged stale cache for ${provider} provider`,
      );
    } catch (e: any) {
      console.error(
        `[ModsService] Failed to purge stale cache for ${provider} provider:`,
        e.message || "Unknown error",
      );
      throw e;
    }
  }

  // Helpers for UI
  getMods() {
    const provider = get(modsProvider);
    if (!provider) return [];
    return get(modsByProvider)[provider] || [];
  }
  isLoading() {
    return get(modsLoading);
  }
  getError() {
    return get(modsError);
  }

  //  Concurrency-limited queue for extended mod info requests
  static #pending: (() => Promise<void>)[] = [];
  static #inFlight = 0;
  static #MAX_CONCURRENT = 3; // You can tune this (2-4 is safe)

  static async getExtendedModInfo(
    modJarInfo: ModJarInfo,
  ): Promise<ExtendedModInfo | null> {
    const fileName = modJarInfo.file_name;
    if (get(extendedModInfo)[fileName]) {
      return get(extendedModInfo)[fileName] || null;
    }

    console.log(`[ModsService] Fetching extended mod info for ${fileName}`);

    // Wrap the actual fetch in a function for the queue
    return new Promise<ExtendedModInfo | null>((resolve) => {
      const task = async () => {
        try {
          const result = await modsApi.getExtendedModInfo(modJarInfo);
          extendedModInfo.set({
            ...get(extendedModInfo),
            [fileName]: result,
          });
          console.log(
            `[ModsService] Successfully fetched extended mod info for ${fileName}`,
          );
          resolve(result || null);
        } catch (e) {
          // Store a null value in the store to prevent infinite retries
          extendedModInfo.set({
            ...get(extendedModInfo),
            [fileName]: null,
          });
          console.warn(
            `[ModsService] Failed to fetch extended mod info for ${fileName}:`,
            e instanceof Error ? e.message : "Unknown error",
          );
          // Optionally: log error, set error state, etc.
          console.warn(`Failed to fetch extended mod info for ${fileName}:`, e);
          resolve(null);
        } finally {
          ModsService.#inFlight--;
          ModsService.#runQueue();
        }
      };
      ModsService.#pending.push(task);
      ModsService.#runQueue();
    });
  }

  static #runQueue() {
    while (
      ModsService.#inFlight < ModsService.#MAX_CONCURRENT &&
      ModsService.#pending.length > 0
    ) {
      const next = ModsService.#pending.shift();
      if (next) {
        ModsService.#inFlight++;
        next();
      }
    }
  }
}
