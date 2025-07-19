import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import type { ShaderPack, ShaderDownload, ShaderSettings } from '../types';
import { settings } from '../stores/settings';

export class ShadersManager {
  /**
   * Get all installed shader packs
   */
  static async getInstalledShaders(): Promise<ShaderPack[]> {
    const minecraftPath = get(settings).general.game_directory;
    if (!minecraftPath) {
      throw new Error('Minecraft directory not found. Please set the Minecraft path in settings.');
    }
    return invoke('get_installed_shaders', { minecraftPath });
  }

  /**
   * Toggle shader pack enabled/disabled state
   */
  static async toggleShader(shaderFile: string, enabled: boolean): Promise<void> {
    const minecraftPath = get(settings).general.game_directory;
    if (!minecraftPath) {
      throw new Error('Minecraft directory not found. Please set the Minecraft path in settings.');
    }
    return invoke('toggle_shader', { minecraftPath, shaderFile, enabled });
  }

  /**
   * Delete a shader pack
   */
  static async deleteShader(shaderFile: string): Promise<void> {
    const minecraftPath = get(settings).general.game_directory;
    if (!minecraftPath) {
      throw new Error('Minecraft directory not found. Please set the Minecraft path in settings.');
    }
    return invoke('delete_shader', { minecraftPath, shaderFile });
  }

  /**
   * Install shader pack from file
   */
  static async installShaderPack(shaderFilePath: string): Promise<string> {
    const minecraftPath = get(settings).general.game_directory;
    if (!minecraftPath) {
      throw new Error('Minecraft directory not found. Please set the Minecraft path in settings.');
    }
    return invoke('install_shader_pack', { minecraftPath, shaderFilePath });
  }

  /**
   * Get detailed info about a shader pack
   */
  static async getShaderInfo(shaderFile: string): Promise<ShaderPack> {
    const minecraftPath = get(settings).general.game_directory;
    if (!minecraftPath) {
      throw new Error('Minecraft directory not found. Please set the Minecraft path in settings.');
    }
    return invoke('get_shader_info', { minecraftPath, shaderFile });
  }

  /**
   * Get shader statistics
   */
  static getShaderStats(shaders: ShaderPack[]) {
    const totalShaders = shaders.length;
    const totalSize = shaders.reduce((sum, shader) => sum + shader.file_size, 0);
    const enabledShaders = shaders.filter(shader => shader.enabled).length;
    
    const loaderCounts = shaders.reduce((counts, shader) => {
      counts[shader.shader_loader] = (counts[shader.shader_loader] || 0) + 1;
      return counts;
    }, {} as Record<string, number>);

    const mostRecentShader = shaders.reduce((latest, shader) => 
      shader.installed_date > latest.installed_date ? shader : latest
    , shaders[0]);

    return {
      totalShaders,
      totalSizeBytes: totalSize,
      enabledShaders,
      loaderCounts,
      mostRecentShader
    };
  }

  /**
   * Filter shaders by criteria
   */
  static filterShaders(shaders: ShaderPack[], filters: {
    loader?: string;
    enabled?: boolean;
    minSize?: number;
    maxSize?: number;
    searchTerm?: string;
  }): ShaderPack[] {
    return shaders.filter(shader => {
      if (filters.loader && shader.shader_loader !== filters.loader) {
        return false;
      }
      
      if (filters.enabled !== undefined && shader.enabled !== filters.enabled) {
        return false;
      }
      
      if (filters.minSize && shader.file_size < filters.minSize) {
        return false;
      }
      
      if (filters.maxSize && shader.file_size > filters.maxSize) {
        return false;
      }
      
      if (filters.searchTerm) {
        const term = filters.searchTerm.toLowerCase();
        if (!shader.name.toLowerCase().includes(term) && 
            !shader.author.toLowerCase().includes(term) &&
            !shader.file_name.toLowerCase().includes(term)) {
          return false;
        }
      }
      
      return true;
    });
  }

  /**
   * Sort shaders by different criteria
   */
  static sortShaders(shaders: ShaderPack[], sortBy: 'name' | 'author' | 'size' | 'installed_date', ascending = true): ShaderPack[] {
    const sorted = [...shaders].sort((a, b) => {
      let comparison = 0;
      
      switch (sortBy) {
        case 'name':
          comparison = a.name.localeCompare(b.name);
          break;
        case 'author':
          comparison = a.author.localeCompare(b.author);
          break;
        case 'size':
          comparison = a.file_size - b.file_size;
          break;
        case 'installed_date':
          comparison = a.installed_date - b.installed_date;
          break;
      }
      
      return ascending ? comparison : -comparison;
    });
    
    return sorted;
  }

  /**
   * Format shader file size for display
   */
  static formatShaderSize(sizeBytes: number): string {
    const sizeMB = sizeBytes / (1024 * 1024);
    if (sizeMB < 1) {
      return `${(sizeBytes / 1024).toFixed(1)} KB`;
    } else if (sizeMB < 1024) {
      return `${sizeMB.toFixed(1)} MB`;
    } else {
      return `${(sizeMB / 1024).toFixed(1)} GB`;
    }
  }

  /**
   * Format installation date for display
   */
  static formatInstallDate(timestamp: number): string {
    if (timestamp === 0) {
      return 'Unknown';
    }
    
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString();
  }

  /**
   * Check if shader is compatible with Minecraft version
   */
  static isCompatible(shader: ShaderPack, minecraftVersion: string): boolean {
    if (!shader.compatible_versions || shader.compatible_versions.length === 0) {
      return true; // Assume compatible if no specific versions listed
    }
    
    return shader.compatible_versions.some(version => 
      version === minecraftVersion || 
      minecraftVersion.startsWith(version.split('.').slice(0, 2).join('.'))
    );
  }

  /**
   * Get shader loader display name
   */
  static getLoaderDisplayName(loader: string): string {
    switch (loader) {
      case 'OptiFine':
        return 'OptiFine';
      case 'Iris':
        return 'Iris Shaders';
      case 'Sodium':
        return 'Sodium';
      default:
        return loader;
    }
  }

  /**
   * Validate shader file before installation
   */
  static validateShaderFile(filePath: string): { valid: boolean; error?: string } {
    const fileName = filePath.split(/[/\\]/).pop() || '';
    const extension = fileName.split('.').pop()?.toLowerCase();
    
    if (!extension || !['zip', 'jar'].includes(extension)) {
      return {
        valid: false,
        error: 'Shader files must be .zip or .jar format'
      };
    }
    
    // Check for common shader pack indicators in filename
    const shaderIndicators = ['shader', 'shaders', 'seus', 'bsl', 'complementary', 'chocapic', 'sildur'];
    const hasShaderIndicator = shaderIndicators.some(indicator => 
      fileName.toLowerCase().includes(indicator)
    );
    
    if (!hasShaderIndicator) {
      return {
        valid: true,
        error: 'Warning: This file may not be a shader pack'
      };
    }
    
    return { valid: true };
  }
}
