import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { currentAccount } from '../stores/auth';
import type { MinecraftSkin, SkinDownload } from '../types';

export class SkinsManager {
  /**
   * Get all local skins
   */
  static async getLocalSkins(): Promise<MinecraftSkin[]> {
    return invoke('get_local_skins');
  }

  /**
   * Save skin data to local storage
   */
  static async saveSkin(skinData: number[], skinName: string, isSlim: boolean): Promise<string> {
    return invoke('save_skin', { skinData, skinName, isSlim });
  }

  /**
   * Delete a skin from local storage
   */
  static async deleteSkin(skinFile: string): Promise<void> {
    return invoke('delete_skin', { skinFile });
  }

  /**
   * Install skin from file path
   */
  static async installSkin(skinFilePath: string, skinName?: string): Promise<string> {
    return invoke('install_skin', { skinFilePath, skinName });
  }

  /**
   * Get skin file data
   */
  static async getSkinData(skinFile: string): Promise<number[]> {
    return invoke('get_skin_data', { skinFile });
  }

  /**
   * Get current Minecraft skin from account
   */
  static async getCurrentMinecraftSkin(accessToken: string, uuid: string): Promise<string | null> {
    return invoke('get_current_minecraft_skin', { accessToken, uuid });
  }

  /**
   * Upload skin to Minecraft account
   */
  static async uploadSkinToMinecraft(accessToken: string, skinFile: string, isSlim: boolean): Promise<void> {
    return invoke('upload_skin_to_minecraft', { accessToken, skinFile, isSlim });
  }

  /**
   * Get all skins (alias for getLocalSkins for compatibility)
   */
  static async getSkins(): Promise<MinecraftSkin[]> {
    return this.getLocalSkins();
  }

  /**
   * Apply skin to player account
   */
  static async applySkin(skinId: string): Promise<void> {
    const account = get(currentAccount);
    if (!account) {
      throw new Error('No account selected');
    }

    const skins = await this.getLocalSkins();
    const skin = skins.find(s => s.file_name === skinId);
    
    if (!skin) {
      throw new Error('Skin not found');
    }

    await this.uploadSkinToMinecraft(
      account.access_token,
      skin.file_name,
      skin.is_slim
    );
  }

  /**
   * Upload new skin (opens file dialog)
   */
  static async uploadSkin(): Promise<MinecraftSkin | null> {
    try {
      // Use Tauri dialog to select file
      const skinFilePath = await invoke<string | null>('select_skin_file');
      
      if (!skinFilePath) {
        return null; // User cancelled
      }

      // Validate the file
      const validation = this.validateSkinFile(skinFilePath);
      if (!validation.valid) {
        throw new Error(validation.error);
      }

      // Install the skin
      const fileName = await this.installSkin(skinFilePath);
      
      // Get the newly installed skin
      const skins = await this.getLocalSkins();
      return skins.find(skin => skin.file_name === fileName) || null;
    } catch (error) {
      console.error('Error uploading skin:', error);
      throw error;
    }
  }

  /**
   * Get skin statistics
   */
  static getSkinStats(skins: MinecraftSkin[]) {
    const totalSkins = skins.length;
    const slimSkins = skins.filter(skin => skin.is_slim).length;
    const classicSkins = totalSkins - slimSkins;
    
    const sourceCounts = skins.reduce((counts, skin) => {
      counts[skin.source] = (counts[skin.source] || 0) + 1;
      return counts;
    }, {} as Record<string, number>);

    const mostRecentSkin = skins.reduce((latest, skin) => 
      skin.created_date > latest.created_date ? skin : latest
    , skins[0]);

    return {
      totalSkins,
      slimSkins,
      classicSkins,
      sourceCounts,
      mostRecentSkin
    };
  }

