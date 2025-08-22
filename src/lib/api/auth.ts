import { invoke } from '@tauri-apps/api/core';
import type { LauncherAccount, LauncherAccountsJson, MicrosoftToken, AuthCodeResponse, DeviceCodeResponse, AuthMethod } from '../types';

/**
 * Authentication API
 * Pure Tauri invoke calls for Microsoft authentication and account management
 */


// Refresh Microsoft token using backend
export async function refreshMicrosoftToken(localId: string): Promise<LauncherAccount> {
  return await invoke('refresh_microsoft_token', { localId });
}


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

