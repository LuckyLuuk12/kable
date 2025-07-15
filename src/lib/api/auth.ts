import { invoke } from '@tauri-apps/api/core';
import type { MicrosoftAccount, LauncherAccountsJson } from '../types';

/**
 * Authentication API
 * Pure Tauri invoke calls for Microsoft authentication and account management
 */

// Microsoft OAuth Authentication
export async function startMicrosoftAuth(): Promise<string> {
  return await invoke('start_microsoft_auth');
}

export async function completeMicrosoftAuth(authCode: string): Promise<MicrosoftAccount> {
  return await invoke('complete_microsoft_auth', { authCode });
}

export async function getOAuthCallbackResult(): Promise<string | null> {
  return await invoke('get_oauth_callback_result');
}

// Device Code Flow Authentication
export async function startDeviceCodeAuth(): Promise<string> {
  return await invoke('start_device_code_auth');
}

export async function pollDeviceCodeAuth(): Promise<MicrosoftAccount | null> {
  return await invoke('poll_device_code_auth');
}

// Token Management
export async function refreshMinecraftToken(accountId: string): Promise<MicrosoftAccount> {
  return await invoke('refresh_minecraft_token', { accountId });
}

export async function validateMinecraftToken(accessToken: string): Promise<boolean> {
  return await invoke('validate_minecraft_token', { accessToken });
}

// Account Management (launcher_accounts.json)
export async function readLauncherAccounts(): Promise<LauncherAccountsJson> {
  return invoke('read_launcher_accounts');
}

export async function writeLauncherAccount(account: MicrosoftAccount): Promise<void> {
  return invoke('write_launcher_account', { account });
}

export async function removeLauncherAccount(accountId: string): Promise<void> {
  return invoke('remove_launcher_account', { accountId });
}

export async function setActiveLauncherAccount(accountId: string): Promise<void> {
  return invoke('set_active_launcher_account', { accountId });
}

export async function getActiveLauncherAccount(): Promise<MicrosoftAccount | null> {
  return invoke('get_active_launcher_account');
}

export async function getAllLauncherAccounts(): Promise<MicrosoftAccount[]> {
  return invoke('get_all_launcher_accounts');
}
