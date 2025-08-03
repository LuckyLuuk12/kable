import { invoke } from '@tauri-apps/api/core';
import type { 
  LaunchOptions, 
  MinecraftDirectoryInfo,
  LauncherProfiles,
  MicrosoftAccount
} from '../types';


// Directory and Path Management
export async function getDefaultMinecraftDir(): Promise<string> {
  return await invoke('get_default_minecraft_dir');
}

export async function validateMinecraftDirectory(path: string): Promise<MinecraftDirectoryInfo> {
  return await invoke('validate_minecraft_directory', { path });
}

