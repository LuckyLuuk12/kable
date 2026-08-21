<script lang="ts">
import type { World, WorldDatapack } from "$lib";

interface Props {
  world: World;
  onselect?: (datapack: WorldDatapack) => void;
}

let { world, onselect }: Props = $props();

const formatFormat = (datapack: WorldDatapack) => {
  if (datapack.min_format !== null && datapack.min_format !== undefined && datapack.max_format !== null && datapack.max_format !== undefined) {
    if (datapack.min_format === datapack.max_format) {
      return String(datapack.min_format);
    }

    return `${datapack.min_format} - ${datapack.max_format}`;
  }

  if (datapack.pack_format !== null && datapack.pack_format !== undefined) {
    return String(datapack.pack_format);
  }

  return "Unknown";
};

const datapacks = $derived(
  [...world.datapacks].sort((a, b) => {
    if (a.enabled !== b.enabled) {
      return a.enabled ? -1 : 1;
    }

    return (a.name ?? a.id).localeCompare(b.name ?? b.id);
  }),
);
</script>

<div class="datapacks">
  <div class="header">
    <div>
      <h2>Datapacks</h2>
      <p>
        {datapacks.length}
        {datapacks.length === 1 ? "datapack" : "datapacks"} found
      </p>
    </div>

    {#if datapacks.length > 0}
      <div class="summary">
        <span class="enabled">
          {datapacks.filter((datapack) => datapack.enabled).length} enabled
        </span>
        <span class="separator">·</span>
        <span>
          {datapacks.filter((datapack) => !datapack.enabled).length} disabled
        </span>
      </div>
    {/if}
  </div>

  {#if datapacks.length === 0}
    <div class="empty">
      <span class="empty-title">No datapacks found</span>
      <span class="empty-description"> This world does not contain any datapack metadata. </span>
    </div>
  {:else}
    <div class="datapack-list">
      {#each datapacks as datapack (datapack.id)}
        <button type="button" class:disabled={!datapack.enabled} class="datapack" onclick={() => onselect?.(datapack)}>
          <div class:active={datapack.enabled} class="status"></div>

          <div class="identity">
            <span class="name">
              {datapack.name ?? datapack.id}
            </span>

            <span class="id" title={datapack.id}>
              {datapack.id}
            </span>
          </div>

          <div class="description">
            {#if datapack.description}
              {datapack.description}
            {:else}
              <span>No description</span>
            {/if}
          </div>

          <div class="metadata">
            <div class="meta">
              <span class="label">Format</span>
              <span class="value">{formatFormat(datapack)}</span>
            </div>

            <div class="meta">
              <span class="label">Status</span>
              <span class:enabled={datapack.enabled} class="value">
                {datapack.enabled ? "Enabled" : "Disabled"}
              </span>
            </div>
          </div>

          <span class="chevron">›</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style lang="scss">
@use "$lib/styles/variables" as *;

.datapacks {
  display: flex;
  flex-direction: column;
  gap: $space-4;

  width: 100%;
  padding: $space-5;
  box-sizing: border-box;
}

.header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: $space-3;
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

.summary {
  display: flex;
  align-items: center;
  gap: $space-1;

  color: $color-text-muted;
  font-size: $font-sm;
}

.summary .enabled {
  color: $color-success;
}

.separator {
  color: $color-border-muted;
}

.datapack-list {
  display: flex;
  flex-direction: column;
  gap: $space-2;
}

.datapack {
  display: grid;
  grid-template-columns: 6px minmax(180px, 1fr) minmax(180px, 1.5fr) auto auto;
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
    border-color 120ms ease,
    opacity 120ms ease;
}

.datapack:hover {
  background: $color-hover;
  border-color: $color-border-muted;
}

.datapack:focus-visible {
  outline: 2px solid $color-focus;
  outline-offset: 2px;
}

.datapack.disabled {
  opacity: 0.7;
}

.status {
  width: 6px;
  height: 36px;

  border-radius: $radius-round;
  background: $color-border-muted;
}

.status.active {
  background: $color-success;
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
  font-size: $font-md;
  font-weight: $font-weight-semibold;
}

.id {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;

  color: $color-text-muted;
  font-family: $font-mono;
  font-size: $font-xs;
}

.description {
  min-width: 0;

  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;

  color: $color-text-muted;
  font-size: $font-sm;
}

.metadata {
  display: flex;
  gap: $space-4;
}

.meta {
  display: flex;
  flex-direction: column;
  gap: $space-1;
  min-width: 70px;
}

.label {
  color: $color-text-muted;
  font-size: $font-xs;
  font-weight: $font-weight-medium;
}

.value {
  color: $color-text;
  font-family: $font-mono;
  font-size: $font-sm;
}

.value.enabled {
  color: $color-success;
  font-family: $font-sans;
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
  .datapack {
    grid-template-columns: 6px minmax(180px, 1fr) auto auto;
  }

  .description {
    display: none;
  }
}

@media (max-width: 600px) {
  .datapacks {
    padding: $space-3;
  }

  .header {
    align-items: flex-start;
    flex-direction: column;
  }

  .datapack {
    grid-template-columns: 6px minmax(0, 1fr) auto;
  }

  .metadata {
    display: none;
  }
}
</style>
