<script lang="ts">
import { app, type World, type WorldChunk, type WorldRegion, type WorldRegionStorage } from "$lib";

let { world }: { world: World } = $props();

let storages = $state<WorldRegionStorage[]>([]);
let loading = $state(false);
let error = $state<string | null>(null);

let selectedStorage = $state<WorldRegionStorage | null>(null);
let selectedRegion = $state<WorldRegion | null>(null);
let selectedChunk = $state<WorldChunk | null>(null);

let requestId = 0;

async function load() {
  const id = ++requestId;
  const path = world.path;

  loading = true;
  error = null;

  try {
    const result = await app.worldsService.loadRegionStorage(path);

    if (id !== requestId || world.path !== path) {
      return;
    }

    storages = result;

    if (selectedStorage) {
      const updatedStorage = result.find((storage) => storage.path === selectedStorage!.path) ?? null;

      selectedStorage = updatedStorage;

      if (updatedStorage && selectedRegion) {
        const updatedRegion = updatedStorage.regions.find((region) => region.path === selectedRegion!.path) ?? null;

        selectedRegion = updatedRegion;

        if (updatedRegion && selectedChunk) {
          selectedChunk = updatedRegion.chunks.find((chunk) => chunk.x === selectedChunk!.x && chunk.z === selectedChunk!.z) ?? null;
        } else {
          selectedChunk = null;
        }
      } else {
        selectedRegion = null;
        selectedChunk = null;
      }
    }
  } catch (e) {
    if (id !== requestId || world.path !== path) {
      return;
    }

    error = e instanceof Error ? e.message : String(e);
    storages = [];
    selectedStorage = null;
    selectedRegion = null;
    selectedChunk = null;
  } finally {
    if (id === requestId) {
      loading = false;
    }
  }
}

function selectStorage(storage: WorldRegionStorage) {
  selectedStorage = storage;
  selectedRegion = null;
  selectedChunk = null;
}

function selectRegion(region: WorldRegion) {
  selectedRegion = region;
  selectedChunk = null;
}

function selectChunk(chunk: WorldChunk) {
  selectedChunk = chunk;
}

function formatSize(value: string): string {
  const size = Number(value);

  if (!Number.isFinite(size)) return value;
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  if (size < 1024 * 1024 * 1024) {
    return `${(size / 1024 / 1024).toFixed(1)} MB`;
  }

  return `${(size / 1024 / 1024 / 1024).toFixed(1)} GB`;
}

$effect(() => {
  const path = world.path;

  if (!path) {
    return;
  }

  load();
});
</script>

