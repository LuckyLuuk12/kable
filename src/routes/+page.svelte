<script lang="ts">
  import { onMount } from 'svelte';
  import { SettingsManager, Icon } from '$lib';
  import { installations, isLoadingInstallations, installationsError, GameManager } from '$lib/game';
  import { quickLaunchDefault, launchInstallation, prepareForLaunch, formatLaunchResult } from '$lib/launcher';
  import type { MinecraftInstallation } from '$lib/types';

  // State variables
  let lastPlayedInstallations: MinecraftInstallation[] = [];
  let error: string | null = null;
  let viewMode: 'grid' | 'list' = 'grid';
  let isLaunching = false;
  let launchStatus = '';

  // Subscribe to the installations store
  $: {
    console.log('Total installations:', $installations.length);
    console.log('Valid installations:', $installations.filter(i => i.is_valid).length);
    
    lastPlayedInstallations = $installations
      .filter((installation: MinecraftInstallation) => installation.is_valid)
      .sort((a: MinecraftInstallation, b: MinecraftInstallation) => {
        const aTime = new Date(a.last_played || 0).getTime();
        const bTime = new Date(b.last_played || 0).getTime();
        return bTime - aTime;
      })
      .slice(0, 8); // Show up to 8 installations
      
    console.log('Last played installations:', lastPlayedInstallations.length);
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

  async function handlePlay() {
    isLaunching = true;
    launchStatus = 'Preparing to launch...';
    let result;
    
    try {
      // Check if we're ready to launch
      const prep = await prepareForLaunch();
      if (!prep.ready) {
        launchStatus = prep.message;
        setTimeout(() => {
          launchStatus = '';
          isLaunching = false;
        }, 10000);
        return;
      }
      
      // Try to launch the most recent installation
      if (lastPlayedInstallations.length > 0) {
        console.log('Launching installation:', lastPlayedInstallations[0]);
        launchStatus = `Launching ${lastPlayedInstallations[0].name}...`;
        result = await launchInstallation(lastPlayedInstallations[0].id);
      } else {
        launchStatus = 'Launching default Minecraft...';
        result = await quickLaunchDefault();
      }
      
      launchStatus = formatLaunchResult(result);
      
      if (result.success) {
        // Refresh installations to update last played
        setTimeout(() => {
          GameManager.loadInstallations();
        }, 1000);
      }
      
    } catch (err) {
      console.error('Launch error:', err);
      launchStatus = `Launch failed: ${err}`;
    } finally {
      setTimeout(() => {
        launchStatus = '';
        isLaunching = false;
      }, result?.success ? 2000 : 5000);
    }
  }

  async function handleInstallationLaunch(installation: MinecraftInstallation) {
    const launchButton = event?.target as HTMLButtonElement;
    if (launchButton) {
      launchButton.disabled = true;
    }
    
    try {
      const prep = await prepareForLaunch();
      if (!prep.ready) {
        alert(prep.message);
        return;
      }
      
      const result = await launchInstallation(installation.id);
      
      if (result.success) {
        // Refresh installations to update last played
        setTimeout(() => {
          GameManager.loadInstallations();
        }, 1000);
      } else {
        alert(formatLaunchResult(result));
      }
    } catch (err) {
      console.error('Installation launch error:', err);
      alert(`Launch failed: ${err}`);
    } finally {
      if (launchButton) {
        launchButton.disabled = false;
      }
    }
  }
</script>

<div class="page-wrapper">
  <!-- Header -->
  <div class="header">
    <h1>Kable Launcher</h1>
    <p>Your Minecraft launcher for all installations</p>
  </div>

  <!-- Play Button Section - Fixed -->
  <div class="play-section">
    <button class="play-button" on:click={handlePlay} disabled={isLaunching || lastPlayedInstallations.length === 0}>
      <Icon name={isLaunching ? "refresh" : "play"} size="lg" />
      <span>{isLaunching ? 'Launching...' : 'Play Minecraft'}</span>
    </button>
    {#if lastPlayedInstallations.length === 0}
      <p class="no-installations">No installations found. Please check your Minecraft directory in settings.</p>
    {/if}
    {#if launchStatus}
      <p class="launch-status" class:error={launchStatus.includes('fail') || launchStatus.includes('error')}>
        {launchStatus}
      </p>
    {/if}
  </div>

  <!-- Last Played Section - Scrollable -->
  <div class="installations-section">
    {#if error}
      <div class="error-state">
        <Icon name="warning" size="md" />
        {error}
      </div>
    {:else if isLoading}
      <div class="loading-state">
        <Icon name="refresh" size="md" />
        <span>Loading installations...</span>
      </div>
    {:else if lastPlayedInstallations.length > 0}
      <div class="section-header">
        <h2>Last Played Installations</h2>
        <button class="view-toggle" on:click={toggleViewMode} title="Toggle view mode">
          <Icon name={viewMode === 'grid' ? 'list' : 'grid'} size="sm" />
        </button>
      </div>
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
                  <button class="action-btn" title="Launch this installation" on:click={() => handleInstallationLaunch(installation)}>
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
        <Icon name="home" size="lg" />
        <h2>No Installations Found</h2>
        <p>It looks like you don't have any Minecraft installations yet.</p>
        <button class="btn btn-primary">Add Installation</button>
      </div>
    {/if}
  </div>
</div>

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;
  .page-wrapper {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: $container;
    overflow: hidden;
  }

  .header {
    text-align: center;
    padding: 2rem 2rem 1rem;
    background: $container;
    border-bottom: 1px solid $dark-600;
    flex-shrink: 0;

    h1 {
      margin: 0 0 0.5rem 0;
      font-size: 2.5rem;
      font-weight: 700;
      background: linear-gradient(135deg, $primary, $tertiary);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      background-clip: text;
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
    background: $container;
    flex-shrink: 0;

    .play-button {
      display: inline-flex;
      align-items: center;
      gap: 0.75rem;
      padding: 1rem 2rem;
      background: $primary;
      color: white;
      border: none;
      border-radius: 12px;
      font-size: 1.1rem;
      font-weight: 600;
      cursor: pointer;
      transition: all 0.2s ease;
      min-width: 200px;

      &:hover:not(:disabled) {
        background: $primary-600;
        transform: translateY(-2px);
      }

      &:disabled {
        opacity: 0.6;
        cursor: not-allowed;
        transform: none;
      }
    }

    .no-installations {
      margin: 1rem 0 0;
      color: $placeholder;
      font-size: 0.875rem;
    }
    
    .launch-status {
      margin: 1rem 0 0;
      padding: 0.75rem 1rem;
      border-radius: 8px;
      font-size: 0.875rem;
      background: rgba($green, 0.1);
      color: $green;
      border: 1px solid rgba($green, 0.3);
      
      &.error {
        background: rgba($red, 0.1);
        color: $red;
        border-color: rgba($red, 0.3);
      }
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
        font-size: 1.5rem;
        font-weight: 600;
      }

      .view-toggle {
        padding: 0.5rem;
        background: $dark-600;
        border: none;
        border-radius: 6px;
        color: $text;
        cursor: pointer;
        transition: background 0.2s ease;

        &:hover {
          background: $dark-500;
        }
      }
    }

    .installations-container {
      flex: 1;
      overflow-y: auto;
      padding-right: 0.5rem;
    }

    .installations-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
      gap: 1rem;
    }

    .installations-list {
      display: flex;
      flex-direction: column;
      gap: 0.75rem;
    }

    .installation-card {
      background: $dark-700;
      border: 1px solid $dark-600;
      border-radius: 12px;
      padding: 1.5rem;
      transition: all 0.2s ease;
      cursor: pointer;

      &:hover {
        border-color: $primary;
        transform: translateY(-2px);
      }

      &.selected {
        border-color: $primary;
        background: rgba($primary, 0.05);
      }

      .installation-header {
        display: flex;
        align-items: flex-start;
        gap: 1rem;
        margin-bottom: 1rem;

        .installation-icon {
          flex-shrink: 0;
          width: 40px;
          height: 40px;
          background: $primary;
          border-radius: 8px;
          display: flex;
          align-items: center;
          justify-content: center;
          color: white;
        }

        .installation-info {
          flex: 1;
          min-width: 0;

          h3 {
            margin: 0 0 0.25rem 0;
            font-size: 1.1rem;
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
          gap: 0.5rem;

          .action-btn {
            padding: 0.5rem;
            background: $dark-600;
            border: none;
            border-radius: 6px;
            color: $text;
            cursor: pointer;
            transition: all 0.2s ease;

            &:hover {
              background: $primary;
              color: white;
            }
          }
        }
      }
    }
  }

  .error-state,
  .loading-state,
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    color: $placeholder;
    gap: 1rem;

    h2 {
      margin: 0;
      color: $text;
    }

    p {
      margin: 0;
    }

    .btn {
      margin-top: 1rem;
    }
  }

  .error-state {
    color: $red;
  }

  .loading-state {
    .icon {
      animation: spin 1s linear infinite;
    }
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  // Responsive design
  @media (max-width: 768px) {
    .header,
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
