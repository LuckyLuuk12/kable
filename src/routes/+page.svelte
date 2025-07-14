<script lang="ts">
  import { onMount } from 'svelte';
  import { SettingsManager, Icon } from '$lib';
  import { installations, isLoadingInstallations, installationsError, GameManager } from '$lib/game';
  import type { MinecraftInstallation } from '$lib/types';

  // State variables
  let lastPlayedInstallations: MinecraftInstallation[] = [];
  let error: string | null = null;
  let viewMode: 'grid' | 'list' = 'grid';

  // Subscribe to the installations store
  $: {
    lastPlayedInstallations = $installations
      .filter((installation: MinecraftInstallation) => installation.is_valid)
      .sort((a: MinecraftInstallation, b: MinecraftInstallation) => {
        const aTime = new Date(a.last_played || 0).getTime();
        const bTime = new Date(b.last_played || 0).getTime();
        return bTime - aTime;
      })
      .slice(0, 8); // Show up to 8 installations
  }

  // Subscribe to loading and error states
  $: isLoading = $isLoadingInstallations;
  $: if ($installationsError) {
    error = $installationsError;
  }

  // Initialize on component mount
  onMount(async () => {
    console.log('Home page mounted, initializing...');
    try {
      // GameManager should already be initialized by the layout, 
      // but trigger a refresh to ensure we have the latest data
      await GameManager.loadInstallations();
    } catch (err) {
      console.error('Error during initialization:', err);
      error = `Initialization failed: ${err}`;
    }
  });

  function toggleViewMode() {
    viewMode = viewMode === 'grid' ? 'list' : 'grid';
  }

  function handlePlay() {
    // TODO: Implement play functionality once auth is ready
    alert('Play functionality coming soon! Authentication system needed first.');
  }
</script>

