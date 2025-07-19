import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import type { LocalWorld, WorldDownload, MinecraftDirectoryInfo } from '../types';
import { settings } from '../stores/settings';

export class MapsManager {
  /**
   * Load worlds data (alias for getLocalWorlds for consistency with other managers)
   */
  static async loadWorlds(): Promise<LocalWorld[]> {
    return this.getLocalWorlds();
  }

  /**
   * Get all local worlds from the Minecraft saves directory
   */
  static async getLocalWorlds(): Promise<LocalWorld[]> {
    const minecraftPath = get(settings).general.game_directory;
    if (!minecraftPath) {
      throw new Error('Minecraft directory not found. Please set the Minecraft path in settings.');
    }
    return invoke('get_local_worlds', { minecraftPath });
  }

  /**
   * Delete a world from the saves directory
   */
  static async deleteWorld(worldFolder: string): Promise<void> {
    const minecraftPath = get(settings).general.game_directory;
    if (!minecraftPath) {
      throw new Error('Minecraft directory not found. Please set the Minecraft path in settings.');
    }
    return invoke('delete_world', { minecraftPath, worldFolder });
  }

  /**
   * Create a backup of a world
   */
  static async backupWorld(worldFolder: string): Promise<string> {
    const minecraftPath = get(settings).general.game_directory;
    if (!minecraftPath) {
      throw new Error('Minecraft directory not found. Please set the Minecraft path in settings.');
    }
    return invoke('backup_world', { minecraftPath, worldFolder });
  }

  /**
   * Get world statistics
   */
  static getWorldStats(worlds: LocalWorld[]) {
    const totalWorlds = worlds.length;
    const totalSize = worlds.reduce((sum, world) => sum + world.size_mb, 0);
    const lastPlayedWorld = worlds.reduce((latest, world) => 
      (world?.last_played || 0) > (latest?.last_played || 0) ? world : latest
    , worlds[0]);

    const gameModeCounts = worlds.reduce((counts, world) => {
      counts[world?.game_mode as string] = (counts[world?.game_mode as string] || 0) + 1;
      return counts;
    }, {} as Record<string, number>);

    return {
      totalWorlds,
      totalSizeMB: totalSize,
      lastPlayedWorld,
      gameModeCounts
    };
  }

  /**
   * Filter worlds by criteria
   */
  static filterWorlds(worlds: LocalWorld[], filters: {
    gameMode?: string;
    minSize?: number;
    maxSize?: number;
    searchTerm?: string;
  }): LocalWorld[] {
    return worlds.filter(world => {
      if (filters.gameMode && world.game_mode !== filters.gameMode) {
        return false;
      }
      
      if (filters.minSize && world.size_mb < filters.minSize) {
        return false;
      }
      
      if (filters.maxSize && world.size_mb > filters.maxSize) {
        return false;
      }
      
      if (filters.searchTerm) {
        const term = filters.searchTerm.toLowerCase();
        if (!world.name.toLowerCase().includes(term) && 
            !world.folder_name.toLowerCase().includes(term)) {
          return false;
        }
      }
      
      return true;
    });
  }

  /**
   * Sort worlds by different criteria
   */
  static sortWorlds(worlds: LocalWorld[], sortBy: 'name' | 'last_played' | 'size' | 'created', ascending = true): LocalWorld[] {
    const sorted = [...worlds].sort((a, b) => {
      let comparison = 0;
      
      switch (sortBy) {
        case 'name':
          comparison = a.name.localeCompare(b.name);
          break;
        case 'last_played':
          comparison = (a.last_played || 0) - (b.last_played || 0);
          break;
        case 'size':
          comparison = (a.size_mb || 0) - (b.size_mb || 0);
          break;
        case 'created':
          comparison = (a.created || 0) - (b.created || 0);
          break;
      }
      
      return ascending ? comparison : -comparison;
    });
    
    return sorted;
  }

  /**
   * Format world size for display
   */
  static formatWorldSize(sizeMB: number): string {
    if (sizeMB < 1024) {
      return `${sizeMB.toFixed(1)} MB`;
    } else {
      return `${(sizeMB / 1024).toFixed(1)} GB`;
    }
  }

  /**
   * Format last played time for display
   */
  static formatLastPlayed(timestamp: number): string {
    if (timestamp === 0) {
      return 'Never';
    }
    
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    
    if (diffDays === 0) {
      return 'Today';
    } else if (diffDays === 1) {
      return 'Yesterday';
    } else if (diffDays < 7) {
      return `${diffDays} days ago`;
    } else {
      return date.toLocaleDateString();
    }
  }
}
