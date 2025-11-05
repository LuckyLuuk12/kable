// src/lib/stores/shaders.ts
import { type ShaderDownload, type ShaderPack, type KableInstallation } from '$lib';
import { writable, derived } from 'svelte/store';

// Holds shader downloads from Modrinth
export const shaderDownloads = writable<ShaderDownload[]>([]);

// Holds installed shader packs
export const installedShaders = writable<ShaderPack[]>([]);

// Loading state
export const shadersLoading = writable<boolean>(false);
export const shadersError = writable<string | null>(null);

// Pagination and filter state
export const shadersLimit = writable<number>(20);
export const shadersOffset = writable<number>(0);

// Current installation for shader browsing (null = global)
export const shadersInstallation = writable<KableInstallation | null>(null);

// Installation mode: 'dedicated' or 'global'
export const shadersInstallMode = writable<'dedicated' | 'global'>('dedicated');

// Create a "global installation" fake entry for global shader management
export const GLOBAL_SHADER_INSTALLATION: KableInstallation = {
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
export const shadersInstallationList = derived(
  shadersInstallation,
  ($shadersInstallation) => {
    // Return array with global installation always first
    return [GLOBAL_SHADER_INSTALLATION];
  }
);
