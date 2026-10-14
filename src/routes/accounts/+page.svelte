<script lang="ts">
import { AccountManager, app, Icon, type LoaderKind } from "$lib";
import { onMount } from "svelte";

let error: string | null = null;

let stats = $derived(app.profilesService.getStatistics());

// let worldStats: Awaited<ReturnType<typeof app.mapsService.getStatistics>> | null = null;
let worldStats: any = null; // TODO: Replace with proper type once we rework the maps service

onMount(async () => {
  try {
    // worldStats = await app.mapsService.getStatistics();
  } catch (err) {
    console.error("Failed to load world statistics:", err);
  }
});

function formatPlaytime(ms: number): string {
  if (!ms || ms < 0) return "0h 0m";

  const hours = Math.floor(ms / 3600000);
  const minutes = Math.floor((ms % 3600000) / 60000);

  if (hours === 0) return `${minutes}m`;
  if (minutes === 0) return `${hours}h`;

  return `${hours}h ${minutes}m`;
}

function formatPlaytimeLong(ms: number): string {
  if (!ms || ms < 0) return "No playtime yet";

  const hours = Math.floor(ms / 3600000);

  if (hours < 24) {
    const minutes = Math.floor((ms % 3600000) / 60000);

    return minutes > 0 ? `${hours} hours, ${minutes} minutes` : `${hours} hours`;
  }

  const days = Math.floor(hours / 24);
  const remainingHours = hours % 24;

  if (remainingHours === 0) return `${days} days`;

  return `${days} days, ${remainingHours} hours`;
}

function formatSize(sizeMB: number): string {
  if (sizeMB < 1024) {
    return `${sizeMB.toFixed(1)} MB`;
  }

  if (sizeMB < 1024 * 1024) {
    return `${(sizeMB / 1024).toFixed(2)} GB`;
  }

  return `${(sizeMB / (1024 * 1024)).toFixed(2)} TB`;
}
</script>

