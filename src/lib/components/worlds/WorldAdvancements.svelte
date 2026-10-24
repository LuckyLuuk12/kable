<script lang="ts">
import { app, type WorldAdvancement } from "$lib";
import { SvelteSet } from "svelte/reactivity";

let { path }: { path: string } = $props();

let advancements = $state<WorldAdvancement[]>([]);
let loading = $state(false);
let error = $state<string | null>(null);
let expanded = $state<Set<string>>(new Set());

async function load() {
  loading = true;
  error = null;

  try {
    const worldData = await app.worldsService.loadWorld(path);
    advancements = worldData?.advancements ?? [];
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    advancements = [];
  } finally {
    loading = false;
  }
}

function toggle(id: string) {
  const next = new SvelteSet(expanded);

  if (next.has(id)) {
    next.delete(id);
  } else {
    next.add(id);
  }

  expanded = next;
}

function playerAdvancements(uuid: string) {
  return advancements.filter((advancement) => advancement.player_uuid === uuid);
}

function playerIds() {
  return [...new Set(advancements.map((advancement) => advancement.player_uuid))].sort();
}

function completedCount(uuid: string) {
  return playerAdvancements(uuid).filter((advancement) => advancement.done).length;
}

function totalCriteria(advancement: WorldAdvancement) {
  return advancement.criteria.length;
}

function completedCriteria(advancement: WorldAdvancement) {
  return advancement.criteria.filter((criterion) => criterion.achieved_at !== null).length;
}

$effect(() => {
  if (path) {
    load();
  }
});
</script>

<section class="advancements">
  <header class="header">
    <div>
      <h2>Advancements</h2>
      <p>Player advancement progress</p>
    </div>

    <button type="button" onclick={load} disabled={loading}>
      {loading ? "Loading..." : "Reload"}
    </button>
  </header>

  {#if loading}
    <div class="state">Loading advancements...</div>
  {:else if error}
    <div class="state error">{error}</div>
  {:else if advancements.length === 0}
    <div class="state">No advancement data available.</div>
  {:else}
    <div class="players">
      {#each playerIds() as uuid (uuid)}
        {@const player = playerAdvancements(uuid)}
        {@const completed = completedCount(uuid)}

        <section class="player">
          <header class="player-header">
            <div class="player-info">
              <span class="player-name">{uuid}</span>
              <span class="player-progress">
                {completed} / {player.length} completed
              </span>
            </div>
          </header>

          <div class="list">
            {#each player as advancement (advancement.id)}
              {@const criteriaCompleted = completedCriteria(advancement)}
              {@const criteriaTotal = totalCriteria(advancement)}
              {@const isExpanded = expanded.has(`${uuid}:${advancement.id}`)}

              <article class:completed={advancement.done} class="advancement">
                <button type="button" class="advancement-header" onclick={() => toggle(`${uuid}:${advancement.id}`)}>
                  <span class="indicator">
                    {advancement.done ? "✓" : "○"}
                  </span>

                  <span class="advancement-info">
                    <span class="advancement-id">{advancement.id}</span>

                    <span class="progress">
                      {criteriaCompleted} / {criteriaTotal} criteria
                    </span>
                  </span>

                  <span class="chevron" class:open={isExpanded}> › </span>
                </button>

                {#if isExpanded}
                  <div class="details">
                    {#if advancement.criteria.length > 0}
                      <div class="criteria">
                        {#each advancement.criteria as criterion (criterion.id)}
                          <div class:achieved={criterion.achieved_at !== null} class="criterion">
                            <span class="criterion-status">
                              {criterion.achieved_at !== null ? "✓" : "○"}
                            </span>

                            <span class="criterion-id">{criterion.id}</span>

                            {#if criterion.achieved_at}
                              <span class="achieved-at">
                                {criterion.achieved_at}
                              </span>
                            {/if}
                          </div>
                        {/each}
                      </div>
                    {:else}
                      <span class="no-criteria">No criteria recorded.</span>
                    {/if}
                  </div>
                {/if}
              </article>
            {/each}
          </div>
        </section>
      {/each}
    </div>
  {/if}
</section>

<style lang="scss">
@use "$lib/styles/variables" as *;

.advancements {
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

.players {
  display: flex;
  flex-direction: column;
  gap: $space-lg;
}

.player {
  overflow: hidden;
  border: 1px solid $color-border;
  border-radius: $radius-lg;
  background: $color-surface-1;
}

.player-header {
  padding: $space-md $space-lg;
  border-bottom: 1px solid $color-border;
  background: $color-surface-2;
}

.player-info {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-md;
}

.player-name {
  color: $color-text;
  font-family: $font-mono;
  font-size: $font-sm;
  font-weight: $font-weight-medium;
}

.player-progress {
  color: $color-text-muted;
  font-size: $font-sm;
}

.list {
  display: flex;
  flex-direction: column;
}

.advancement {
  border-bottom: 1px solid $color-border-muted;

  &:last-child {
    border-bottom: 0;
  }

  &.completed {
    .indicator,
    .advancement-id {
      color: $color-success;
    }
  }
}

.advancement-header {
  display: flex;
  align-items: center;
  width: 100%;
  gap: $space-md;
  padding: $space-md $space-lg;
  border: 0;
  background: transparent;
  color: inherit;
  font: inherit;
  text-align: left;
  cursor: pointer;

  &:hover {
    background: $color-hover;
  }
}

.indicator {
  flex: 0 0 1.25rem;
  color: $color-text-muted;
  font-size: $font-lg;
  text-align: center;
}

.advancement-info {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
  gap: $space-1;
}

.advancement-id {
  overflow: hidden;
  color: $color-text;
  font-family: $font-mono;
  font-size: $font-sm;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.progress {
  color: $color-text-muted;
  font-size: $font-xs;
}

.chevron {
  color: $color-text-muted;
  font-size: $font-lg;
  transform: rotate(0deg);
  transition: transform 0.15s ease;

  &.open {
    transform: rotate(90deg);
  }
}

.details {
  padding: 0 $space-lg $space-md calc($space-lg + 1.25rem + $space-md);
}

.criteria {
  display: flex;
  flex-direction: column;
  gap: $space-1;
  padding: $space-sm;
  border-radius: $radius-md;
  background: $color-surface-2;
}

.criterion {
  display: flex;
  align-items: center;
  gap: $space-sm;
  min-height: 1.75rem;

  &.achieved {
    .criterion-status {
      color: $color-success;
    }

    .criterion-id {
      color: $color-text;
    }
  }
}

.criterion-status {
  flex: 0 0 1rem;
  color: $color-text-muted;
  text-align: center;
}

.criterion-id {
  flex: 1;
  color: $color-text-muted;
  font-family: $font-mono;
  font-size: $font-xs;
}

.achieved-at {
  color: $color-text-muted;
  font-size: $font-xs;
}

.no-criteria {
  color: $color-text-muted;
  font-size: $font-sm;
}

.state {
  padding: $space-xl;
  border: 1px solid $color-border;
  border-radius: $radius-lg;
  background: $color-surface-1;
  color: $color-text-muted;
  text-align: center;

  &.error {
    color: $color-error;
  }
}
</style>
