<script lang="ts">
import { app } from "$lib";
import WorldsNavigation from "./WorldsNavigation.svelte";

let {
  worldId,
}: {
  worldId?: string;
} = $props();

type WorldFeature = "overview" | "players" | "dimensions" | "regions" | "datapacks" | "advancements" | "statistics" | "nbt";

let activeFeature = $state<WorldFeature>("overview");

let world = $derived(worldId ? app.worldsService.getWorld(worldId) : (app.worldsService.worlds[0] ?? null));
</script>

<div class="worlds-browser">
  {#if world}
    <header class="worlds-header">
      <div class="world-identity">
        {#if world.icon}
          <img class="world-icon" src={world.icon} alt="" />
        {:else}
          <div class="world-icon-placeholder">
            <span>W</span>
          </div>
        {/if}

        <div class="world-title">
          <h1>{world.name}</h1>
          <span>{world.path}</span>
        </div>
      </div>

      <WorldsNavigation
        {world}
        active={activeFeature}
        onchange={(event) => {
          activeFeature = event.at(-1) as WorldFeature;
        }} />
    </header>

    <main class="worlds-content">
      {#if activeFeature === "overview"}
        <div></div>
      {:else if activeFeature === "players"}
        <div></div>
      {:else if activeFeature === "dimensions"}
        <div></div>
      {:else if activeFeature === "regions"}
        <div></div>
      {:else if activeFeature === "datapacks"}
        <div></div>
      {:else if activeFeature === "advancements"}
        <div></div>
      {:else if activeFeature === "statistics"}
        <div></div>
      {:else if activeFeature === "nbt"}
        <div></div>
      {/if}
    </main>
  {:else if app.worldsService.loading}
    <div class="worlds-state">
      <span>Loading worlds...</span>
    </div>
  {:else}
    <div class="worlds-state">
      <span>No world selected</span>
    </div>
  {/if}
</div>

<style lang="scss">
@use "$lib/styles/variables" as *;

.worlds-browser {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-height: 0;
  overflow: hidden;

  color: $color-text;
  font-family: $font-sans;
}

.worlds-header {
  display: flex;
  flex-direction: column;
  gap: $space-lg;
  flex-shrink: 0;

  padding: $space-lg $space-xl;
  border-bottom: 1px solid $color-border;
  background: $color-surface-1;
}

.world-identity {
  display: flex;
  align-items: center;
  gap: $space-md;
  min-width: 0;
}

.world-icon,
.world-icon-placeholder {
  width: 48px;
  height: 48px;
  flex-shrink: 0;

  border-radius: $radius-md;
  border: 1px solid $color-border;
}

.world-icon {
  display: block;
  object-fit: cover;
}

.world-icon-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;

  background: $color-surface-2;
  color: $color-text-muted;
  font-family: $font-display;
  font-size: $font-xl;
  font-weight: $font-weight-bold;
}

.world-title {
  display: flex;
  flex-direction: column;
  gap: $space-1;
  min-width: 0;
}

.world-title h1 {
  margin: 0;
  overflow: hidden;

  color: $color-text;
  font-size: $font-xl;
  font-weight: $font-weight-semibold;
  line-height: $font-line-tight;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.world-title span {
  overflow: hidden;

  color: $color-text-muted;
  font-family: $font-mono;
  font-size: $font-xs;
  line-height: $font-line-normal;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.worlds-content {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: $space-xl;
  background: $color-background;
}

.worlds-state {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;

  color: $color-text-muted;
  font-size: $font-md;
}
</style>
