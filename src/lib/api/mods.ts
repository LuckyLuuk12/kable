import type {
  ExtendedModInfo,
  KableInstallation,
  ModFilter,
  ModInfoKind,
  ModJarInfo,
  ModpackContext,
  ModpackPrepareResult,
  ModrinthVersion,
  ProviderKind,
} from "$lib";
import { invoke } from "@tauri-apps/api/core";

// Unified mod/modpack download/prepare API
export async function downloadOrPrepareMod(
  provider: ProviderKind,
  modId: string,
  versionId: string | null,
  installation: KableInstallation,
): Promise<ModpackPrepareResult> {
  return await invoke("download_or_prepare_mod", {
    provider,
    modId,
    versionId,
    installation,
  });
}

import type { ModpackSelection } from "$lib";

export async function applyModpackSelection(
  installation: KableInstallation,
  selection: ModpackSelection,
  context: ModpackContext,
): Promise<void> {
  return await invoke("apply_modpack_selection", {
    installation,
    selection,
    context,
  });
}

export async function getMods(
  provider: ProviderKind,
  offset: number,
): Promise<ModInfoKind[]> {
  console.log(
    `[ModsAPI] Calling get_mods with provider: ${provider}, offset: ${offset}`,
  );
  const result = (await invoke("get_mods", {
    provider,
    offset,
  })) as ModInfoKind[];
  console.log(
    `[ModsAPI] get_mods returned ${Array.isArray(result) ? result.length : "non-array"} results:`,
    result,
  );
  return result;
}

export async function downloadMod(
  provider: ProviderKind,
  modId: string,
  versionId: string | null,
  installation: KableInstallation,
): Promise<void> {
  return invoke("download_mod", { provider, modId, versionId, installation });
}

export async function getProjects(
  provider: ProviderKind,
  projectIds: string[],
): Promise<ModInfoKind[]> {
  console.log(
    `[ModsAPI] Calling get_projects with provider: ${provider}, projectIds:`,
    projectIds,
  );
  const result = (await invoke("get_projects", {
    provider,
    projectIds,
  })) as ModInfoKind[];
  console.log(`[ModsAPI] get_projects returned ${result.length} projects`);
  return result;
}

export async function getProjectVersions(
  provider: ProviderKind,
  projectId: string,
  loaders?: string[],
  gameVersions?: string[],
): Promise<ModrinthVersion[]> {
  console.log(
    `[ModsAPI] Calling get_project_versions with provider: ${provider}, projectId: ${projectId}, loaders:`,
    loaders,
    "gameVersions:",
    gameVersions,
  );
  const result = (await invoke("get_project_versions", {
    provider,
    projectId,
    loaders: loaders || null,
    gameVersions: gameVersions || null,
  })) as ModrinthVersion[];
  console.log(
    `[ModsAPI] get_project_versions returned ${result.length} versions`,
  );
  return result;
}

export async function setProviderFilter(
  provider: ProviderKind,
  installation: KableInstallation | null,
  filter: ModFilter | null,
): Promise<void> {
  console.log(
    `[ModsAPI] Calling set_provider_filter with provider: ${provider}, installation:`,
    installation,
    "filter:",
    filter,
  );
  const result = await invoke("set_provider_filter", {
    provider,
    installation,
    filter,
  });
  console.log(`[ModsAPI] set_provider_filter completed:`, result);
  return result as void;
}

export async function setProviderLimit(
  provider: ProviderKind,
  limit: number,
): Promise<void> {
  return invoke("set_provider_limit", { provider, limit });
}

export async function clearProviderCache(
  provider: ProviderKind,
): Promise<void> {
  return invoke("clear_provider_cache", { provider });
}

export async function purgeStaleProviderCache(
  provider: ProviderKind,
): Promise<void> {
  return invoke("purge_stale_provider_cache", { provider });
}

export async function getExtendedModInfo(
  modJarInfo: ModJarInfo,
): Promise<ExtendedModInfo> {
  return invoke("get_extended_mod_info", { modJarInfo });
}

export interface ModMetadata {
  project_id: string;
  file_name: string;
  version_number: string;
  modrinth_version_id?: string | null;
  download_time: string;
}

export interface ModpackSourceRecord {
  provider: ProviderKind;
  mod_id: string;
  version_id?: string | null;
  modpack_name?: string | null;
  modpack_version?: string | null;
  installed_at: string;
  managed_project_ids: string[];
}

export async function getModMetadata(
  installation: KableInstallation,
  jarFilename: string,
): Promise<ModMetadata> {
  return invoke("get_mod_metadata", { installation, jarFilename });
}

export async function getModpackSourceRecords(
  installation: KableInstallation,
): Promise<ModpackSourceRecord[]> {
  return invoke("get_modpack_source_records", { installation });
}
