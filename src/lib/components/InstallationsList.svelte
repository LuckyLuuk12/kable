<script lang="ts">
  import { Icon, InstallationManager, installations } from '$lib';

  export let isGrid: boolean = false;
  export let isSmall: boolean = false;
  export let isLoading: boolean = false;
  export let error: string | null = null;
  export let limit: number | null = null;

  $: limitedInstallations = $installations.slice(0, limit || $installations.length);
  $: loaderIcons = Object.fromEntries(
    $installations.map(installation => [
      installation.id,
      InstallationManager.getLoaderIcon(installation.version.loader)
    ])
  );
  $: loaderColors = Object.fromEntries(
    $installations.map(installation => [
      installation.id,
      InstallationManager.getLoaderColor(installation.version.loader)
    ])
  );
  // let modLoaderIcons: Record<string, string> = {};
  // let modLoaderColors: Record<string, string> = {};
  // let modLoaderDisplays: Record<string, string> = {};

  // onMount(async () => {
  //   installations = await InstallationManager.getInstallations();
  //   for (const installation of installations) {
  //     const detection = await InstallationManager.analyzeInstallation(installation);
  //     modLoaderIcons[installation.id] = detection
  //       ? ModDetectionService.getModLoaderIcon(detection.modLoaderType)
  //       : ModDetectionService.getModLoaderIcon(installation.mod_loader);
  //     modLoaderColors[installation.id] = detection
  //       ? ModDetectionService.getModLoaderColor(detection.modLoaderType)
  //       : ModDetectionService.getModLoaderColor(installation.mod_loader);
  //     modLoaderDisplays[installation.id] = detection
  //       ? ModDetectionService.getModdingStatusDescription(detection)
  //       : (installation.mod_loader === 'vanilla'
  //           ? 'Vanilla Minecraft'
  //           : installation.mod_loader.charAt(0).toUpperCase() + installation.mod_loader.slice(1) + (installation.loader_version ? ` ${installation.loader_version}` : '')
  //         );
  //   }
  // });

</script>

<div class=installations-list>
  {#if error}
    <div class="error-message">
      <Icon name="alert" size="sm" />
      {error}
    </div>
  {/if}

  {#if isLoading && limitedInstallations.length === 0}
    <div class="loading-state">
      <Icon name="refresh" size="md" />
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
      {#each limitedInstallations as installation}
        <div class={isSmall ? 'installation-card small' : 'installation-card'}>
          <div class="installation-header">
            <div class="installation-icon" style="background-color: {loaderColors[installation.id]}20; color: {loaderColors[installation.id]};">
              <Icon name={loaderIcons[installation.id]} size="lg" />
            </div>
            <div class="installation-info">
              <h3>{installation.name}</h3>
              {#if !isSmall}
                <div class="installation-details">
                  <span class="version">{installation.version.id}</span>
                  <span class="mod-loader" style="color: {loaderColors[installation.id]};">{installation.version.loader}</span>
                </div>
              {/if}
            </div>
          </div>

          {#if !isSmall}
            <div class="installation-stats">
              <div class="stat">
                <Icon name="clock" size="sm" />
                <span>Last played: {installation.last_used ? new Date(installation.last_used).toLocaleDateString() : 'Never'}</span>
              </div>
            </div>
          {/if}

          <div class="installation-actions">
            <!-- TODO: Implement launch button -->
            <!-- <button 
              class="btn btn-primary" 
              on:click={async () => await InstallationManager.launchInstallation(installation)}
              disabled={isLoading}
            >
              <Icon name="play" size="sm" />
              Play
            </button> -->
            <div class="dropdown">
              <button class="btn btn-secondary dropdown-toggle">
                <Icon name="more-horizontal" size="sm" />
              </button>
              <div class="dropdown-menu">
                <button on:click={async () => await InstallationManager.updateInstallation(installation.id, installation)}>
                  <Icon name="edit" size="sm" />
                  Edit
                </button>
                <button on:click={async () => await InstallationManager.createInstallation(installation.version.id)}>
                  <Icon name="duplicate" size="sm" />
                  Duplicate
                </button>
                <div class="dropdown-separator"></div>
                <button 
                  class="danger" 
                  on:click={async () => await InstallationManager.deleteInstallation(installation.id)}
                >
                  <Icon name="trash" size="sm" />
                  Delete
                </button>
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style lang="scss">
  @use "@kablan/clean-ui/scss/_variables.scss" as *;
  .installations-list {
    padding: 2rem;
    border-radius: $border-radius;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    overflow: scroll;
  }
  .installations-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 1rem;
  }
  .installations-flex {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .empty-state {
    text-align: center;
    padding: 4rem 2rem;
    .empty-icon {
      margin-bottom: 1.5rem;
      color: $placeholder;
    }
    h3 {
      margin: 0 0 1rem;
      font-size: 1.5rem;
      font-weight: 600;
      color: $text;
    }
    p {
      margin: 0 0 2rem;
      color: $placeholder;
      font-size: 1rem;
    }
  }
  .loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 4rem 2rem;
    color: $placeholder;
    :global(.icon) {
      animation: spin 1s linear infinite;
    }
  }
  .error-message {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem;
    background: rgba($red, 0.1);
    border: 1px solid $red;
    border-radius: $border-radius;
    color: $red;
    margin-bottom: 1rem;
  }
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  @media (max-width: 768px) {
    .installations-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
