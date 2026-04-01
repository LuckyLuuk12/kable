import type {
  CurseForgeInfo,
  ExtendedModInfo,
  KableInstallation,
  ModFilter,
  ModInfoKind,
  ModJarInfo,
  ModpackContext,
  ModpackPrepareResult,
  ModrinthInfo,
  MrPackDetailed,
} from "$lib";
import {
  extendedModInfo,
  modsByProvider,
  modsError,
  modsFilter,
  modsInstallation,
  modsLimit,
  modsLoading,
  modsOffset,
  modsProvider,
  NotificationService,
  ProviderKind,
} from "$lib";
import * as semver from "semver";
import { get } from "svelte/store";
import * as modsApi from "../api/mods";

export interface NormalizedModInfo {
  provider: ProviderKind;
  providerName: "Modrinth" | "CurseForge";
  projectId: string;
  versionId: string | null;
  data: ModrinthInfo | CurseForgeInfo;
}

export interface ModDisplayInfo {
  title: string;
  description: string;
  author: string;
  downloads: number;
  icon_url: string | null | undefined;
  categories: string[];
  project_type: string;
  follows?: number;
  client_side?: string;
  server_side?: string;
  game_versions: string[];
  loaders?: string[];
  source_url?: string;
  wiki_url?: string;
  license?: string;
  date_created?: string;
  date_modified?: string;
  latest_version?: string;
}

/**
 * Version comparison utilities for mod version management
 * Handles various version formats found in mod JARs and metadata:
 * - Standard semver: 1.2.3
 * - With metadata: 1.2.3+fabric-1.21.11
 * - Reversed formats: 1.21.11-fabric-1.2.3 or vmc1.21.10-0.7.3-fabric
 * - Pre-release versions: 1.2.3-beta.1
 */
export class VersionUtils {
  /**
   * Normalize a version string by extracting and cleaning the version number
   * This is used for exact string comparison before attempting semver parsing
   * Handles various formats:
   * - vmc1.21.10-1.2.3-fabric → 1.2.3
   * - 1.2.3-mc1.21.10-fabric → 1.2.3
   * - 1.2.3+fabric+mc1.21.10 → 1.2.3
   * - 1.2.3+mc1.21.10-fabric → 1.2.3
   */
  static normalizeForComparison(version: string, mcVersion?: string): string {
    if (!version) return "";

    // First extract the version number (removes MC versions, loaders, etc.)
    const extracted = this.extractVersion(version, mcVersion);
    if (!extracted) return version.toLowerCase().trim();

    // Further normalize by removing prefixes and build metadata
    let normalized = extracted.toLowerCase();

    // Remove leading 'v' characters (avoid polynomial regex)
    while (normalized.startsWith("v")) {
      normalized = normalized.substring(1);
    }

    // Remove build metadata after '+' (avoid polynomial regex)
    const plusIndex = normalized.indexOf("+");
    if (plusIndex !== -1) {
      normalized = normalized.substring(0, plusIndex);
    }

    return normalized.trim();
  }

  /**
   * Extract version string from various formats
   * Examples:
   * - "somemodname-1.2.3+fabric-1.21.11.jar" -> "1.2.3"
   * - "mod-fabric-1.21.11-1.2.3.jar" -> "1.2.3"
   * - "vmc1.21.10-0.7.3-fabric" -> "0.7.3"
   * - "1.2.3-beta" -> "1.2.3-beta"
   * - "v1.2.3" -> "1.2.3"
   */
  static extractVersion(input: string, mcVersion?: string): string | null {
    if (!input) return null;

    // Remove .jar extension if present
    let str = input.replace(/\.jar$/i, "");

    // Split on common delimiters to get individual segments
    const segments = str.split(/[-_+]/);

    // Filter segments to find likely mod version candidates
    const candidates = segments.filter((seg) => {
      if (!seg) return false;

      // Remove segments containing mc/vmc (Minecraft version indicators)
      if (/^(v)?mc/i.test(seg)) return false;

      // Remove segments without any numbers
      if (!/\d/.test(seg)) return false;

      // Remove segments that are just loader names or other non-version text
      if (
        /^(fabric|forge|neoforge|quilt|snapshot|alpha|beta|release)$/i.test(seg)
      )
        return false;

      // Remove segments that match the known MC version
      if (mcVersion) {
        const normalizedMc = mcVersion.replace(/^v+/i, "");
        const normalizedSeg = seg.replace(/^v+/i, "");
        if (normalizedSeg === normalizedMc) return false;
        if (normalizedSeg.startsWith(normalizedMc + ".")) return false;
      }

      // Keep segments that look like version numbers (contain dots and numbers)
      // Use simple string operations to avoid polynomial regex complexity
      const dotIndex = seg.indexOf(".");
      if (dotIndex > 0 && dotIndex < seg.length - 1) {
        // Check if there's at least one digit before and after the dot
        const beforeDot = seg.substring(0, dotIndex);
        const afterDot = seg.substring(dotIndex + 1);
        if (/\d/.test(beforeDot) && /\d/.test(afterDot)) {
          return true;
        }
      }

      // Skip likely Minecraft versions by pattern (1.12, 1.16, 1.17, etc.)
      if (/^v?1\.(1[2-9]|2[0-9]|3[0-9])(\.\d+)?$/i.test(seg)) return false;

      return true;
    });

    if (candidates.length === 0) return null;

    // Take the first candidate (usually the mod version comes first)
    let version = candidates[0];

    // Remove common prefixes like "v", "version"
    version = version.replace(/^(v+|version)/i, "");

    return version || null;
  }

