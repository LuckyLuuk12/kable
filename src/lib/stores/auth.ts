import { writable, derived } from 'svelte/store';
import type { MicrosoftAccount } from '../types';

/**
 * Authentication Stores
 * Svelte stores for authentication state management
 */

// Core authentication state
export const currentAccount = writable<MicrosoftAccount | null>(null);
export const isAuthenticated = derived(currentAccount, $account => $account !== null);
export const isAuthenticating = writable(false);
export const availableAccounts = writable<MicrosoftAccount[]>([]);

// Convenience stores
export const isSignedIn = isAuthenticated;
