import { invoke } from '@tauri-apps/api/core';
import type { LauncherAccount, LauncherAccountsJson } from '../types';

/**
 * Authentication API
 * Pure Tauri invoke calls for Microsoft authentication and account management
 */

// Modern Authentication Flows
export interface AuthCodeResponse {
  auth_url: string;
  state: string;
  local_server_port: number;
}

export interface DeviceCodeResponse {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
}

export interface MicrosoftToken {
  access_token: string;
  expires_at: string;
}

// ...existing code...

// Authentication Methods
export type AuthMethod = 'DeviceCodeFlow' | 'AuthCodeFlow' | 'Custom' | 'Offline';

// Main Authentication Functions
export async function getMinecraftAccount(authMethod?: AuthMethod): Promise<LauncherAccount> {
  return await invoke('get_minecraft_account', { authMethod });
}

export async function getLaunchAuthAccount(): Promise<LauncherAccount> {
  return await invoke('get_launch_auth_account');
}

export async function refreshMinecraftAccount(): Promise<LauncherAccount> {
  return await invoke('refresh_minecraft_account');
}

// Authorization Code Flow (Recommended for Desktop)
export async function startMicrosoftAuthCode(): Promise<AuthCodeResponse> {
  return await invoke('start_microsoft_auth_code');
}

export async function completeMicrosoftAuthCode(microsoftToken: MicrosoftToken): Promise<LauncherAccount> {
  return await invoke('complete_minecraft_auth_code', { microsoftToken });
}

export async function pollMicrosoftAuthCode(state: string): Promise<MicrosoftToken | null> {
  return await invoke('poll_microsoft_auth_code', { state });
}

// Device Code Flow (Fallback)
export async function startMicrosoftDeviceAuth(): Promise<DeviceCodeResponse> {
  return await invoke('start_microsoft_device_auth');
}

export async function pollMicrosoftDeviceAuth(deviceCode: string): Promise<MicrosoftToken | null> {
  return await invoke('poll_microsoft_device_auth', { deviceCode });
}

export async function completeMicrosoftAuth(microsoftToken: MicrosoftToken): Promise<LauncherAccount> {
  return await invoke('complete_minecraft_auth', { microsoftToken });
}

// Account Management (launcher_accounts.json)
export async function readLauncherAccounts(): Promise<LauncherAccountsJson> {
  return invoke('read_launcher_accounts');
}

export async function writeLauncherAccount(account: LauncherAccount): Promise<void> {
  return invoke('write_launcher_account', { account });
}

export async function removeLauncherAccount(accountId: string): Promise<void> {
  return invoke('remove_launcher_account', { accountId });
}

export async function setActiveLauncherAccount(accountId: string): Promise<void> {
  return invoke('set_active_launcher_account', { accountId });
}

export async function getActiveLauncherAccount(): Promise<LauncherAccount | null> {
  return invoke('get_active_launcher_account');
}

export async function getAllLauncherAccounts(): Promise<LauncherAccount[]> {
  return invoke('get_all_launcher_accounts');
}

// Utility Functions for Type Conversion

// Removed convertToMinecraftAccount and convertToLauncherAccount. Use LauncherAccount directly.

/**
 * Check if an account is a valid account (not a fallback state)
 * This includes legitimate offline accounts from launcher_accounts.json
 */
export function isValidAuthenticatedAccount(account: LauncherAccount): boolean {
  // Validate using LauncherAccount fields only
  return !!(
    account.minecraft_profile?.id &&
    account.minecraft_profile.id !== '00000000-0000-0000-0000-000000000000' &&
    account.minecraft_profile?.name &&
    account.minecraft_profile.name.trim() !== ''
  );
}

// Legacy API for backward compatibility (deprecated)
export async function startMicrosoftAuth(): Promise<string> {
  console.warn('⚠️ startMicrosoftAuth is deprecated, use startMicrosoftAuthCode instead');
  const response = await startMicrosoftAuthCode();
  return response.auth_url;
}

export async function getOAuthCallbackResult(): Promise<string | null> {
  console.warn('⚠️ getOAuthCallbackResult is deprecated, auth code flow handles this automatically');
  return null;
}

export async function startDeviceCodeAuth(): Promise<string> {
  console.warn('⚠️ startDeviceCodeAuth is deprecated, use startMicrosoftDeviceAuth instead');
  const response = await startMicrosoftDeviceAuth();
  return response.user_code;
}

export async function pollDeviceCodeAuth(): Promise<LauncherAccount | null> {
  console.warn('⚠️ pollDeviceCodeAuth is deprecated, use pollMicrosoftDeviceAuth instead');
  return null;
}

export async function validateMinecraftToken(accessToken: string): Promise<boolean> {
  console.warn('⚠️ validateMinecraftToken is deprecated, token validation is built into getMinecraftAccount');
  return false;
}
