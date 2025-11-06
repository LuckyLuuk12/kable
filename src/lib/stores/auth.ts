import { writable, derived } from "svelte/store";
import type { LauncherAccount } from "$lib";

/**
 * Authentication Stores
 * Svelte stores for authentication state management
 * Updated to use modern LauncherAccount type
 */

// Core authentication state
export const currentAccount = writable<LauncherAccount | null>(null);
export const isAuthenticated = derived(
  currentAccount,
  ($account) => $account !== null,
);
export const isAuthenticating = writable(false);
export const availableAccounts = writable<LauncherAccount[]>([]);

// Convenience stores
export const isSignedIn = isAuthenticated;
