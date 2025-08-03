import { invoke } from '@tauri-apps/api/core';
import type { KableInstallation, ModFilter, ModInfoKind, ProviderKind } from '$lib';

export async function getMods(provider: ProviderKind, offset: number): Promise<ModInfoKind[]> {
  return invoke('get_mods_command', { provider, offset });
}

export async function downloadMod(provider: ProviderKind, modId: string, versionId: string | null, targetDir: string): Promise<void> {
  return invoke('download_mod_command', { provider, modId, versionId, targetDir });
}

export async function setProviderFilter(provider: ProviderKind, installation: KableInstallation | null, filter: ModFilter | null): Promise<void> {
  return invoke('set_provider_filter_command', { provider, installation, filter });
}

export async function setProviderLimit(provider: ProviderKind, limit: number): Promise<void> {
  return invoke('set_provider_limit_command', { provider, limit });
}

export async function clearProviderCache(provider: ProviderKind): Promise<void> {
  return invoke('clear_provider_cache_command', { provider });
}

export async function purgeStaleProviderCache(provider: ProviderKind): Promise<void> {
  return invoke('purge_stale_provider_cache_command', { provider });
}