  /**
   * Normalize version for semver comparison
   * Converts various formats into valid semver strings
   */
  static normalizeVersion(version: string, mcVersion?: string): string | null {
    const extracted = this.extractVersion(version, mcVersion);
    if (!extracted) return null;

    // Split on + to separate build metadata (ignored in semver comparison)
    const [versionPart] = extracted.split("+");

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

    // Validate it's proper semver
    const coerced = semver.coerce(normalized);
    if (!coerced) return null;

    return normalized;
  }

  /**
   * Compare two version strings
   * Returns:
   * - positive number if v1 > v2 (v1 is newer)
   * - negative number if v1 < v2 (v2 is newer)
   * - 0 if equal
   */
  static compareVersions(v1: string, v2: string, mcVersion?: string): number {
    // First check if they're exactly the same after normalization
    const norm1Str = this.normalizeForComparison(v1, mcVersion);
    const norm2Str = this.normalizeForComparison(v2, mcVersion);

    if (norm1Str === norm2Str) return 0;

    // Try semver comparison
    const norm1 = this.normalizeVersion(v1, mcVersion);
    const norm2 = this.normalizeVersion(v2, mcVersion);

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
  static isNewer(
    version1: string,
    version2: string,
    mcVersion?: string,
  ): boolean {
    return this.compareVersions(version1, version2, mcVersion) > 0;
  }

  /**
   * Check if version1 is older than version2
   */
  static isOlder(
    version1: string,
    version2: string,
    mcVersion?: string,
  ): boolean {
    return this.compareVersions(version1, version2, mcVersion) < 0;
  }

  /**
   * Check if two versions are equal
   */
  static isEqual(
    version1: string,
    version2: string,
    mcVersion?: string,
  ): boolean {
    return this.compareVersions(version1, version2, mcVersion) === 0;
  }

  /**
   * Find the latest version from an array of version strings
   */
  static findLatest(versions: string[], mcVersion?: string): string | null {
    if (!versions || versions.length === 0) return null;

    return versions.reduce((latest, current) => {
      if (!latest) return current;
      return this.isNewer(current, latest, mcVersion) ? current : latest;
    }, versions[0]);
  }
}

export class ModsService {
  initialized = false;

  static normalizeMod(mod: ModInfoKind): NormalizedModInfo {
    if ("Modrinth" in mod) {
      return {
        provider: ProviderKind.Modrinth,
        providerName: "Modrinth",
        projectId: mod.Modrinth.project_id,
        versionId: mod.Modrinth.latest_version || null,
        data: mod.Modrinth,
      };
    }

    if ("CurseForge" in mod) {
      return {
        provider: ProviderKind.CurseForge,
        providerName: "CurseForge",
        projectId: mod.CurseForge.id.toString(),
        versionId: mod.CurseForge.main_file_id?.toString() || null,
        data: mod.CurseForge,
      };
    }

    throw new Error("Unknown mod provider format");
  }

  static getProjectId(mod: ModInfoKind): string | null {
    try {
      return ModsService.normalizeMod(mod).projectId;
    } catch {
      return null;
    }
  }

  static getModKey(mod: ModInfoKind): string {
    const projectId = ModsService.getProjectId(mod);
    if (!projectId) {
      return `unknown-${Math.random().toString(36).slice(2)}`;
    }

    try {
      const normalized = ModsService.normalizeMod(mod);
      return `${normalized.providerName.toLowerCase()}-${projectId}`;
    } catch {
      return `unknown-${projectId}`;
    }
  }

