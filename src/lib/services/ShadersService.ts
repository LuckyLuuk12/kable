import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import type { ShaderPack, ShaderDownload, ShaderSettings } from '../types';
import { settings } from '../stores/settings';

export class ShadersService {
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
}
