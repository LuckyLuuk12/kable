import { api, app, type CacheEntryInfo } from "$lib";
import type { Service } from "./app.svelte";

export class CacheService implements Service {
  entries = $state<CacheEntryInfo[]>([]);
  loading = $state(false);
  invalidating = $state(false);

  /**
   * Total size of the persistent cache in bytes.
   */
  get totalSizeBytes(): number {
    return this.entries.reduce((total, entry) => total + (entry.size_bytes ?? 0), 0);
  }

  /**
   * Number of persistent cache entries.
   */
  get entryCount(): number {
    return this.entries.length;
  }

  init(): Promise<void> {
    return this.loadEntries().then(() => {
      console.log("Cache service initialized.");
    });
  }


  /**
   * Load persistent cache entries.
   */
  async loadEntries(): Promise<CacheEntryInfo[]> {
    this.loading = true;

    try {
      this.entries = await api.listEntries();
      return this.entries;
    } catch (error) {
      app.notificationService.error("Failed to load cache entries.");
      throw error;
    } finally {
      this.loading = false;
    }
  }

  /**
   * Refresh the cache entry listing.
   */
  async refreshEntries(): Promise<CacheEntryInfo[]> {
    return this.loadEntries();
  }

  /**
   * Invalidate a specific persistent cache entry by its relative path.
   */
  async invalidateEntry(path: string): Promise<void> {
    this.invalidating = true;

    try {
      await api.invalidateEntry(path);
      this.entries = this.entries.filter((entry) => entry.path !== path);

      app.notificationService.success("Cache entry invalidated.");
    } catch (error) {
      app.notificationService.error("Failed to invalidate cache entry.");
      throw error;
    } finally {
      this.invalidating = false;
    }
  }

  /**
   * Invalidate all entries in a namespace.
   */
  async invalidateNamespace(namespace: string): Promise<void> {
    this.invalidating = true;

    try {
      await api.invalidateNamespace(namespace);
      this.entries = this.entries.filter(
        (entry) =>
          entry.namespace !== namespace &&
          !entry.namespace.startsWith(`${namespace}/`)
      );

      app.notificationService.success(
        `Cache namespace "${namespace}" invalidated.`
      );
    } catch (error) {
      app.notificationService.error(
        `Failed to invalidate cache namespace "${namespace}".`
      );
      throw error;
    } finally {
      this.invalidating = false;
    }
  }

  /**
   * Clear only the in-memory cache.
   *
   * Persistent cache files remain untouched.
   */
  async invalidateMemory(): Promise<void> {
    this.invalidating = true;

    try {
      await api.invalidateMemory();
      app.notificationService.success("In-memory cache cleared.");
    } catch (error) {
      app.notificationService.error("Failed to clear in-memory cache.");
      throw error;
    } finally {
      this.invalidating = false;
    }
  }

  /**
   * Clear the entire persistent and in-memory cache.
   */
  async invalidateAll(): Promise<void> {
    this.invalidating = true;

    try {
      await api.invalidateAll();
      this.entries = [];

      app.notificationService.success("Entire cache cleared.");
    } catch (error) {
      app.notificationService.error("Failed to clear the cache.");
      throw error;
    } finally {
      this.invalidating = false;
    }
  }

  /**
   * Reset service state.
   */
  async destroy(): Promise<void> {
    this.entries = [];
    this.loading = false;
    this.invalidating = false;
  }
}
