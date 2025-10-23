import { invoke } from '@tauri-apps/api/core';

export interface SymlinkInfo {
  id: string | null; // Custom symlink ID if stored
  source: string;
  destination: string;
  is_global: boolean;
  installation_id: string | null;
  symlink_type: 'resourcepack' | 'shader' | 'world' | 'mod' | 'custom';
  is_disabled: boolean;
  exists: boolean;
}

export class SymlinksAPI {
  /**
   * List all managed symlinks
   */
  static async listSymlinks(): Promise<SymlinkInfo[]> {
    return invoke<SymlinkInfo[]>('list_symlinks');
  }

  /**
   * Create a custom symlink
   * @param source - The file or folder to link from
   * @param destinationParent - The parent folder where the symlink will be created (symlink will have the same name as source)
   * @param installationId - Optional installation ID (null/undefined = global, always active)
   * @returns The ID of the created custom symlink
   */
  static async createCustomSymlink(source: string, destinationParent: string, installationId?: string | null): Promise<string> {
    return invoke<string>('create_custom_symlink', { source, destinationParent, installationId: installationId || null });
  }

  /**
   * Remove a symlink by its destination path
   * @param destination - The full path to the symlink
   * @param id - Optional custom symlink ID (for stored symlinks)
   */
  static async removeSymlink(destination: string, id?: string | null): Promise<void> {
    return invoke('remove_symlink', { destination, id: id || null });
  }

  /**
   * Toggle symlink disabled state
   * Returns the new disabled state (true = disabled, false = enabled)
   * @param destination - The full path to the symlink
   * @param id - Optional custom symlink ID (for stored symlinks)
   */
  static async toggleSymlinkDisabled(destination: string, id?: string | null): Promise<boolean> {
    return invoke<boolean>('toggle_symlink_disabled', { destination, id: id || null });
  }

  /**
   * Update an existing symlink with new source/destination paths
   * @param id - Optional custom symlink ID (for stored symlinks)
   * @param oldDestination - The full path to the existing symlink
   * @param newSource - The new source file/folder path
   * @param newDestinationParent - The parent folder where the symlink will be created (symlink will have the same name as source)
   * @param newInstallationId - Optional new installation ID (null = global, undefined = don't change)
   */
  static async updateSymlink(
    id: string | null | undefined,
    oldDestination: string,
    newSource: string,
    newDestinationParent: string,
    newInstallationId?: string | null
  ): Promise<void> {
    return invoke('update_symlink', {
      id: id || null,
      oldDestination,
      newSource,
      newDestinationParent,
      newInstallationId: newInstallationId === undefined ? null : newInstallationId
    });
  }

  /**
   * Select a file using the system file dialog
   * @returns The selected file path, or null if cancelled
   */
  static async selectFileForSymlink(): Promise<string | null> {
    return invoke<string | null>('select_file_for_symlink');
  }

  /**
   * Select a folder using the system file dialog
   * @returns The selected folder path, or null if cancelled
   */
  static async selectFolderForSymlink(): Promise<string | null> {
    return invoke<string | null>('select_folder_for_symlink');
  }
}
