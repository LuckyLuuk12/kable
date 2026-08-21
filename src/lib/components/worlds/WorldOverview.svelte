<script lang="ts">
import { type World, Image } from "$lib";

interface Props {
  world: World;
}

let { world }: Props = $props();

const formatBytes = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
  return `${(bytes / 1024 ** 3).toFixed(1)} GB`;
};

const formatGameMode = (mode: World["level"] extends infer T ? (T extends { game_mode?: infer M } ? M : never) : never) => {
  if (!mode) return "Unknown";

  return mode
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
};
</script>

<div class="overview">
  <section class="hero">
    <div class="hero-icon">
      {#if world.icon}
        <Image key={world.icon} alt="" />
      {:else}
        <div class="icon-placeholder"></div>
      {/if}
    </div>

    <div class="hero-content">
      <h1>{world.name}</h1>
      <span class="path" title={world.path}>{world.path}</span>
    </div>
  </section>

  {#if world.level}
    <section class="section">
      <h2>World</h2>

      <div class="properties">
        <div class="property">
          <span class="label">Level Name</span>
          <span class="value">{world.level.level_name ?? "Unknown"}</span>
        </div>

        <div class="property">
          <span class="label">Version</span>
          <span class="value">
            {world.level.version?.name ?? "Unknown"}
            {#if world.level.version?.id !== undefined}
              <span class="muted">({world.level.version.id})</span>
            {/if}
          </span>
        </div>

        <div class="property">
          <span class="label">Data Version</span>
          <span class="value">{world.level.data_version ?? "Unknown"}</span>
        </div>

        <div class="property">
          <span class="label">Game Mode</span>
          <span class="value">
            {formatGameMode(world.level.game_mode as never)}
          </span>
        </div>

        <div class="property">
          <span class="label">Difficulty</span>
          <span class="value">
            {world.level.difficulty ? world.level.difficulty.charAt(0).toUpperCase() + world.level.difficulty.slice(1) : "Unknown"}
          </span>
        </div>

        <div class="property">
          <span class="label">Hardcore</span>
          <span class="value">{world.level.hardcore ? "Yes" : "No"}</span>
        </div>

        <div class="property">
          <span class="label">Commands</span>
          <span class="value">
            {world.level.allow_commands ? "Enabled" : "Disabled"}
          </span>
        </div>

        <div class="property">
          <span class="label">Seed</span>
          <span class="value mono">{world.level.seed ?? "Unknown"}</span>
        </div>
      </div>
    </section>
  {/if}

  <section class="section">
    <h2>Storage</h2>

    <div class="cards">
      <div class="stat-card">
        <span class="stat-value">{world.players.length}</span>
        <span class="stat-label">Players</span>
      </div>

      <div class="stat-card">
        <span class="stat-value">{world.dimensions.length}</span>
        <span class="stat-label">Dimensions</span>
      </div>

      <div class="stat-card">
        <span class="stat-value">{world.datapacks.length}</span>
        <span class="stat-label">Datapacks</span>
      </div>

      <div class="stat-card">
        <span class="stat-value">{world.advancements.length}</span>
        <span class="stat-label">Advancements</span>
      </div>

      <div class="stat-card">
        <span class="stat-value">{world.statistics.length}</span>
        <span class="stat-label">Statistics Files</span>
      </div>

      <div class="stat-card">
        <span class="stat-value">
          {world.region_storage.reduce((total, storage) => total + Number(storage.chunk_count), 0)}
        </span>
        <span class="stat-label">Chunks</span>
      </div>

      <div class="stat-card">
        <span class="stat-value">
          {formatBytes(world.region_storage.reduce((total, storage) => total + Number(storage.total_size), 0))}
        </span>
        <span class="stat-label">Region Data</span>
      </div>
    </div>
  </section>

  {#if world.level?.time || world.level?.spawn || world.level?.weather}
    <section class="section">
      <h2>World State</h2>

      <div class="properties">
        {#if world.level.time}
          <div class="property">
            <span class="label">Game Time</span>
            <span class="value mono">
              {world.level.time.game_time ?? "Unknown"}
            </span>
          </div>

          <div class="property">
            <span class="label">Day Time</span>
            <span class="value mono">
              {world.level.time.day_time ?? "Unknown"}
            </span>
          </div>
        {/if}

        {#if world.level.spawn}
          <div class="property">
            <span class="label">Spawn</span>
            <span class="value mono">
              {world.level.spawn.x}, {world.level.spawn.y},
              {world.level.spawn.z}
            </span>
          </div>
        {/if}

        {#if world.level.weather}
          <div class="property">
            <span class="label">Rain</span>
            <span class="value">
              {world.level.weather.raining ? "Raining" : "Clear"}
            </span>
          </div>

          <div class="property">
            <span class="label">Thunder</span>
            <span class="value">
              {world.level.weather.thundering ? "Thunderstorm" : "None"}
            </span>
          </div>
        {/if}
      </div>
    </section>
  {/if}
</div>

<style lang="scss">
@use "$lib/styles/variables" as *;

.overview {
  display: flex;
  flex-direction: column;
  gap: $space-5;

  width: 100%;
  padding: $space-5;
  box-sizing: border-box;
}

.hero {
  display: flex;
  align-items: center;
  gap: $space-4;

  padding: $space-4;

  background: $color-surface-1;
  border: 1px solid $color-border;
  border-radius: $radius-lg;
}

.hero-icon {
  width: 72px;
  height: 72px;
  flex-shrink: 0;

  overflow: hidden;
  border-radius: $radius-lg;
}

.hero-icon img,
.icon-placeholder {
  display: block;

  width: 100%;
  height: 100%;

  object-fit: cover;
}

.icon-placeholder {
  background: $color-surface-3;
}

.hero-content {
  min-width: 0;
}

h1 {
  margin: 0 0 $space-1;

  overflow: hidden;
  text-overflow: ellipsis;

  color: $color-text;
  font-family: $font-sans;
  font-size: $font-2xl;
  font-weight: $font-weight-bold;
}

.path {
  display: block;

  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;

  color: $color-text-muted;
  font-family: $font-mono;
  font-size: $font-xs;
}

.section {
  display: flex;
  flex-direction: column;
  gap: $space-3;
}

h2 {
  margin: 0;

  color: $color-text;
  font-family: $font-sans;
  font-size: $font-lg;
  font-weight: $font-weight-semibold;
}

.properties {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 1px;

  overflow: hidden;

  background: $color-border;
  border: 1px solid $color-border;
  border-radius: $radius-lg;
}

.property {
  display: flex;
  flex-direction: column;
  gap: $space-1;

  min-width: 0;
  padding: $space-3;

  background: $color-surface-1;
}

.label,
.stat-label {
  color: $color-text-muted;
  font-size: $font-xs;
  font-weight: $font-weight-medium;
}

.value {
  min-width: 0;

  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;

  color: $color-text;
  font-size: $font-sm;
}

.muted {
  color: $color-text-muted;
}

.mono {
  font-family: $font-mono;
}

.cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: $space-2;
}

.stat-card {
  display: flex;
  flex-direction: column;
  gap: $space-1;

  padding: $space-3;

  background: $color-surface-1;
  border: 1px solid $color-border;
  border-radius: $radius-lg;
}

.stat-value {
  color: $color-text;
  font-size: $font-xl;
  font-weight: $font-weight-semibold;
}
</style>
