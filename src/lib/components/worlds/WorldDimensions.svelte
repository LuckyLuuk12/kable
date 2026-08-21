<script lang="ts">
import type { World, WorldDimension, WorldDimensionKind } from "$lib";

interface Props {
  world: World;
  onselect?: (dimension: WorldDimension) => void;
}

let { world, onselect }: Props = $props();

const formatKind = (kind: WorldDimensionKind) =>
  kind
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");

const formatNumber = (value: string | number | null | undefined) => {
  if (value === null || value === undefined) return "Unknown";

  const number = Number(value);

  return Number.isFinite(number) ? number.toLocaleString() : String(value);
};

const dimensions = $derived([...world.dimensions].sort((a, b) => a.id.localeCompare(b.id)));
</script>

<div class="dimensions">
  <div class="header">
    <div>
      <h2>Dimensions</h2>
      <p>
        {dimensions.length}
        {dimensions.length === 1 ? "dimension" : "dimensions"} found
      </p>
    </div>
  </div>

  {#if dimensions.length === 0}
    <div class="empty">
      <span class="empty-title">No dimensions found</span>
      <span class="empty-description"> No dimension data could be found in this world. </span>
    </div>
  {:else}
    <div class="dimension-list">
      {#each dimensions as dimension (dimension.id)}
        <button type="button" class="dimension" onclick={() => onselect?.(dimension)}>
          <div class="icon">
            {#if dimension.kind === "overworld"}
              O
            {:else if dimension.kind === "nether"}
              N
            {:else if dimension.kind === "end"}
              E
            {:else}
              ?
            {/if}
          </div>

          <div class="identity">
            <span class="name">{dimension.id}</span>
            <span class="kind">{formatKind(dimension.kind)}</span>
          </div>

          <div class="stats">
            <div class="stat">
              <span class="label">Regions</span>
              <span class="value">
                {formatNumber(dimension.region_count)}
              </span>
            </div>

            <div class="stat">
              <span class="label">Chunks</span>
              <span class="value">
                {formatNumber(dimension.chunk_count)}
              </span>
            </div>

            {#if dimension.region_storage}
              <div class="stat">
                <span class="label">Storage</span>
                <span class="value">
                  {formatNumber(dimension.region_storage.total_size)}
                </span>
              </div>
            {/if}
          </div>

          <span class="chevron">›</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style lang="scss">
@use "$lib/styles/variables" as *;

.dimensions {
  display: flex;
  flex-direction: column;
  gap: $space-4;

  width: 100%;
  padding: $space-5;
  box-sizing: border-box;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

h2 {
  margin: 0;

  color: $color-text;
  font-size: $font-xl;
  font-weight: $font-weight-semibold;
}

p {
  margin: $space-1 0 0;

  color: $color-text-muted;
  font-size: $font-sm;
}

.dimension-list {
  display: flex;
  flex-direction: column;
  gap: $space-2;
}

.dimension {
  display: grid;
  grid-template-columns: auto minmax(180px, 1fr) minmax(300px, 2fr) auto;
  align-items: center;
  gap: $space-4;

  width: 100%;
  padding: $space-3 $space-4;

  border: 1px solid $color-border;
  border-radius: $radius-lg;
  background: $color-surface-1;

  color: inherit;
  text-align: left;

  cursor: pointer;
  transition:
    background-color 120ms ease,
    border-color 120ms ease;
}

.dimension:hover {
  background: $color-hover;
  border-color: $color-border-muted;
}

.dimension:focus-visible {
  outline: 2px solid $color-focus;
  outline-offset: 2px;
}

.icon {
  display: grid;
  place-items: center;

  width: 44px;
  height: 44px;

  border-radius: $radius-md;
  background: $color-accent-muted;

  color: $color-accent;
  font-size: $font-lg;
  font-weight: $font-weight-bold;
}

.identity {
  display: flex;
  flex-direction: column;
  gap: $space-1;

  min-width: 0;
}

.name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;

  color: $color-text;
  font-family: $font-mono;
  font-size: $font-sm;
  font-weight: $font-weight-medium;
}

.kind {
  color: $color-text-muted;
  font-size: $font-xs;
}

.stats {
  display: grid;
  grid-template-columns: repeat(3, minmax(80px, 1fr));
  gap: $space-3;
}

.stat {
  display: flex;
  flex-direction: column;
  gap: $space-1;
  min-width: 0;
}

.label {
  color: $color-text-muted;
  font-size: $font-xs;
  font-weight: $font-weight-medium;
}

.value {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;

  color: $color-text;
  font-family: $font-mono;
  font-size: $font-sm;
}

.chevron {
  color: $color-text-muted;
  font-size: $font-xl;
  line-height: 1;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: $space-1;

  min-height: 240px;
  padding: $space-5;

  border: 1px dashed $color-border;
  border-radius: $radius-lg;
  background: $color-surface-1;

  text-align: center;
}

.empty-title {
  color: $color-text;
  font-size: $font-md;
  font-weight: $font-weight-semibold;
}

.empty-description {
  color: $color-text-muted;
  font-size: $font-sm;
}

@media (max-width: 900px) {
  .dimension {
    grid-template-columns: auto minmax(0, 1fr) auto;
  }

  .stats {
    grid-column: 2 / -1;
    grid-row: 2;
  }
}

@media (max-width: 600px) {
  .dimensions {
    padding: $space-3;
  }

  .dimension {
    grid-template-columns: auto minmax(0, 1fr) auto;
  }

  .stats {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
