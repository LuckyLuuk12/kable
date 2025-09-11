import { invoke } from '@tauri-apps/api/core';
import type { KableInstallation, ModJarInfo, VersionData } from '../types';

// Get all versions (optionally force refresh)
export async function getAllVersions(force = false): Promise<VersionData[]> {
  return await invoke('get_all_versions', { force });
}

// Get a single version by id
export async function getVersion(version_id: string): Promise<VersionData | null> {
  return await invoke('get_version', { version_id });
}

// Get all Kable installations
export async function getInstallations(): Promise<KableInstallation[]> {
  return await invoke('get_installations');
}

// Get a single Kable installation by id
export async function getInstallation(id: string): Promise<KableInstallation | null> {
  return await invoke('get_installation', { id });
}

// Modify an existing Kable installation
export async function modifyInstallation(id: string, new_installation: KableInstallation): Promise<void> {
  return await invoke('modify_installation', { id, newInstallation: new_installation });
}

// Delete a Kable installation by id
export async function deleteInstallation(id: string): Promise<void> {
  return await invoke('delete_installation', { id });
}

// Create a new Kable installation for a given version_id
export async function createInstallation(version_id: string): Promise<KableInstallation> {
  return await invoke('create_installation', { version_id });
}

export async function getModInfo(installation: KableInstallation): Promise<ModJarInfo[] | null> {
  return await invoke('get_mod_info', { installation });
}

// Disable a mod by moving the jar into the installation's disabled/ subfolder
export async function disableMod(installation: KableInstallation, fileName: string): Promise<void> {
  return await invoke('disable_mod', { installation, file_name: fileName, fileName });
}

// Enable a mod by moving the jar out of the installation's disabled/ subfolder
export async function enableMod(installation: KableInstallation, fileName: string): Promise<void> {
  return await invoke('enable_mod', { installation, file_name: fileName, fileName });
}

// Toggle the disabled state for a mod; returns the new disabled state (true = disabled)
export async function toggleModDisabled(installation: KableInstallation, fileName: string): Promise<boolean> {
  return await invoke('toggle_mod_disabled', { installation, file_name: fileName, fileName });
}