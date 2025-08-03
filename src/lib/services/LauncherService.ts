
import { currentAccount, installations, settings } from '$lib/stores';
import type { KableInstallation, LaunchResult } from '$lib';
import { get } from 'svelte/store';
import * as launcherApi from '../api/launcher';


export class Launcher {
    /**
     * Launch latest Minecraft version
     */
    static async launchLatest(): Promise<LaunchResult> {
        try {
            let latest = get(installations).sort((a: KableInstallation, b: KableInstallation) => b.last_used.localeCompare(a.last_used))[0];
            return await Launcher.launchInstallation(latest);
        } catch (error) {
            console.error('Launch latest failed:', error);
            return {
                pid: -1,
                success: false,
                error: error instanceof Error ? error.message : String(error)
            };
        }
    }

    /**
     * Launch specific installation
     */
    static async launchInstallation(installation: KableInstallation): Promise<LaunchResult> {
        try {
            const account = get(currentAccount);
            if (!account) throw new Error('No account selected');
            return await launcherApi.launchInstallation(installation, get(settings), account);
        } catch (error) {
            console.error('Launching ' + installation.name + ' failed with account ' + JSON.stringify(get(currentAccount)), error);
            return {
                pid: -1,
                success: false,
                error: error instanceof Error ? error.message : String(error)
            };
        }
    }

    /**
     * Check if Minecraft is currently running
     */
    static async isMinecraftRunning(): Promise<boolean> {
        try {
            return await launcherApi.isMinecraftRunning();
        } catch (error) {
            console.error('Failed to check Minecraft status:', error);
            return false;
        }
    }

    /**
     * Get all running Minecraft processes
     */
    static async getRunningProcesses(): Promise<number[]> {
        try {
            return await launcherApi.getRunningMinecraftProcesses();
        } catch (error) {
            console.error('Failed to get running processes:', error);
            return [];
        }
    }

    /**
     * Kill a specific Minecraft process
     */
    static async killProcess(processId: number): Promise<LaunchResult> {
        try {
            await launcherApi.killMinecraftProcess(processId);
            return { success: true, pid: processId };
        } catch (error) {
            console.error('Failed to kill process:', error);
            return {
                pid: processId,
                success: false,
                error: error instanceof Error ? error.message : String(error)
            };
        }
    }

    /**
     * Wait for a Minecraft process to exit
     */
    static async waitForExit(processId: number): Promise<void> {
        try {
            await launcherApi.waitForMinecraftExit(processId);
        } catch (error) {
            console.error('Failed to wait for process exit:', error);
        }
    }
}
