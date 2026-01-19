import { invoke } from "@tauri-apps/api/core";
import type { KableInstallation, ModJarInfo, VersionData } from "../types";

// Get all versions (optionally force refresh)
export async function getAllVersions(force = false): Promise<VersionData[]> {
  return await invoke("get_all_versions", { force });
}

// Force refresh version manifests from the network
export async function refreshVersionManifests(): Promise<VersionData[]> {
  return await invoke("refresh_version_manifests");
}

// Get a single version by id
export async function getVersion(
  version_id: string,
): Promise<VersionData | null> {
  return await invoke("get_version", { version_id });
}

// Get all Kable installations
export async function getInstallations(): Promise<KableInstallation[]> {
  return await invoke("get_installations");
}

// Force refresh installations bypassing cache
export async function refreshInstallations(): Promise<KableInstallation[]> {
  return await invoke("get_installations_force");
}

// Get a single Kable installation by id
export async function getInstallation(
  id: string,
): Promise<KableInstallation | null> {
  return await invoke("get_installation", { id });
}

// Modify an existing Kable installation
export async function modifyInstallation(
  id: string,
  new_installation: KableInstallation,
): Promise<void> {
  return await invoke("modify_installation", {
    id: id,
    newInstallation: new_installation,
  });
}

// Delete a Kable installation by id
export async function deleteInstallation(id: string): Promise<void> {
  return await invoke("delete_installation", { id });
}

// Create a new Kable installation for a given versionId
export async function createInstallation(
  versionId: string,
): Promise<KableInstallation> {
  return await invoke("create_installation", { versionId });
}

// Create a new Kable installation by copying from an existing one
export async function createInstallationFromExisting(
  versionId: string,
  sourceInstallationId: string,
  options: {
    copyMods: boolean;
    copyResourcePacks: boolean;
    copyShaders: boolean;
  },
): Promise<KableInstallation> {
  return await invoke("create_installation_from_existing", {
    versionId,
    sourceInstallationId,
    copyMods: options.copyMods,
    copyResourcePacks: options.copyResourcePacks,
    copyShaders: options.copyShaders,
  });
}

export async function getModInfo(
  installation: KableInstallation,
): Promise<ModJarInfo[] | null> {
  return await invoke("get_mod_info", { installation });
}

// Disable a mod by moving the jar into the installation's disabled/ subfolder
export async function disableMod(
  installation: KableInstallation,
  fileName: string,
): Promise<void> {
  return await invoke("disable_mod", {
    installation,
    fileName,
  });
}

// Enable a mod by moving the jar out of the installation's disabled/ subfolder
export async function enableMod(
  installation: KableInstallation,
  fileName: string,
): Promise<void> {
  return await invoke("enable_mod", {
    installation,
    fileName,
  });
}

// Toggle the disabled state for a mod; returns the new disabled state (true = disabled)
export async function toggleModDisabled(
  installation: KableInstallation,
  fileName: string,
): Promise<boolean> {
  return await invoke("toggle_mod_disabled", {
    installation,
    fileName,
  });
}

// Delete/remove a mod from installation
export async function deleteMod(
  installation: KableInstallation,
  fileName: string,
): Promise<void> {
  return await invoke("delete_mod", {
    installation,
    fileName,
  });
}

// Disable a resource pack by moving it into the disabled/ subfolder
export async function disableResourcePack(
  installation: KableInstallation,
  fileName: string,
): Promise<void> {
  return await invoke("disable_resourcepack_for_installation", {
    installation,
    fileName,
  });
}

// Enable a resource pack by moving it out of the disabled/ subfolder
export async function enableResourcePack(
  installation: KableInstallation,
  fileName: string,
): Promise<void> {
  return await invoke("enable_resourcepack_for_installation", {
    installation,
    fileName,
  });
}

// Toggle the disabled state for a resource pack; returns the new disabled state (true = disabled)
export async function toggleResourcePackDisabled(
  installation: KableInstallation,
  fileName: string,
): Promise<boolean> {
  return await invoke("toggle_resourcepack_disabled_for_installation", {
    installation,
    fileName,
  });
}

// Delete/remove a resource pack from installation
export async function deleteResourcePack(
  installation: KableInstallation,
  fileName: string,
): Promise<void> {
  return await invoke("delete_resourcepack_for_installation", {
    installation,
    fileName,
  });
}

// Get resource pack info for an installation
export async function getResourcePackInfo(
  installation: KableInstallation,
): Promise<any[]> {
  return await invoke("get_resourcepack_info_for_installation", {
    installation,
  });
}

// Get global resource packs from .minecraft/resourcepacks
export async function getGlobalResourcePacks(): Promise<any[]> {
  return await invoke("get_global_resourcepacks");
}

// Update resource pack settings (order and merging)
export async function updateResourcePackSettings(
  installationId: string,
  enablePackMerging: boolean,
  packOrder: string[],
): Promise<void> {
  return await invoke("update_resourcepack_settings", {
    installationId,
    enablePackMerging,
    packOrder,
  });
}

// Disable a shader pack by moving it into the disabled/ subfolder
export async function disableShader(
  installation: KableInstallation,
  fileName: string,
): Promise<void> {
  return await invoke("disable_shader_for_installation", {
    installation,
    fileName,
  });
}

// Enable a shader pack by moving it out of the disabled/ subfolder
export async function enableShader(
  installation: KableInstallation,
  fileName: string,
): Promise<void> {
  return await invoke("enable_shader_for_installation", {
    installation,
    fileName,
  });
}

// Toggle the disabled state for a shader pack; returns the new disabled state (true = disabled)
export async function toggleShaderDisabled(
  installation: KableInstallation,
  fileName: string,
): Promise<boolean> {
  return await invoke("toggle_shader_disabled_for_installation", {
    installation,
    fileName,
  });
}

// Delete/remove a shader pack from installation
export async function deleteShaderFromInstallation(
  installation: KableInstallation,
  fileName: string,
): Promise<void> {
  return await invoke("delete_shader_for_installation", {
    installation,
    fileName,
  });
}

// Get shader pack info for an installation
export async function getShaderPackInfo(
  installation: KableInstallation,
): Promise<any[]> {
  return await invoke("get_shaderpack_info_for_installation", { installation });
}

// Get global shader packs from .minecraft/shaderpacks
export async function getGlobalShaderPacks(): Promise<any[]> {
  return await invoke("get_global_shaderpacks");
}

// Import an installation from a path
export async function importInstallation(
  path: string,
): Promise<KableInstallation> {
  return await invoke("import", { path });
}

// Import installations from a .minecraft folder
export async function importFromMinecraftFolder(
  path: string,
): Promise<KableInstallation[]> {
  return await invoke("import_from_minecraft_folder", { path });
}

// Export an installation as a string (serialized)
export async function exportInstallation(
  installation: KableInstallation,
): Promise<string> {
  return await invoke("export", { installation });
}

// Duplicate an installation and return the new list of installations
export async function duplicateInstallation(
  installation: KableInstallation,
): Promise<KableInstallation[]> {
  return await invoke("duplicate", { installation });
}

// Create a desktop shortcut for an installation
export async function createShortcut(
  installation: KableInstallation,
): Promise<string> {
  return await invoke("create_shortcut", { installation });
}

// Select a zip file for importing a Kable installation
export async function selectInstallationZip(): Promise<string | null> {
  return await invoke("select_installation_zip");
}

// Select a .minecraft folder for importing installations
export async function selectMinecraftFolder(): Promise<string | null> {
  return await invoke("select_minecraft_folder");
}
