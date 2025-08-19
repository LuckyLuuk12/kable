import { invoke } from '@tauri-apps/api/core';
import type { SkinUploadConfig, SkinUploadResponse, CurrentSkin, SkinModelType, AccountSkin } from '../types';

/**
 * Skins API
 * Pure Tauri invoke calls for Minecraft skin management
 */

// Skin management functions

/**
 * Upload a skin file to the authenticated Microsoft/Mojang account
 */
export async function uploadSkinToAccount(config: SkinUploadConfig): Promise<SkinUploadResponse> {
  return await invoke('upload_skin_to_account', { config });
}

/**
 * Change the skin model (slim/classic) for the current skin
 */
export async function changeSkinModel(newModel: SkinModelType): Promise<SkinUploadResponse> {
  return await invoke('change_skin_model', { newModel });
}

/**
 * Get the current skin information from Mojang
 */
export async function getCurrentSkinInfo(): Promise<CurrentSkin> {
  return await invoke('get_current_skin_info');
}

/**
 * Select a skin file using the system file dialog
 * @returns The selected file path, or null if cancelled
 */
export async function selectSkinFile(): Promise<string | null> {
  return await invoke('select_skin_file');
}

/**
 * Get all skins stored in the user's Microsoft/Mojang account
 */
export async function getAllAccountSkins(): Promise<AccountSkin[]> {
  return await invoke('get_all_account_skins');
}

/**
 * Apply an account skin (set it as the current skin)
 */
export async function applyAccountSkin(skinId: string): Promise<SkinUploadResponse> {
  return await invoke('apply_account_skin', { skinId });
}
