<script lang="ts">
  import { Icon } from '$lib';
  import type { MinecraftInstallation, ModDetectionResult } from '$lib';

  export let installations: MinecraftInstallation[] = [];
  export let modDetectionResults: Map<string, ModDetectionResult> = new Map();
  export let isLoading: boolean = false;
  export let error: string | null = null;
  export let isGrid: boolean = false;
  export let small: boolean = false;
  export let onPlay: (installation: MinecraftInstallation) => void;
  export let onOpenFolder: (installation: MinecraftInstallation) => void;
  export let onEdit: (installation: MinecraftInstallation) => void;
  export let onDelete: (installationId: string) => void;
  export let onDuplicate: (installation: MinecraftInstallation) => void;
  export let getModLoaderIcon: (installation: MinecraftInstallation) => string;
  export let getModLoaderDisplay: (installation: MinecraftInstallation) => string;
  export let getModLoaderColor: (installation: MinecraftInstallation) => string;

  function getLastPlayed(installation: MinecraftInstallation) {
    return installation.last_played ? new Date(installation.last_played).toLocaleDateString() : 'Never';
  }
</script>

{#if error}
  <div class="error-message">
    <Icon name="alert" size="sm" />
    {error}
  </div>
{/if}

{#if isLoading && installations.length === 0}
  <div class="loading-state">
    <Icon name="refresh" size="md" />
    <span>Loading installations...</span>
  </div>
{:else if installations.length === 0}
  <div class="empty-state">
    <div class="empty-icon">
      <Icon name="cube" size="xl" />
    </div>
    <h3>No installations found</h3>
    <p>Create your first Minecraft installation to get started</p>
  </div>
{:else}
  <div class={isGrid ? 'installations-grid' : 'installations-flex'}>
    {#each installations as installation}
      <div class={small ? 'installation-card small' : 'installation-card'}>
        <div class="installation-header">
          <div class="installation-icon" style="background-color: {getModLoaderColor(installation)}20; color: {getModLoaderColor(installation)};">
            <Icon name={getModLoaderIcon(installation)} size="lg" />
          </div>
          <div class="installation-info">
            <h3>{installation.name}</h3>
            {#if !small}
              <div class="installation-details">
                <span class="version">{installation.version}</span>
                <span class="mod-loader" style="color: {getModLoaderColor(installation)};">{getModLoaderDisplay(installation)}</span>
              </div>
              {#if installation.description}
                <p class="description">{installation.description}</p>
              {/if}
            {/if}
          </div>
        </div>

        {#if !small}
          <div class="installation-stats">
            <div class="stat">
              <Icon name="clock" size="sm" />
              <span>Last played: {getLastPlayed(installation)}</span>
            </div>
            <div class="stat">
              <Icon name="folder" size="sm" />
              <span>Game directory: {installation.game_directory || 'Default'}</span>
            </div>
          </div>
        {/if}

        <div class="installation-actions">
          <button 
            class="btn btn-primary" 
            on:click={() => onPlay(installation)}
            disabled={isLoading}
          >
            <Icon name="play" size="sm" />
            Play
          </button>
          <button 
            class="btn btn-secondary" 
            on:click={() => onOpenFolder(installation)}
          >
            <Icon name="folder-open" size="sm" />
            Open Folder
          </button>
          <div class="dropdown">
            <button class="btn btn-secondary dropdown-toggle">
              <Icon name="more-horizontal" size="sm" />
            </button>
            <div class="dropdown-menu">
              <button on:click={() => onEdit(installation)}>
                <Icon name="edit" size="sm" />
                Edit
              </button>
              <button on:click={() => onDuplicate(installation)}>
                <Icon name="duplicate" size="sm" />
                Duplicate
              </button>
              <div class="dropdown-separator"></div>
              <button 
                class="danger" 
                on:click={() => onDelete(installation.id)}
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

<style lang="scss">
  @use "@kablan/clean-ui/scss/_variables.scss" as *;
  .installations-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
    gap: 1.5rem;
  }
  .installations-flex {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  .installation-card {
    background: $container;
    border: 1px solid $dark-600;
    border-radius: $border-radius;
    padding: 1.5rem;
    transition: all 0.2s ease;
    &:hover {
      border-color: $primary;
      transform: translateY(-2px);
    }
    .installation-header {
      display: flex;
      gap: 1rem;
      margin-bottom: 1rem;
      .installation-icon {
        width: 60px;
        height: 60px;
        border-radius: $border-radius;
        background: rgba($primary, 0.1);
        display: flex;
        align-items: center;
        justify-content: center;
        color: $primary;
        flex-shrink: 0;
      }
      .installation-info {
        flex: 1;
        h3 {
          margin: 0 0 0.5rem;
          font-size: 1.25rem;
          font-weight: 600;
          color: $text;
        }
        .installation-details {
          display: flex;
          gap: 1rem;
          margin-bottom: 0.5rem;
          .version {
            font-weight: 500;
            color: $green;
            font-family: monospace;
          }
          .mod-loader {
            font-size: 0.875rem;
            color: $placeholder;
            text-transform: capitalize;
          }
        }
        .description {
          margin: 0;
          font-size: 0.875rem;
          color: $placeholder;
          line-height: 1.4;
        }
      }
    }
    .installation-stats {
      margin-bottom: 1rem;
      padding: 1rem 0;
      border-top: 1px solid $dark-600;
      border-bottom: 1px solid $dark-600;
      .stat {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        margin-bottom: 0.5rem;
        font-size: 0.875rem;
        color: $placeholder;
        &:last-child {
          margin-bottom: 0;
        }
      }
    }
    .installation-actions {
      display: flex;
      gap: 0.75rem;
      align-items: center;
      .dropdown {
        position: relative;
        margin-left: auto;
        .dropdown-toggle {
          padding: 0.5rem;
        }
        .dropdown-menu {
          position: absolute;
          top: 100%;
          right: 0;
          margin-top: 0.25rem;
          background: $container;
          border: 1px solid $dark-600;
          border-radius: $border-radius;
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
          min-width: 150px;
          z-index: 10;
          opacity: 0;
          visibility: hidden;
          transform: translateY(-10px);
          transition: opacity 0.2s ease, visibility 0.2s ease, transform 0.2s ease;
          transition-delay: 0s;
          button {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            width: 100%;
            padding: 0.75rem 1rem;
            border: none;
            background: none;
            color: $text;
            font-size: 0.875rem;
            cursor: pointer;
            transition: background-color 0.2s ease;
            &:hover {
              background: rgba($primary, 0.1);
            }
            &.danger {
              color: $red;
            }
          }
          .dropdown-separator {
            height: 1px;
            background: $dark-600;
            margin: 0.5rem 0;
          }
        }
        &:hover .dropdown-menu {
          opacity: 1;
          visibility: visible;
          transform: translateY(0);
          transition-delay: 0.1s;
        }
        &::before {
          content: '';
          position: absolute;
          top: 100%;
          right: 0;
          width: 100%;
          height: 0.25rem;
          background: transparent;
          z-index: 9;
        }
      }
    }
    &.small {
      padding: 0.75rem 1rem;
      .installation-header {
        gap: 0.5rem;
        .installation-icon {
          width: 40px;
          height: 40px;
        }
        h3 {
          font-size: 1rem;
        }
      }
      .installation-actions {
        gap: 0.5rem;
        button {
          font-size: 0.85rem;
          padding: 0.3rem 0.7rem;
        }
      }
    }
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
    .installation-card {
      .installation-actions {
        flex-wrap: wrap;
      }
    }
  }
</style>
