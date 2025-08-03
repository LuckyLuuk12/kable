import { invoke } from '@tauri-apps/api/core';
import type { KableInstallation, ModJarInfo, VersionData } from '../types';

// Get all versions (optionally force refresh)
export async function get_all_versions(force = false): Promise<VersionData[]> {
  return await invoke('get_all_versions', { force });
}

// Get a single version by id
export async function get_version(version_id: string): Promise<VersionData | null> {
  return await invoke('get_version', { version_id });
}

// Get all Kable installations
export async function get_installations(): Promise<KableInstallation[]> {
  return await invoke('get_installations');
}

// Get a single Kable installation by id
export async function get_installation(id: string): Promise<KableInstallation | null> {
  return await invoke('get_installation', { id });
}

// Modify an existing Kable installation
export async function modify_installation(id: string, new_installation: KableInstallation): Promise<void> {
  return await invoke('modify_installation', { id, newInstallation: new_installation });
}

// Delete a Kable installation by id
export async function delete_installation(id: string): Promise<void> {
  return await invoke('delete_installation', { id });
}

// Create a new Kable installation for a given version_id
export async function create_installation(version_id: string): Promise<KableInstallation> {
  return await invoke('create_installation', { version_id });
}

export async function get_mod_info(installation: KableInstallation): Promise<ModJarInfo[] | null> {
  return await invoke('get_mod_info', { installation });
}