<script lang="ts">
  import { Icon, InstallationService, installations, isLoadingInstallations, isLoadingVersions, Launcher } from '$lib';
  import { onMount, onDestroy } from 'svelte';
  
  export let isGrid: boolean = false;
  export let isSmall: boolean = false;
  
  export let error: string | null = null;
  export let limit: number | null = null;

  $: isLoading = $isLoadingInstallations || $isLoadingVersions

  $: limitedInstallations = $installations
    .slice()
    .sort((a, b) => {
      // Favorites first
      if ((a.favorite ? 1 : 0) !== (b.favorite ? 1 : 0)) {
        return (b.favorite ? 1 : 0) - (a.favorite ? 1 : 0);
      }
      // Then by last_used (most recent first)
      const aTime = a.last_used ? new Date(a.last_used).getTime() : 0;
      const bTime = b.last_used ? new Date(b.last_used).getTime() : 0;
      return bTime - aTime;
    })
    .slice(0, limit || $installations.length);
  $: loaderIcons = Object.fromEntries(
    $installations.map(installation => [
      installation.id,
      InstallationService.getLoaderIcon(InstallationService.getVersionData(installation).loader)
    ])
  );
  $: loaderColors = Object.fromEntries(
    $installations.map(installation => [
      installation.id,
      InstallationService.getLoaderColor(InstallationService.getVersionData(installation).loader)
    ])
  );

  // Dynamic action display logic
  let useDropdownForActions: { [key: string]: boolean } = {};
  let resizeObserver: ResizeObserver | null = null;

  function checkActionsFit() {
    if (isGrid) return; // Only applies to list view

    const newUseDropdown: { [key: string]: boolean } = {};

    // If compact mode is enabled, force all to dropdown
    if (isSmall) {
      limitedInstallations.forEach(installation => {
        newUseDropdown[installation.id] = true;
      });
      useDropdownForActions = newUseDropdown;
      return;
    }

    // If compact mode is disabled, show inline actions by default
    // Only use dropdown if space is really tight
    limitedInstallations.forEach(installation => {
      newUseDropdown[installation.id] = false; // Default to inline actions
    });

    const listItems = document.querySelectorAll('.list-title-actions-row');

    listItems.forEach((item, index) => {
      const installation = limitedInstallations[index];
      if (!installation) return;

      const titleElement = item.querySelector('h3') as HTMLElement;
      const actionsSection = item.querySelector('.list-actions-section') as HTMLElement;
      const inlineActions = item.querySelector('.list-inline-actions') as HTMLElement;
      
      if (!titleElement || !actionsSection || !inlineActions) return;

      const containerWidth = (item as HTMLElement).offsetWidth;
      const titleWidth = titleElement.offsetWidth;
      const actionsSectionWidth = actionsSection.offsetWidth;
      
      // Check if there's enough space (with some padding)
      const availableSpace = containerWidth - titleWidth - 32; // 32px for gaps and padding
      const neededSpace = actionsSectionWidth;
      
      newUseDropdown[installation.id] = neededSpace > availableSpace || containerWidth < 800;
    });

    useDropdownForActions = newUseDropdown;
  }

  function handleResize() {
    checkActionsFit();
  }

  onMount(() => {
    // Initial check
    setTimeout(checkActionsFit, 100);

    // Set up resize observer for more accurate detection
    if (typeof ResizeObserver !== 'undefined') {
      resizeObserver = new ResizeObserver(() => {
        checkActionsFit();
      });

      const container = document.querySelector('.installations-list');
      if (container) {
        resizeObserver.observe(container);
      }
    }

    // Fallback to window resize
    window.addEventListener('resize', handleResize);
  });

  onDestroy(() => {
    if (resizeObserver) {
      resizeObserver.disconnect();
    }
    window.removeEventListener('resize', handleResize);
  });

  // Re-check when installations change or isSmall prop changes
  $: {
    if (limitedInstallations.length >= 0) {
      setTimeout(checkActionsFit, 100);
    }
  }

  // Also re-check when isSmall changes specifically
  $: {
    if (typeof isSmall === 'boolean') {
      // Immediate check for isSmall changes
      if (isSmall) {
        // Force all to dropdown when compact mode is enabled
        const newUseDropdown: { [key: string]: boolean } = {};
        limitedInstallations.forEach(installation => {
          newUseDropdown[installation.id] = true;
        });
        useDropdownForActions = newUseDropdown;
      } else {
        // Reset to fit-based detection when compact mode is disabled
        setTimeout(checkActionsFit, 10);
      }
    }
  }

</script>

