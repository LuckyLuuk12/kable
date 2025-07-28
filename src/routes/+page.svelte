<script lang="ts">
  import { onMount } from 'svelte';
  import { Icon, installations, isLoadingInstallations, installationsError, type KableInstallation, Launcher, InstallationManager } from '$lib';
    import InstallationsList from '$lib/components/InstallationsList.svelte';


  // State variables
  let lastPlayedInstallations: KableInstallation[] = [];
  let error: string | null = null;
  let viewMode: 'grid' | 'list' = 'grid';
  let isLaunching = false;
  let launchStatus = '';
  let openDropdownId: string | null = null;

  // Subscribe to the installations store
  $: {
    console.log('Total installations:', $installations.length);
    
    lastPlayedInstallations = $installations
      .sort((a: KableInstallation, b: KableInstallation) => {
        const aTime = new Date(a.last_used || 0).getTime();
        const bTime = new Date(b.last_used || 0).getTime();
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
  onMount(() => {
    console.log('Home page mounted');
    // GameManager is already initialized by the layout with installations loaded

    // Add click outside handler for dropdown
    function handleClickOutside(event: MouseEvent) {
      const target = event.target as Element;
      if (!target.closest('.dropdown-container')) {
        openDropdownId = null;
      }
    }
    document.addEventListener('click', handleClickOutside);
    
    // Return cleanup function
    return () => {
      document.removeEventListener('click', handleClickOutside);
    };
  });

  function toggleViewMode() {
    viewMode = viewMode === 'grid' ? 'list' : 'grid';
  }

  function getModLoaderIcon(modLoader: string) {
    switch (modLoader) {
      case 'fabric': return 'fabric';
      case 'forge': return 'hammer';
      default: return 'cube';
    }
  }

  function toggleDropdown(installationId: string) {
    openDropdownId = openDropdownId === installationId ? null : installationId;
  }

  function closeDropdown() {
    openDropdownId = null;
  }

  async function handlePlay() {
    isLaunching = true;
    launchStatus = 'Preparing to launch...';
    let result;
    
    try {      
      // Try to launch the most recent installation
      if (lastPlayedInstallations.length > 0) {
        console.log('Launching installation:', lastPlayedInstallations[0]);
        launchStatus = `Launching ${lastPlayedInstallations[0].name}...`;
        // Select the installation and launch with GameManager
        InstallationManager.selectInstallation(lastPlayedInstallations[0]);
        // await InstallationManager.launchGame();
        result = { success: true };
      } else {
        launchStatus = 'Launching default Minecraft...';
        // Use Launcher for quick launch fallback
        result = await Launcher.launchLatest();
      }
      
      if (result.success) {         
        launchStatus = 'Launched Minecraft!';
        // Refresh installations to update last played
        setTimeout(() => {
          InstallationManager.loadInstallations();
        }, 1000);
      } else {
        launchStatus = `Launch failed: ${result.error || 'Unknown error'}`;
      }
      
    } catch (err) {
      console.error('Launch error:', err);
      launchStatus = `Launch failed: ${err}`;
    } finally {
      // Reset the button state quickly since Minecraft is now running independently
      setTimeout(() => {
        launchStatus = '';
        isLaunching = false;
      }, result?.success ? 2000 : 5000);
    }
  }

  async function handleInstallationLaunch(installation: KableInstallation) {
    const launchButton = event?.target as HTMLButtonElement;
    const originalText = launchButton?.textContent || '';
    
    if (launchButton) {
      launchButton.disabled = true;
      launchButton.textContent = 'Launching...';
    }
    
    try {
      // Select the installation and check if we can launch
      InstallationManager.selectInstallation(installation);
      // const { canLaunch, reason } = await InstallationManager.canLaunch();
      // if (!canLaunch) {
      //   alert(reason || 'Cannot launch');
      //   return;
      // }
      // TODO: make launchService work with selected installation
      // await InstallationManager.launchGame();
      
      if (launchButton) {
        launchButton.textContent = 'Launched!';
      }
      // Refresh installations to update last played
      setTimeout(() => {
        InstallationManager.loadInstallations();
      }, 1000);
    } catch (err) {
      console.error('Installation launch error:', err);
      alert(`Launch failed: ${err}`);
    } finally {
      // Reset button state after a short delay
      setTimeout(() => {
        if (launchButton) {
          launchButton.disabled = false;
          launchButton.textContent = originalText;
        }
      }, 2000);
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
      <Icon name={isLaunching ? "refresh" : "play"} size="md" forceType="svg" />
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
  <!-- <div class="installations-section">
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
                  <Icon name={getModLoaderIcon(installation.mod_loader)} size="md" />
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
                    <Icon name="play" size="sm" forceType="svg" />
                  </button>
                  <div class="dropdown-container">
                    <button 
                      class="action-btn dropdown-trigger" 
                      title="More options"
                      on:click={() => toggleDropdown(installation.id)}
                    >
                      <Icon name="more" size="sm" />
                    </button>
                    {#if openDropdownId === installation.id}
                      <div 
                        class="dropdown-menu"
                        role="menu"
                        tabindex="-1"
                        on:mouseleave={closeDropdown}
                      >
                        <button class="dropdown-item" role="menuitem" on:click={() => { /* Edit installation */ closeDropdown(); }}>
                          <Icon name="edit" size="sm" />
                          Edit
                        </button>
                        <button class="dropdown-item" role="menuitem" on:click={() => { /* Duplicate installation */ closeDropdown(); }}>
                          <Icon name="copy" size="sm" />
                          Duplicate
                        </button>
                        <button class="dropdown-item danger" role="menuitem" on:click={() => { /* Delete installation */ closeDropdown(); }}>
                          <Icon name="trash" size="sm" />
                          Delete
                        </button>
                      </div>
                    {/if}
                  </div>
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
  </div> -->
  
  <div class="section-header">
    <h2>Last Played Installations</h2>
  </div>
  <InstallationsList isGrid isSmall limit={12}/>
</div>

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;
  .page-wrapper {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: $container;
    overflow: hidden;
    border-radius: $border-radius;
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

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem 2rem 0 2rem;
    flex-shrink: 0;

    h2 {
      margin: 0;
      font-size: 1.5rem;
      font-weight: 600;
    }
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  @keyframes dropdownSlide {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  // Responsive design
  @media (max-width: 768px) {
    .header,
    .play-section {
      padding-left: 1rem;
      padding-right: 1rem;
    }

    .play-button {
      min-width: auto;
      width: 100%;
      max-width: 300px;
    }
  }
</style>
