import * as modsApi from "../api/mods";
import { get } from "svelte/store";
import * as semver from "semver";
import {
  modsByProvider,
  modsLoading,
  modsError,
  modsLimit,
  modsOffset,
  modsFilter,
  modsInstallation,
  modsProvider,
  extendedModInfo,
  NotificationService,
} from "$lib";
import type {
  ProviderKind,
  ModInfoKind,
  KableInstallation,
  ModFilter,
  ModJarInfo,
  ExtendedModInfo,
} from "$lib";

/**
 * Version comparison utilities for mod version management
 * Handles various version formats found in mod JARs and metadata:
 * - Standard semver: 1.2.3
 * - With metadata: 1.2.3+fabric-1.21.11
 * - Reversed formats: 1.21.11-fabric-1.2.3
 * - Pre-release versions: 1.2.3-beta.1
 */
export class VersionUtils {
  /**
   * Extract version string from various formats
   * Examples:
   * - "somemodname-1.2.3+fabric-1.21.11.jar" -> "1.2.3+fabric-1.21.11"
   * - "mod-fabric-1.21.11-1.2.3.jar" -> "1.2.3"
   * - "1.2.3-beta" -> "1.2.3-beta"
   * - "v1.2.3" -> "1.2.3"
   */
  static extractVersion(input: string): string | null {
    if (!input) return null;

    // Remove .jar extension if present
    let str = input.replace(/\.jar$/i, "");

    // Remove common prefixes like "v", "version-", etc.
    str = str.replace(/^(v|version[-_]?)/i, "");

    // Try to find semver-like patterns (major.minor.patch with optional pre-release/build metadata)
    // Match patterns like: 1.2.3, 1.2.3-beta.1, 1.2.3+fabric-1.21.11, etc.
    const semverPattern = /(\d+\.\d+(?:\.\d+)?(?:[-+][a-zA-Z0-9.-]+)*)/g;
    const matches = str.match(semverPattern);

    if (!matches || matches.length === 0) return null;

    // If we have multiple version-like strings, try to identify the mod version
    // Usually the mod version comes before loader/minecraft version
    // Heuristic: prefer versions that don't start with "1.1[0-9]" or "1.20" (likely Minecraft versions)
    const modVersions = matches.filter((v) => {
      // Skip likely Minecraft versions (1.12, 1.16, 1.17, 1.18, 1.19, 1.20, 1.21, etc.)
      if (/^1\.(1[2-9]|2[0-9])/.test(v)) return false;
      return true;
    });

    // Return the first non-Minecraft version, or first match if all look like MC versions
    return modVersions.length > 0 ? modVersions[0] : matches[0];
  }

  /**
   * Normalize version for semver comparison
   * Converts various formats into valid semver strings
   */
  static normalizeVersion(version: string): string | null {
    const extracted = this.extractVersion(version);
    if (!extracted) return null;

    // Split on + to separate build metadata
    const [versionPart, ...buildParts] = extracted.split("+");
    const buildMetadata = buildParts.join("+");

    // Ensure we have at least major.minor.patch format
    const parts = versionPart.split(/[-.]/).filter((p) => p);
    if (parts.length < 2) return null;

    // Take first 3 numeric parts as major.minor.patch
    const numericParts = parts.filter((p) => /^\d+$/.test(p)).slice(0, 3);
    while (numericParts.length < 3) {
      numericParts.push("0");
    }

    // Find pre-release identifier (everything after first non-numeric part)
    const firstNonNumericIndex = parts.findIndex((p) => !/^\d+$/.test(p));
    const preRelease =
      firstNonNumericIndex >= 0
        ? parts.slice(firstNonNumericIndex).join(".")
        : "";

    let normalized = numericParts.join(".");
    if (preRelease) normalized += `-${preRelease}`;
    if (buildMetadata) normalized += `+${buildMetadata}`;

    // Validate it's proper semver
    if (!semver.valid(semver.coerce(normalized))) {
      return null;
    }

    return normalized;
  }

  /**
   * Compare two version strings
   * Returns:
   * - positive number if v1 > v2 (v1 is newer)
   * - negative number if v1 < v2 (v2 is newer)
   * - 0 if equal
   */
  static compareVersions(v1: string, v2: string): number {
    const norm1 = this.normalizeVersion(v1);
    const norm2 = this.normalizeVersion(v2);

    if (!norm1 && !norm2) return 0;
    if (!norm1) return -1;
    if (!norm2) return 1;

    try {
      const result = semver.compare(norm1, norm2);
      return result;
    } catch (e) {
      console.warn(
        `[VersionUtils] Failed to compare versions: ${v1} vs ${v2}`,
        e,
      );
      // Fallback to string comparison
      return v1.localeCompare(v2);
    }
  }

