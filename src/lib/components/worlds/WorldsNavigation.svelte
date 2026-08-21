<script lang="ts">
import { app, Image, type WorldSummary } from "$lib";

type Section = "overview" | "players" | "dimensions" | "datapacks" | "regions" | "advancements" | "statistics" | "nbt";

let {
  world = null,
  active = "overview",
  onchange,
  onworldchange,
}: {
  world?: WorldSummary | null;
  active?: Section;
  onchange?: (section: Section) => void;
  onworldchange?: (world: string) => void;
} = $props();

const sections: { id: Section; label: string }[] = [
  { id: "overview", label: "Overview" },
  { id: "players", label: "Players" },
  { id: "dimensions", label: "Dimensions" },
  { id: "datapacks", label: "Datapacks" },
  { id: "regions", label: "Regions" },
  { id: "advancements", label: "Advancements" },
  { id: "statistics", label: "Statistics" },
  { id: "nbt", label: "NBT" },
];

let worlds = $derived(app.worldsService.worlds);
let refreshing = $derived(app.worldsService.loading);

function select(section: Section) {
  onchange?.(section);
}

function selectWorld(id: string) {
  if (!id) return;

  const selected = worlds.find((availableWorld) => availableWorld.id === id);

  if (selected) {
    onworldchange?.(selected.id);
  }
}

async function refreshWorlds() {
  await app.worldsService.refreshWorlds();
}
</script>

<nav class="navigation" aria-label="World navigation">
  <div class="world-selector">
    <div class="world">
      {#if world}
        {#if world.icon}
          <Image key={world.icon} alt="" className="icon" />
        {:else}
          <div class="icon-placeholder">W</div>
        {/if}

        <div class="world-info">
          <span class="world-name">{world.name}</span>
          <span class="world-path">{world.path}</span>
        </div>
      {:else}
        <div class="icon-placeholder">W</div>

        <div class="world-info">
          <span class="world-name">No world selected</span>
          <span class="world-path">Select a world to continue</span>
        </div>
      {/if}
    </div>

    <div class="world-controls">
      <select
        value={world?.id ?? ""}
        disabled={worlds.length === 0 || refreshing}
        aria-label="Select world"
        onchange={(event) => selectWorld((event.currentTarget as HTMLSelectElement).value)}>
        {#if worlds.length === 0}
          <option value="">No worlds available</option>
        {:else}
          {#if !world}
            <option value="" disabled>Select a world</option>
          {/if}

          {#each worlds as availableWorld (availableWorld.id)}
            <option value={availableWorld.id}>
              {availableWorld.name}
            </option>
          {/each}
        {/if}
      </select>

      <button type="button" class="refresh" disabled={refreshing} aria-label="Refresh worlds" title="Refresh worlds" onclick={refreshWorlds}>
        {#if refreshing}
          <span class="spinner"></span>
        {:else}
          ↻
        {/if}
      </button>
    </div>
  </div>

  {#if world}
    <div class="sections">
      {#each sections as section (section.id)}
        <button type="button" class:active={active === section.id} aria-current={active === section.id ? "page" : undefined} onclick={() => select(section.id)}>
          {section.label}
        </button>
      {/each}
    </div>
  {/if}
</nav>

<style lang="scss">
@use "$lib/styles/variables" as *;

.navigation {
  display: flex;
  flex-direction: column;
  gap: $space-md;
  padding: $space-md;
  border-bottom: 1px solid $color-border;
  background: $color-surface-1;
}

.world-selector {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-lg;
  min-width: 0;
}

.world {
  display: flex;
  align-items: center;
  min-width: 0;
  gap: $space-md;
}

.icon,
.icon-placeholder {
  width: 42px;
  height: 42px;
  flex: 0 0 42px;
  border-radius: $radius-md;
}

.icon {
  display: block;
  object-fit: cover;
}

.icon-placeholder {
  display: grid;
  place-items: center;
  background: $color-surface-3;
  color: $color-text-muted;
  font-family: $font-display;
  font-size: $font-lg;
  font-weight: $font-weight-bold;
}

.world-info {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.world-name {
  overflow: hidden;
  color: $color-text;
  font-size: $font-lg;
  font-weight: $font-weight-semibold;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.world-path {
  overflow: hidden;
  color: $color-text-muted;
  font-family: $font-mono;
  font-size: $font-xs;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.world-controls {
  display: flex;
  align-items: center;
  gap: $space-sm;
  flex: 0 0 auto;
}

.world-controls select {
  min-width: 180px;
  max-width: 280px;
  padding: $space-2 $space-md;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  outline: none;
  background: $color-surface-2;
  color: $color-text;
  font: inherit;
  font-size: $font-sm;
  cursor: pointer;

  &:hover:not(:disabled) {
    background: $color-hover;
  }

  &:focus {
    border-color: $color-focus;
  }

  &:disabled {
    color: $color-disabled;
    cursor: default;
  }
}

.refresh {
  display: grid;
  place-items: center;
  width: 34px;
  height: 34px;
  flex: 0 0 34px;
  padding: 0;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  background: $color-surface-2;
  color: $color-text-muted;
  font: inherit;
  font-size: $font-lg;
  cursor: pointer;

  &:hover:not(:disabled) {
    background: $color-hover;
    color: $color-text;
  }

  &:disabled {
    color: $color-disabled;
    cursor: default;
  }
}

.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid $color-border;
  border-top-color: $color-accent;
  border-radius: $radius-round;
  animation: spin 700ms linear infinite;
}

.sections {
  display: flex;
  gap: $space-1;
  overflow-x: auto;
  scrollbar-width: thin;
}

.sections button {
  flex: 0 0 auto;
  padding: $space-sm $space-md;
  border: 1px solid transparent;
  border-radius: $radius-md;
  background: transparent;
  color: $color-text-muted;
  font: inherit;
  font-size: $font-sm;
  font-weight: $font-weight-medium;
  white-space: nowrap;
  cursor: pointer;
  transition:
    background 120ms ease,
    color 120ms ease,
    border-color 120ms ease;

  &:hover {
    background: $color-hover;
    color: $color-text;
  }

  &.active {
    border-color: $color-border;
    background: $color-selected;
    color: $color-text;
  }
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 600px) {
  .navigation {
    gap: $space-sm;
    padding: $space-sm;
  }

  .world-selector {
    align-items: stretch;
    flex-direction: column;
    gap: $space-sm;
  }

  .world {
    gap: $space-sm;
  }

  .world-controls {
    width: 100%;
  }

  .world-controls select {
    min-width: 0;
    width: 100%;
    max-width: none;
  }

  .refresh {
    flex: 0 0 36px;
    width: 36px;
    height: 36px;
  }

  .icon,
  .icon-placeholder {
    width: 36px;
    height: 36px;
    flex-basis: 36px;
  }

  .world-name {
    font-size: $font-md;
  }
}
</style>
