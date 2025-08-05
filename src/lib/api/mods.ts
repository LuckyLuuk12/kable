import { invoke } from '@tauri-apps/api/core';
import type { ExtendedModInfo, KableInstallation, ModFilter, ModInfoKind, ModJarInfo, ProviderKind } from '$lib';

export async function getMods(provider: ProviderKind, offset: number): Promise<ModInfoKind[]> {
  return invoke('get_mods', { provider, offset });
}

export async function downloadMod(provider: ProviderKind, modId: string, versionId: string | null, installation: KableInstallation): Promise<void> {
  return invoke('download_mod', { provider, modId, versionId, installation });
}

export async function setProviderFilter(provider: ProviderKind, installation: KableInstallation | null, filter: ModFilter | null): Promise<void> {
  return invoke('set_provider_filter', { provider, installation, filter });
}

export async function setProviderLimit(provider: ProviderKind, limit: number): Promise<void> {
  return invoke('set_provider_limit', { provider, limit });
}

export async function clearProviderCache(provider: ProviderKind): Promise<void> {
  return invoke('clear_provider_cache', { provider });
}

export async function purgeStaleProviderCache(provider: ProviderKind): Promise<void> {
  return invoke('purge_stale_provider_cache', { provider });
}

export async function getExtendedModInfo(modJarInfo: ModJarInfo): Promise<ExtendedModInfo> {
  return invoke('get_extended_mod_info', { modJarInfo });
}