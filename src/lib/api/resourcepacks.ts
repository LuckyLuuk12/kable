import { invoke } from '@tauri-apps/api/core';
import type { ResourcePack, ResourcePackDownload, ResourcePackFilterFacets } from '$lib';

export async function getInstalledResourcepacks(minecraftPath: string): Promise<ResourcePack[]> {
  return invoke('get_installed_resourcepacks', { minecraftPath });
}

export async function deleteResourcepack(minecraftPath: string, packFile: string): Promise<void> {
  return invoke('delete_resourcepack', { minecraftPath, packFile });
}

export async function installResourcepack(minecraftPath: string, packFilePath: string): Promise<string> {
  return invoke('install_resourcepack', { minecraftPath, packFilePath });
}

export async function getResourcepackInfo(minecraftPath: string, packFile: string): Promise<ResourcePack> {
  return invoke('get_resourcepack_info', { minecraftPath, packFile });
}

export async function searchModrinthResourcepacks(
  query: string,
  minecraftVersion: string | null,
  limit: number,
  offset: number
): Promise<ResourcePackDownload[]> {
  return invoke('search_modrinth_resourcepacks', { query, minecraftVersion, limit, offset });
}

export async function searchModrinthResourcepacksWithFacets(
  query: string,
  minecraftVersion: string | null,
  facets: ResourcePackFilterFacets | null,
  limit: number,
  offset: number
): Promise<ResourcePackDownload[]> {
  return invoke('search_modrinth_resourcepacks_with_facets', { query, minecraftVersion, facets, limit, offset });
}

export async function getModrinthResourcepackDetails(projectId: string): Promise<ResourcePackDownload> {
  return invoke('get_modrinth_resourcepack_details', { projectId });
}

export async function downloadAndInstallResourcepack(
  minecraftPath: string,
  downloadUrl: string,
  filename: string
): Promise<string> {
  return invoke('download_and_install_resourcepack', { minecraftPath, downloadUrl, filename });
}

export async function downloadAndInstallResourcepackToDedicated(
  minecraftPath: string,
  dedicatedFolder: string,
  downloadUrl: string,
  filename: string
): Promise<string> {
  return invoke('download_and_install_resourcepack_to_dedicated', { 
    minecraftPath, 
    dedicatedFolder, 
    downloadUrl, 
    filename 
  });
}

export async function setupResourcepackSymlink(
  minecraftPath: string,
  dedicatedFolder: string,
  symlinkName: string
): Promise<void> {
  return invoke('setup_resourcepack_symlink', { minecraftPath, dedicatedFolder, symlinkName });
}

export async function removeResourcepackSymlink(minecraftPath: string, symlinkName: string): Promise<void> {
  return invoke('remove_resourcepack_symlink', { minecraftPath, symlinkName });
}

export async function deleteResourcepackFromDedicated(
  minecraftPath: string,
  dedicatedFolder: string,
  packFile: string,
  symlinkName: string | null
): Promise<void> {
  return invoke('delete_resourcepack_from_dedicated', { 
    minecraftPath, 
    dedicatedFolder, 
    packFile, 
    symlinkName 
  });
}
