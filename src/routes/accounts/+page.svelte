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
