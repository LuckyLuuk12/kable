import { invoke } from '@tauri-apps/api/core';
import type { LauncherSettings } from '../types';

/**
 * Launcher API
 * Pure Tauri invoke calls for Minecraft launching operations
 */

export interface ExtendedLaunchOptions {
    version?: string;
    installationId?: string;
    customGameDir?: string;
    javaPath?: string;
    jvmArgs?: string[];
    windowWidth?: number;
    windowHeight?: number;
}

export interface LaunchResult {
    success: boolean;
    error?: string;
    processId?: number;
}

// Quick launch functions
export async function quickLaunchMinecraft(versionName: string): Promise<void> {
    return await invoke('quick_launch_minecraft', { versionName });
}

export async function launchWithOptions(options: ExtendedLaunchOptions): Promise<void> {
    return await invoke('launch_with_options', { options });
}

export async function launchLatestVersion(): Promise<void> {
    return await invoke('launch_latest_version');
}

export async function launchInstallation(installationId: string): Promise<void> {
    return await invoke('launch_installation', { installationId });
}

// Process management
export async function killMinecraftProcess(processId: number): Promise<void> {
    return await invoke('kill_minecraft_process', { processId });
}

export async function getRunningMinecraftProcesses(): Promise<number[]> {
    return await invoke('get_running_minecraft_processes');
}

// Launch monitoring
export async function isMinecraftRunning(): Promise<boolean> {
    return await invoke('is_minecraft_running');
}

export async function waitForMinecraftExit(processId: number): Promise<void> {
    return await invoke('wait_for_minecraft_exit', { processId });
}
