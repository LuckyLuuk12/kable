import { invoke } from '@tauri-apps/api/core';
import type { KableInstallation, VersionData } from '../types';

// Get all versions (optionally force refresh)
export async function get_all_versions(force = false): Promise<VersionData[]> {
  return await invoke('get_all_versions', { force });
}

// Get a single version by id
export async function get_version(version_id: string): Promise<VersionData | null> {
  return await invoke('get_version', { version_id });
}

// Get all Kable installations
export async function get_installations(): Promise<KableInstallation[]> {
  return await invoke('get_installations');
}

// Get a single Kable installation by id
export async function get_installation(id: string): Promise<KableInstallation | null> {
  return await invoke('get_installation', { id });
}

// Modify an existing Kable installation
export async function modify_installation(id: string, new_installation: KableInstallation): Promise<void> {
  return await invoke('modify_installation', { id, newInstallation: new_installation });
}

// Delete a Kable installation by id
export async function delete_installation(id: string): Promise<void> {
  return await invoke('delete_installation', { id });
}

// Create a new Kable installation for a given version_id
export async function create_installation(version_id: string): Promise<KableInstallation> {
  return await invoke('create_installation', { version_id });
}

// import { invoke } from '@tauri-apps/api/core';
// import type { MinecraftInstallation, MinecraftVersion } from '../types';

// /**
//  * Installation Management API
//  * Pure Tauri invoke calls for Minecraft installation operations
//  */

// export async function getInstallations(): Promise<MinecraftInstallation[]> {
//   return await invoke('get_installations');
// }

// export async function createInstallation(
//   name: string,
//   version: string,
//   modLoader: string,
//   gameDirectory?: string,
//   javaPath?: string,
//   jvmArgs?: string,
//   memory?: number,
//   description?: string
// ): Promise<MinecraftInstallation> {
//   return await invoke('create_installation', {
//     request: {
//       name,
//       version,
//       mod_loader: modLoader,
//       game_directory: gameDirectory,
//       java_path: javaPath,
//       jvm_args: jvmArgs,
//       memory,
//       description
//     }
//   });
// }

// export async function updateInstallation(
//   installationId: string,
//   name: string,
//   version: string,
//   modLoader: string,
//   gameDirectory?: string,
//   javaPath?: string,
//   jvmArgs?: string,
//   memory?: number,
//   description?: string
// ): Promise<MinecraftInstallation> {
//   return await invoke('update_installation', {
//     installationId,
//     request: {
//       name,
//       version,
//       mod_loader: modLoader,
//       game_directory: gameDirectory,
//       java_path: javaPath,
//       jvm_args: jvmArgs,
//       memory,
//       description
//     }
//   });
// }

// export async function deleteInstallation(installationId: string): Promise<void> {
//   return await invoke('delete_installation', { installationId });
// }

// export async function launchMinecraftInstallation(installationId: string): Promise<void> {
//   return await invoke('launch_minecraft_installation', { installationId });
// }

// export async function openInstallationFolder(installationId: string): Promise<void> {
//   return await invoke('open_installation_folder', { installationId });
// }

// export async function getMinecraftVersions(): Promise<MinecraftVersion[]> {
//   return await invoke('get_minecraft_versions');
// }

// export async function getInstalledMods(minecraftPath: string, installationId: string): Promise<any[]> {
//   return await invoke('get_installed_mods', { minecraftPath, installationId });
// }

// export async function detectInstallationModLoader(installationId: string): Promise<{ modLoader: string; loaderVersion?: string }> {
//   return await invoke('detect_installation_mod_loader', { installationId });
// }

// export async function convertVanillaToKable(installationId: string): Promise<MinecraftInstallation> {
//   return await invoke('convert_vanilla_to_kable', { installationId });
// }