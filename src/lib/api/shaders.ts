import { invoke } from '@tauri-apps/api/core';
import type { ShaderPack, ShaderDownload, ShaderFilterFacets } from '$lib';

export async function getInstalledShaders(minecraftPath: string): Promise<ShaderPack[]> {
  return invoke('get_installed_shaders', { minecraftPath });
}

export async function toggleShader(minecraftPath: string, shaderFile: string, enabled: boolean): Promise<void> {
  return invoke('toggle_shader', { minecraftPath, shaderFile, enabled });
}

export async function deleteShader(minecraftPath: string, shaderFile: string): Promise<void> {
  return invoke('delete_shader', { minecraftPath, shaderFile });
}

export async function installShaderPack(minecraftPath: string, shaderFilePath: string): Promise<string> {
  return invoke('install_shader_pack', { minecraftPath, shaderFilePath });
}

export async function getShaderInfo(minecraftPath: string, shaderFile: string): Promise<ShaderPack> {
  return invoke('get_shader_info', { minecraftPath, shaderFile });
}

export async function searchModrinthShaders(
  query: string,
  minecraftVersion: string | null,
  limit: number,
  offset: number
): Promise<ShaderDownload[]> {
  return invoke('search_modrinth_shaders', { query, minecraftVersion, limit, offset });
}

export async function searchModrinthShadersWithFacets(
  query: string,
  minecraftVersion: string | null,
  facets: ShaderFilterFacets | null,
  limit: number,
  offset: number
): Promise<ShaderDownload[]> {
  return invoke('search_modrinth_shaders_with_facets', { query, minecraftVersion, facets, limit, offset });
}

export async function getModrinthShaderDetails(projectId: string): Promise<ShaderDownload> {
  return invoke('get_modrinth_shader_details', { projectId });
}

export async function downloadAndInstallShader(
  minecraftPath: string,
  downloadUrl: string,
  filename: string
): Promise<string> {
  return invoke('download_and_install_shader', { minecraftPath, downloadUrl, filename });
}

export async function downloadAndInstallShaderToDedicated(
  minecraftPath: string,
  dedicatedFolder: string,
  downloadUrl: string,
  filename: string
): Promise<string> {
  return invoke('download_and_install_shader_to_dedicated', { 
    minecraftPath, 
    dedicatedFolder, 
    downloadUrl, 
    filename 
  });
}

export async function setupShaderSymlink(
  minecraftPath: string,
  dedicatedFolder: string,
  symlinkName: string
): Promise<void> {
  return invoke('setup_shader_symlink', { minecraftPath, dedicatedFolder, symlinkName });
}

export async function removeShaderSymlink(minecraftPath: string, symlinkName: string): Promise<void> {
  return invoke('remove_shader_symlink', { minecraftPath, symlinkName });
}

export async function deleteShaderFromDedicated(
  minecraftPath: string,
  dedicatedFolder: string,
  shaderFile: string,
  symlinkName: string | null
): Promise<void> {
  return invoke('delete_shader_from_dedicated', { 
    minecraftPath, 
    dedicatedFolder, 
    shaderFile, 
    symlinkName 
  });
}