<div class="profile-page">
  <header class="page-header">
    <div class="page-header-content">
      <span class="page-eyebrow">Account</span>
      <h1>Profile & Account</h1>
      <p>Manage your Minecraft accounts and view your launcher statistics.</p>
    </div>
  </header>

  {#if error}
    <div class="error-message">
      <Icon name="alert" size="sm" />
      <span>{error}</span>
    </div>
  {/if}

  <main class="profile-content">
    <section class="account-section">
      <div class="section-heading">
        <div class="section-heading-icon">
          <Icon name="user-plus" forceType="svg" />
        </div>

        <div>
          <h2>Accounts</h2>
          <p>Manage your Microsoft accounts and choose which one to use with Kable.</p>
        </div>
      </div>

      <div class="account-container">
        <AccountManager />
      </div>
    </section>

    <section class="stats-section">
      <div class="section-heading">
        <div class="section-heading-icon">
          <Icon name="chart" />
        </div>

        <div>
          <h2>Minecraft Statistics</h2>
          <p>An overview of your Minecraft usage across Kable.</p>
        </div>
      </div>

      <div class="stats-grid">
        <div class="stat-card">
          <div class="stat-icon">
            <Icon name="clock" size="md" />
          </div>

          <div class="stat-content">
            <h4>Total Playtime</h4>
            <p class="stat-value">
              {formatPlaytimeLong(stats.profiles.totalPlaytimeMs)}
            </p>
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-icon">
            <Icon name="cube" size="md" />
          </div>

          <div class="stat-content">
            <h4>Total Profiles</h4>
            <p class="stat-value">{stats.profiles.totalProfiles}</p>
          </div>
        </div>

        {#if worldStats}
          <div class="stat-card">
            <div class="stat-icon">
              <Icon name="map" size="md" />
            </div>

            <div class="stat-content">
              <h4>Total Worlds</h4>
              <p class="stat-value">{worldStats.totalWorlds}</p>
            </div>
          </div>
        {/if}

        <div class="stat-card">
          <div class="stat-icon">
            <Icon name="calendar" size="md" />
          </div>

          <div class="stat-content">
            <h4>Last Played</h4>
            <p class="stat-value">
              {stats.profiles.lastPlayedDate ? new Date(stats.profiles.lastPlayedDate).toLocaleDateString() : "Never"}
            </p>
          </div>
        </div>

        {#if worldStats?.lastPlayedWorld}
          <div class="stat-card">
            <div class="stat-icon">
              <Icon name="clock" size="md" />
            </div>

            <div class="stat-content">
              <h4>Last Played World</h4>

              <p class="stat-value" title={worldStats.lastPlayedWorld.name}>
                {worldStats.lastPlayedWorld.name.length > 20 ? `${worldStats.lastPlayedWorld.name.substring(0, 20)}...` : worldStats.lastPlayedWorld.name}
              </p>

              <p class="stat-subtext">
                {new Date(worldStats.lastPlayedWorld.last_played || 0).toLocaleDateString()}
              </p>
            </div>
          </div>
        {/if}

        <div class="stat-card">
          <div class="stat-icon">
            <Icon name="play" size="md" />
          </div>

          <div class="stat-content">
            <h4>Total Launches</h4>
            <p class="stat-value">{stats.profiles.totalLaunches}</p>
          </div>
        </div>

        {#if stats.profiles.mostPlayedProfile}
          <div class="stat-card">
            <div class="stat-icon">
              <Icon name="cube" size="md" />
            </div>

            <div class="stat-content">
              <h4>Most Played</h4>

              <p class="stat-value" title={formatPlaytimeLong(stats.profiles.mostPlayedProfile.metadata.total_time_played_ms)}>
                {stats.profiles.mostPlayedProfile.metadata.name || stats.profiles.mostPlayedProfile.version.id}
              </p>

              <p class="stat-subtext">
                {formatPlaytime(stats.profiles.mostPlayedProfile.metadata.total_time_played_ms)}
              </p>
            </div>
          </div>
        {/if}

        {#if worldStats?.largestWorld}
          <div class="stat-card">
            <div class="stat-icon">
              <Icon name="maximize" size="md" />
            </div>

            <div class="stat-content">
              <h4>Largest World</h4>

              <p class="stat-value" title={worldStats.largestWorld.name}>
                {worldStats.largestWorld.name.length > 20 ? `${worldStats.largestWorld.name.substring(0, 20)}...` : worldStats.largestWorld.name}
              </p>

              <p class="stat-subtext">
                {formatSize(worldStats.largestWorld.size_mb)}
              </p>
            </div>
          </div>
        {/if}

        {#if worldStats}
          <div class="stat-card">
            <div class="stat-icon">
              <Icon name="hard-drive" size="md" />
            </div>

            <div class="stat-content">
              <h4>Total World Size</h4>
              <p class="stat-value">{formatSize(worldStats.totalSizeMB)}</p>
            </div>
          </div>
        {/if}

        <div class="stat-card">
          <div class="stat-icon">
            <Icon name="star" size="md" />
          </div>

          <div class="stat-content">
            <h4>Favorite Profiles</h4>
            <p class="stat-value">{stats.profiles.favoriteCount}</p>
          </div>
        </div>

        {#if worldStats}
          <div class="stat-card">
            <div class="stat-icon">
              <Icon name="archive" size="md" />
            </div>

            <div class="stat-content">
              <h4>Total Backups</h4>
              <p class="stat-value">{worldStats.totalBackups}</p>
            </div>
          </div>
        {/if}

        <div class="stat-card">
          <div class="stat-icon">
            <Icon name="activity" size="md" />
          </div>

          <div class="stat-content">
            <h4>Average Launches</h4>
            <p class="stat-value">
              {stats.profiles.averageLaunchesPerProfile}
            </p>
            <p class="stat-subtext">per profile</p>
          </div>
        </div>

        {#if worldStats}
          <div class="stat-card">
            <div class="stat-icon">
              <Icon name="layers" size="md" />
            </div>

            <div class="stat-content">
              <h4>Average Backups</h4>
              <p class="stat-value">{worldStats.averageBackupsPerWorld}</p>
              <p class="stat-subtext">per world</p>
            </div>
          </div>
        {/if}

        {#if stats.profiles.mostUsedLoader}
          <div class="stat-card">
            <div class="stat-icon">
              <Icon name={app.profilesService.getLoaderIcon(stats.profiles.mostUsedLoader as LoaderKind)} size="md" />
            </div>

            <div class="stat-content">
              <h4>Favorite Mod Loader</h4>
              <p class="stat-value">{stats.profiles.mostUsedLoader}</p>
              <p class="stat-subtext">
                {stats.profiles.loaderCounts[stats.profiles.mostUsedLoader]}
                profiles
              </p>
            </div>
          </div>
        {/if}

        {#if worldStats?.mostCommonGameMode}
          <div class="stat-card">
            <div class="stat-icon">
              <Icon name="gamepad" size="md" />
            </div>

            <div class="stat-content">
              <h4>Favorite Game Mode</h4>
              <p class="stat-value">{worldStats.mostCommonGameMode}</p>
              <p class="stat-subtext">
                {worldStats.gameModeCounts[worldStats.mostCommonGameMode]}
                worlds
              </p>
            </div>
          </div>
        {/if}

        {#if worldStats && worldStats.hardcoreCount > 0}
          <div class="stat-card hardcore">
            <div class="stat-icon">
              <Icon name="skull" size="md" />
            </div>

            <div class="stat-content">
              <h4>Hardcore Worlds</h4>
              <p class="stat-value">{worldStats.hardcoreCount}</p>
            </div>
          </div>
        {/if}
      </div>
    </section>
  </main>
</div>

<style lang="scss">
.profile-page {
  width: 100%;
  max-width: 1500px;
  margin: 0 auto;
  padding: 2.5rem 2.5rem 5rem;
  box-sizing: border-box;
}

.page-header {
  margin-bottom: 2.5rem;
}

.page-header-content {
  max-width: 720px;
}

.page-eyebrow {
  display: block;
  margin-bottom: 0.5rem;

  color: $color-accent;
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.page-header h1 {
  margin: 0 0 0.6rem;

  color: var(--text);
  font-size: clamp(2rem, 3vw, 2.75rem);
  font-weight: 700;
  letter-spacing: -0.04em;
  line-height: 1.1;
}

.page-header p {
  margin: 0;

  color: var(--placeholder);
  font-size: 1rem;
  line-height: 1.5;
}

.profile-content {
  display: flex;
  flex-direction: column;
  gap: 2.5rem;
}

/* Account */

.account-section,
.stats-section {
  min-width: 0;
}

.account-section {
  padding: 1.5rem;

  background: $color-surface-1;
  border: 1px solid $color-border;
  border-radius: $radius-lg;

  box-shadow:
    0 12px 40px rgba(0, 0, 0, 0.08),
    0 1px 2px rgba(0, 0, 0, 0.08);
}

.section-heading {
  display: flex;
  align-items: center;
  gap: 0.85rem;

  margin-bottom: 1.5rem;
}

.section-heading-icon {
  display: flex;
  align-items: center;
  justify-content: center;

  width: 2.5rem;
  height: 2.5rem;
  flex-shrink: 0;

  color: $color-accent;
  background: color-mix(in srgb, $color-accent 9%, transparent);
  border: 1px solid color-mix(in srgb, $color-accent 18%, transparent);
  border-radius: $radius-md;
}

.section-heading h2 {
  margin: 0 0 0.2rem;

  color: var(--text);
  font-size: 1.15rem;
  font-weight: 650;
  letter-spacing: -0.015em;
}

.section-heading p {
  margin: 0;

  color: var(--placeholder);
  font-size: 0.8rem;
  line-height: 1.4;
}

.account-container {
  min-width: 0;
}

.account-container :global(.account-manager) {
  width: 100%;
}

/* Statistics */

.stats-section {
  padding: 1.5rem;

  background: $color-surface-2;
  border: 1px solid $color-border;
  border-radius: $radius-lg;
}

.stats-section .section-heading {
  padding-bottom: 1.25rem;
  margin-bottom: 1.25rem;

  border-bottom: 1px solid $color-border;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 0.75rem;
}

.stat-card {
  position: relative;

  display: flex;
  align-items: center;
  gap: 0.9rem;

  min-width: 0;
  min-height: 82px;
  padding: 1rem;

  background: color-mix(in srgb, $color-surface-2 72%, $color-surface-3);

  border: 1px solid $color-border;
  border-radius: $radius-md;

  overflow: hidden;

  transition:
    background 0.15s ease,
    border-color 0.15s ease,
    transform 0.15s ease,
    box-shadow 0.15s ease;

  &::after {
    content: "";

    position: absolute;
    inset: 0;

    pointer-events: none;

    background: linear-gradient(135deg, color-mix(in srgb, $color-accent 4%, transparent), transparent 50%);

    opacity: 0;
    transition: opacity 0.15s ease;
  }

  &:hover {
    background: color-mix(in srgb, $color-accent 3%, var(--container));

    border-color: color-mix(in srgb, $color-accent 25%, $color-border);

    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.08);
    transform: translateY(-1px);

    &::after {
      opacity: 1;
    }
  }

  &.hardcore {
    .stat-icon {
      color: $color-error;
      background: color-mix(in srgb, $color-error 8%, transparent);
      border-color: color-mix(in srgb, $color-error 18%, transparent);
    }
  }
}

.stat-icon {
  position: relative;
  z-index: 1;

  display: flex;
  align-items: center;
  justify-content: center;

  width: 42px;
  height: 42px;
  flex-shrink: 0;

  color: $color-accent;
  background: color-mix(in srgb, $color-accent 8%, transparent);
  border: 1px solid color-mix(in srgb, $color-accent 15%, transparent);
  border-radius: $radius-md;
}

.stat-content {
  position: relative;
  z-index: 1;

  min-width: 0;
  flex: 1;
}

.stat-content h4 {
  margin: 0 0 0.2rem;

  color: $color-text-muted;
  font-size: 0.68rem;
  font-weight: 650;
  letter-spacing: 0.045em;
  line-height: 1.2;
  text-transform: uppercase;
}

.stat-value {
  margin: 0;

  color: $color-text;
  font-size: 1.1rem;
  font-weight: 650;
  line-height: 1.3;

  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.stat-subtext {
  margin: 0.15rem 0 0;

  color: $color-text-muted;
  font-size: 0.7rem;
  line-height: 1.2;
}

/* Error */

.error-message {
  display: flex;
  align-items: center;
  gap: 0.6rem;

  margin-bottom: 1.5rem;
  padding: 0.8rem 1rem;

  color: $color-error;
  background: color-mix(in srgb, $color-error 8%, transparent);
  border: 1px solid color-mix(in srgb, $color-error 30%, transparent);
  border-radius: $radius-md;

  font-size: 0.85rem;
}

/* Responsive */

@media (max-width: 900px) {
  .profile-page {
    padding: 2rem 1.5rem 4rem;
  }

  .stats-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 600px) {
  .profile-page {
    padding: 1.5rem 1rem 3rem;
  }

  .page-header {
    margin-bottom: 2rem;
  }

  .account-section,
  .stats-section {
    padding: 1rem;
  }

  .section-heading {
    align-items: flex-start;
  }

  .section-heading-icon {
    width: 2.25rem;
    height: 2.25rem;
  }

  .stats-grid {
    grid-template-columns: 1fr;
  }
}
</style>
