import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { settings } from '../stores/settings';
import type { LocalWorld, WorldDownload, MinecraftDirectoryInfo } from '../types';

export class MapsService {
  static async loadWorlds(): Promise<LocalWorld[]> {
    return this.getLocalWorlds();
  }

  static async getLocalWorlds(): Promise<LocalWorld[]> {
    const minecraftPath = get(settings).general.game_directory;
    if (!minecraftPath) {
      throw new Error('Minecraft directory not found. Please set the Minecraft path in settings.');
    }
    return invoke('get_local_worlds', { minecraftPath });
  }

  static async deleteWorld(worldFolder: string): Promise<void> {
    const minecraftPath = get(settings).general.game_directory;
    if (!minecraftPath) {
      throw new Error('Minecraft directory not found. Please set the Minecraft path in settings.');
    }
    return invoke('delete_world', { minecraftPath, worldFolder });
  }

  static async backupWorld(worldFolder: string): Promise<string> {
    const minecraftPath = get(settings).general.game_directory;
    if (!minecraftPath) {
      throw new Error('Minecraft directory not found. Please set the Minecraft path in settings.');
    }
    return invoke('backup_world', { minecraftPath, worldFolder });
  }

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

  static sortWorlds(worlds: LocalWorld[], sortBy: 'name' | 'last_played' | 'size' | 'created', ascending = true): LocalWorld[] {
    return [...worlds].sort((a, b) => {
      let cmp = 0;
      switch (sortBy) {
        case 'name':
          cmp = a.name.localeCompare(b.name);
          break;
        case 'last_played':
          cmp = (a.last_played || 0) - (b.last_played || 0);
          break;
        case 'size':
          cmp = a.size_mb - b.size_mb;
          break;
        case 'created':
          cmp = a.created - b.created;
          break;
      }
      return ascending ? cmp : -cmp;
    });
  }

  static formatWorldSize(sizeMB: number): string {
    if (sizeMB < 1024) {
      return `${sizeMB.toFixed(1)} MB`;
    } else {
      return `${(sizeMB / 1024).toFixed(1)} GB`;
    }
  }

  static formatLastPlayed(timestamp: number): string {
    if (!timestamp) return 'Never';
    const date = new Date(timestamp);
    return date.toLocaleString();
  }
}
