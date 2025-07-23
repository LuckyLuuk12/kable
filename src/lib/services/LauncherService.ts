import type { KableInstallation } from '$lib/types';
import type { ExtendedLaunchOptions, LaunchResult } from '../api/launcher';
import * as launcherApi from '../api/launcher';

/**
 * Launch Service
 * High-level business logic for Minecraft launching operations
 */

export class LaunchService {
    private static instance: LaunchService;
    
    public static getInstance(): LaunchService {
        if (!LaunchService.instance) {
            LaunchService.instance = new LaunchService();
        }
        return LaunchService.instance;
    }
    
    /**
     * Quick launch Minecraft by version name
     * Used for home page "Play" buttons
     */
    async quickLaunch(versionName: string): Promise<LaunchResult> {
        try {
            await launcherApi.quickLaunchMinecraft(versionName);
            return { success: true };
        } catch (error) {
            console.error('Quick launch failed:', error);
            return { 
                success: false, 
                error: error instanceof Error ? error.message : String(error)
            };
        }
    }
    
    /**
     * Launch with custom options
     */
    async launchWithOptions(options: ExtendedLaunchOptions): Promise<LaunchResult> {
        try {
            await launcherApi.launchWithOptions(options);
            return { success: true };
        } catch (error) {
            console.error('Launch with options failed:', error);
            return { 
                success: false, 
                error: error instanceof Error ? error.message : String(error)
            };
        }
    }
    
    /**
     * Launch latest Minecraft version
     */
    async launchLatest(): Promise<LaunchResult> {
        try {
            await launcherApi.launchLatestVersion();
            return { success: true };
        } catch (error) {
            console.error('Launch latest failed:', error);
            return { 
                success: false, 
                error: error instanceof Error ? error.message : String(error)
            };
        }
    }
    
    /**
     * Launch specific installation
     */
    async launchInstallationById(installationId: string): Promise<LaunchResult> {
        try {
            await launcherApi.launchInstallation(installationId);
            return { success: true };
        } catch (error) {
            console.error('Launch installation failed:', error);
            return { 
                success: false, 
                error: error instanceof Error ? error.message : String(error)
            };
        }
    }
    async launchInstallation(installation: KableInstallation): Promise<LaunchResult> {
        return this.launchInstallationById(installation.id);
    }
    
    /**
     * Check if Minecraft is currently running
     */
    async isMinecraftRunning(): Promise<boolean> {
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
    async getRunningProcesses(): Promise<number[]> {
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
    async killProcess(processId: number): Promise<LaunchResult> {
        try {
            await launcherApi.killMinecraftProcess(processId);
            return { success: true };
        } catch (error) {
            console.error('Failed to kill process:', error);
            return { 
                success: false, 
                error: error instanceof Error ? error.message : String(error)
            };
        }
    }
    
    /**
     * Wait for a Minecraft process to exit
     */
    async waitForExit(processId: number): Promise<void> {
        try {
            await launcherApi.waitForMinecraftExit(processId);
        } catch (error) {
            console.error('Failed to wait for process exit:', error);
        }
    }
}
