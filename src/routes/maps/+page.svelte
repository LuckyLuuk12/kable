<script lang="ts">
  import { onMount } from 'svelte';
import { MapsService, SettingsService, Icon } from '$lib';
  import type { LocalWorld } from '$lib/types';

  let searchQuery = '';
  let selectedCategory = 'all';
  let sortBy = 'recent';
  let localWorlds: LocalWorld[] = [];
  let filteredWorlds: LocalWorld[] = [];
  let isLoading = false;
  let isRefreshing = false;
  let error: string | null = null;
  
  // Categories based on actual game modes and world types
  const categories = [
    { id: 'all', name: 'All Worlds', icon: 'map' },
    { id: 'survival', name: 'Survival', icon: 'home' },
    { id: 'creative', name: 'Creative', icon: 'palette' },
    { id: 'adventure', name: 'Adventure', icon: 'compass' },
    { id: 'spectator', name: 'Spectator', icon: 'eye' },
    { id: 'hardcore', name: 'Hardcore', icon: 'skull' }
  ];

  onMount(async () => {
    // SettingsManager is already initialized in the layout
    await loadWorlds();
  });

  async function loadWorlds() {
    isLoading = true;
    isRefreshing = true;
    error = null;
    try {
      localWorlds = await MapsService.getLocalWorlds();
      // filteredWorlds will update automatically via reactive statement
    } catch (err) {
      console.error('Failed to load worlds:', err);
      error = `Failed to load worlds: ${err}`;
    } finally {
      isLoading = false;
      isRefreshing = false;
    }
  }

  // Reactive filtering - automatically updates when dependencies change
  $: filteredWorlds = (() => {
    let filtered = [...localWorlds];

    // Filter by category (game mode)
    if (selectedCategory !== 'all') {
      filtered = filtered.filter(world => 
        world.game_mode?.toLowerCase() === selectedCategory.toLowerCase()
      );
    }

    // Filter by search query
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(world =>
        world.name.toLowerCase().includes(query) ||
        world.game_mode?.toLowerCase().includes(query)
      );
    }

    // Sort worlds
    filtered.sort((a, b) => {
      switch (sortBy) {
        case 'recent':
          return (b.last_played || 0) - (a.last_played || 0);
        case 'name':
          return a.name.localeCompare(b.name);
        case 'size':
          return (b.size_mb || 0) - (a.size_mb || 0);
        default:
          return 0;
      }
    });

    return filtered;
  })();

  async function deleteWorld(worldName: string) {
    if (!confirm(`Are you sure you want to delete the world "${worldName}"? This action cannot be undone.`)) {
      return;
    }

    try {
      await MapsService.deleteWorld(worldName);
      await loadWorlds(); // Refresh the list
    } catch (err) {
      console.error('Failed to delete world:', err);
      error = `Failed to delete world: ${err}`;
    }
  }

  async function backupWorld(worldName: string) {
    try {
      const backupName = await MapsService.backupWorld(worldName);
      // Show success message with backup location info
      const successMessage = `World "${worldName}" backed up successfully as "${backupName}".\nBackups are stored in .minecraft/kable/world-backups/`;
      alert(successMessage);
      console.log(successMessage);
      // Refresh the worlds list to update backup counts
      await loadWorlds();
    } catch (err) {
      console.error('Failed to backup world:', err);
      error = `Failed to backup world: ${err}`;
    }
  }

  function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }
</script>

