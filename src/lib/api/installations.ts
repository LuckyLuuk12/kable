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
  console.log('[API] modifyInstallation called with:', { id, new_installation });
  return await invoke('modify_installation', { id: id, new_installation: new_installation });
}

// Delete a Kable installation by id
export async function deleteInstallation(id: string): Promise<void> {
  return await invoke('delete_installation', { id });
}

// Create a new Kable installation for a given versionId
export async function createInstallation(versionId: string): Promise<KableInstallation> {
  return await invoke('create_installation', { versionId });
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

// Import an installation from a path
export async function importInstallation(path: string): Promise<KableInstallation> {
  return await invoke('import', { path });
}

// Export an installation as a string (serialized)
export async function exportInstallation(installation: KableInstallation): Promise<string> {
  return await invoke('export', { installation });
}

// Duplicate an installation and return the new list of installations
export async function duplicateInstallation(installation: KableInstallation): Promise<KableInstallation[]> {
  return await invoke('duplicate', { installation });
}