<div class="installations-list" class:compact={isSmall && !isGrid}>
  {#if error}
    <div class="error-message">
      <Icon name="alert" size="sm" />
      {error}
    </div>
  {/if}

  {#if isLoading && limitedInstallations.length === 0}
    <div class="loading-state">
      <Icon name="refresh" size="md" forceType="svg"/>
      <span>Loading installations...</span>
    </div>
  {:else if limitedInstallations.length === 0}
    <div class="empty-state">
      <div class="empty-icon">
        <Icon name="cube" size="xl" />
      </div>
      <h3>No installations found</h3>
      <p>Create your first Minecraft installation to get started</p>
    </div>
  {:else}
    <div class={isGrid ? 'installations-grid' : 'installations-flex'}>
      {#each limitedInstallations as installation, i (installation.id)}
        {#if isGrid}
          <!-- Grid Layout - existing card design -->
          <div class={isSmall ? 'installation-card small' : 'installation-card'} style="background: linear-gradient(135deg, {loaderColors[installation.id]}22 0%, {loaderColors[installation.id]}08 40%); --loader-color: {loaderColors[installation.id]}55; z-index: {(limitedInstallations.length - i) * 2}; position: relative;">
            <div class="card-top-actions">
              <button class="star-btn" title={installation.favorite ? "Unfavorite" : "Favorite"} on:click={async (e) => { e.stopPropagation(); await InstallationService.toggleFavorite(installation); }}>
                <Icon name="star" forceType={installation.favorite ? 'emoji' : 'svg'} size="md" />
              </button>
              {#if isSmall}
                <div class="dropdown installation-dropdown actions-dropdown small-actions-dropdown">
                  <button class="btn btn-secondary dropdown-toggle">
                    <Icon name="more-horizontal" size="sm" />
                  </button>
                  <div class="dropdown-menu" style="z-index: {(limitedInstallations.length - i) * 2 - 1};">
                    <button on:click={async () => await InstallationService.updateInstallation(installation.id, installation)}>
                      <Icon name="edit" size="sm" />
                      Edit
                    </button>
                    <button on:click={async () => await InstallationService.createInstallation(installation.version_id)}>
                      <Icon name="duplicate" size="sm" />
                      Duplicate
                    </button>
                    <button on:click={async () => {/* TODO: implement export logic */}}>
                      <Icon name="download" size="sm" />
                      Export
                    </button>
                    <div class="dropdown-separator"></div>
                    <button 
                      class="danger" 
                      on:click={async () => await InstallationService.deleteInstallation(installation.id)}
                    >
                      <Icon name="trash" size="sm" />
                      Delete
                    </button>
                  </div>
                </div>
              {/if}
            </div>
            <div class="installation-main">
              <div class="installation-icon-column">
                <div class="installation-icon icon-tooltip-wrapper" style="color: {loaderColors[installation.id]}; background: rgba(0,0,0,0.0);">
                  <Icon name={loaderIcons[installation.id]} size="lg" />
                  <span class="icon-tooltip">{InstallationService.getVersionData(installation).loader}</span>
                </div>
                <button 
                  class="btn btn-primary play-below-icon" 
                  style="background: linear-gradient(90deg, {loaderColors[installation.id] || 'var(--loader-primary)'} 60%, {loaderColors[installation.id] ? `${loaderColors[installation.id]}cc` : 'var(--loader-secondary)'} 100%); color: var(--text-white) !important;"
                  on:click={async () => { await Launcher.launchInstallation(installation); }}
                  disabled={isLoading}
                >
                  Play
                </button>
              </div>
              <div class="installation-meta">
                <div class="installation-title-row">
                  <h3>{installation.name || installation.version_id}</h3>
                </div>
                {#if installation.version_id}
                  <div class="loader-version-row">
                    <span class="loader-version" style="color: {loaderColors[installation.id]};">{installation.version_id}</span>
                  </div>
                {/if}
                {#if isSmall}
                  <div class="installation-meta-grid small-meta-grid">
                    <div class="meta-cell small-meta-cell">
                      <span class="meta-key">Total time:</span>
                      <span class="meta-value last-played small-meta-value"><Icon name="clock" size="sm" /> {installation.total_time_played_ms ? new Date(installation.total_time_played_ms).toLocaleDateString() : 'Unknown'}</span>
                    </div>
                  </div>
                {:else}
                  <div class="installation-meta-grid">
                    <div class="meta-cell">
                      <span class="meta-key">Created:</span>
                      <span class="meta-value created-date"><Icon name="calendar" size="sm" /> {installation.created ? new Date(installation.created).toLocaleDateString() : 'Unknown'}</span>
                    </div>
                    <div class="meta-cell">
                      <span class="meta-key">Last played:</span>
                      <span class="meta-value last-played"><Icon name="clock" size="sm" /> {installation.last_used ? new Date(installation.last_used).toLocaleDateString() : 'Never'}</span>
                    </div>
                    <div class="meta-cell">
                      <span class="meta-key">Total time:</span>
                      <span class="meta-value total-time"><Icon name="clock" size="sm" /> {installation.total_time_played_ms ? new Date(installation.total_time_played_ms).toLocaleDateString() : '---'}</span>
                    </div>
                  </div>
                {/if}
              </div>
            </div>

            {#if !isSmall}
              <div class="installation-actions">
                <button class="btn btn-secondary" on:click={async () => await InstallationService.updateInstallation(installation.id, installation)}>
                  <Icon name="edit" size="sm" />
                  Edit
                </button>
                <button class="btn btn-secondary" on:click={async () => await InstallationService.createInstallation(installation.version_id)}>
                  <Icon name="duplicate" size="sm" />
                  Duplicate
                </button>
                <button class="btn btn-secondary" on:click={async () => {/* TODO: implement export logic */}}>
                  <Icon name="download" size="sm" />
                  Export
                </button>
                <button class="btn btn-danger" on:click={async () => await InstallationService.deleteInstallation(installation.id)}>
                  <Icon name="trash" size="sm" />
                  Delete
                </button>
              </div>
            {/if}
          </div>
        {:else}
          <!-- List Layout - horizontal compact design -->
          <div class="installation-list-item" class:dropdown-active={useDropdownForActions[installation.id]} style="background: linear-gradient(135deg, {loaderColors[installation.id]}15 0%, {loaderColors[installation.id]}05 40%); --loader-color: {loaderColors[installation.id]}55;">
            <div class="list-item-main">
              <!-- Icon and Play Button -->
              <div class="list-item-icon-section">
                <div class="installation-icon icon-tooltip-wrapper" style="color: {loaderColors[installation.id]};">
                  <Icon name={loaderIcons[installation.id]} size="md" />
                  <span class="icon-tooltip">{InstallationService.getVersionData(installation).loader}</span>
                </div>
                <button 
                  class="btn btn-primary list-play-btn" 
                  style="background: linear-gradient(90deg, {loaderColors[installation.id] || 'var(--loader-primary)'} 60%, {loaderColors[installation.id] ? `${loaderColors[installation.id]}cc` : 'var(--loader-secondary)'} 100%); color: var(--text-white) !important;"
                  on:click={async () => { await Launcher.launchInstallation(installation); }}
                  disabled={isLoading}
                >
                  Play
                </button>
              </div>

              <!-- Title and Actions Section -->
              <div class="list-item-content">
                <!-- Title Row with Actions -->
                <div class="list-title-actions-row">
                  <h3>{installation.name || installation.version_id}</h3>
                  <div class="list-actions-section">
                    <button class="star-btn" title={installation.favorite ? "Unfavorite" : "Favorite"} on:click={async (e) => { e.stopPropagation(); await InstallationService.toggleFavorite(installation); }}>
                      <Icon name="star" forceType={installation.favorite ? 'emoji' : 'svg'} size="sm" />
                    </button>
                    
                    <!-- Inline Actions (shown when they fit) -->
                    <div class="list-inline-actions" class:hidden={useDropdownForActions[installation.id]}>
                      <button class="list-action-btn" on:click={async () => await InstallationService.updateInstallation(installation.id, installation)}>
                        <Icon name="edit" size="sm" />
                        Edit
                      </button>
                      <button class="list-action-btn" on:click={async () => await InstallationService.createInstallation(installation.version_id)}>
                        <Icon name="duplicate" size="sm" />
                        Duplicate
                      </button>
                      <button class="list-action-btn" on:click={async () => {/* TODO: implement export logic */}}>
                        <Icon name="download" size="sm" />
                        Export
                      </button>
                      <button class="list-action-btn danger" on:click={async () => await InstallationService.deleteInstallation(installation.id)}>
                        <Icon name="trash" size="sm" />
                        Delete
                      </button>
                    </div>
                    
                    <!-- Dropdown (shown when actions don't fit) -->
                    <div class="dropdown list-dropdown" class:visible={useDropdownForActions[installation.id]}>
                      <button class="btn btn-secondary dropdown-toggle">
                        <Icon name="more-horizontal" size="sm" />
                      </button>
                      <div class="dropdown-menu">
                        <button on:click={async () => await InstallationService.updateInstallation(installation.id, installation)}>
                          <Icon name="edit" size="sm" />
                          Edit
                        </button>
                        <button on:click={async () => await InstallationService.createInstallation(installation.version_id)}>
                          <Icon name="duplicate" size="sm" />
                          Duplicate
                        </button>
                        <button on:click={async () => {/* TODO: implement export logic */}}>
                          <Icon name="download" size="sm" />
                          Export
                        </button>
                        <div class="dropdown-separator"></div>
                        <button 
                          class="danger" 
                          on:click={async () => await InstallationService.deleteInstallation(installation.id)}
                        >
                          <Icon name="trash" size="sm" />
                          Delete
                        </button>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- Version and Stats Row -->
                <div class="list-version-stats-row">
                  {#if installation.version_id && installation.name}
                    <span class="list-version" style="color: {loaderColors[installation.id]};">{installation.version_id}</span>
                  {/if}
                  <div class="list-stats-section">
                    <div class="list-meta-item">
                      <Icon name="calendar" size="sm" />
                      <span>{installation.created ? new Date(installation.created).toLocaleDateString() : 'Unknown'}</span>
                    </div>
                    <div class="list-meta-item">
                      <Icon name="clock" size="sm" />
                      <span>{installation.last_used ? new Date(installation.last_used).toLocaleDateString() : 'Never'}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style lang="scss">
  @use "@kablan/clean-ui/scss/_variables.scss" as *;
  .installations-list {
    padding: 2rem;
    border-radius: var(--border-radius);
    border: 1px solid color-mix(in srgb, var(--dark-400), 3%, transparent);
    background:
      radial-gradient(circle at var(--dot1-x, 30%) var(--dot1-y, 40%), #{'color-mix(in srgb, var(--primary-900), 4.5%, transparent)'} 0%, transparent 18%),
      radial-gradient(circle at var(--dot2-x, 70%) var(--dot2-y, 60%), #{'color-mix(in srgb, var(--secondary), 3.5%, transparent)'} 0%, transparent 15%),
      radial-gradient(circle at var(--dot3-x, 60%) var(--dot3-y, 20%), #{'color-mix(in srgb, var(--tertiary), 3%, transparent)'} 0%, transparent 13%),
      radial-gradient(circle at var(--dot4-x, 80%) var(--dot4-y, 80%), #{'color-mix(in srgb, var(--quaternary), 3.5%, transparent)'} 0%, transparent 16%),
      linear-gradient(120deg, #{'color-mix(in srgb, var(--container), 98%, transparent)'} 60%, #{'color-mix(in srgb, var(--primary), 4%, transparent)'} 100%);
    box-shadow: 0 0.125rem 0.25rem rgba(0, 0, 0, 0.08); // 2px 4px
    overflow: scroll;
    animation: move-dots 32s ease infinite alternate;
  }

  .installations-list.compact {
    padding: 1rem;
    
    .installation-list-item {
      margin-bottom: 0.5rem;
      
      .list-item-main {
        padding: 0.75rem;
        gap: 0.5rem;
      }
      
      .list-item-content {
        h3 {
          font-size: 0.9rem;
          margin-bottom: 0.25rem;
        }
        
        .list-version-stats-row {
          font-size: 0.8rem;
          opacity: 0.8;
          
          .list-meta-item {
            font-size: 0.8rem;
          }
        }
      }
    }
  }

  @keyframes move-dots {
    0% {
      --dot1-x: 18%; --dot1-y: 18%;
      --dot2-x: 68%; --dot2-y: 58%;
      --dot3-x: 12%; --dot3-y: 8%;
      --dot4-x: 78%; --dot4-y: 78%;
      --dot5-x: 22%; --dot5-y: 68%;
      --dot6-x: 28%; --dot6-y: 12%;
      --dot7-x: 10%; --dot7-y: 80%;
      --dot8-x: 85%; --dot8-y: 30%;
      --dot9-x: 40%; --dot9-y: 85%;
      --dot10-x: 75%; --dot10-y: 15%;
    }
    10% {
      --dot1-x: 20%; --dot1-y: 20%;
      --dot2-x: 66%; --dot2-y: 60%;
      --dot3-x: 14%; --dot3-y: 10%;
      --dot4-x: 76%; --dot4-y: 76%;
      --dot5-x: 24%; --dot5-y: 70%;
      --dot6-x: 30%; --dot6-y: 14%;
      --dot7-x: 12%; --dot7-y: 82%;
      --dot8-x: 87%; --dot8-y: 32%;
      --dot9-x: 42%; --dot9-y: 87%;
      --dot10-x: 77%; --dot10-y: 17%;
    }
    20% {
      --dot1-x: 22%; --dot1-y: 22%;
      --dot2-x: 64%; --dot2-y: 62%;
      --dot3-x: 16%; --dot3-y: 12%;
      --dot4-x: 74%; --dot4-y: 74%;
      --dot5-x: 26%; --dot5-y: 72%;
      --dot6-x: 32%; --dot6-y: 16%;
      --dot7-x: 14%; --dot7-y: 84%;
      --dot8-x: 89%; --dot8-y: 34%;
      --dot9-x: 44%; --dot9-y: 89%;
      --dot10-x: 79%; --dot10-y: 19%;
    }
    30% {
      --dot1-x: 24%; --dot1-y: 24%;
      --dot2-x: 62%; --dot2-y: 64%;
      --dot3-x: 18%; --dot3-y: 14%;
      --dot4-x: 72%; --dot4-y: 72%;
      --dot5-x: 28%; --dot5-y: 74%;
      --dot6-x: 34%; --dot6-y: 18%;
      --dot7-x: 16%; --dot7-y: 86%;
      --dot8-x: 91%; --dot8-y: 36%;
      --dot9-x: 46%; --dot9-y: 91%;
      --dot10-x: 81%; --dot10-y: 21%;
    }
    40% {
      --dot1-x: 26%; --dot1-y: 26%;
      --dot2-x: 60%; --dot2-y: 66%;
      --dot3-x: 20%; --dot3-y: 16%;
      --dot4-x: 70%; --dot4-y: 70%;
      --dot5-x: 30%; --dot5-y: 76%;
      --dot6-x: 36%; --dot6-y: 20%;
      --dot7-x: 18%; --dot7-y: 88%;
      --dot8-x: 93%; --dot8-y: 38%;
      --dot9-x: 48%; --dot9-y: 93%;
      --dot10-x: 83%; --dot10-y: 23%;
    }
    50% {
      --dot1-x: 24%; --dot1-y: 24%;
      --dot2-x: 62%; --dot2-y: 64%;
      --dot3-x: 18%; --dot3-y: 14%;
      --dot4-x: 72%; --dot4-y: 72%;
      --dot5-x: 28%; --dot5-y: 74%;
      --dot6-x: 34%; --dot6-y: 18%;
      --dot7-x: 16%; --dot7-y: 86%;
      --dot8-x: 91%; --dot8-y: 36%;
      --dot9-x: 46%; --dot9-y: 91%;
      --dot10-x: 81%; --dot10-y: 21%;
    }
    60% {
      --dot1-x: 22%; --dot1-y: 22%;
      --dot2-x: 64%; --dot2-y: 62%;
      --dot3-x: 16%; --dot3-y: 12%;
      --dot4-x: 74%; --dot4-y: 74%;
      --dot5-x: 26%; --dot5-y: 72%;
      --dot6-x: 32%; --dot6-y: 16%;
      --dot7-x: 14%; --dot7-y: 84%;
      --dot8-x: 89%; --dot8-y: 34%;
      --dot9-x: 44%; --dot9-y: 89%;
      --dot10-x: 79%; --dot10-y: 19%;
    }
    70% {
      --dot1-x: 20%; --dot1-y: 20%;
      --dot2-x: 66%; --dot2-y: 60%;
      --dot3-x: 14%; --dot3-y: 10%;
      --dot4-x: 76%; --dot4-y: 76%;
      --dot5-x: 24%; --dot5-y: 70%;
      --dot6-x: 30%; --dot6-y: 14%;
      --dot7-x: 12%; --dot7-y: 82%;
      --dot8-x: 87%; --dot8-y: 32%;
      --dot9-x: 42%; --dot9-y: 87%;
      --dot10-x: 77%; --dot10-y: 17%;
    }
    80% {
      --dot1-x: 18%; --dot1-y: 18%;
      --dot2-x: 68%; --dot2-y: 58%;
      --dot3-x: 12%; --dot3-y: 8%;
      --dot4-x: 78%; --dot4-y: 78%;
      --dot5-x: 22%; --dot5-y: 68%;
      --dot6-x: 28%; --dot6-y: 12%;
      --dot7-x: 10%; --dot7-y: 80%;
      --dot8-x: 85%; --dot8-y: 30%;
      --dot9-x: 40%; --dot9-y: 85%;
      --dot10-x: 75%; --dot10-y: 15%;
    }
    100% {
      --dot1-x: 32%; --dot1-y: 42%;
      --dot2-x: 68%; --dot2-y: 58%;
      --dot3-x: 58%; --dot3-y: 22%;
      --dot4-x: 78%; --dot4-y: 78%;
      --dot5-x: 22%; --dot5-y: 68%;
      --dot6-x: 50%; --dot6-y: 10%;
      --dot7-x: 10%; --dot7-y: 80%;
      --dot8-x: 85%; --dot8-y: 30%;
      --dot9-x: 40%; --dot9-y: 85%;
      --dot10-x: 75%; --dot10-y: 15%;
    }
  }
  .installations-grid {
    display: grid;
    gap: 1.25rem;
    grid-template-columns: repeat(auto-fill, minmax(20.5rem, 1fr));
  }
  .installations-flex {
    display: flex;
    gap: 0.75rem;
    flex-direction: column;
  }

  /* List Layout Styles */
  .installation-list-item {
    background: var(--card);
    border-radius: var(--border-radius);
    box-shadow: 0 0.125rem 0.5rem rgba(80,80,90,0.06), 0 0.0625rem 0.125rem rgba(80,80,90,0.04);
    border: 0.0625rem solid transparent;
    transition: all 0.15s, border 0.15s;
    position: relative;
    backdrop-filter: blur(0.5rem);
    -webkit-backdrop-filter: blur(0.5rem);
    cursor: pointer;
    z-index: 1;
    
    &.dropdown-active {
      z-index: 3001; // Higher than dropdown menu z-index
    }
    
    &:hover {
      box-shadow: 0 0.25rem 1rem rgba(80,80,90,0.1), 0 0.125rem 0.25rem rgba(80,80,90,0.06);
      border-color: var(--loader-color, var(--primary));
    }
  }

  .list-item-main {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 1rem;
    background: color-mix(in srgb, var(--red), 10%, transparent);
  }

  .list-item-icon-section {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-shrink: 0;
    
    .installation-icon {
      width: 2.25rem;
      height: 2.25rem;
      border-radius: 0.5rem;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 1.25rem;
      background: var(--container);
      box-shadow: 0 0.0625rem 0.25rem rgba(0,0,0,0.04);
      position: relative;
    }
  }

  .list-play-btn {
    font-size: 0.875rem;
    font-weight: 600;
    border: none;
    border-radius: 0.5rem;
    box-shadow: 0 0.125rem 0.5rem rgba(80,80,90,0.08);
    transition: all 0.15s;
    letter-spacing: 0.02em;
    color: var(--text-white) !important;
    padding: 0.5rem 1rem;
    
    &:hover, &:focus {
      filter: brightness(1.1) saturate(1.15);
      box-shadow: 0 0.25rem 1rem rgba(80,80,90,0.15);
      transform: translateY(-1px);
      outline: none;
    }
  }

  .list-item-content {
    flex: 1 1 0%;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .list-title-actions-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    
    h3 {
      margin: 0;
      font-size: 1rem;
      font-weight: 700;
      color: var(--text);
      text-overflow: ellipsis;
      overflow: hidden;
      white-space: nowrap;
      flex: 1 1 0%;
      min-width: 0;
    }
  }

  .list-actions-section {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
    
    .star-btn {
      background: none;
      border: none;
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      cursor: pointer;
      &:hover {
        transform: scale(1.3);
      }
    }
  }

  .list-inline-actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    transition: opacity 0.2s ease;
    
    &.hidden {
      display: none;
    }
    
    .list-action-btn {
      background: none;
      border: none;
      padding: 0.375rem 0.75rem;
      border-radius: 0.375rem;
      cursor: pointer;
      color: var(--placeholder);
      font-size: 0.8rem;
      font-weight: 500;
      transition: all 0.15s;
      display: flex;
      align-items: center;
      gap: 0.25rem;
      white-space: nowrap;
      
      &:hover, &:focus {
        color: var(--text);
  background: color-mix(in srgb, var(--dark-200), 8%, transparent);
        outline: none;
      }
      
      &.danger {
        color: var(--red-700);
        
        &:hover, &:focus {
          background: color-mix(in srgb, var(--red-700), 8%, transparent);
        }
      }
    }
  }

  .list-dropdown {
    display: none;
    
    &.visible {
      display: block;
    }
  }

  .list-version-stats-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    
    .list-version {
      font-size: 0.75rem;
      font-weight: 500;
      background: var(--container);
      border-radius: 0.375rem;
      padding: 0.125rem 0.5rem;
      opacity: 0.8;
      flex-shrink: 0;
    }
  }

  .list-stats-section {
    display: flex;
    gap: 1.5rem;
    align-items: center;
    
    .list-meta-item {
      display: flex;
      align-items: center;
      gap: 0.375rem;
      font-size: 0.8rem;
      color: var(--placeholder);
      white-space: nowrap;
      
      span {
        font-weight: 500;
      }
    }
  }

  .installation-card {
    position: relative;
    .card-top-actions {
      position: absolute;
      top: 0.5rem;
      right: 0.5rem;
      display: flex;
      flex-direction: row;
      gap: 0.25rem;
      z-index: 10;
      align-items: center;
      height: 2rem;
    }
    .star-btn {
      background: none;
      border: none;
      border-radius: 50%;
      width: 2rem;
      height: 2rem;
      display: flex;
      align-items: center;
      justify-content: center;
      box-shadow: none;
      cursor: pointer;
      transition: background 0.15s;
      padding: 0;
      &:hover, &:focus {
        background: none;
        outline: none;
      }
    }
    background: var(--card);
    border-radius: var(--border-radius);
    box-shadow: 0 0.125rem 0.75rem rgba(80,80,90,0.07), 0 0.09375rem 0.25rem rgba(80,80,90,0.04); // 2px 12px, 1.5px 4px
    border: 0.0625rem solid transparent; // 1px
    display: flex;
    flex-direction: column;
    padding: 1rem 0.75rem 0.5rem 0.75rem;
    transition: all 0.15s, border 0.15s;
    position: relative;
    min-width: 0;
    backdrop-filter: blur(0.5rem); // 8px
    -webkit-backdrop-filter: blur(0.5rem);
    cursor: pointer;
    z-index: 1;
    * {
      cursor: pointer !important;
    }
    &:hover {
      box-shadow: 0 0.375rem 1.5rem rgba(80,80,90,0.13), 0 0.125rem 0.5rem rgba(80,80,90,0.07); // 6px 24px, 2px 8px
      border-color: var(--loader-color, var(--primary));
    }
    &.small {
      padding: 0.6rem 0.6rem 0.6rem 0.6rem;
      min-height: 0.625rem; // 10px
    }
    .installation-dropdown.actions-dropdown {
      position: absolute;
      bottom: 1.25rem;
      right: 1.25rem;
      z-index: 2;
    }
    &.small .installation-dropdown.actions-dropdown {
      margin-right: 0.5rem;
      bottom: auto;
      left: auto;
    }
    .small-actions-dropdown {
      left: auto;
      bottom: auto;
      position: absolute;
      z-index: 3;
    }
  }
  .installation-actions {
    display: flex;
    align-items: center;
    justify-content: flex-start;
    margin-top: 1rem;
    min-height: 2.5rem;
    gap: 0.5rem;
    position: relative;
    button {
      background: none !important;
      border: none;
      box-shadow: none;
      color: var(--primary);
      font-weight: 600;
      transition: color 0.13s;
      padding: 0;
      color: var(--placeholder);
      &:hover, &:focus {
        color: var(--text);
      }
      &.btn-danger, &.btn.btn-danger, &.btn-danger:focus, &.btn-danger:hover {
        color: var(--red-700);
      }
    }
    .btn-danger, .btn.btn-danger {
      color: var(--red-700);
    }
  }
  .installation-main {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }
  .installation-icon-column {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    min-width: 40px;
  }
  .installation-icon {
    width: 2.5rem; // 40px
    height: 2.5rem;
    border-radius: 0.625rem; // 10px
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.5rem;
    background: var(--container);
    box-shadow: 0 0.0625rem 0.25rem rgba(0,0,0,0.04); // 1px 4px
    flex-shrink: 0;
    position: relative;
    cursor: pointer;
  }
  .play-below-icon {
    margin-top: 0.5rem;
    width: 100%;
    min-width: 2.5rem; // 40px
    max-width: 5rem;   // 80px
    align-self: center;
    font-weight: 700;
    border: none;
    border-radius: 0.7em;
    box-shadow: 0 0.125rem 0.75rem rgba(80,80,90,0.10); // 2px 12px
    transition: background 0.15s, box-shadow 0.15s, filter 0.15s;
    letter-spacing: 0.02em;
    color: var(--text-white) !important;
    &:hover, &:focus {
      filter: brightness(1.13) saturate(1.18);
      box-shadow: 0 0.375rem 2rem 0 rgba(80,80,90,0.18), 0 0 0 0.125rem rgba(0,0,0,0.08); // 6px 32px 2px
      opacity: 1;
      transform: scale(1.045);
      outline: none;
      z-index: 2;
    }
  }
  .icon-tooltip-wrapper:hover .icon-tooltip,
  .icon-tooltip-wrapper:focus-within .icon-tooltip {
    opacity: 1;
    pointer-events: auto;
    transform: none;
  }
  .icon-tooltip {
    opacity: 0;
    pointer-events: none;
    position: absolute;
    left: -0.75rem;
    top: -1rem;
    background: var(--container);
    color: var(--text);
    border: 0.0625rem solid var(--dark-200); // 1px
    border-radius: 0.22em;
    padding: 0.03em 0.28em;
    font-size: 0.7em;
    font-weight: 500;
    white-space: nowrap;
    box-shadow: 0 0.0625rem 0.125rem rgba(0,0,0,0.06); // 1px 2px
    z-index: 10;
    transition: opacity 0.13s;
    margin: 0;
  }
  .installation-meta {
    flex: 1 1 0%;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }
    .installation-title-row {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      h3 {
        margin: 0;
        font-size: 1.18rem;
        font-weight: 800;
        color: var(--text);
        text-overflow: ellipsis;
        overflow: hidden;
        white-space: nowrap;
        max-width: 80%;
        display: block;
      }
    }
    .loader-version-row {
      width: 100%;
      display: flex;
      align-items: center;
      margin: 0.1rem 0 0.2rem 0;
      justify-content: flex-start;
    }
  .installation-meta-grid {
    display: flex;
    flex-wrap: wrap;
    grid-template-columns: 1fr 1fr;
    font-size: 0.65rem;
    margin-top: 0.35rem;
    .meta-cell {
      display: flex;
      align-items: center;
      min-width: 0;
      flex-wrap: nowrap;
      overflow: hidden;
      justify-content: space-between;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
  }
    .meta-key {
      color: var(--text);
      font-weight: 500;
      opacity: 0.8;
      text-align: left;
      display: inline-block;
      min-width: fit-content;
      white-space: nowrap;
      overflow: visible;
    }
    .meta-cell:nth-child(2n) .meta-key {
      text-align: right;
    }
    .meta-value {
      color: var(--placeholder);
      font-weight: 400;
  background: color-mix(in srgb, var(--dark-200), 8%, transparent);
      border-radius: 0.4em;
      padding: 0.05em 0.45em;
      display: flex;
      align-items: center;
      gap: 0.3em;
      min-width: fit-content;
      flex-wrap: nowrap;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }
    .meta-cell:nth-child(2n) .meta-value {
      justify-content: flex-end;
      text-align: right;
    }

  // Responsive fallback: switch to flex column if grid can't fit
  @media (max-width: 32rem) {
    .installation-meta-grid {
      display: flex;
      flex-direction: column;
      gap: 0.18rem 0;
    }
    .meta-cell {
      justify-content: flex-start !important;
      text-align: left !important;
    }
    .meta-key, .meta-value {
      text-align: left !important;
    }
  }
  .loader-version {
    font-size: 0.82rem;
    color: var(--placeholder);
    font-weight: 500;
    background: var(--container);
    border-radius: 0.5em;
    padding: 0.08em 0.4em;
    text-align: left;
    margin-left: 0;
    line-height: 1.2;
    word-break: break-all;
    max-width: 100%;
    display: inline-block;
  }
  .small-meta-grid {
    grid-template-columns: 1fr !important;
    font-size: 0.82rem !important;
  }
  .small-meta-cell {
    width: auto;
    display: flex;
    align-items: baseline;
    gap: 0.25rem;
  }
  .small-meta-value {
    font-size: 0.82rem;
    flex: 0 0 auto;
    min-width: 0;
    max-width: 100%;
    white-space: nowrap;
    text-overflow: ellipsis;
    overflow: hidden;
  }
  .installation-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 1rem;
    min-height: 2.5rem;
    gap: 0.5rem;
  }
  /* Reserve space for play button */
    .dropdown {
      position: relative;
      .dropdown-toggle {
        background: none;
        border: none;
        padding: 0.5rem 0.75rem;
        border-radius: var(--border-radius);
        cursor: pointer;
        color: var(--text);
        transition: background 0.12s;
        height: 2.25rem;
        min-height: 2.25rem;
        max-height: 2.25rem;
        display: flex;
        align-items: center;
      }
      .dropdown-menu {
        opacity: 0;
        pointer-events: none;
        transition: opacity 0.15s cubic-bezier(0.4,0,0.2,1), z-index 0s linear 0.15s;
        position: absolute;
        right: 0;
        top: 100%;
        min-width: 10rem;
  background: color-mix(in srgb, var(--card), 94%, transparent);
        border: 1px solid var(--dark-200);
        border-radius: var(--border-radius);
        box-shadow: 0 0.25rem 1rem rgba(0,0,0,0.15), 0 0.125rem 0.5rem rgba(0,0,0,0.08);
        z-index: 1;
        flex-direction: column;
        padding: 0.5rem 0;
        backdrop-filter: blur(0.7rem) saturate(1.2);
        -webkit-backdrop-filter: blur(0.7rem) saturate(1.2);
        
        .dropdown-separator {
          height: 1px;
          background: var(--dark-200);
          margin: 0.3rem 0;
        }
        
        button {
          width: 100%;
          background: none;
          border: none;
          padding: 0.5rem 1rem;
          text-align: left;
          color: var(--text);
          font-size: 0.875rem;
          border-radius: 0;
          cursor: pointer;
          display: flex;
          align-items: center;
          gap: 0.5rem;
          transition: background 0.12s;
          
          &:hover {
            background: color-mix(in srgb, var(--dark-200), 10%, transparent);
          }
        }
        
        .danger {
          color: var(--red-700);
        }
      }
      &:hover .dropdown-menu,
      &:focus-within .dropdown-menu,
      .dropdown-menu:hover,
      .dropdown-menu:focus-within {
        opacity: 1;
        pointer-events: auto;
        z-index: 3000;
        transition: opacity 0.12s cubic-bezier(0.4,0,0.2,1), z-index 0s;
        display: flex;
      }
      .dropdown-menu {
        display: flex;
        transition: opacity 0.4s cubic-bezier(0.4,0,0.2,1), z-index 0s linear 0.4s;
      }
    }
  .empty-state {
    text-align: center;
    padding: 4rem 2rem;
    .empty-icon {
      margin-bottom: 1.5rem;
      color: var(--placeholder);
    }
    h3 {
      margin: 0 0 1rem;
      font-size: 1.5rem;
      font-weight: 600;
      color: var(--text);
    }
    p {
      margin: 0 0 2rem;
      color: var(--placeholder);
      font-size: 1rem;
    }
  }
  .loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 4rem 2rem;
    color: var(--placeholder);
    :global(.icon) {
      animation: spin 1s linear infinite;
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
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  @media (max-width: 48rem) { // 768px
    .installations-grid {
      grid-template-columns: 1fr;
    }
    .installation-card {
      padding: 1rem 0.5rem 1rem 0.5rem;
    }
    .installation-icon {
      width: 2.5rem;
      height: 2.5rem;
      font-size: 1.5rem;
    }
    
    /* List layout responsive adjustments */
    .list-item-main {
      flex-direction: column;
      align-items: stretch;
      gap: 0.75rem;
      padding: 0.75rem;
    }
    
    .list-item-icon-section {
      justify-content: center;
      
      .list-play-btn {
        flex: 1;
        justify-content: center;
      }
    }
    
    .list-title-actions-row {
      justify-content: center;
      text-align: center;
    }
    
    .list-version-stats-row {
      justify-content: center;
      gap: 1rem;
    }
    
    .list-stats-section {
      gap: 1rem;
    }
  }
  
  @media (max-width: 64rem) { // 1024px - tablet breakpoint
    .list-stats-section {
      gap: 1rem;
      
      .list-meta-item {
        font-size: 0.75rem;
      }
    }
  }
</style>
