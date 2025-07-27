import { writable } from 'svelte/store';
import type { KableInstallation, VersionData } from '../types';

/**
 * Game Stores
 * Svelte stores for game and installation state management
 */

// Installation state
export const installations = writable<KableInstallation[]>([]);
export const selectedInstallation = writable<KableInstallation | null>(null);
export const isLoadingInstallations = writable(false);
export const installationsError = writable<string | null>(null);

export const versions = writable<VersionData[]>([]);
export const isLoadingVersions = writable(false);
export const versionsError = writable<string | null>(null);