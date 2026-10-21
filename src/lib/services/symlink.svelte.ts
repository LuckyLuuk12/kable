import type { Symlink, SymlinkCreateRequest } from "$lib";
import { api, app, error, log } from "$lib";
import type { Service } from "./app.svelte";

export class SymlinkService implements Service {
  initialized = $state(false);
  loading = $state(false);
  modifying = $state(false);

  error = $state<string | null>(null);

  persistentSymlinks = $state<Symlink[]>([]);
  runtimeSymlinks = $state<Record<string, Symlink[]>>({});

  /**
   * Number of modification operations currently in progress.
   * Prevents overlapping operations from incorrectly clearing `modifying`.
   */
  private modificationCount = 0;

  /**
   * Run an operation while maintaining the shared modification state.
   */
  private async withModification<T>(
    operation: () => Promise<T>,
  ): Promise<T> {
    this.modificationCount++;
    this.modifying = true;

    try {
      return await operation();
    } finally {
      this.modificationCount = Math.max(0, this.modificationCount - 1);
      this.modifying = this.modificationCount > 0;
    }
  }

  /**
   * Initialize the service and load persistent and runtime symlinks.
   */
  async init(): Promise<void> {
    await this.refresh();
    const errMessage = await api.checkSymlinkSupport();
    if (errMessage) {
      app.notificationService.error(`${errMessage}`, undefined,
        false,
        async () => {
          try {
            await api.openDeveloperSettings();
          } catch (err) {
            error("Failed to open developer settings:", err);
          }
        });
      return;
    }
    this.initialized = true;
    log("[SymlinkService] initialized successfully.");
  }

  /**
   * Refresh all symlink data.
   */
  async refresh(): Promise<void> {
    this.loading = true;
    this.error = null;

    try {
      const [persistentSymlinks, runtimeSymlinks] = await Promise.all([
        api.getSymlinks(),
        api.runtimeSymlinks(),
      ]);

      this.persistentSymlinks = persistentSymlinks;
      this.runtimeSymlinks = runtimeSymlinks;
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  /**
   * Refresh persistent custom symlinks.
   */
  async refreshPersistent(): Promise<void> {
    this.error = null;

    try {
      this.persistentSymlinks = await api.getSymlinks();
    } catch (e) {
      this.error = String(e);
    }
  }

  /**
   * Refresh runtime symlinks currently known to the backend.
   */
  async refreshRuntime(): Promise<void> {
    this.error = null;

    try {
      this.runtimeSymlinks = await api.runtimeSymlinks();
    } catch (e) {
      this.error = String(e);
    }
  }

  /**
   * Load persisted runtime symlinks for a specific profile.
   */
  async refreshRuntimeForProfile(profileId: string): Promise<Symlink[]> {
    this.error = null;

    try {
      const symlinks = await api.runtimeSymlinksForProfile(profileId);

      this.runtimeSymlinks = {
        ...this.runtimeSymlinks,
        [profileId]: symlinks,
      };

      return symlinks;
    } catch (e) {
      this.error = String(e);
      return [];
    }
  }

  /**
   * Create a persistent custom symlink.
   */
  async create(link: SymlinkCreateRequest): Promise<Symlink | null> {
    this.error = null;

    return this.withModification(async () => {
      try {
        const created = await api.create(link);
        await this.refreshPersistent();
        return created;
      } catch (e) {
        this.error = String(e);
        return null;
      }
    });
  }

  /**
   * Update a persistent custom symlink.
   */
  async update(old: Symlink, updated: Symlink): Promise<Symlink | null> {
    this.error = null;

    return this.withModification(async () => {
      try {
        const result = await api.update(old, updated);
        await this.refreshPersistent();
        return result;
      } catch (e) {
        this.error = String(e);
        return null;
      }
    });
  }

  /**
   * Enable or disable a persistent custom symlink.
   */
  async toggle(link: Symlink): Promise<Symlink | null> {
    this.error = null;

    return this.withModification(async () => {
      try {
        const result = await api.toggle(link);
        await this.refreshPersistent();
        return result;
      } catch (e) {
        this.error = String(e);
        return null;
      }
    });
  }

  /**
   * Remove a persistent custom symlink.
   */
  async remove(link: Symlink): Promise<boolean> {
    this.error = null;

    return this.withModification(async () => {
      try {
        await api.remove(link);
        await this.refreshPersistent();
        return true;
      } catch (e) {
        this.error = String(e);
        return false;
      }
    });
  }

  /**
   * Create a runtime symlink for a profile.
   */
  async createRuntime(
    profileId: string,
    source: string,
    destination: string,
  ): Promise<Symlink | null> {
    this.error = null;

    return this.withModification(async () => {
      try {
        const created = await api.createRuntimeSymlink(
          profileId,
          source,
          destination,
        );

        await this.refreshRuntimeForProfile(profileId);
        return created;
      } catch (e) {
        this.error = String(e);
        return null;
      }
    });
  }

  /**
   * Reconcile runtime symlinks with the complete desired set for a profile.
   */
  async reconcileRuntime(
    profileId: string,
    desired: Array<[string, string]>,
  ): Promise<boolean> {
    this.error = null;

    return this.withModification(async () => {
      try {
        await api.reconcileRuntimeSymlinks(profileId, desired);
        await this.refreshRuntimeForProfile(profileId);
        return true;
      } catch (e) {
        this.error = String(e);
        return false;
      }
    });
  }

  /**
   * Remove registered runtime symlinks for a profile.
   */
  async cleanupRuntimeForProfile(profileId: string): Promise<boolean> {
    this.error = null;

    return this.withModification(async () => {
      try {
        await api.cleanupProfileSymlinks(profileId);
        await this.refreshRuntimeForProfile(profileId);
        return true;
      } catch (e) {
        this.error = String(e);
        return false;
      }
    });
  }

  /**
   * Get a persistent symlink by ID.
   */
  getPersistentById(id: string): Symlink | undefined {
    return this.persistentSymlinks.find((link) => link.id === id);
  }

  /**
   * Get runtime symlinks cached for a profile.
   */
  getRuntimeForProfile(profileId: string): Symlink[] {
    return this.runtimeSymlinks[profileId] ?? [];
  }

  /**
   * Clear service state.
   */
  clear(): void {
    this.persistentSymlinks = [];
    this.runtimeSymlinks = {};
    this.initialized = false;
    this.loading = false;
    this.modificationCount = 0;
    this.modifying = false;
    this.error = null;
  }

  /**
   * Cleanup.
   */
  async destroy(): Promise<void> {
    this.clear();
  }
}