<div class="maps-page">
  <!-- Page Header -->
  <div class="page-header">
    <div class="header-left">
      <h1>
        <Icon name="map" size="md" />
        Minecraft Worlds
      </h1>
      <div class="header-meta">
        <p>Manage your Minecraft worlds and save files</p>
        {#if !isLoading && localWorlds.length > 0}
          <div class="stat-badge">
            <Icon name="folder" size="sm" />
            {localWorlds.length} world{localWorlds.length !== 1 ? 's' : ''} found
          </div>
        {/if}
      </div>
    </div>
    <div class="header-actions">
      <button 
        on:click={loadWorlds} 
        class="btn btn-secondary {isRefreshing ? 'spinning' : ''}"
        disabled={isLoading}
        title="Refresh worlds list"
      >
        <Icon name="refresh" size="sm" forceType="svg" />
      </button>
    </div>
  </div>

  {#if error}
    <div class="error-message">
      <Icon name="alert" size="sm" />
      {error}
    </div>
  {/if}

  <!-- Search and Filters -->
  <div class="filters-section">
    <div class="search-container">
      <Icon name="search" size="sm" />
      <input
        type="text"
        placeholder="Search worlds by name or game mode..."
        bind:value={searchQuery}
        class="search-input"
      />
    </div>
    <div class="filter-controls">
      <select bind:value={selectedCategory} class="filter-select">
        {#each categories as category}
          <option value={category.id}>
            {category.name}
          </option>
        {/each}
      </select>
      
      <select bind:value={sortBy} class="filter-select">
        <option value="recent">Recently Played</option>
        <option value="name">Name (A-Z)</option>
        <option value="size">File Size</option>
      </select>
    </div>
  </div>

  <!-- Results Header -->
  <div class="results-header">
    <h2>
      {#if searchQuery.trim() || selectedCategory !== 'all'}
        Filtered Results
      {:else}
        All Worlds
      {/if}
      <span class="count">({filteredWorlds.length})</span>
    </h2>
  </div>

  <!-- Worlds Grid -->
  <div class="worlds-content">
    {#if isLoading}
      <div class="loading-state">
        <Icon name="loader" size="xl" />
        <p>Loading worlds...</p>
      </div>
    {:else if filteredWorlds.length > 0}
      <div class="worlds-grid">
        {#each filteredWorlds as world}
          <div class="world-card">
            <div class="world-header">
              <div class="world-icon">
                <Icon name={world.game_mode?.toLowerCase() === 'hardcore' ? 'skull' : 'world'} size="lg" />
              </div>
              <div class="world-info">
                <h3 class="world-name">{world.name}</h3>
                <div class="world-meta">
                  <span class="badge badge-{world.game_mode?.toLowerCase() || 'survival'}">
                    <Icon name={categories.find(c => c.id === world.game_mode?.toLowerCase())?.icon || 'map'} size="sm" />
                    {world.game_mode || 'Unknown'}
                  </span>
                  {#if world.version}
                    <span class="world-version">{world.version}</span>
                  {/if}
                </div>
              </div>
            </div>
            
            <div class="world-stats">
              {#if world.last_played}
                <div class="stat-item">
                  <Icon name="clock" size="sm" />
                  <span class="stat-label">Last Played:</span>
                  <span class="stat-value">{new Date(world.last_played).toLocaleDateString()}</span>
                </div>
              {/if}
              
              {#if world.size_mb}
                <div class="stat-item">
                  <Icon name="folder" size="sm" />
                  <span class="stat-label">Size:</span>
                  <span class="stat-value">{formatFileSize(world.size_mb * 1024 * 1024)}</span>
                </div>
              {/if}
              
              {#if world.difficulty}
                <div class="stat-item">
                  <Icon name="gamepad" size="sm" />
                  <span class="stat-label">Difficulty:</span>
                  <span class="stat-value">{world.difficulty}</span>
                </div>
              {/if}
              
              <div class="stat-item">
                <Icon name="archive" size="sm" />
                <span class="stat-label">Backups:</span>
                <span class="stat-value">{world.backup_count || 0}</span>
              </div>
            </div>
            
            <div class="world-actions">
              <button 
                on:click={() => backupWorld(world.name)}
                class="btn btn-secondary btn-sm"
                title="Create backup"
              >
                <Icon name="archive" size="sm" />
                Backup
              </button>
              
              <button 
                on:click={() => deleteWorld(world.name)}
                class="btn btn-danger btn-sm"
                title="Delete world"
              >
                <Icon name="trash" size="sm" />
                Delete
              </button>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="empty-state">
        <div class="empty-icon">
          {#if searchQuery.trim() || selectedCategory !== 'all'}
            <Icon name="search" size="xl" />
          {:else}
            <Icon name="map" size="xl" />
          {/if}
        </div>
        <h3>
          {#if searchQuery.trim() || selectedCategory !== 'all'}
            No worlds found
          {:else}
            No Minecraft worlds
          {/if}
        </h3>
        <p>
          {#if searchQuery.trim() || selectedCategory !== 'all'}
            No worlds match your current filters. Try adjusting your search or category.
          {:else}
            No Minecraft worlds were found. Make sure Minecraft is installed and you have created some worlds.
          {/if}
        </p>
        <button on:click={loadWorlds} class="btn btn-primary">
          <Icon name="refresh" size="sm" />
          Refresh Worlds
        </button>
      </div>
    {/if}
  </div>
</div>

<style lang="scss">
  .maps-page {
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
    gap: 2rem;
    
    .header-left {
      flex: 1;
      
      h1 {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        margin: 0 0 0.75rem 0;
        font-size: 2rem;
        font-weight: 700;
      }
      
      .header-meta {
        display: flex;
        align-items: center;
        gap: 1rem;
        flex-wrap: wrap;
        
        p {
          margin: 0;
          color: var(--placeholder);
          font-size: 1.1rem;
        }
        
        .stat-badge {
          display: inline-flex;
          align-items: center;
          gap: 0.25rem;
          padding: 0.25rem 0.75rem;
          background: color-mix(in srgb, var(--primary), 10%, transparent);
          color: var(--primary);
          border-radius: var(--border-radius);
          font-size: 0.875rem;
          font-weight: 500;
        }
      }
    }
    
    .header-actions {
      display: flex;
      gap: 0.5rem;
      
      .spinning {
        animation: spin 1s linear infinite;
      }
    }
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error-message {
    background: color-mix(in srgb, var(--red), 10%, transparent);
    color: var(--red);
    padding: 1rem;
    border-radius: var(--border-radius);
    margin-bottom: 1rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 500;
    border: 1px solid var(--dark-200);
  }

  .filters-section {
    display: flex;
    gap: 1rem;
    align-items: center;
    margin-bottom: 1rem;
    padding: 1rem;
    background: var(--container);
    border-radius: var(--border-radius);
    border: 1px solid var(--dark-200);

    .search-container {
      flex: 1;
      position: relative;
      display: flex;
      align-items: center;
      gap: 0.5rem;
      
      .search-input {
        flex: 1;
        padding: 0.5rem 0.75rem;
        font-size: 0.9rem;
        background: var(--card);
        border: 1px solid var(--dark-200);
        border-radius: var(--border-radius-small);
        color: var(--text);
        
        &:focus {
          outline: none;
          border-color: var(--primary);
        }
        
        &::placeholder {
          color: var(--placeholder);
        }
      }
    }
    
    .filter-controls {
      display: flex;
      gap: 0.5rem;
      align-items: center;
      
      .filter-select {
        padding: 0.5rem 0.75rem;
        background: var(--card);
        border: 1px solid var(--dark-200);
        border-radius: var(--border-radius-small);
        color: var(--text);
        font-size: 0.9rem;
        cursor: pointer;
        
        &:focus {
          outline: none;
          border-color: var(--primary);
        }
      }
    }
  }

  .results-header {
    margin-bottom: 1rem;
    
    h2 {
      margin: 0;
      font-size: 1.25rem;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: 0.5rem;
      
      .count {
        color: var(--placeholder);
        font-weight: 400;
      }
    }
  }

  .worlds-content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 1rem;
    text-align: center;
    
    :global(svg) {
      color: var(--primary);
      margin-bottom: 1rem;
      animation: spin 1s linear infinite;
    }
    
    p {
      color: var(--placeholder);
      font-size: 1.1rem;
      margin: 0;
    }
  }

  .worlds-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: 1rem;
    padding-bottom: 1rem;
  }

  .world-card {
    background: var(--container);
    border: 1px solid var(--dark-200);
    border-radius: var(--border-radius);
    padding: 1.25rem;
    transition: all 0.2s ease;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    
    &:hover {
      border-color: var(--primary);
      transform: translateY(-2px);
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    }
    
    .world-header {
      display: flex;
      align-items: flex-start;
      gap: 1rem;
      
      .world-icon {
        flex-shrink: 0;
        width: 3rem;
        height: 3rem;
        display: flex;
        align-items: center;
        justify-content: center;
        background: color-mix(in srgb, var(--primary), 10%, transparent);
        border-radius: var(--border-radius);
        color: var(--primary);
      }
      
      .world-info {
        flex: 1;
        min-width: 0;
        
        .world-name {
          margin: 0 0 0.5rem 0;
          font-size: 1.125rem;
          font-weight: 600;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }
        
        .world-meta {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          flex-wrap: wrap;
          
          .badge {
            display: inline-flex;
            align-items: center;
            gap: 0.25rem;
            padding: 0.125rem 0.5rem;
            border-radius: var(--border-radius-small);
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: capitalize;
            
            &.badge-survival {
              background: color-mix(in srgb, var(--green), 10%, transparent);
              color: var(--green);
            }
            
            &.badge-creative {
              background: color-mix(in srgb, var(--blue), 10%, transparent);
              color: var(--blue);
            }
            
            &.badge-adventure {
              background: color-mix(in srgb, var(--purple), 10%, transparent);
              color: var(--purple);
            }
            
            &.badge-spectator {
              background: color-mix(in srgb, var(--text), 10%, transparent);
              color: var(--text);
            }
            
            &.badge-hardcore {
              background: color-mix(in srgb, var(--red), 10%, transparent);
              color: var(--red);
            }
          }
          
          .world-version {
            font-size: 0.75rem;
            color: var(--placeholder);
            font-weight: 500;
          }
        }
      }
    }
    
    .world-stats {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      padding: 0.75rem;
      background: var(--card);
      border-radius: var(--border-radius-small);
      
      .stat-item {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.875rem;
        
        :global(svg) {
          flex-shrink: 0;
          color: var(--placeholder);
        }
        
        .stat-label {
          color: var(--placeholder);
          min-width: 5rem;
        }
        
        .stat-value {
          color: var(--text);
          font-weight: 500;
        }
      }
    }
    
    .world-actions {
      display: flex;
      gap: 0.5rem;
      
      button {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 0.5rem 1rem;
      }
    }
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 1rem;
    text-align: center;
    
    .empty-icon {
      color: var(--placeholder);
      opacity: 0.5;
      margin-bottom: 1rem;
    }
    
    h3 {
      margin: 0 0 0.5rem 0;
      font-size: 1.5rem;
      font-weight: 600;
    }
    
    p {
      margin: 0 0 1.5rem 0;
      color: var(--placeholder);
      font-size: 1rem;
      max-width: 500px;
    }
  }

  @media (max-width: 768px) {
    .filters-section {
      flex-direction: column;
      align-items: stretch;
      
      .filter-controls {
        flex-wrap: wrap;
      }
    }
    
    .worlds-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
