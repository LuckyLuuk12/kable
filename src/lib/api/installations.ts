import { invoke } from '@tauri-apps/api/core';
import type { MinecraftInstallation, MinecraftVersion } from '../types';

/**
 * Installation Management API
 * Pure Tauri invoke calls for Minecraft installation operations
 */

export async function getInstallations(): Promise<MinecraftInstallation[]> {
  return await invoke('get_installations');
}

export async function createInstallation(
  name: string,
  version: string,
  modLoader: string,
  gameDirectory?: string,
  javaPath?: string,
  jvmArgs?: string,
  memory?: number,
  description?: string
): Promise<MinecraftInstallation> {
  return await invoke('create_installation', {
    request: {
      name,
      version,
      mod_loader: modLoader,
      game_directory: gameDirectory,
      java_path: javaPath,
      jvm_args: jvmArgs,
      memory,
      description
    }
  });
}

export async function updateInstallation(
  installationId: string,
  name: string,
  version: string,
  modLoader: string,
  gameDirectory?: string,
  javaPath?: string,
  jvmArgs?: string,
  memory?: number,
  description?: string
): Promise<MinecraftInstallation> {
  return await invoke('update_installation', {
    installationId,
    request: {
      name,
      version,
      mod_loader: modLoader,
      game_directory: gameDirectory,
      java_path: javaPath,
      jvm_args: jvmArgs,
      memory,
      description
    }
  });
}

export async function deleteInstallation(installationId: string): Promise<void> {
  return await invoke('delete_installation', { installationId });
}

export async function launchMinecraftInstallation(installationId: string): Promise<void> {
  return await invoke('launch_minecraft_installation', { installationId });
}

export async function openInstallationFolder(installationId: string): Promise<void> {
  return await invoke('open_installation_folder', { installationId });
}

export async function getMinecraftVersions(): Promise<MinecraftVersion[]> {
  return await invoke('get_minecraft_versions');
}

export async function getInstalledMods(minecraftPath: string, installationId: string): Promise<any[]> {
  return await invoke('get_installed_mods', { minecraftPath, installationId });
}

export async function detectInstallationModLoader(installationId: string): Promise<{ modLoader: string; loaderVersion?: string }> {
  return await invoke('detect_installation_mod_loader', { installationId });
}
