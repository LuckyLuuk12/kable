<script lang="ts">
import {
  app,
  WorldAdvancements,
  WorldDatapacks,
  WorldDimensions,
  WorldNbt,
  WorldOverview,
  WorldPlayers,
  WorldRegions,
  WorldsNavigation,
  WorldStatistics,
  type World,
} from "$lib";

type Section = "overview" | "players" | "dimensions" | "datapacks" | "regions" | "advancements" | "statistics" | "nbt";

let selectedWorldId = $state<string | null>(app.worldsService.worlds[0]?.id ?? null);

let activeSection = $state<Section>("overview");

let world = $state<World | null>(null);
let loadingWorld = $state(false);

let selectedWorld = $derived(app.worldsService.worlds.find((world) => world.id === selectedWorldId) ?? null);

async function loadSelectedWorld() {
  const summary = selectedWorld;

  if (!summary) {
    world = null;
    return;
  }

  loadingWorld = true;
  world = null;

  try {
    world = await app.worldsService.loadWorld(summary.path);
  } finally {
    loadingWorld = false;
  }
}

function changeWorld(id: string) {
  if (id === selectedWorldId) return;

  selectedWorldId = id;
  activeSection = "overview";
}

$effect(() => {
  if (!selectedWorldId && app.worldsService.worlds.length > 0) {
    selectedWorldId = app.worldsService.worlds[0].id;
  }
});

$effect(() => {
  if (selectedWorld) {
    loadSelectedWorld();
  } else {
    world = null;
  }
});
</script>

<div class="world-page">
  <WorldsNavigation world={selectedWorld} active={activeSection} onchange={(section) => (activeSection = section)} onworldchange={changeWorld} />

  {#if loadingWorld}
    <main class="content">
      <div class="state">
        <span class="spinner"></span>
        <span>Loading world...</span>
      </div>
    </main>
  {:else if world}
    <main class="content">
      {#if activeSection === "overview"}
        <WorldOverview {world} />
      {:else if activeSection === "players"}
        <WorldPlayers {world} />
      {:else if activeSection === "dimensions"}
        <WorldDimensions {world} />
      {:else if activeSection === "datapacks"}
        <WorldDatapacks {world} />
      {:else if activeSection === "regions"}
        <WorldRegions {world} />
      {:else if activeSection === "advancements"}
        <WorldAdvancements path={world.path} />
      {:else if activeSection === "statistics"}
        <WorldStatistics {world} />
      {:else if activeSection === "nbt"}
        <WorldNbt path={`${world.path}/level.dat`} />
      {/if}
    </main>
  {:else}
    <main class="content">
      <div class="empty">
        <span>No worlds available.</span>
      </div>
    </main>
  {/if}
</div>

<style lang="scss">
@use "$lib/styles/variables" as *;

.world-page {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-height: 0;
  background: $color-background;
  color: $color-text;
}

.content {
  display: flex;
  flex: 1;
  min-width: 100%;
  min-height: 0;
  overflow: auto;
  padding: $space-lg;
}

.state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: $space-md;
  width: 100%;
  height: 100%;
  color: $color-text-muted;
}

.spinner {
  width: 18px;
  height: 18px;
  flex: 0 0 18px;
  border: 2px solid $color-border;
  border-top-color: $color-accent;
  border-radius: $radius-round;
  animation: spin 700ms linear infinite;
}

.empty {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  color: $color-text-muted;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
