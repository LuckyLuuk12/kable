import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import type { ShaderPack, ShaderDownload, KableInstallation, ShaderFilterFacets } from '../types';
import { settings } from '../stores/settings';
import { shaderDownloads, shadersLoading, shadersError, shadersLimit, shadersOffset, shadersInstallation, shadersInstallMode } from '../stores/shaders';
import * as ShadersAPI from '../api/shaders';

export interface ShaderFilter {
  loader?: ('Canvas' | 'Iris' | 'OptiFine' | 'Vanilla')[];
  performanceImpact?: ('High' | 'Low' | 'Medium' | 'Potato' | 'Screenshot')[];
  features?: ('Atmosphere' | 'Bloom' | 'Colored Lighting' | 'Foliage' | 'Path Tracing' | 'PBR' | 'Reflections' | 'Shadows')[];
  categories?: ('Cartoon' | 'Cursed' | 'Fantasy' | 'Realistic' | 'Semi-realistic' | 'Vanilla-like')[];
  minecraftVersion?: string;
  searchTerm?: string;
}

export class ShadersService {
  private static minecraftPath: string | null = null;
  private currentFilter: ShaderFilterFacets | null = null;
  initialized = false;

  constructor() {
    console.log('[ShadersService] Initializing shader service');
  }

  async initialize() {
    if (this.initialized) return;
    
    const currentSettings = get(settings);
    ShadersService.minecraftPath = currentSettings.general.game_directory || null;
    
    await this.loadShaders();
    this.initialized = true;
  }

  async loadShaders() {
    shadersLoading.set(true);
    shadersError.set(null);
    const offset = get(shadersOffset);
    const limit = get(shadersLimit);

    console.log(`[ShadersService] Loading shaders (offset: ${offset}, limit: ${limit})`);

    try {
      // Search Modrinth for shaders with current filter
      const shaders = this.currentFilter
        ? await ShadersAPI.searchModrinthShadersWithFacets('', null, this.currentFilter, limit, offset)
        : await ShadersAPI.searchModrinthShaders('', null, limit, offset);
      console.log(`[ShadersService] Successfully loaded ${shaders.length} shaders`);
      shaderDownloads.set(shaders);
    } catch (e: any) {
      console.error('[ShadersService] Failed to load shaders:', e.message || 'Unknown error');
      shadersError.set(e.message || 'Failed to load shaders');
    } finally {
      shadersLoading.set(false);
    }
  }

  async setFilter(filter: ShaderFilterFacets | null) {
    this.currentFilter = filter;
    console.log('[ShadersService] Setting shader filter:', JSON.stringify(filter, null, 2));
    shadersOffset.set(0); // Reset to first page
    await this.loadShaders();
  }

  async setLimit(limit: number) {
    shadersLimit.set(limit);
    console.log(`[ShadersService] Setting shader limit to ${limit}`);
    await this.loadShaders();
  }

  async nextPage() {
    const limit = get(shadersLimit);
    const offset = get(shadersOffset) + limit;
    
    console.log(`[ShadersService] Moving to next page (new offset: ${offset})`);
    
    shadersOffset.set(offset);
    await this.loadShaders();
  }

  async prevPage() {
    const limit = get(shadersLimit);
    const offset = Math.max(0, get(shadersOffset) - limit);
    
    console.log(`[ShadersService] Moving to previous page (new offset: ${offset})`);
    
    shadersOffset.set(offset);
    await this.loadShaders();
  }

  async searchShaders(query: string) {
    shadersLoading.set(true);
    shadersError.set(null);
    const limit = get(shadersLimit);

    console.log(`[ShadersService] Searching shaders: "${query}"`);

    try {
      const shaders = this.currentFilter
        ? await ShadersAPI.searchModrinthShadersWithFacets(query, null, this.currentFilter, limit, 0)
        : await ShadersAPI.searchModrinthShaders(query, null, limit, 0);
      console.log(`[ShadersService] Found ${shaders.length} shaders matching "${query}"`);
      shaderDownloads.set(shaders);
      shadersOffset.set(0); // Reset to first page on new search
    } catch (e: any) {
      console.error('[ShadersService] Failed to search shaders:', e.message || 'Unknown error');
      shadersError.set(e.message || 'Failed to search shaders');
    } finally {
      shadersLoading.set(false);
    }
  }

  async downloadShader(shader: ShaderDownload, installation: KableInstallation | null) {
    shadersLoading.set(true);
    shadersError.set(null);
    const mode = get(shadersInstallMode);

    const isGlobal = !installation || installation.id === '__global__' || mode === 'global';
    
    console.log(`[ShadersService] Downloading shader "${shader.name}" to ${isGlobal ? 'global' : installation?.name || 'unknown'}`);

    try {
      if (isGlobal) {
        await ShadersService.downloadShaderGlobal(shader);
      } else if (installation) {
        await ShadersService.downloadShaderToDedicated(shader, installation);
      } else {
        throw new Error('No installation selected for dedicated shader installation');
      }
      console.log(`[ShadersService] Successfully downloaded shader "${shader.name}"`);
    } catch (e: any) {
      console.error(`[ShadersService] Failed to download shader "${shader.name}":`, e.message || 'Unknown error');
      shadersError.set(e.message || 'Failed to download shader');
      throw e;
    } finally {
      shadersLoading.set(false);
    }
  }

