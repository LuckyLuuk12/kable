// src/lib/stores/resourcepacks.ts
import { type ResourcePackDownload, type ResourcePack, type KableInstallation } from '$lib';
import { writable, derived } from 'svelte/store';

// Holds resourcepack downloads from Modrinth
export const resourcepackDownloads = writable<ResourcePackDownload[]>([]);

// Holds installed resource packs
export const installedResourcepacks = writable<ResourcePack[]>([]);

// Loading state
export const resourcepacksLoading = writable<boolean>(false);
export const resourcepacksError = writable<string | null>(null);

// Pagination and filter state
export const resourcepacksLimit = writable<number>(20);
export const resourcepacksOffset = writable<number>(0);

// Current installation for resourcepack browsing (null = global)
export const resourcepacksInstallation = writable<KableInstallation | null>(null);

// Installation mode: 'dedicated' or 'global'
export const resourcepacksInstallMode = writable<'dedicated' | 'global'>('dedicated');

// Create a "global installation" fake entry for global resourcepack management
export const GLOBAL_INSTALLATION: KableInstallation = {
  id: '__global__',
  name: 'Global (All Installations)',
  version_id: '',
  created: new Date().toISOString(),
  last_used: new Date().toISOString(),
  java_args: [],
  favorite: false,
  total_time_played_ms: 0,
  parameters_map: {},
  times_launched: 0
};

// Derived store that adds the global installation to the list
export const resourcepacksInstallationList = derived(
  resourcepacksInstallation,
  ($resourcepacksInstallation) => {
    // Return array with global installation always first
    return [GLOBAL_INSTALLATION];
  }
);
