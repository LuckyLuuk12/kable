import type { KableInstallation, CategorizedLauncherSettings, LauncherAccount } from '$lib';
import { invoke } from '@tauri-apps/api/core';

// Types matching backend LaunchResult
export interface LaunchResult {
  pid: number;
  success: boolean;
  error?: string;
}

/**
 * Launch a Minecraft installation (matches tauri::command launch_installation)
 */
export async function launchInstallation(
  installation: KableInstallation,
  settings: CategorizedLauncherSettings,
  account: LauncherAccount
): Promise<LaunchResult> {
  return await invoke<LaunchResult>('launch_installation', {
    installation,
    settings,
    account
  });
}

/**
 * Kill a Minecraft process by PID (matches tauri::command kill_minecraft_process)
 */
export async function killMinecraftProcess(processId: number): Promise<void> {
  return await invoke('kill_minecraft_process', { processId });
}

/**
 * Get all running Minecraft process IDs (matches tauri::command get_running_minecraft_processes)
 */
export async function getRunningMinecraftProcesses(): Promise<number[]> {
  return await invoke<number[]>('get_running_minecraft_processes');
}

/**
 * Check if any Minecraft process is running (matches tauri::command is_minecraft_running)
 */
export async function isMinecraftRunning(): Promise<boolean> {
  return await invoke<boolean>('is_minecraft_running');
}

/**
 * Wait for a Minecraft process to exit (matches tauri::command wait_for_minecraft_exit)
 */
export async function waitForMinecraftExit(processId: number): Promise<void> {
  return await invoke('wait_for_minecraft_exit', { processId });
}