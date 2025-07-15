import { writable } from 'svelte/store';
import type { MinecraftInstallation } from '../types';

/**
 * Game Stores
 * Svelte stores for game and installation state management
 */

// Installation state
export const installations = writable<MinecraftInstallation[]>([]);
export const selectedInstallation = writable<MinecraftInstallation | null>(null);
export const isLoadingInstallations = writable(false);
export const installationsError = writable<string | null>(null);

// Launch state
export const isLaunching = writable(false);
export const launchError = writable<string | null>(null);

// Java state
export const javaStatus = writable<string>('Checking...');
