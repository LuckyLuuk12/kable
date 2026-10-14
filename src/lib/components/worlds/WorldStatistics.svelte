<script lang="ts">
import { app, type WorldPlayerStatistics } from "$lib";
import { SvelteMap } from "svelte/reactivity";

let { world }: { world: { path: string } } = $props();

let statistics = $state<WorldPlayerStatistics[]>([]);
let loading = $state(false);
let error = $state<string | null>(null);

let selectedUuid = $state<string | null>(null);
let search = $state("");

async function load() {
  loading = true;
  error = null;

  try {
    const worldData = await app.worldsService.loadWorld(world.path);
    statistics = worldData?.statistics ?? [];

    if (selectedUuid && !statistics.some((player) => player.uuid === selectedUuid)) {
      selectedUuid = null;
    }
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    statistics = [];
  } finally {
    loading = false;
  }
}

let filteredPlayers = $derived(
  statistics.filter((player) => {
    const query = search.trim().toLowerCase();

    if (!query) return true;

    return (
      player.uuid.toLowerCase().includes(query) ||
      player.statistics.some((stat) => stat.category.toLowerCase().includes(query) || stat.name.toLowerCase().includes(query))
    );
  }),
);

let selectedPlayer = $derived(statistics.find((player) => player.uuid === selectedUuid) ?? null);

let groupedStatistics = $derived.by(() => {
  if (!selectedPlayer) return [];

  const groups = new SvelteMap<string, typeof selectedPlayer.statistics>();

  for (const statistic of selectedPlayer.statistics) {
    const group = groups.get(statistic.category);

    if (group) {
      group.push(statistic);
    } else {
      groups.set(statistic.category, [statistic]);
    }
  }

  return [...groups.entries()];
});

function selectPlayer(uuid: string) {
  selectedUuid = uuid;
}

function formatName(name: string) {
  const value = name.split(".").at(-1) ?? name;

  return value.replace(/_/g, " ").replace(/\b\w/g, (character) => character.toUpperCase());
}

$effect(() => {
  if (world.path) {
    load();
  }
});
</script>

<section class="statistics">
  <header class="header">
    <div>
      <h2>Statistics</h2>
      <p>Player statistics recorded by Minecraft</p>
    </div>

    <button type="button" onclick={load} disabled={loading}>
      {loading ? "Loading..." : "Reload"}
    </button>
  </header>

  {#if loading}
    <div class="state">Loading statistics...</div>
  {:else if error}
    <div class="state error">{error}</div>
  {:else if statistics.length === 0}
    <div class="state">No player statistics found.</div>
  {:else}
    <div class="content">
      <aside class="players">
        <div class="players-header">
          <span>Players</span>
          <span>{filteredPlayers.length}</span>
        </div>

        <input type="search" bind:value={search} placeholder="Search players..." aria-label="Search players" />

        <div class="player-list">
          {#each filteredPlayers as player (player.uuid)}
            <button type="button" class:selected={selectedUuid === player.uuid} onclick={() => selectPlayer(player.uuid)}>
              <span>{player.uuid}</span>
              <small>{player.statistics.length} statistics</small>
            </button>
          {/each}
        </div>
      </aside>

      <div class="details">
        {#if selectedPlayer}
          <header class="details-header">
            <div>
              <h3>{selectedPlayer.uuid}</h3>
              <p>
                {selectedPlayer.statistics.length} statistics
              </p>
            </div>
          </header>

          {#if groupedStatistics.length === 0}
            <div class="empty">This player has no statistics.</div>
          {:else}
            <div class="groups">
              {#each groupedStatistics as [category, entries] (category)}
                <section class="category">
                  <h4>{formatName(category)}</h4>

                  <div class="stat-list">
                    {#each entries as statistic (statistic.name)}
                      <div class="statistic">
                        <span class="stat-name">
                          {formatName(statistic.name)}
                        </span>

                        <span class="stat-value">
                          {statistic.value}
                        </span>
                      </div>
                    {/each}
                  </div>
                </section>
              {/each}
            </div>
          {/if}
        {:else}
          <div class="empty">Select a player to view their statistics.</div>
        {/if}
      </div>
    </div>
  {/if}
</section>

<style lang="scss">
@use "$lib/styles/variables" as *;

.statistics {
  display: flex;
  flex-direction: column;
  min-height: 0;
  gap: $space-lg;
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

  > button {
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
  grid-template-columns: 280px minmax(0, 1fr);
  min-height: 500px;
  overflow: hidden;
  border: 1px solid $color-border;
  border-radius: $radius-lg;
  background: $color-surface-1;
}

.players {
  display: flex;
  flex-direction: column;
  min-width: 0;
  padding: $space-md;
  border-right: 1px solid $color-border;
  gap: $space-md;
}

.players-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: $color-text;
  font-size: $font-sm;
  font-weight: $font-weight-semibold;

  span:last-child {
    color: $color-text-muted;
    font-weight: $font-weight-normal;
  }
}

input {
  width: 100%;
  box-sizing: border-box;
  padding: $space-sm $space-md;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  outline: none;
  background: $color-surface-2;
  color: $color-text;
  font: inherit;

  &::placeholder {
    color: $color-placeholder;
  }

  &:focus {
    border-color: $color-focus;
  }
}

.player-list {
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow-y: auto;
  gap: $space-1;
}

.player-list button {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: $space-1;
  width: 100%;
  padding: $space-sm;
  border: 0;
  border-radius: $radius-md;
  background: transparent;
  color: $color-text;
  font: inherit;
  text-align: left;
  cursor: pointer;

  &:hover {
    background: $color-hover;
  }

  &.selected {
    background: $color-selected;
  }

  span {
    width: 100%;
    overflow: hidden;
    font-family: $font-mono;
    font-size: $font-xs;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  small {
    color: $color-text-muted;
    font-size: $font-xs;
  }
}

.details {
  min-width: 0;
  overflow-y: auto;
  padding: $space-lg;
}

.details-header {
  margin-bottom: $space-lg;

  h3 {
    margin: 0;
    color: $color-text;
    font-family: $font-mono;
    font-size: $font-md;
    font-weight: $font-weight-medium;
    overflow-wrap: anywhere;
  }

  p {
    margin: $space-1 0 0;
    color: $color-text-muted;
    font-size: $font-sm;
  }
}

.groups {
  display: flex;
  flex-direction: column;
  gap: $space-lg;
}

.category {
  h4 {
    margin: 0 0 $space-sm;
    color: $color-text;
    font-size: $font-md;
    font-weight: $font-weight-semibold;
  }
}

.stat-list {
  overflow: hidden;
  border: 1px solid $color-border-muted;
  border-radius: $radius-md;
  background: $color-surface-2;
}

.statistic {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-md;
  padding: $space-sm $space-md;

  &:not(:last-child) {
    border-bottom: 1px solid $color-border-muted;
  }
}

.stat-name {
  color: $color-text-muted;
  font-size: $font-sm;
}

.stat-value {
  color: $color-text;
  font-family: $font-mono;
  font-size: $font-sm;
  font-weight: $font-weight-medium;
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

@media (max-width: 700px) {
  .content {
    grid-template-columns: 1fr;
  }

  .players {
    max-height: 280px;
    border-right: 0;
    border-bottom: 1px solid $color-border;
  }
}
</style>
