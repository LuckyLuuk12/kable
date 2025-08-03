import * as modsApi from '../api/mods';
import { get } from 'svelte/store';
import { modsByProvider, modsLoading, modsError, modsLimit, modsOffset, modsFilter, modsInstallation, modsProvider } from '$lib';
import type { ProviderKind, ModInfoKind, KableInstallation, ModFilter, ModJarInfo } from '$lib';

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
}
