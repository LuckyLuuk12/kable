import { invoke } from '@tauri-apps/api/core';
import type { 
  LaunchOptions, 
  MinecraftDirectoryInfo,
  LauncherProfiles,
  MicrosoftAccount
} from '../types';

/**
 * Minecraft API
 * Pure Tauri invoke calls for Minecraft game operations
 */

// Game Launch
export async function launchMinecraft(options: LaunchOptions, minecraftPath: string): Promise<string> {
  return await invoke('launch_minecraft', { options, minecraftPath });
}

// Directory and Path Management
export async function getDefaultMinecraftDir(): Promise<string> {
  return await invoke('get_default_minecraft_dir');
}

export async function validateMinecraftDirectory(path: string): Promise<MinecraftDirectoryInfo> {
  return await invoke('validate_minecraft_directory', { path });
}

export async function getCachedUsernames(minecraftPath: string): Promise<string[]> {
  return await invoke('get_cached_usernames', { minecraftPath });
}

// Java Management
export async function checkJavaInstallation(): Promise<string> {
  return await invoke('check_java_installation');
}

// Session Management
export async function getMinecraftSessionPath(): Promise<string> {
  return await invoke('get_minecraft_session_path');
}

export async function readMinecraftSessions(): Promise<LauncherProfiles> {
  return await invoke('read_minecraft_sessions');
}

export async function writeMinecraftSession(account: MicrosoftAccount): Promise<void> {
  return await invoke('write_minecraft_session', { account });
}

export async function getMinecraftLaunchArgs(account: MicrosoftAccount): Promise<string[]> {
  return await invoke('get_minecraft_launch_args', { account });
}
