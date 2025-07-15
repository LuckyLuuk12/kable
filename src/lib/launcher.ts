import { invoke } from '@tauri-apps/api/core';
import type { LauncherSettings, MicrosoftAccount } from './types';

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

/**
 * Launch System - Handles Minecraft launching across the app
 */
export class LaunchSystem {
    private static instance: LaunchSystem;
    
    public static getInstance(): LaunchSystem {
        if (!LaunchSystem.instance) {
            LaunchSystem.instance = new LaunchSystem();
        }
        return LaunchSystem.instance;
    }
    
    /**
     * Quick launch Minecraft by version name
     * Used for home page "Play" buttons
     */
    async quickLaunch(versionName: string): Promise<LaunchResult> {
        try {
            await invoke('quick_launch_minecraft', { versionName });
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
     * Launch a specific installation
     * Used for installations page and detailed launch options
     */
    async launchInstallation(installationId: string): Promise<LaunchResult> {
        try {
            await invoke('launch_minecraft_installation', { installationId });
            return { success: true };
        } catch (error) {
            console.error('Installation launch failed:', error);
            return { 
                success: false, 
                error: error instanceof Error ? error.message : String(error)
            };
        }
    }
    
    /**
     * Launch with custom options
     * Advanced launch method with full customization
     */
    async launchWithOptions(options: ExtendedLaunchOptions): Promise<LaunchResult> {
        if (options.installationId) {
            return this.launchInstallation(options.installationId);
        } else if (options.version) {
            return this.quickLaunch(options.version);
        } else {
            return {
                success: false,
                error: 'Either version or installationId must be specified'
            };
        }
    }
    
    /**
     * Get available Minecraft versions from .minecraft/versions/
     */
    async getAvailableVersions(): Promise<string[]> {
        try {
            return await invoke('get_minecraft_versions');
        } catch (error) {
            console.error('Failed to get versions:', error);
            return [];
        }
    }
    
    /**
     * Check if a version exists and is launchable
     */
    async isVersionAvailable(versionName: string): Promise<boolean> {
        const versions = await this.getAvailableVersions();
        return versions.includes(versionName);
    }
    
    /**
     * Validate launch prerequisites (auth, settings, etc.)
     */
    async validateLaunchPrerequisites(): Promise<{ valid: boolean; errors: string[] }> {
        const errors: string[] = [];
        
        try {
            // Check authentication using simplified auth status
            const authStatus = await invoke('check_auth_status') as { authenticated: boolean; username?: string };
            if (!authStatus.authenticated) {
                errors.push('No authenticated Microsoft account found. Please log in first.');
            }
            
            // Check settings
            const settings = await invoke('load_settings') as LauncherSettings;
            if (!settings.minecraft_path) {
                errors.push('Minecraft directory not configured. Please check settings.');
            }
            
            // Check Java using simplified Java detection
            try {
                await invoke('get_java_path', { javaPath: settings.java_path || null });
            } catch (error) {
                errors.push('Java not found. Please install Java or configure Java path in settings.');
            }
            
        } catch (error) {
            errors.push(`Failed to validate prerequisites: ${error}`);
        }
        
        return {
            valid: errors.length === 0,
            errors
        };
    }
}

/**
 * Exported functions for easy use across components
 */

/**
 * Quick launch for home page - launches the most recent/default installation
 */
export async function quickLaunchDefault(): Promise<LaunchResult> {
    try {
        await invoke('launch_most_recent_installation');
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
 * Launch by version name - for specific version selection
 */
export async function launchVersion(versionName: string): Promise<LaunchResult> {
    const launcher = LaunchSystem.getInstance();
    return launcher.quickLaunch(versionName);
}

/**
 * Launch installation - for installations page
 */
export async function launchInstallation(installationId: string): Promise<LaunchResult> {
    const launcher = LaunchSystem.getInstance();
    return launcher.launchInstallation(installationId);
}

/**
 * Get launch status and show user-friendly messages
 */
export function formatLaunchResult(result: LaunchResult): string {
    if (result.success) {
        return 'Launched Minecraft!';
    } else {
        return result.error || 'Unknown launch error occurred';
    }
}

/**
 * Validate and prepare for launch
 */
export async function prepareForLaunch(): Promise<{ ready: boolean; message: string }> {
    const launcher = LaunchSystem.getInstance();
    const validation = await launcher.validateLaunchPrerequisites();
    
    if (validation.valid) {
        return {
            ready: true,
            message: 'Ready to launch Minecraft'
        };
    } else {
        return {
            ready: false,
            message: validation.errors.join('\n')
        };
    }
}

// Default export
export default LaunchSystem;