  // Helpers for UI
  getShaders() {
    return get(shaderDownloads);
  }
  
  isLoading() { 
    return get(shadersLoading); 
  }
  
  getError() { 
    return get(shadersError); 
  }

  static async ensureInitialized() {
    if (!this.minecraftPath) {
      const currentSettings = get(settings);
      this.minecraftPath = currentSettings.general.game_directory || null;
    }
  }

  static async getInstalledShaders(): Promise<ShaderPack[]> {
    await this.ensureInitialized();
    if (!this.minecraftPath) {
      throw new Error('Minecraft directory not configured');
    }
    return ShadersAPI.getInstalledShaders(this.minecraftPath);
  }

  static async searchShaders(
    query: string,
    minecraftVersion: string | null,
    limit: number,
    offset: number
  ): Promise<ShaderDownload[]> {
    return ShadersAPI.searchModrinthShaders(query, minecraftVersion, limit, offset);
  }

  static async downloadShaderGlobal(shader: ShaderDownload): Promise<string> {
    await this.ensureInitialized();
    if (!this.minecraftPath) {
      throw new Error('Minecraft directory not configured');
    }
    
    // Extract filename from download URL
    const filename = shader.download_url.split('/').pop() || `${shader.name}.zip`;
    
    return ShadersAPI.downloadAndInstallShader(
      this.minecraftPath,
      shader.download_url,
      filename
    );
  }

  static async downloadShaderToDedicated(
    shader: ShaderDownload,
    installation: KableInstallation
  ): Promise<string> {
    await this.ensureInitialized();
    if (!this.minecraftPath) {
      throw new Error('Minecraft directory not configured');
    }

    const dedicatedFolder = installation.dedicated_shaders_folder || installation.id;
    const filename = shader.download_url.split('/').pop() || `${shader.name}.zip`;
    
    // Download to dedicated folder
    // Note: Symlinks are managed dynamically by symlink_manager when launching the game
    await ShadersAPI.downloadAndInstallShaderToDedicated(
      this.minecraftPath,
      dedicatedFolder,
      shader.download_url,
      filename
    );

    return filename;
  }

  static async deleteShader(shaderFile: string): Promise<void> {
    await this.ensureInitialized();
    if (!this.minecraftPath) {
      throw new Error('Minecraft directory not configured');
    }
    return ShadersAPI.deleteShader(this.minecraftPath, shaderFile);
  }

  static async deleteShaderFromDedicated(
    shaderFile: string,
    installation: KableInstallation
  ): Promise<void> {
    await this.ensureInitialized();
    if (!this.minecraftPath) {
      throw new Error('Minecraft directory not configured');
    }

    const dedicatedFolder = installation.dedicated_shaders_folder || installation.id;
    const symlinkName = installation.id;

    return ShadersAPI.deleteShaderFromDedicated(
      this.minecraftPath,
      dedicatedFolder,
      shaderFile,
      symlinkName
    );
  }

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

  static filterShaderDownloads(shaders: ShaderDownload[], filters: ShaderFilter): ShaderDownload[] {
    return shaders.filter(shader => {
      // Loader filter
      if (filters.loader && filters.loader.length > 0) {
        if (!filters.loader.includes(shader.shader_loader)) {
          return false;
        }
      }

      // Performance Impact filter (based on tags)
      if (filters.performanceImpact && filters.performanceImpact.length > 0) {
        const hasPerformanceTag = filters.performanceImpact.some(perf =>
          shader.tags.some(tag => tag.toLowerCase() === perf.toLowerCase())
        );
        if (!hasPerformanceTag) return false;
      }

      // Features filter (based on tags)
      if (filters.features && filters.features.length > 0) {
        const hasFeature = filters.features.some(feature =>
          shader.tags.some(tag => tag.toLowerCase().includes(feature.toLowerCase()))
        );
        if (!hasFeature) return false;
      }

      // Categories filter (based on tags)
      if (filters.categories && filters.categories.length > 0) {
        const hasCategory = filters.categories.some(category =>
          shader.tags.some(tag => tag.toLowerCase() === category.toLowerCase())
        );
        if (!hasCategory) return false;
      }

      // Minecraft version filter
      if (filters.minecraftVersion) {
        if (!shader.minecraft_versions.includes(filters.minecraftVersion)) {
          return false;
        }
      }

      // Search term filter
      if (filters.searchTerm) {
        const term = filters.searchTerm.toLowerCase();
        if (
          !shader.name.toLowerCase().includes(term) &&
          !shader.author.toLowerCase().includes(term) &&
          !shader.description.toLowerCase().includes(term)
        ) {
          return false;
        }
      }

      return true;
    });
  }
}
