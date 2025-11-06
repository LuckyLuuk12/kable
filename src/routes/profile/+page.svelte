<script lang="ts">
import { onMount } from "svelte";
import {
  Icon,
  AccountSwitcher,
  AccountManager,
  InstallationService,
  MapsService,
} from "$lib";

// State variables
let error: string | null = null;
let stats = InstallationService.getStatistics();
let worldStats: Awaited<ReturnType<typeof MapsService.getStatistics>> | null =
  null;

// Reactively update stats when installations change
$: {
  // This will re-run whenever the installations store changes
  stats = InstallationService.getStatistics();
}

onMount(async () => {
  await loadWorldStats();
});

async function loadWorldStats() {
  try {
    worldStats = await MapsService.getStatistics();
  } catch (err) {
    console.error("Failed to load world statistics:", err);
  }
}

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
    return minutes > 0
      ? `${hours} hours, ${minutes} minutes`
      : `${hours} hours`;
  }

  const days = Math.floor(hours / 24);
  const remainingHours = hours % 24;

  if (remainingHours === 0) return `${days} days`;
  return `${days} days, ${remainingHours} hours`;
}

function formatSize(sizeMB: number): string {
  if (sizeMB < 1024) {
    return `${sizeMB.toFixed(1)} MB`;
  } else if (sizeMB < 1024 * 1024) {
    return `${(sizeMB / 1024).toFixed(2)} GB`;
  } else {
    return `${(sizeMB / (1024 * 1024)).toFixed(2)} TB`;
  }
}
</script>