  static getModInfoUrl(mod: ModInfoKind): string | null {
    let normalized: NormalizedModInfo;
    try {
      normalized = ModsService.normalizeMod(mod);
    } catch {
      return null;
    }

    if (normalized.providerName === "Modrinth") {
      const data = normalized.data as ModrinthInfo;
      return data.source_url || data.wiki_url || `https://modrinth.com/mod/${data.slug}`;
    }

    const data = normalized.data as CurseForgeInfo;
    return (
      data.links?.website_url ||
      data.links?.source_url ||
      data.links?.wiki_url ||
      `https://www.curseforge.com/minecraft/mc-mods/${data.slug}`
    );
  }

  static getDisplayInfo(mod: ModInfoKind): ModDisplayInfo {
    let normalized: NormalizedModInfo;
    try {
      normalized = ModsService.normalizeMod(mod);
    } catch {
      return {
        title: "Unknown Mod",
        description: "No description available.",
        author: "Unknown Author",
        downloads: 0,
        icon_url: null,
        categories: [],
        project_type: "mod",
        follows: undefined,
        client_side: undefined,
        server_side: undefined,
        game_versions: [],
        source_url: undefined,
        wiki_url: undefined,
        license: undefined,
        date_created: undefined,
        date_modified: undefined,
        latest_version: undefined,
      };
    }

    if (normalized.providerName === "Modrinth") {
      const data = normalized.data as ModrinthInfo;
      return {
        title: data.title || "Unknown Mod",
        description: data.description || "No description available.",
        author: data.author || "Unknown Author",
        downloads: data.downloads || 0,
        icon_url: data.icon_url,
        categories: data.categories || [],
        project_type: data.project_type || "mod",
        follows: data.follows,
        client_side: data.client_side,
        server_side: data.server_side,
        game_versions: data.game_versions || [],
        loaders: data.loaders,
        source_url: data.source_url,
        wiki_url: data.wiki_url,
        license: data.license,
        date_created: data.date_created,
        date_modified: data.date_modified,
        latest_version: data.latest_version,
      };
    }

    const data = normalized.data as CurseForgeInfo;
    return {
      title: data.name || "Unknown Mod",
      description: data.summary || "No description available.",
      author: data.authors?.[0]?.name || "Unknown Author",
      downloads: data.download_count || 0,
      icon_url: data.logo?.url || data.logo?.thumbnail_url,
      categories: data.categories?.map((cat) => cat.name) || [],
      project_type: "mod",
      follows: data.thumbs_up_count,
      client_side: undefined,
      server_side: undefined,
      game_versions: data.latest_files_indexes?.map((file) => file.game_version) || [],
      loaders: undefined,
      source_url: data.links?.source_url,
      wiki_url: data.links?.wiki_url,
      license: undefined,
      date_created: data.date_created,
      date_modified: data.date_modified,
      latest_version: data.latest_files?.[0]?.display_name,
    };
  }

  private static getDownloadTarget(mod: ModInfoKind): {
    provider: ProviderKind;
    modId: string;
    versionId: string | null;
  } {
    const normalized = ModsService.normalizeMod(mod);
    return {
      provider: normalized.provider,
      modId: normalized.projectId,
      versionId: normalized.versionId,
    };
  }

  async downloadOrPrepareFromMod(
    mod: ModInfoKind,
    installation: KableInstallation,
  ): Promise<
    | { kind: "downloaded" }
    | { kind: "modpack"; modpack: MrPackDetailed; context: ModpackContext }
  > {
    const { provider, modId, versionId } = ModsService.getDownloadTarget(mod);
    const result: ModpackPrepareResult = await modsApi.downloadOrPrepareMod(
      provider,
      modId,
      versionId,
      installation,
    );

    if ("success" in result && result.success) {
      return { kind: "downloaded" };
    }

    if ("modpack" in result && "context" in result) {
      return {
        kind: "modpack",
        modpack: result.modpack,
        context: result.context,
      };
    }

    throw new Error("Unknown response from backend.");
  }

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
    // Only check Svelte store (for UI reactivity)
    if (get(extendedModInfo)[fileName]) {
      return get(extendedModInfo)[fileName] || null;
    }
    // Fetch and update store, using concurrency-limited queue
    console.log(`[ModsService] Fetching extended mod info for ${fileName}`);
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
