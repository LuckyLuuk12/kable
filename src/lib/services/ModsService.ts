import * as modsApi from '../api/mods';
import { get } from 'svelte/store';
import { modsByProvider, modsLoading, modsError, modsLimit, modsOffset, modsFilter, modsInstallation, modsProvider, extendedModInfo } from '$lib';
import type { ProviderKind, ModInfoKind, KableInstallation, ModFilter, ModJarInfo, ExtendedModInfo } from '$lib';

export class ModsService {
  initialized = false;

  constructor(provider: ProviderKind) {
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
    if (!provider) {
      modsError.set('No provider selected');
      modsLoading.set(false);
      return;
    }
    try {
      const mods = await modsApi.getMods(provider, offset);
      modsByProvider.update(map => ({ ...map, [provider]: mods }));
    } catch (e: any) {
      modsError.set(e.message || 'Failed to load mods');
    } finally {
      modsLoading.set(false);
    }
  }

  async setLimit(limit: number) {
    modsLimit.set(limit);
    const provider = get(modsProvider);
    if (!provider) {
      modsError.set('No provider selected');
      return;
    }
    await modsApi.setProviderLimit(provider, limit);
    await this.loadMods();
  }

  async setFilter(filter: ModFilter | null, installation: KableInstallation | null) {
    modsFilter.set(filter);
    modsInstallation.set(installation);
    const provider = get(modsProvider);
    if (!provider) {
      modsError.set('No provider selected');
      return;
    }
    await modsApi.setProviderFilter(provider, installation, filter);
    await this.loadMods();
  }

  async nextPage() {
    const limit = get(modsLimit);
    const offset = get(modsOffset) + limit;
    modsOffset.set(offset);
    await this.loadMods();
  }

  async prevPage() {
    const limit = get(modsLimit);
    const offset = Math.max(0, get(modsOffset) - limit);
    modsOffset.set(offset);
    await this.loadMods();
  }

  async downloadMod(modId: string, versionId: string | null, installation: KableInstallation) {
    modsLoading.set(true);
    modsError.set(null);
    const provider = get(modsProvider);
    if (!provider) {
      modsError.set('No provider selected');
      modsLoading.set(false);
      return;
    }
    try {
      await modsApi.downloadMod(provider, modId, versionId, installation);
    } catch (e: any) {
      modsError.set(e.message || 'Failed to download mod');
    } finally {
      modsLoading.set(false);
    }
  }

  async clearProviderCache() {
    const provider = get(modsProvider);
    if (!provider) {
      modsError.set('No provider selected');
      return;
    }
    await modsApi.clearProviderCache(provider);
    await this.loadMods();
  }

  async purgeStaleProviderCache() {
    const provider = get(modsProvider);
    if (!provider) {
      modsError.set('No provider selected');
      return;
    }
    await modsApi.purgeStaleProviderCache(provider);
    await this.loadMods();
  }

  // Helpers for UI
  getMods() {
    const provider = get(modsProvider);
    if (!provider) return [];
    return get(modsByProvider)[provider] || [];
  }
  isLoading() { return get(modsLoading); }
  getError() { return get(modsError); }

  // --- Concurrency-limited queue for extended mod info requests ---
  static #pending: (() => Promise<void>)[] = [];
  static #inFlight = 0;
  static #MAX_CONCURRENT = 3; // You can tune this (2-4 is safe)

  static async getExtendedModInfo(modJarInfo: ModJarInfo): Promise<ExtendedModInfo | null> {
    const fileName = modJarInfo.file_name;
    if (get(extendedModInfo)[fileName]) {
      return get(extendedModInfo)[fileName] || null;
    }
    // Wrap the actual fetch in a function for the queue
    return new Promise<ExtendedModInfo | null>((resolve) => {
      const task = async () => {
        try {
          const result = await modsApi.getExtendedModInfo(modJarInfo);
          extendedModInfo.set({
            ...get(extendedModInfo),
            [fileName]: result
          });
          resolve(result || null);
        } catch (e) {
          // Store a null value in the store to prevent infinite retries
          extendedModInfo.set({
            ...get(extendedModInfo),
            [fileName]: null
          });
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
    while (ModsService.#inFlight < ModsService.#MAX_CONCURRENT && ModsService.#pending.length > 0) {
      const next = ModsService.#pending.shift();
      if (next) {
        ModsService.#inFlight++;
        next();
      }
    }
  }
}
