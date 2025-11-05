// src/lib/stores/mods.ts
import {
  type ModInfoKind,
  ProviderKind,
  type ModFilter,
  type KableInstallation,
  type ExtendedModInfo,
} from "$lib";
import { writable } from "svelte/store";

// Holds mods for each provider (e.g. { Modrinth: ModInfoKind[] })
export const modsByProvider = writable<{ [provider: string]: ModInfoKind[] }>(
  {},
);
// Loading state per provider (optional: can be a single boolean if only one provider is active at a time)
export const modsLoading = writable<boolean>(false);
export const modsError = writable<string | null>(null);
// Pagination and filter state
export const modsLimit = writable<number>(20);
export const modsOffset = writable<number>(0);
export const modsFilter = writable<ModFilter | null>(null);
export const modsInstallation = writable<KableInstallation | null>(null);
export const modsProvider = writable<ProviderKind | null>(null); // if we default here to an Enum value then the tauri app does not open
export const extendedModInfo = writable<{
  [modId: string]: ExtendedModInfo | null;
}>({});
