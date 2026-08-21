<script lang="ts">
import type { World, WorldPlayer } from "$lib";

interface Props {
  world: World;
  onselect?: (player: WorldPlayer) => void;
}

let { world, onselect }: Props = $props();

const formatGameMode = (mode: WorldPlayer["game_mode"]) => {
  if (!mode) return "Unknown";

  return mode
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
};

const formatDimension = (dimension: string | null) => {
  if (!dimension) return "Unknown";

  const name = dimension.includes(":") ? (dimension.split(":").at(-1) ?? dimension) : dimension;

  return name
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
};

const formatNumber = (value: number | string | null | undefined) => {
  if (value === null || value === undefined) return "Unknown";

  const number = typeof value === "string" ? Number(value) : value;

  return Number.isFinite(number) ? number.toLocaleString() : String(value);
};

const getPlayerName = (player: WorldPlayer) => {
  return player.name || player.uuid;
};

const sortedPlayers = $derived([...world.players].sort((a, b) => getPlayerName(a).localeCompare(getPlayerName(b))));
</script>

<div class="players">
  <div class="header">
    <div>
      <h2>Players</h2>
      <p>
        {world.players.length}
        {world.players.length === 1 ? "player" : "players"} found
      </p>
    </div>
  </div>

  {#if sortedPlayers.length === 0}
    <div class="empty">
      <span class="empty-title">No players found</span>
      <span class="empty-description"> This world does not contain any player data. </span>
    </div>
  {:else}
    <div class="player-list">
      {#each sortedPlayers as player (player.uuid)}
        <button type="button" class="player" onclick={() => onselect?.(player)}>
          <div class="identity">
            <div class="avatar">
              {getPlayerName(player).charAt(0).toUpperCase()}
            </div>

            <div class="identity-text">
              <span class="name">{getPlayerName(player)}</span>
              <span class="uuid">{player.uuid}</span>
            </div>
          </div>

          <div class="details">
            <div class="detail">
              <span class="label">Dimension</span>
              <span class="value">
                {formatDimension(player.dimension)}
              </span>
            </div>

            <div class="detail">
              <span class="label">Position</span>
              <span class="value mono">
                {#if player.position}
                  {player.position.x?.toFixed(1)},
                  {player.position.y?.toFixed(1)},
                  {player.position.z?.toFixed(1)}
                {:else}
                  Unknown
                {/if}
              </span>
            </div>

            <div class="detail">
              <span class="label">Game Mode</span>
              <span class="value">
                {formatGameMode(player.game_mode)}
              </span>
            </div>

            <div class="detail">
              <span class="label">Level</span>
              <span class="value">
                {formatNumber(player.experience_level)}
              </span>
            </div>

            <div class="detail">
              <span class="label">Health</span>
              <span class="value">
                {player.health !== null && player.health !== undefined ? `${player.health.toFixed(1)}` : "Unknown"}
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

.players {
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

.player-list {
  display: flex;
  flex-direction: column;
  gap: $space-2;
}

.player {
  display: grid;
  grid-template-columns: minmax(220px, 1.2fr) minmax(0, 2fr) auto;
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

.player:hover {
  background: $color-hover;
  border-color: $color-border-muted;
}

.player:focus-visible {
  outline: 2px solid $color-focus;
  outline-offset: 2px;
}

.identity {
  display: flex;
  align-items: center;
  gap: $space-3;

  min-width: 0;
}

.avatar {
  display: grid;
  place-items: center;

  width: 42px;
  height: 42px;
  flex-shrink: 0;

  border-radius: $radius-md;
  background: $color-accent-muted;

  color: $color-accent;
  font-size: $font-lg;
  font-weight: $font-weight-semibold;
}

.identity-text {
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

.uuid {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;

  color: $color-text-muted;
  font-family: $font-mono;
  font-size: $font-xs;
}

.details {
  display: grid;
  grid-template-columns: repeat(5, minmax(80px, 1fr));
  gap: $space-3;

  min-width: 0;
}

.detail {
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
  font-size: $font-sm;
}

.mono {
  font-family: $font-mono;
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

@media (max-width: 1100px) {
  .player {
    grid-template-columns: minmax(200px, 1fr) auto;
  }

  .details {
    grid-column: 1 / -1;
    grid-row: 2;

    grid-template-columns: repeat(5, minmax(100px, 1fr));
  }

  .chevron {
    grid-column: 2;
    grid-row: 1;
  }
}

@media (max-width: 700px) {
  .players {
    padding: $space-3;
  }

  .details {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
