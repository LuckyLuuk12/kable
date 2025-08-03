import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { currentAccount } from '../stores/auth';
import type { MinecraftSkin, SkinDownload } from '../types';

export class SkinsService {
  /**
   * Get all local skins (alias for getSkins)
   */
  static async getSkins(): Promise<MinecraftSkin[]> {
    return await invoke('get_local_skins');
  }

  /**
   * Apply skin to player account
   */
  static async applySkin(skinId: string): Promise<void> {
    const account = get(currentAccount);
    if (!account) {
      throw new Error('No account selected');
    }
    const skins = await SkinsService.getSkins();
    const skin = skins.find(s => s.file_name === skinId);
    if (!skin) {
      throw new Error('Skin not found');
    }
    await invoke('upload_skin_to_minecraft', {
      accessToken: account.access_token,
      skinFile: skin.file_name,
      isSlim: skin.is_slim
    });
  }

  /**
   * Upload new skin (opens file dialog)
   */
  static async uploadSkin(): Promise<MinecraftSkin | null> {
    // Use Tauri dialog to select file
    const skinFilePath = await invoke<string | null>('select_skin_file');
    if (!skinFilePath) {
      return null; // User cancelled
    }
    // Validate the file
    const fileName = skinFilePath.split(/[/\\]/).pop() || '';
    const extension = fileName.split('.').pop()?.toLowerCase();
    if (!extension || extension !== 'png') {
      throw new Error('Skin files must be in PNG format');
    }
    // Install the skin
    const installedFileName = await invoke<string>('install_skin', { skinFilePath });
    // Get the newly installed skin
    const skins = await SkinsService.getSkins();
    return skins.find(skin => skin.file_name === installedFileName) || null;
  }

  /**
   * Delete a skin from local storage
   */
  static async deleteSkin(skinFile: string): Promise<void> {
    await invoke('delete_skin', { skinFile });
  }
}