  /**
   * Filter skins by criteria
   */
  static filterSkins(skins: MinecraftSkin[], filters: {
    modelType?: 'slim' | 'classic';
    source?: string;
    searchTerm?: string;
  }): MinecraftSkin[] {
    return skins.filter(skin => {
      if (filters.modelType === 'slim' && !skin.is_slim) {
        return false;
      }
      
      if (filters.modelType === 'classic' && skin.is_slim) {
        return false;
      }
      
      if (filters.source && skin.source !== filters.source) {
        return false;
      }
      
      if (filters.searchTerm) {
        const term = filters.searchTerm.toLowerCase();
        if (!skin.name.toLowerCase().includes(term) && 
            !skin.file_name.toLowerCase().includes(term)) {
          return false;
        }
      }
      
      return true;
    });
  }

  /**
   * Sort skins by different criteria
   */
  static sortSkins(skins: MinecraftSkin[], sortBy: 'name' | 'created_date' | 'last_used', ascending = true): MinecraftSkin[] {
    const sorted = [...skins].sort((a, b) => {
      let comparison = 0;
      
      switch (sortBy) {
        case 'name':
          comparison = a.name.localeCompare(b.name);
          break;
        case 'created_date':
          comparison = a.created_date - b.created_date;
          break;
        case 'last_used':
          const aLastUsed = a.last_used || 0;
          const bLastUsed = b.last_used || 0;
          comparison = aLastUsed - bLastUsed;
          break;
      }
      
      return ascending ? comparison : -comparison;
    });
    
    return sorted;
  }

  /**
   * Format creation date for display
   */
  static formatCreatedDate(timestamp: number): string {
    if (timestamp === 0) {
      return 'Unknown';
    }
    
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString();
  }

  /**
   * Get model type display name
   */
  static getModelDisplayName(isSlim: boolean): string {
    return isSlim ? 'Alex (Slim)' : 'Steve (Classic)';
  }

  /**
   * Get source display name
   */
  static getSourceDisplayName(source: string): string {
    switch (source) {
      case 'Local':
        return 'Local File';
      case 'Mojang':
        return 'Mojang Account';
      case 'Custom':
        return 'Custom Upload';
      default:
        return source;
    }
  }

  /**
   * Validate skin file before installation
   */
  static validateSkinFile(filePath: string): { valid: boolean; error?: string } {
    const fileName = filePath.split(/[/\\]/).pop() || '';
    const extension = fileName.split('.').pop()?.toLowerCase();
    
    if (!extension || extension !== 'png') {
      return {
        valid: false,
        error: 'Skin files must be in PNG format'
      };
    }
    
    return { valid: true };
  }

  /**
   * Generate skin preview URL (placeholder for base64 data URL)
   */
  static async generateSkinPreview(skinData: number[]): Promise<string> {
    // Convert number array back to Uint8Array
    const uint8Array = new Uint8Array(skinData);
    
    // Create blob and convert to data URL
    const blob = new Blob([uint8Array], { type: 'image/png' });
    const arrayBuffer = await blob.arrayBuffer();
    const base64 = btoa(String.fromCharCode(...new Uint8Array(arrayBuffer)));
    
    return `data:image/png;base64,${base64}`;
  }

  /**
   * Create default skin names based on model type
   */
  static generateDefaultSkinName(isSlim: boolean, index?: number): string {
    const baseName = isSlim ? 'Custom Alex Skin' : 'Custom Steve Skin';
    return index ? `${baseName} ${index}` : baseName;
  }

  /**
   * Check if skin name already exists
   */
  static isNameTaken(skins: MinecraftSkin[], name: string): boolean {
    return skins.some(skin => skin.name.toLowerCase() === name.toLowerCase());
  }

  /**
   * Generate unique skin name
   */
  static generateUniqueName(skins: MinecraftSkin[], baseName: string): string {
    if (!this.isNameTaken(skins, baseName)) {
      return baseName;
    }
    
    let counter = 1;
    let uniqueName = `${baseName} ${counter}`;
    
    while (this.isNameTaken(skins, uniqueName)) {
      counter++;
      uniqueName = `${baseName} ${counter}`;
    }
    
    return uniqueName;
  }
}
