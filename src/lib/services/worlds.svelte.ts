import {
  type NbtValue,
  type World,
  type WorldDatapack,
  type WorldDimension,
  type WorldPlayer,
  type WorldRegionStorage,
  type WorldSummary,
  api,
} from "$lib";
import type { Service } from "./app.svelte";

export class WorldsService implements Service {
  worlds = $state<WorldSummary[]>([]);

  loaded = $state(false);
  loading = $state(false);

  private loadingPromise: Promise<void> | null = null;

  async init() {
    if (this.loaded) return;

    await this.loadWorlds();
  }

  async refreshWorlds() {
    await this.loadWorlds();
  }

  private async loadWorlds(): Promise<void> {
    if (this.loadingPromise) {
      return this.loadingPromise;
    }

    this.loadingPromise = this.performLoadWorlds();

    try {
      await this.loadingPromise;
    } finally {
      this.loadingPromise = null;
    }
  }

  private async performLoadWorlds(): Promise<void> {
    this.loading = true;

    try {
      console.debug("[WorldsService] Loading world summaries...");

      const worlds = await api.loadWorlds();

      console.debug(
        `[WorldsService] Loaded ${worlds.length} world summaries`,
      );

      this.worlds = worlds;
      this.loaded = true;
    } catch (e) {
      console.error("[WorldsService] Failed to load world summaries", e);
    } finally {
      this.loading = false;
    }
  }

  async loadWorld(path: string): Promise<World | null> {
    try {
      console.debug(`[WorldsService] Loading full world from path: ${path}`);

      return await api.loadWorld(path);
    } catch (e) {
      console.error(`[WorldsService] Failed to load world ${path}`, e);
      return null;
    }
  }

  async loadLevel(path: string) {
    try {
      console.debug(`[WorldsService] Loading world level from path: ${path}`);

      return await api.loadWorldLevel(path);
    } catch (e) {
      console.error(
        `[WorldsService] Failed to load world level ${path}`,
        e,
      );

      return null;
    }
  }

  async loadPlayers(path: string): Promise<WorldPlayer[]> {
    try {
      console.debug(
        `[WorldsService] Loading world players from path: ${path}`,
      );

      return await api.loadWorldPlayers(path);
    } catch (e) {
      console.error(
        `[WorldsService] Failed to load world players ${path}`,
        e,
      );

      return [];
    }
  }

  async loadPlayer(
    path: string,
    uuid: string,
  ): Promise<WorldPlayer | null> {
    try {
      console.debug(
        `[WorldsService] Loading world player from path: ${path}, UUID: ${uuid}`,
      );

      return await api.loadWorldPlayer(path, uuid);
    } catch (e) {
      console.error(
        `[WorldsService] Failed to load player ${uuid} from world ${path}`,
        e,
      );

      return null;
    }
  }

  async loadDimensions(path: string): Promise<WorldDimension[]> {
    try {
      console.debug(
        `[WorldsService] Loading world dimensions from path: ${path}`,
      );

      return await api.loadWorldDimensions(path);
    } catch (e) {
      console.error(
        `[WorldsService] Failed to load world dimensions ${path}`,
        e,
      );

      return [];
    }
  }

  async loadRegionStorage(path: string): Promise<WorldRegionStorage[]> {
    try {
      console.debug(
        `[WorldsService] Loading world region storage from path: ${path}`,
      );

      return await api.loadWorldRegionStorage(path);
    } catch (e) {
      console.error(
        `[WorldsService] Failed to load world region storage ${path}`,
        e,
      );

      return [];
    }
  }

  async loadDatapacks(path: string): Promise<WorldDatapack[]> {
    try {
      console.debug(
        `[WorldsService] Loading world datapacks from path: ${path}`,
      );

      return await api.loadWorldDatapacks(path);
    } catch (e) {
      console.error(
        `[WorldsService] Failed to load world datapacks ${path}`,
        e,
      );

      return [];
    }
  }

  async loadNbt(path: string): Promise<NbtValue | null> {
    try {
      console.debug(`[WorldsService] Loading NBT from path: ${path}`);

      return await api.loadWorldNbt(path);
    } catch (e) {
      console.error(`[WorldsService] Failed to load NBT ${path}`, e);

      return null;
    }
  }

  getWorld(id: string): WorldSummary | null {
    return this.worlds.find((world) => world.id === id) ?? null;
  }

  memoryClone<T>(value: T): T {
    return JSON.parse(JSON.stringify(value));
  }

  async destroy() {
    this.worlds = [];
    this.loaded = false;
    this.loading = false;
    this.loadingPromise = null;
  }
}