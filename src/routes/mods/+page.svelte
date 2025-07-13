<script lang="ts">
  import { AuthManager } from '$lib';
  import { currentAccount } from '$lib/auth';
  import { onMount } from 'svelte';

  let searchQuery = '';
  let selectedCategory = 'all';
  let sortBy = 'popular';
  
  // Mock mod data - this would come from an API
  const categories = [
    { id: 'all', name: 'All Categories', icon: '📂' },
    { id: 'technology', name: 'Technology', icon: '⚙️' },
    { id: 'magic', name: 'Magic', icon: '✨' },
    { id: 'adventure', name: 'Adventure', icon: '🗺️' },
    { id: 'decoration', name: 'Decoration', icon: '🎨' },
    { id: 'utility', name: 'Utility', icon: '🔧' },
    { id: 'world-gen', name: 'World Generation', icon: '🌍' }
  ];

  const mockMods = [
    {
      id: 'jei',
      name: 'Just Enough Items (JEI)',
      description: 'JEI is an item and recipe viewing mod for Minecraft, built from the ground up for stability and performance.',
      author: 'mezz',
      downloads: '234M',
      version: '15.2.0.27',
      category: 'utility',
      icon: '🔍',
      installed: false
    },
    {
      id: 'optifine',
      name: 'OptiFine',
      description: 'OptiFine is a Minecraft optimization mod. It allows Minecraft to run faster and look better with full support for HD textures.',
      author: 'sp614x',
      downloads: '456M',
      version: 'HD U I5',
      category: 'utility',
      icon: '⚡',
      installed: true
    },
    {
      id: 'thermal',
      name: 'Thermal Expansion',
      description: 'Expanding Minecraft Thermally! A server-friendly and content-rich blend of magic and technology!',
      author: 'TeamCoFH',
      downloads: '89M',
      version: '10.0.2.18',
      category: 'technology',
      icon: '🔥',
      installed: false
    }
  ];

  let filteredMods = mockMods;

  onMount(async () => {
    await AuthManager.initialize();
    updateFilter();
  });

  function updateFilter() {
    filteredMods = mockMods.filter(mod => {
      const matchesSearch = mod.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                           mod.description.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesCategory = selectedCategory === 'all' || mod.category === selectedCategory;
      return matchesSearch && matchesCategory;
    });

    // Sort mods
    if (sortBy === 'popular') {
      filteredMods.sort((a, b) => parseFloat(b.downloads) - parseFloat(a.downloads));
    } else if (sortBy === 'name') {
      filteredMods.sort((a, b) => a.name.localeCompare(b.name));
    } else if (sortBy === 'installed') {
      filteredMods.sort((a, b) => (b.installed ? 1 : 0) - (a.installed ? 1 : 0));
    }
  }

  async function installMod(modId: string) {
    try {
      console.log('Installing mod:', modId);
      // Update mock data
      const mod = mockMods.find(m => m.id === modId);
      if (mod) mod.installed = true;
      updateFilter();
    } catch (error) {
      console.error('Failed to install mod:', error);
    }
  }

  async function uninstallMod(modId: string) {
    try {
      console.log('Uninstalling mod:', modId);
      // Update mock data
      const mod = mockMods.find(m => m.id === modId);
      if (mod) mod.installed = false;
      updateFilter();
    } catch (error) {
      console.error('Failed to uninstall mod:', error);
    }
  }

  // React to changes
  $: searchQuery, selectedCategory, sortBy, updateFilter();
</script>