  /**
   * Check if version1 is newer than version2
   */
  static isNewer(version1: string, version2: string): boolean {
    return this.compareVersions(version1, version2) > 0;
  }

  /**
   * Check if version1 is older than version2
   */
  static isOlder(version1: string, version2: string): boolean {
    return this.compareVersions(version1, version2) < 0;
  }

  /**
   * Check if two versions are equal
   */
  static isEqual(version1: string, version2: string): boolean {
    return this.compareVersions(version1, version2) === 0;
  }

  /**
   * Find the latest version from an array of version strings
   */
  static findLatest(versions: string[]): string | null {
    if (!versions || versions.length === 0) return null;

    return versions.reduce((latest, current) => {
      if (!latest) return current;
      return this.isNewer(current, latest) ? current : latest;
    }, versions[0]);
  }
}

export class ModsService {
  initialized = false;

  constructor(provider: ProviderKind) {
    console.log(`[ModsService] Initializing with ${provider} provider`);
    modsProvider.set(provider);
  }

  async initialize() {
    if (this.initialized) return;
    await this.loadMods();
    this.initialized = true;
  }

  async loadMods() {
    modsLoading.set(true);
    modsError.set(null);
    const provider = get(modsProvider);
    const offset = get(modsOffset);
    const filter = get(modsFilter);
    const installation = get(modsInstallation);

    if (!provider) {
      modsError.set("No provider selected");
      modsLoading.set(false);
      return;
    }

    // Log the request details
    console.log(
      `[ModsService] Loading mods from ${provider} provider (offset: ${offset}, filter: ${filter ? JSON.stringify(filter) : "none"}, installation: ${installation?.name || "none"})`,
    );

    try {
      const mods = await modsApi.getMods(provider, offset);
      console.log(
        `[ModsService] Successfully loaded ${mods.length} mods from ${provider} provider`,
      );
      modsByProvider.update((map) => ({ ...map, [provider]: mods }));
    } catch (e: any) {
      console.error(
        `[ModsService] Failed to load mods from ${provider} provider:`,
        e.message || "Unknown error",
      );
      modsError.set(e.message || "Failed to load mods");
    } finally {
      modsLoading.set(false);
    }
  }

  async setLimit(limit: number) {
    modsLimit.set(limit);
    const provider = get(modsProvider);
    if (!provider) {
      modsError.set("No provider selected");
      return;
    }

    console.log(
      `[ModsService] Setting mod limit to ${limit} for ${provider} provider`,
    );

    try {
      await modsApi.setProviderLimit(provider, limit);
      await this.loadMods();
    } catch (e: any) {
      console.error(
        `[ModsService] Failed to set limit for ${provider} provider:`,
        e.message || "Unknown error",
      );
      throw e;
    }
  }

  async setFilter(
    filter: ModFilter | null,
    installation: KableInstallation | null,
  ) {
    modsFilter.set(filter);
    modsInstallation.set(installation);
    const provider = get(modsProvider);
    if (!provider) {
      modsError.set("No provider selected");
      return;
    }

    console.log(
      `[ModsService] Setting filter for ${provider} provider: ${filter ? JSON.stringify(filter) : "none"} (installation: ${installation?.name || "none"})`,
    );

    try {
      await modsApi.setProviderFilter(provider, installation, filter);
      await this.loadMods();
    } catch (e: any) {
      console.error(
        `[ModsService] Failed to set filter for ${provider} provider:`,
        e.message || "Unknown error",
      );
      throw e;
    }
  }

  async nextPage() {
    const limit = get(modsLimit);
    const offset = get(modsOffset) + limit;
    const provider = get(modsProvider);

    console.log(
      `[ModsService] Moving to next page for ${provider} provider (new offset: ${offset})`,
    );

    modsOffset.set(offset);
    await this.loadMods();
  }

  async prevPage() {
    const limit = get(modsLimit);
    const offset = Math.max(0, get(modsOffset) - limit);
    const provider = get(modsProvider);

    console.log(
      `[ModsService] Moving to previous page for ${provider} provider (new offset: ${offset})`,
    );

    modsOffset.set(offset);
    await this.loadMods();
  }

