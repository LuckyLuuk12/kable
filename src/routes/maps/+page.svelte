<script lang="ts">
  import { onMount } from 'svelte';
  import { MapsManager, SettingsManager, Icon } from '$lib';
  import type { LocalWorld } from '$lib/types';

  let searchQuery = '';
  let selectedCategory = 'all';
  let sortBy = 'recent';
  let localWorlds: LocalWorld[] = [];
  let filteredWorlds: LocalWorld[] = [];
  let isLoading = false;
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
    // Initialize settings first
    await SettingsManager.initialize();
    await loadWorlds();
  });

  async function loadWorlds() {
    isLoading = true;
    error = null;
    try {
      localWorlds = await MapsManager.getLocalWorlds();
      updateFilteredWorlds();
    } catch (err) {
      console.error('Failed to load worlds:', err);
      error = `Failed to load worlds: ${err}`;
    } finally {
      isLoading = false;
    }
  }

  function updateFilteredWorlds() {
    let filtered = [...localWorlds];

    // Filter by category
    if (selectedCategory !== 'all') {
      filtered = filtered.filter(world => 
        world.game_mode?.toLowerCase() === selectedCategory
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
          return new Date(b.last_played || 0).getTime() - new Date(a.last_played || 0).getTime();
        case 'name':
          return a.name.localeCompare(b.name);
        case 'size':
          return (b.size_mb || 0) - (a.size_mb || 0);
        default:
          return 0;
      }
    });

    filteredWorlds = filtered;
  }

  $: {
    updateFilteredWorlds();
  }

  async function deleteWorld(worldName: string) {
    if (!confirm(`Are you sure you want to delete the world "${worldName}"? This action cannot be undone.`)) {
      return;
    }

    try {
      await MapsManager.deleteWorld(worldName);
      await loadWorlds(); // Refresh the list
    } catch (err) {
      console.error('Failed to delete world:', err);
      error = `Failed to delete world: ${err}`;
    }
  }

  async function backupWorld(worldName: string) {
    try {
      const backupName = await MapsManager.backupWorld(worldName);
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
  <div class="page-header">
    <h1>Local Worlds</h1>
    <p>Manage your Minecraft worlds and save files</p>
  </div>

  {#if error}
    <div class="error-message">
      <Icon name="alert" size="sm" />
      {error}
    </div>
  {/if}

  <!-- Search and Filters -->
  <section class="search-section">
    <div class="search-bar">
      <div class="search-input-wrapper">
        <Icon name="search" size="sm" className="search-icon" />
        <input 
          type="text" 
          placeholder="Search worlds..." 
          bind:value={searchQuery}
          class="search-input"
        />
      </div>
      
      <div class="filter-controls">
        <select bind:value={selectedCategory} class="category-select">
          {#each categories as category}
            <option value={category.id}>
              {category.name}
            </option>
          {/each}
        </select>
        
        <select bind:value={sortBy} class="sort-select">
          <option value="recent">Recently Played</option>
          <option value="name">Name (A-Z)</option>
          <option value="size">File Size</option>
        </select>
        
        <button on:click={loadWorlds} class="btn btn-secondary btn-sm" disabled={isLoading}>
          <Icon name="refresh" size="sm" />
          {isLoading ? 'Loading...' : 'Refresh'}
        </button>
      </div>
    </div>
  </section>

  <!-- Local Worlds Grid -->
  <section class="worlds-section">
    <div class="section-header">
      <h2>Your Worlds ({filteredWorlds.length})</h2>
    </div>

    {#if isLoading}
      <div class="loading-state">
        <Icon name="loader" size="lg" />
        <p>Loading worlds...</p>
      </div>
    {:else if filteredWorlds.length > 0}
      <div class="worlds-grid">
        {#each filteredWorlds as world}
          <div class="world-card">
            <div class="world-header">
              <Icon name="map" size="md" />
              <h3 class="world-name">{world.name}</h3>
            </div>
            
            <div class="world-details">
              <div class="detail-row">
                <span class="label">Game Mode:</span>
                <span class="value">{world.game_mode || 'Unknown'}</span>
              </div>
              
              {#if world.version}
                <div class="detail-row">
                  <span class="label">Version:</span>
                  <span class="value">{world.version}</span>
                </div>
              {/if}
              
              {#if world.last_played}
                <div class="detail-row">
                  <span class="label">Last Played:</span>
                  <span class="value">{new Date(world.last_played).toLocaleDateString()}</span>
                </div>
              {/if}
              
              {#if world.size_mb}
                <div class="detail-row">
                  <span class="label">Size:</span>
                  <span class="value">{formatFileSize(world.size_mb * 1024 * 1024)}</span>
                </div>
              {/if}
              
              {#if world.difficulty}
                <div class="detail-row">
                  <span class="label">Difficulty:</span>
                  <span class="value">{world.difficulty || 'Unknown'}</span>
                </div>
              {/if}
              
              <div class="detail-row">
                <span class="label">Backups:</span>
                <span class="value">{world.backup_count || 0}</span>
              </div>
            </div>
            
            <div class="world-actions">
              <button 
                on:click={() => backupWorld(world.name)}
                class="action-btn secondary-btn"
              >
                <Icon name="backup" size="sm" />
                Backup
              </button>
              
              <button 
                on:click={() => deleteWorld(world.name)}
                class="action-btn danger-btn"
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
        <Icon name="map" size="xl" className="empty-icon" />
        <h3>No worlds found</h3>
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
  </section>
</div>

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;

  .maps-page {
    max-width: 1200px;
    margin: 0 auto;
    padding: 1rem;
  }

  .page-header {
    text-align: center;
    margin-bottom: 2rem;
    
    h1 {
      margin: 0 0 0.5rem 0;
      color: $text;
      font-size: 2.5rem;
      font-weight: 700;
    }
    
    p {
      margin: 0;
      color: $placeholder;
      font-size: 1.1rem;
    }
  }

  .error-message {
    background: rgba($red, 0.1);
    color: $red;
    padding: 1rem;
    border-radius: 0.5rem;
    margin-bottom: 1rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 500;
  }

  .search-section {
    background: $card;
    border: 1px solid $dark-600;
    border-radius: 1rem;
    padding: 1.5rem;
    margin-bottom: 2rem;
  }

  .search-bar {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
    align-items: center;
    
    @media (max-width: 768px) {
      flex-direction: column;
    }
  }

  .search-input-wrapper {
    flex: 1;
    position: relative;
    min-width: 250px;
    
    :global(.search-icon) {
      position: absolute;
      left: 1rem;
      top: 50%;
      transform: translateY(-50%);
      color: $placeholder;
    }
    
    .search-input {
      width: 100%;
      padding: 0.75rem 1rem 0.75rem 2.5rem;
      border: 1px solid $dark-600;
      border-radius: 0.75rem;
      background: $input;
      color: $text;
      font-size: 1rem;
      
      &:focus {
        outline: none;
        border-color: $primary;
      }
    }
  }

  .filter-controls {
    display: flex;
    gap: 0.75rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .category-select, .sort-select {
    padding: 0.75rem 1rem;
    border: 1px solid $dark-600;
    border-radius: 0.75rem;
    background: $input;
    color: $text;
    font-size: 0.9rem;
    cursor: pointer;
    
    &:focus {
      outline: none;
      border-color: $primary;
    }
  }

  .worlds-section {
    background: $card;
    border: 1px solid $dark-600;
    border-radius: 1rem;
    padding: 2rem;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
    
    h2 {
      margin: 0;
      color: $text;
      font-size: 1.5rem;
      font-weight: 600;
    }
  }

  .loading-state {
    text-align: center;
    padding: 3rem 1rem;
    
    :global(.loader) {
      color: $primary;
      margin-bottom: 1rem;
      animation: spin 1s linear infinite;
    }
    
    p {
      color: $placeholder;
      font-size: 1.1rem;
    }
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .worlds-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 1.5rem;
  }

  .world-card {
    background: $container;
    border: 1px solid $dark-600;
    border-radius: 0.75rem;
    padding: 1.5rem;
    transition: all 0.2s ease;
    
    &:hover {
      border-color: $primary;
      transform: translateY(-2px);
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    }
    
    .world-header {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      margin-bottom: 1rem;
      
      .world-name {
        margin: 0;
        color: $text;
        font-size: 1.25rem;
        font-weight: 600;
        flex: 1;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }
      

    }
    
    .world-details {
      margin-bottom: 1.5rem;
      
      .detail-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.5rem;
        
        .label {
          color: $placeholder;
          font-size: 0.875rem;
          font-weight: 500;
        }
        
        .value {
          color: $text;
          font-size: 0.875rem;
          font-weight: 500;
        }
      }
    }
    
    .world-actions {
      display: flex;
      gap: 0.75rem;
      
      .action-btn {
        flex: 1;
        padding: 0.75rem;
        border-radius: 0.5rem;
        font-weight: 600;
        font-size: 0.875rem;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        border: none;
        cursor: pointer;
        transition: all 0.2s ease;
        
        &.danger-btn {
          background: $red;
          color: white;
          
          &:hover {
            background: $red-600;
          }
        }
      }
    }
  }

  .empty-state {
    text-align: center;
    padding: 3rem 1rem;
    
    :global(.empty-icon) {
      color: $placeholder;
      margin-bottom: 1rem;
    }
    
    h3 {
      margin: 0 0 0.5rem 0;
      color: $text;
      font-size: 1.25rem;
      font-weight: 600;
    }
    
    p {
      margin: 0 0 1.5rem 0;
      color: $placeholder;
      font-size: 1rem;
      max-width: 500px;
      margin-left: auto;
      margin-right: auto;
    }
  }

  // Responsive design
  @media (max-width: 768px) {
    .worlds-grid {
      grid-template-columns: 1fr;
    }
    
    .world-actions {
      flex-direction: column;
    }
  }
</style>