<section class="regions">
  <header class="header">
    <div>
      <h2>Regions</h2>
      <p>Region files and their stored chunks</p>
    </div>

    <button type="button" onclick={load} disabled={loading}>
      {loading ? "Loading..." : "Reload"}
    </button>
  </header>

  {#if loading}
    <div class="state">Loading region storage...</div>
  {:else if error}
    <div class="state error">{error}</div>
  {:else if storages.length === 0}
    <div class="state">No region storage found.</div>
  {:else}
    <div class="content">
      <aside class="storages">
        <div class="section-title">Storage</div>

        {#each storages as storage (storage.path)}
          <button type="button" class:selected={selectedStorage?.path === storage.path} onclick={() => selectStorage(storage)}>
            <span class="storage-name">
              {storage.path.split("/").at(-1) ?? storage.path}
            </span>

            <span class="storage-meta">
              {storage.region_count} regions · {storage.chunk_count} chunks
            </span>
          </button>
        {/each}
      </aside>

      <div class="regions-list">
        {#if selectedStorage}
          {@const storage = selectedStorage}

          <div class="list-header">
            <div>
              <span class="section-title">Regions</span>
              <span class="count">{storage.region_count}</span>
            </div>

            <span class="size">
              {formatSize(storage.total_size)}
            </span>
          </div>

          <div class="region-grid">
            {#each storage.regions as region (region.path)}
              <button type="button" class:selected={selectedRegion?.path === region.path} onclick={() => selectRegion(region)}>
                <span class="coordinates">
                  r.{region.x}.{region.z}
                </span>

                <span class="region-meta">
                  {region.chunk_count} chunks · {formatSize(region.size)}
                </span>
              </button>
            {/each}
          </div>
        {:else}
          <div class="empty">Select a region storage.</div>
        {/if}
      </div>

      <aside class="details">
        {#if selectedChunk}
          {@const chunk = selectedChunk}

          <div class="details-header">
            <span class="section-title">Chunk</span>

            <button type="button" onclick={() => (selectedChunk = null)}> Close </button>
          </div>

          <dl>
            <dt>Coordinates</dt>
            <dd>{chunk.x}, {chunk.z}</dd>

            <dt>Region</dt>
            <dd>{chunk.region_x}, {chunk.region_z}</dd>

            <dt>Sector offset</dt>
            <dd>{chunk.sector_offset}</dd>

            <dt>Sector count</dt>
            <dd>{chunk.sector_count}</dd>

            <dt>Timestamp</dt>
            <dd>{chunk.timestamp ?? "Unknown"}</dd>

            <dt>Compression</dt>
            <dd>{chunk.compression?.toString() ?? "Unknown"}</dd>

            <dt>Compressed size</dt>
            <dd>
              {chunk.compressed_size ? `${chunk.compressed_size} B` : "Unknown"}
            </dd>
          </dl>
        {:else if selectedRegion}
          {@const region = selectedRegion}

          <div class="details-header">
            <div>
              <span class="section-title">
                r.{region.x}.{region.z}
              </span>

              <span class="region-path">
                {region.path}
              </span>
            </div>
          </div>

          <div class="chunk-list">
            {#each region.chunks as chunk (`${chunk.x}:${chunk.z}`)}
              <button type="button" onclick={() => selectChunk(chunk)}>
                <span>
                  {chunk.x}, {chunk.z}
                </span>

                <span>
                  {chunk.compression?.toString() ?? "Unknown"}
                </span>
              </button>
            {/each}
          </div>
        {:else}
          <div class="empty">Select a region to inspect its chunks.</div>
        {/if}
      </aside>
    </div>
  {/if}
</section>

<style lang="scss">
.regions {
  display: flex;
  flex-direction: column;
  gap: $space-lg;
  min-height: 100%;
  width: 100%;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-md;

  h2 {
    margin: 0;
    color: $color-text;
    font-size: $font-lg;
    font-weight: $font-weight-semibold;
  }

  p {
    margin: $space-1 0 0;
    color: $color-text-muted;
    font-size: $font-sm;
  }

  button {
    padding: $space-2 $space-md;
    border: 1px solid $color-border;
    border-radius: $radius-md;
    background: $color-surface-2;
    color: $color-text;
    font: inherit;
    cursor: pointer;

    &:hover:not(:disabled) {
      background: $color-hover;
    }

    &:disabled {
      color: $color-disabled;
      cursor: default;
    }
  }
}

.content {
  display: grid;
  grid-template-columns: 220px minmax(260px, 1fr) minmax(280px, 360px);
  min-height: 500px;
  overflow: hidden;
  border: 1px solid $color-border;
  border-radius: $radius-lg;
  background: $color-surface-1;
}

.storages,
.regions-list,
.details {
  min-width: 0;
  overflow: auto;
}

.storages {
  padding: $space-md;
  border-right: 1px solid $color-border;
}

.regions-list {
  padding: $space-md;
  border-right: 1px solid $color-border;
}

.details {
  padding: $space-md;
}

.section-title {
  color: $color-text;
  font-size: $font-sm;
  font-weight: $font-weight-semibold;
}

.storages > button,
.region-grid > button,
.chunk-list > button {
  display: flex;
  flex-direction: column;
  width: 100%;
  border: 0;
  color: $color-text;
  text-align: left;
  cursor: pointer;
}

.storages > button {
  gap: $space-1;
  margin-top: $space-2;
  padding: $space-sm;
  border-radius: $radius-md;
  background: transparent;
}

.storages > button:hover,
.region-grid > button:hover,
.chunk-list > button:hover {
  background: $color-hover;
}

.storages > button.selected,
.region-grid > button.selected {
  background: $color-selected;
}

.storage-name,
.coordinates {
  font-weight: $font-weight-medium;
}

.storage-meta,
.region-meta,
.size,
.region-path {
  color: $color-text-muted;
  font-size: $font-xs;
}

.list-header,
.details-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-md;
  margin-bottom: $space-md;
}

.list-header > div {
  display: flex;
  align-items: center;
  gap: $space-sm;
}

.count {
  color: $color-text-muted;
  font-size: $font-xs;
}

.region-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: $space-sm;
}

.region-grid > button {
  gap: $space-1;
  padding: $space-md;
  border: 1px solid $color-border-muted;
  border-radius: $radius-md;
  background: $color-surface-2;
}

.chunk-list {
  display: flex;
  flex-direction: column;
  gap: $space-1;
}

.chunk-list > button {
  flex-direction: row;
  justify-content: space-between;
  padding: $space-sm;
  border-radius: $radius-sm;
  background: transparent;
}

.details-header button {
  padding: $space-1 $space-sm;
  border: 1px solid $color-border;
  border-radius: $radius-sm;
  background: $color-surface-2;
  color: $color-text;
  cursor: pointer;
}

.region-path {
  display: block;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

dl {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: $space-sm $space-md;
  margin: 0;
}

dt {
  color: $color-text-muted;
}

dd {
  margin: 0;
  color: $color-text;
  text-align: right;
  overflow-wrap: anywhere;
}

.empty,
.state {
  display: grid;
  place-items: center;
  min-height: 180px;
  color: $color-text-muted;
  text-align: center;
}

.state {
  padding: $space-xl;
  border: 1px solid $color-border;
  border-radius: $radius-lg;
  background: $color-surface-1;

  &.error {
    color: $color-error;
  }
}

@media (max-width: 900px) {
  .content {
    grid-template-columns: 180px minmax(220px, 1fr);
  }

  .details {
    grid-column: 1 / -1;
    border-top: 1px solid $color-border;
  }
}

@media (max-width: 600px) {
  .content {
    grid-template-columns: 1fr;
  }

  .storages,
  .regions-list {
    border-right: 0;
    border-bottom: 1px solid $color-border;
  }
}
</style>
