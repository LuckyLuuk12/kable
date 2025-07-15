import { invoke } from '@tauri-apps/api/core';
import type { ModInstallationConfig, InstalledMod } from '../types';

export class ModsManager {
    /**
     * Get all modded installations (excludes vanilla)
     */
    static async getModdedInstallations(minecraftPath: string): Promise<ModInstallationConfig[]> {
        return await invoke('get_modded_installations', { minecraftPath });
    }

    /**
     * Set up mod folder for an installation
     */
    static async setupInstallationMods(
        minecraftPath: string,
        installationId: string,
        useGlobal: boolean
    ): Promise<string> {
        return await invoke('setup_installation_mods', {
            minecraftPath,
            installationId,
            useGlobal
        });
    }

    /**
     * Get installed mods for a specific installation
     */
    static async getInstalledMods(
        minecraftPath: string,
        installationId: string
    ): Promise<InstalledMod[]> {
        return await invoke('get_installed_mods', {
            minecraftPath,
            installationId
        });
    }

    /**
     * Toggle mod enabled/disabled state
     */
    static async toggleModEnabled(modFilePath: string, enabled: boolean): Promise<void> {
        return await invoke('toggle_mod_enabled', {
            modFilePath,
            enabled
        });
    }

    /**
     * Update installation mod configuration
     */
    static async updateInstallationModConfig(
        minecraftPath: string,
        installationId: string,
        useGlobalMods: boolean,
        customModsPath?: string
    ): Promise<void> {
        return await invoke('update_installation_mod_config', {
            minecraftPath,
            installationId,
            useGlobalMods,
            customModsPath
        });
    }
}