  async downloadMod(
    modId: string,
    versionId: string | null,
    installation: KableInstallation,
  ) {
    modsLoading.set(true);
    modsError.set(null);
    const provider = get(modsProvider);
    if (!provider) {
      modsError.set("No provider selected");
      modsLoading.set(false);
      return;
    }

    console.log(
      `[ModsService] Downloading mod ${modId}${versionId ? ` (version: ${versionId})` : ""} from ${provider} provider to installation "${installation.name}"`,
    );

    try {
      await modsApi.downloadMod(provider, modId, versionId, installation);
      console.log(
        `[ModsService] Successfully downloaded mod ${modId} from ${provider} provider`,
      );
      NotificationService.success(`Mod downloaded successfully`);
    } catch (e: any) {
      console.error(
        `[ModsService] Failed to download mod ${modId} from ${provider} provider to ${installation.dedicated_mods_folder}:`,
        e.message || e || "Unknown error",
      );
      const errorMsg = e.message || "Failed to download mod";
      modsError.set(errorMsg);
      NotificationService.error(`Failed to download mod: ${errorMsg}`);
    } finally {
      modsLoading.set(false);
    }
  }

  async clearProviderCache() {
    const provider = get(modsProvider);
    if (!provider) {
      modsError.set("No provider selected");
      return;
    }

    console.log(`[ModsService] Clearing cache for ${provider} provider`);

    try {
      await modsApi.clearProviderCache(provider);
      await this.loadMods();
      console.log(
        `[ModsService] Successfully cleared cache for ${provider} provider`,
      );
      NotificationService.success(`Cache cleared for ${provider}`);
    } catch (e: any) {
      console.error(
        `[ModsService] Failed to clear cache for ${provider} provider:`,
        e.message || "Unknown error",
      );
      NotificationService.error(
        `Failed to clear cache: ${e.message || "Unknown error"}`,
      );
      throw e;
    }
  }

  async purgeStaleProviderCache() {
    const provider = get(modsProvider);
    if (!provider) {
      modsError.set("No provider selected");
      return;
    }

    console.log(`[ModsService] Purging stale cache for ${provider} provider`);

    try {
      await modsApi.purgeStaleProviderCache(provider);
      await this.loadMods();
      console.log(
        `[ModsService] Successfully purged stale cache for ${provider} provider`,
      );
    } catch (e: any) {
      console.error(
        `[ModsService] Failed to purge stale cache for ${provider} provider:`,
        e.message || "Unknown error",
      );
      throw e;
    }
  }

  // Helpers for UI
  getMods() {
    const provider = get(modsProvider);
    if (!provider) return [];
    return get(modsByProvider)[provider] || [];
  }
  isLoading() {
    return get(modsLoading);
  }
  getError() {
    return get(modsError);
  }

  //  Concurrency-limited queue for extended mod info requests
  static #pending: (() => Promise<void>)[] = [];
  static #inFlight = 0;
  static #MAX_CONCURRENT = 3; // You can tune this (2-4 is safe)

  static async getExtendedModInfo(
    modJarInfo: ModJarInfo,
  ): Promise<ExtendedModInfo | null> {
    const fileName = modJarInfo.file_name;
    if (get(extendedModInfo)[fileName]) {
      return get(extendedModInfo)[fileName] || null;
    }

    console.log(`[ModsService] Fetching extended mod info for ${fileName}`);

    // Wrap the actual fetch in a function for the queue
    return new Promise<ExtendedModInfo | null>((resolve) => {
      const task = async () => {
        try {
          const result = await modsApi.getExtendedModInfo(modJarInfo);
          extendedModInfo.set({
            ...get(extendedModInfo),
            [fileName]: result,
          });
          console.log(
            `[ModsService] Successfully fetched extended mod info for ${fileName}`,
          );
          resolve(result || null);
        } catch (e) {
          // Store a null value in the store to prevent infinite retries
          extendedModInfo.set({
            ...get(extendedModInfo),
            [fileName]: null,
          });
          console.warn(
            `[ModsService] Failed to fetch extended mod info for ${fileName}:`,
            e instanceof Error ? e.message : "Unknown error",
          );
          // Optionally: log error, set error state, etc.
          console.warn(`Failed to fetch extended mod info for ${fileName}:`, e);
          resolve(null);
        } finally {
          ModsService.#inFlight--;
          ModsService.#runQueue();
        }
      };
      ModsService.#pending.push(task);
      ModsService.#runQueue();
    });
  }

  static #runQueue() {
    while (
      ModsService.#inFlight < ModsService.#MAX_CONCURRENT &&
      ModsService.#pending.length > 0
    ) {
      const next = ModsService.#pending.shift();
      if (next) {
        ModsService.#inFlight++;
        next();
      }
    }
  }
}