<div class="mods-page">
  <div class="page-header">
    <h1>Mods</h1>
    <p>Browse, install, and manage your Minecraft mods</p>
  </div>

  {#if !$currentAccount}
    <div class="auth-required">
      <div class="warning-card">
        <div class="warning-icon">🔒</div>
        <div class="warning-content">
          <h3>Authentication Required</h3>
          <p>Sign in with Microsoft to access mod management features</p>
          <button on:click={() => AuthManager.signIn()} class="sign-in-btn">
            Sign in with Microsoft
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Search and Filters -->
  <section class="search-section">
    <div class="search-bar">
      <div class="search-input-wrapper">
        <span class="search-icon">🔍</span>
        <input 
          type="text" 
          placeholder="Search mods..." 
          bind:value={searchQuery}
          class="search-input"
        />
      </div>
      
      <div class="filter-controls">
        <select bind:value={selectedCategory} class="category-select">
          {#each categories as category}
            <option value={category.id}>
              {category.icon} {category.name}
            </option>
          {/each}
        </select>
        
        <select bind:value={sortBy} class="sort-select">
          <option value="popular">Most Popular</option>
          <option value="name">Name (A-Z)</option>
          <option value="installed">Installed First</option>
        </select>
      </div>
    </div>
  </section>

  <!-- Mods Grid -->
  <section class="mods-section">
    <div class="section-header">
      <h2>Available Mods ({filteredMods.length})</h2>
      <div class="view-options">
        <button class="view-btn active">Grid</button>
        <button class="view-btn">List</button>
      </div>
    </div>

    {#if filteredMods.length > 0}
      <div class="mods-grid">
        {#each filteredMods as mod}
          <div class="mod-card" class:installed={mod.installed}>
            <div class="mod-header">
              <div class="mod-icon">{mod.icon}</div>
              <div class="mod-status">
                {#if mod.installed}
                  <span class="status-badge installed">✅ Installed</span>
                {:else}
                  <span class="status-badge available">📦 Available</span>
                {/if}
              </div>
            </div>
            
            <div class="mod-content">
              <h3 class="mod-name">{mod.name}</h3>
              <p class="mod-description">{mod.description}</p>
              
              <div class="mod-meta">
                <div class="meta-item">
                  <span class="meta-label">Author:</span>
                  <span class="meta-value">{mod.author}</span>
                </div>
                <div class="meta-item">
                  <span class="meta-label">Downloads:</span>
                  <span class="meta-value">{mod.downloads}</span>
                </div>
                <div class="meta-item">
                  <span class="meta-label">Version:</span>
                  <span class="meta-value">{mod.version}</span>
                </div>
              </div>
            </div>
            
            <div class="mod-actions">
              {#if mod.installed}
                <button 
                  on:click={() => uninstallMod(mod.id)}
                  class="action-btn uninstall-btn"
                  disabled={!$currentAccount}
                >
                  🗑️ Uninstall
                </button>
              {:else}
                <button 
                  on:click={() => installMod(mod.id)}
                  class="action-btn install-btn"
                  disabled={!$currentAccount}
                >
                  ⬇️ Install
                </button>
              {/if}
              
              <button class="action-btn info-btn">
                ℹ️ Info
              </button>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="no-results">
        <div class="empty-state">
          <div class="empty-icon">🔍</div>
          <h3>No mods found</h3>
          <p>Try adjusting your search criteria or browse different categories.</p>
        </div>
      </div>
    {/if}
  </section>
</div>

<style lang="scss">
  .mods-page {
    max-width: 1400px;
    margin: 0 auto;
  }

  .page-header {
    text-align: center;
    margin-bottom: 2rem;
    
    h1 {
      margin: 0 0 0.5rem 0;
      font-size: 2.5rem;
      font-weight: 700;
      background: linear-gradient(135deg, var(--primary), var(--accent));
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      background-clip: text;
    }
    
    p {
      margin: 0;
      color: var(--text-muted);
      font-size: 1.1rem;
    }
  }

  .auth-required {
    margin-bottom: 2rem;
  }

  .warning-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1.5rem;
    background: var(--warning-light);
    border: 1px solid var(--warning);
    border-radius: 1rem;
    
    .warning-icon {
      font-size: 2rem;
    }
    
    .warning-content {
      flex: 1;
      
      h3 {
        margin: 0 0 0.5rem 0;
        color: var(--warning);
      }
      
      p {
        margin: 0 0 1rem 0;
        color: var(--text-muted);
      }
    }
  }

  .search-section {
    margin-bottom: 2rem;
  }

  .search-bar {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
    
    @media (max-width: 768px) {
      flex-direction: column;
    }
  }

  .search-input-wrapper {
    flex: 1;
    position: relative;
    min-width: 250px;
    
    .search-icon {
      position: absolute;
      left: 1rem;
      top: 50%;
      transform: translateY(-50%);
      color: var(--text-muted);
    }
    
    .search-input {
      width: 100%;
      padding: 0.75rem 1rem 0.75rem 2.5rem;
      border: 1px solid var(--border);
      border-radius: 0.75rem;
      background: var(--surface);
      color: var(--text);
      font-size: 1rem;
      
      &:focus {
        outline: none;
        border-color: var(--primary);
      }
    }
  }

  .filter-controls {
    display: flex;
    gap: 0.5rem;
  }

  .category-select, .sort-select {
    padding: 0.75rem 1rem;
    border: 1px solid var(--border);
    border-radius: 0.75rem;
    background: var(--surface);
    color: var(--text);
    cursor: pointer;
    
    &:focus {
      outline: none;
      border-color: var(--primary);
    }
  }

  .mods-section {
    h2 {
      margin: 0;
      color: var(--text);
      font-size: 1.5rem;
    }
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .view-options {
    display: flex;
    gap: 0.5rem;
    
    .view-btn {
      padding: 0.5rem 1rem;
      border: 1px solid var(--border);
      border-radius: 0.5rem;
      background: var(--surface);
      color: var(--text);
      cursor: pointer;
      transition: all 0.2s ease;
      
      &.active {
        background: var(--primary);
        color: white;
        border-color: var(--primary);
      }
      
      &:hover:not(.active) {
        background: var(--surface-hover);
      }
    }
  }

  .mods-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: 1.5rem;
  }

  .mod-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 1rem;
    padding: 1.5rem;
    transition: all 0.2s ease;
    
    &:hover {
      transform: translateY(-2px);
      box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
    }
    
    &.installed {
      border-color: var(--success);
      background: var(--success-light);
    }
  }

  .mod-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
    
    .mod-icon {
      font-size: 2rem;
      width: 48px;
      height: 48px;
      display: flex;
      align-items: center;
      justify-content: center;
      background: var(--background);
      border-radius: 0.75rem;
      border: 1px solid var(--border);
    }
    
    .status-badge {
      padding: 0.25rem 0.75rem;
      border-radius: 1rem;
      font-size: 0.75rem;
      font-weight: 500;
      
      &.installed {
        background: var(--success);
        color: white;
      }
      
      &.available {
        background: var(--surface-variant);
        color: var(--text);
      }
    }
  }

  .mod-content {
    margin-bottom: 1.5rem;
    
    .mod-name {
      margin: 0 0 0.5rem 0;
      color: var(--text);
      font-size: 1.125rem;
      font-weight: 600;
    }
    
    .mod-description {
      margin: 0 0 1rem 0;
      color: var(--text-muted);
      font-size: 0.875rem;
      line-height: 1.5;
      display: -webkit-box;
      -webkit-line-clamp: 3;
      -webkit-box-orient: vertical;
      overflow: hidden;
    }
    
    .mod-meta {
      display: flex;
      flex-direction: column;
      gap: 0.25rem;
    }
    
    .meta-item {
      display: flex;
      justify-content: space-between;
      font-size: 0.75rem;
      
      .meta-label {
        color: var(--text-muted);
      }
      
      .meta-value {
        color: var(--text);
        font-weight: 500;
      }
    }
  }

  .mod-actions {
    display: flex;
    gap: 0.5rem;
    
    .action-btn {
      flex: 1;
      padding: 0.5rem;
      border: none;
      border-radius: 0.5rem;
      font-size: 0.875rem;
      cursor: pointer;
      transition: all 0.2s ease;
      
      &:disabled {
        opacity: 0.5;
        cursor: not-allowed;
      }
      
      &.install-btn {
        background: var(--primary);
        color: white;
        
        &:hover:not(:disabled) {
          background: var(--primary-hover);
        }
      }
      
      &.uninstall-btn {
        background: var(--error);
        color: white;
        
        &:hover:not(:disabled) {
          background: var(--error-hover);
        }
      }
      
      &.info-btn {
        background: var(--surface-variant);
        color: var(--text);
        
        &:hover {
          background: var(--surface-hover);
        }
      }
    }
  }

  .no-results {
    padding: 3rem 1rem;
  }

  .empty-state {
    text-align: center;
    max-width: 400px;
    margin: 0 auto;
    
    .empty-icon {
      font-size: 4rem;
      margin-bottom: 1rem;
    }
    
    h3 {
      margin: 0 0 1rem 0;
      color: var(--text);
    }
    
    p {
      margin: 0;
      color: var(--text-muted);
      line-height: 1.5;
    }
  }

  .sign-in-btn {
    padding: 0.75rem 1.5rem;
    background: var(--primary);
    color: white;
    border: none;
    border-radius: 0.5rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    
    &:hover {
      background: var(--primary-hover);
      transform: translateY(-1px);
    }
  }
</style>