<div class="profile-page">
  <div class="page-header">
    <h1>Profile & Account</h1>
    <p>Manage your Microsoft account and view your Minecraft statistics</p>
  </div>

  {#if error}
    <div class="error-message">
      <Icon name="alert" size="sm" />
      {error}
    </div>
  {/if}

  <div class="profile-sections">
    <!-- Top Row: Account Switcher and Account Management -->
    <div class="top-row">
      <!-- Account Switcher Section -->
      <section class="profile-section">
        <div class="section-header">
          <h2><Icon name="user" forceType="svg" /> Quick Account Switcher</h2>
        </div>
        <AccountSwitcher />
      </section>

      <!-- Account Management Section -->
      <section class="profile-section">
        <div class="section-header">
          <h2><Icon name="user-plus" forceType="svg" /> Account Management</h2>
        </div>

        <div class="account-management-container">
          <AccountManager />
        </div>
      </section>
    </div>

    <!-- Statistics Section -->
    <section class="profile-section stats-section">
      <div class="section-header">
        <h2><Icon name="chart" /> Minecraft Statistics</h2>
      </div>

      <div class="stats-grid">
        <!-- Total Playtime -->
        <div class="stat-card">
          <div class="stat-icon">
            <Icon name="clock" size="md" />
          </div>
          <div class="stat-content">
            <h4>Total Playtime</h4>
            <p class="stat-value">
              {formatPlaytimeLong(stats.totalPlaytimeMs)}
            </p>
          </div>
        </div>

        <!-- Total Installations -->
        <div class="stat-card">
          <div class="stat-icon">
            <Icon name="cube" size="md" />
          </div>
          <div class="stat-content">
            <h4>Total Installations</h4>
            <p class="stat-value">{stats.totalInstallations}</p>
          </div>
        </div>

        <!-- Total Worlds -->
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

        <!-- Last Played Installation -->
        <div class="stat-card">
          <div class="stat-icon">
            <Icon name="calendar" size="md" />
          </div>
          <div class="stat-content">
            <h4>Last Played</h4>
            <p class="stat-value">
              {stats.lastPlayedDate
                ? new Date(stats.lastPlayedDate).toLocaleDateString()
                : "Never"}
            </p>
          </div>
        </div>

        <!-- Last Played World -->
        {#if worldStats?.lastPlayedWorld}
          <div class="stat-card">
            <div class="stat-icon">
              <Icon name="clock" size="md" />
            </div>
            <div class="stat-content">
              <h4>Last Played World</h4>
              <p class="stat-value" title={worldStats.lastPlayedWorld.name}>
                {worldStats.lastPlayedWorld.name.length > 20
                  ? worldStats.lastPlayedWorld.name.substring(0, 20) + "..."
                  : worldStats.lastPlayedWorld.name}
              </p>
              <p class="stat-subtext">
                {new Date(
                  worldStats.lastPlayedWorld.last_played || 0,
                ).toLocaleDateString()}
              </p>
            </div>
          </div>
        {/if}

        <!-- Total Launches -->
        <div class="stat-card">
          <div class="stat-icon">
            <Icon name="play" size="md" />
          </div>
          <div class="stat-content">
            <h4>Total Launches</h4>
            <p class="stat-value">{stats.totalLaunches}</p>
          </div>
        </div>

        <!-- Most Played Installation -->
        {#if stats.mostPlayedInstallation}
          <div class="stat-card">
            <div
              class="stat-icon"
              style="color: {InstallationService.getLoaderColor(
                InstallationService.getVersionData(stats.mostPlayedInstallation)
                  .loader,
              )}"
            >
              <Icon
                name={InstallationService.getLoaderIcon(
                  InstallationService.getVersionData(
                    stats.mostPlayedInstallation,
                  ).loader,
                )}
                size="md"
              />
            </div>
            <div class="stat-content">
              <h4>Most Played</h4>
              <p
                class="stat-value"
                title={formatPlaytimeLong(
                  stats.mostPlayedInstallation.total_time_played_ms,
                )}
              >
                {stats.mostPlayedInstallation.name}
              </p>
              <p class="stat-subtext">
                {formatPlaytime(
                  stats.mostPlayedInstallation.total_time_played_ms,
                )}
              </p>
            </div>
          </div>
        {/if}

        <!-- Largest World -->
        {#if worldStats?.largestWorld}
          <div class="stat-card">
            <div class="stat-icon">
              <Icon name="maximize" size="md" />
            </div>
            <div class="stat-content">
              <h4>Largest World</h4>
              <p class="stat-value" title={worldStats.largestWorld.name}>
                {worldStats.largestWorld.name.length > 20
                  ? worldStats.largestWorld.name.substring(0, 20) + "..."
                  : worldStats.largestWorld.name}
              </p>
              <p class="stat-subtext">
                {formatSize(worldStats.largestWorld.size_mb)}
              </p>
            </div>
          </div>
        {/if}

        <!-- Total World Size -->
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

        <!-- Favorite Installations -->
        <div class="stat-card">
          <div class="stat-icon">
            <Icon name="star" size="md" />
          </div>
          <div class="stat-content">
            <h4>Favorite Installations</h4>
            <p class="stat-value">{stats.favoriteCount}</p>
          </div>
        </div>

        <!-- Total Backups -->
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

        <!-- Average Launches -->
        <div class="stat-card">
          <div class="stat-icon">
            <Icon name="activity" size="md" />
          </div>
          <div class="stat-content">
            <h4>Average Launches</h4>
            <p class="stat-value">{stats.averageLaunchesPerInstallation}</p>
            <p class="stat-subtext">per installation</p>
          </div>
        </div>

        <!-- Average Backups -->
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

        <!-- Favorite Mod Loader -->
        {#if stats.mostUsedLoader}
          <div class="stat-card">
            <div
              class="stat-icon"
              style="color: {InstallationService.getLoaderColor(
                stats.mostUsedLoader,
              )}"
            >
              <Icon
                name={InstallationService.getLoaderIcon(stats.mostUsedLoader)}
                size="md"
              />
            </div>
            <div class="stat-content">
              <h4>Favorite Mod Loader</h4>
              <p class="stat-value">{stats.mostUsedLoader}</p>
              <p class="stat-subtext">
                {stats.loaderCounts[stats.mostUsedLoader]} installations
              </p>
            </div>
          </div>
        {/if}

        <!-- Favorite Game Mode -->
        {#if worldStats?.mostCommonGameMode}
          <div class="stat-card">
            <div class="stat-icon">
              <Icon name="gamepad" size="md" />
            </div>
            <div class="stat-content">
              <h4>Favorite Game Mode</h4>
              <p class="stat-value">{worldStats.mostCommonGameMode}</p>
              <p class="stat-subtext">
                {worldStats.gameModeCounts[worldStats.mostCommonGameMode]} worlds
              </p>
            </div>
          </div>
        {/if}

        <!-- Hardcore Worlds -->
        {#if worldStats && worldStats.hardcoreCount > 0}
          <div class="stat-card">
            <div class="stat-icon" style="color: var(--red)">
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
  </div>
</div>

<style lang="scss">
.profile-page {
  width: 100%;
  padding: 0 2rem;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

.page-header {
  margin-bottom: 2rem;
  text-align: center;

  h1 {
    margin: 0 0 0.5rem;
    font-size: 2.5rem;
    font-weight: 700;
    background: linear-gradient(135deg, var(--primary), var(--tertiary));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  p {
    margin: 0;
    color: var(--placeholder);
    font-size: 1.125rem;
    line-height: 1.6;
  }
}

.profile-sections {
  display: flex;
  flex-direction: column;
  gap: 2rem;
}

.top-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 2rem;
  align-items: stretch;

  @media (max-width: 1024px) {
    grid-template-columns: 1fr;
  }
}

.profile-section {
  background: var(--container);
  border: 1px solid var(--dark-600);
  border-radius: var(--border-radius-large);
  padding: 2rem;
  transition: all 0.3s ease;
  position: relative;
  overflow: visible;
  word-wrap: break-word;
  overflow-wrap: break-word;

  &::before {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(
      90deg,
      transparent,
      #{"color-mix(in srgb, var(--primary), 30%, transparent)"},
      transparent
    );
  }

  &:hover {
    border-color: color-mix(in srgb, var(--primary), 30%, transparent);
    box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid
      color-mix(in srgb, var(--dark-600), 50%, transparent);

    h2 {
      margin: 0;
      font-size: 1.25rem;
      font-weight: 600;
      color: var(--text);
      display: flex;
      align-items: center;
      gap: 0.5rem;
      position: relative;
      word-wrap: break-word;
    }
  }
}

.stats-section {
  grid-column: 1 / -1; /* Full width in the main container */

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 1.5rem;

    .stat-card {
      background: linear-gradient(
        135deg,
        #{"color-mix(in srgb, var(--primary), 3%, transparent)"} 0%,
        #{"color-mix(in srgb, var(--tertiary), 2%, transparent)"} 100%
      );
      border: 1px solid
        #{"color-mix(in srgb, var(--dark-600), 60%, transparent)"};
      border-radius: var(--border-radius);
      padding: 1.5rem;
      display: flex;
      align-items: center;
      gap: 1rem;
      transition: all 0.3s ease;
      position: relative;
      overflow: hidden;
      word-wrap: break-word;
      overflow-wrap: break-word;

      &::before {
        content: "";
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 2px;
        background: linear-gradient(90deg, var(--primary), var(--tertiary));
        transform: translateX(-100%);
        transition: transform 0.3s ease;
      }

      &:hover {
        border-color: color-mix(in srgb, var(--primary), 30%, transparent);
        transform: translateY(-2px);
        box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);

        &::before {
          transform: translateX(0);
        }
      }

      .stat-icon {
        width: 48px;
        height: 48px;
        border-radius: var(--border-radius);
        background: linear-gradient(
          135deg,
          #{"color-mix(in srgb, var(--primary), 15%, transparent)"},
          #{"color-mix(in srgb, var(--tertiary), 10%, transparent)"}
        );
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--primary);
        flex-shrink: 0;
        position: relative;

        &::after {
          content: "";
          position: absolute;
          inset: -1px;
          border-radius: var(--border-radius);
          background: linear-gradient(135deg, var(--primary), var(--tertiary));
          z-index: -1;
          opacity: 0.3;
          filter: blur(4px);
        }
      }

      .stat-content {
        flex: 1;
        word-wrap: break-word;
        overflow-wrap: break-word;

        h4 {
          margin: 0 0 0.25rem;
          font-size: 0.875rem;
          font-weight: 500;
          color: var(--placeholder);
          text-transform: uppercase;
          letter-spacing: 0.5px;
          word-wrap: break-word;
        }

        .stat-value {
          margin: 0;
          font-size: 1.25rem;
          font-weight: 600;
          color: var(--text);
          background: linear-gradient(135deg, var(--primary), var(--tertiary));
          -webkit-background-clip: text;
          -webkit-text-fill-color: transparent;
          background-clip: text;
          word-wrap: break-word;
          overflow-wrap: break-word;
          line-height: 1.3;
        }

        .stat-subtext {
          margin: 0.25rem 0 0 0;
          font-size: 0.75rem;
          font-weight: 400;
          color: var(--placeholder);
          opacity: 0.8;
        }
      }
    }
  }
}

.error-message {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 1rem;
  background: color-mix(in srgb, var(--red), 10%, transparent);
  border: 1px solid var(--red);
  border-radius: var(--border-radius);
  color: var(--red);
  margin-bottom: 1rem;
}

@media (max-width: 768px) {
  .profile-section {
    padding: 1rem;
  }

  .stats-grid {
    grid-template-columns: 1fr !important;
  }

  .top-row {
    grid-template-columns: 1fr !important;
  }
}
</style>