<div class="home-page">
  <!-- Header Section - Fixed -->
  <div class="page-header">
    <h1>Welcome to Kable</h1>
    <p>Your Minecraft launcher for all installations</p>
  </div>

  <!-- Play Button Section - Fixed -->
  <div class="play-section">
    <button class="play-button" on:click={handlePlay} disabled={lastPlayedInstallations.length === 0}>
      <Icon name="play" size="lg" />
      <span>Play Minecraft</span>
    </button>
    {#if lastPlayedInstallations.length === 0}
      <p class="no-installations">No installations found. Add some Minecraft installations to get started!</p>
    {/if}
  </div>

  <!-- Last Played Section - Scrollable -->
  <div class="installations-section">
    <div class="section-header">
      <h2>Last Played Installations</h2>
      <div class="section-controls">
        <button class="view-toggle" on:click={toggleViewMode} title="Toggle view mode">
          <Icon name={viewMode === 'grid' ? 'list' : 'grid'} size="sm" />
        </button>
      </div>
    </div>

    {#if error}
      <div class="error-message">
        <Icon name="alert" size="sm" />
        {error}
      </div>
    {:else if isLoading}
      <div class="loading-state">
        <Icon name="refresh" size="md" />
        <span>Loading installations...</span>
      </div>
    {:else if lastPlayedInstallations.length > 0}
      <div class="installations-container">
        <div class="installations-{viewMode}">
          {#each lastPlayedInstallations as installation}
            <div class="installation-card" class:selected={false}>
              <div class="installation-header">
                <div class="installation-icon">
                  <Icon name={installation.mod_loader === 'vanilla' ? 'home' : 'mods'} size="md" />
                </div>
                <div class="installation-info">
                  <h3>{installation.name || `Minecraft ${installation.version}`}</h3>
                  <p class="installation-details">
                    {installation.version}
                    {#if installation.mod_loader !== 'vanilla'}
                      • {installation.mod_loader}
                      {#if installation.loader_version}
                        {installation.loader_version}
                      {/if}
                    {/if}
                  </p>
                </div>
              </div>
              
              <div class="installation-meta">
                {#if installation.last_played}
                  <span class="last-played">
                    Last played: {new Date(installation.last_played).toLocaleDateString()}
                  </span>
                {/if}
                <div class="installation-actions">
                  <button class="action-btn" title="Launch this installation">
                    <Icon name="play" size="sm" />
                  </button>
                  <button class="action-btn" title="More options">
                    <Icon name="more" size="sm" />
                  </button>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {:else}
      <div class="empty-state">
        <Icon name="folder" size="xl" />
        <h3>No installations found</h3>
        <p>Add some Minecraft installations to see them here</p>
        <button class="btn btn-primary">Add Installation</button>
      </div>
    {/if}
  </div>
</div>

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;

  .home-page {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .page-header {
    padding: 2rem 2rem 1rem;
    border-bottom: 1px solid $dark-600;
    background: $background;
    flex-shrink: 0;

    h1 {
      margin: 0 0 0.5rem;
      font-size: 2rem;
      font-weight: 600;
      color: $text;
    }

    p {
      margin: 0;
      color: $placeholder;
      font-size: 1rem;
    }
  }

  .play-section {
    padding: 2rem;
    text-align: center;
    border-bottom: 1px solid $dark-600;
    background: $background;
    flex-shrink: 0;

    .play-button {
      display: inline-flex;
      align-items: center;
      gap: 0.75rem;
      padding: 1rem 2rem;
      background: $primary;
      color: white;
      border: none;
      border-radius: $border-radius;
      font-size: 1.125rem;
      font-weight: 600;
      cursor: pointer;
      transition: all 0.2s ease;
      min-width: 200px;

      &:hover:not(:disabled) {
        background: $primary-600;
        transform: translateY(-1px);
      }

      &:disabled {
        background: $dark-600;
        color: $placeholder;
        cursor: not-allowed;
        transform: none;
      }
    }

    .no-installations {
      margin: 1rem 0 0;
      color: $placeholder;
      font-size: 0.875rem;
    }
  }

  .installations-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 0 2rem 2rem;

    .section-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 1.5rem 0 1rem;
      flex-shrink: 0;

      h2 {
        margin: 0;
        font-size: 1.25rem;
        font-weight: 600;
        color: $text;
      }

      .section-controls {
        display: flex;
        gap: 0.5rem;
      }

      .view-toggle {
        padding: 0.5rem;
        background: $container;
        border: 1px solid $dark-600;
        border-radius: $border-radius;
        color: $placeholder;
        cursor: pointer;
        transition: all 0.2s ease;

        &:hover {
          background: $button-hover;
          color: $text;
        }
      }
    }

    .installations-container {
      flex: 1;
      overflow-y: auto;
      margin: 0 -0.5rem;
      padding: 0 0.5rem;

      &::-webkit-scrollbar {
        width: 8px;
      }

      &::-webkit-scrollbar-track {
        background: $container;
      }

      &::-webkit-scrollbar-thumb {
        background: $dark-600;
        border-radius: 4px;
      }

      &::-webkit-scrollbar-thumb:hover {
        background: $placeholder;
      }
    }

    .installations-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
      gap: 1rem;
      padding-bottom: 1rem;
    }

    .installations-list {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      padding-bottom: 1rem;
    }

    .installation-card {
      background: $container;
      border: 1px solid $dark-600;
      border-radius: $border-radius;
      padding: 1rem;
      transition: all 0.2s ease;
      cursor: pointer;

      &:hover {
        background: $button-hover;
        border-color: $primary;
        transform: translateY(-1px);
      }

      &.selected {
        border-color: $primary;
        background: rgba($primary, 0.1);
      }

      .installation-header {
        display: flex;
        align-items: flex-start;
        gap: 0.75rem;
        margin-bottom: 0.75rem;

        .installation-icon {
          padding: 0.5rem;
          background: rgba($primary, 0.1);
          border-radius: $border-radius;
          color: $primary;
          flex-shrink: 0;
        }

        .installation-info {
          flex: 1;
          min-width: 0;

          h3 {
            margin: 0 0 0.25rem;
            font-size: 1rem;
            font-weight: 600;
            color: $text;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
          }

          .installation-details {
            margin: 0;
            font-size: 0.875rem;
            color: $placeholder;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
          }
        }
      }

      .installation-meta {
        display: flex;
        justify-content: space-between;
        align-items: center;

        .last-played {
          font-size: 0.75rem;
          color: $placeholder;
        }

        .installation-actions {
          display: flex;
          gap: 0.25rem;
        }

        .action-btn {
          padding: 0.375rem;
          background: transparent;
          border: 1px solid $dark-600;
          border-radius: $border-radius;
          color: $placeholder;
          cursor: pointer;
          transition: all 0.2s ease;

          &:hover {
            background: $primary;
            border-color: $primary;
            color: white;
          }
        }
      }
    }

    .installations-list .installation-card {
      .installation-header {
        margin-bottom: 0.5rem;
      }

      .installation-meta {
        align-items: flex-end;
      }
    }

    .error-message,
    .loading-state {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.5rem;
      padding: 3rem;
      color: $placeholder;
      font-size: 0.875rem;
    }

    .error-message {
      color: $red;
    }

    .empty-state {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      padding: 3rem;
      text-align: center;
      color: $placeholder;

      h3 {
        margin: 1rem 0 0.5rem;
        font-size: 1.125rem;
        font-weight: 600;
        color: $text;
      }

      p {
        margin: 0 0 1.5rem;
        font-size: 0.875rem;
      }

      .btn {
        padding: 0.75rem 1.5rem;
        background: $primary;
        color: white;
        border: none;
        border-radius: $border-radius;
        font-weight: 500;
        cursor: pointer;
        transition: background 0.2s ease;

        &:hover {
          background: $primary-600;
        }
      }
    }
  }

  // Responsive design
  @media (max-width: 768px) {
    .page-header,
    .play-section,
    .installations-section {
      padding-left: 1rem;
      padding-right: 1rem;
    }

    .installations-grid {
      grid-template-columns: 1fr;
    }

    .play-button {
      min-width: auto;
      width: 100%;
      max-width: 300px;
    }
  }
</style>
