<script lang="ts">
  import { AuthManager } from '$lib';
  import { currentAccount } from '$lib/auth';
  import { onMount } from 'svelte';

  let searchQuery = '';
  let selectedCategory = 'all';
  let sortBy = 'popular';
  
  // Mock map data
  const categories = [
    { id: 'all', name: 'All Maps', icon: '🗺️' },
    { id: 'adventure', name: 'Adventure', icon: '⚔️' },
    { id: 'survival', name: 'Survival', icon: '🏠' },
    { id: 'creative', name: 'Creative', icon: '🎨' },
    { id: 'parkour', name: 'Parkour', icon: '🏃' },
    { id: 'puzzle', name: 'Puzzle', icon: '🧩' },
    { id: 'horror', name: 'Horror', icon: '👻' },
    { id: 'minigame', name: 'Minigame', icon: '🎮' }
  ];

  const mockMaps = [
    {
      id: 'the-dropper',
      name: 'The Dropper',
      description: 'A classic falling map where you must navigate through obstacles and land safely at the bottom.',
      author: 'Bigre',
      downloads: '15M',
      version: '1.19+',
      category: 'minigame',
      image: '/map-preview-dropper.jpg',
      players: '1-4',
      difficulty: 'Medium',
      installed: false,
      size: '25 MB'
    },
    {
      id: 'herobrine-mansion',
      name: 'Herobrine\'s Mansion',
      description: 'Explore the haunted mansion and discover the secrets of Herobrine in this spine-chilling adventure.',
      author: 'Hypixel',
      downloads: '8M',
      version: '1.20+',
      category: 'horror',
      image: '/map-preview-mansion.jpg',
      players: '1-8',
      difficulty: 'Hard',
      installed: true,
      size: '142 MB'
    },
    {
      id: 'skyblock',
      name: 'SkyBlock',
      description: 'Start with nothing but a tree and a chest. Build your island and survive in the void.',
      author: 'Noobcrew',
      downloads: '89M',
      version: '1.16+',
      category: 'survival',
      image: '/map-preview-skyblock.jpg',
      players: '1-∞',
      difficulty: 'Easy',
      installed: false,
      size: '5 MB'
    },
    {
      id: 'diversity-3',
      name: 'Diversity 3',
      description: 'The ultimate variety map featuring 10 unique branches with different gameplay styles.',
      author: 'qmagnet',
      downloads: '12M',
      version: '1.18+',
      category: 'adventure',
      image: '/map-preview-diversity.jpg',
      players: '1-2',
      difficulty: 'Hard',
      installed: false,
      size: '68 MB'
    }
  ];

  let filteredMaps = mockMaps;

  onMount(async () => {
    await AuthManager.initialize();
    updateFilter();
  });

  function updateFilter() {
    filteredMaps = mockMaps.filter(map => {
      const matchesSearch = map.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                           map.description.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesCategory = selectedCategory === 'all' || map.category === selectedCategory;
      return matchesSearch && matchesCategory;
    });

    // Sort maps
    if (sortBy === 'popular') {
      filteredMaps.sort((a, b) => parseFloat(b.downloads) - parseFloat(a.downloads));
    } else if (sortBy === 'name') {
      filteredMaps.sort((a, b) => a.name.localeCompare(b.name));
    } else if (sortBy === 'installed') {
      filteredMaps.sort((a, b) => (b.installed ? 1 : 0) - (a.installed ? 1 : 0));
    } else if (sortBy === 'size') {
      filteredMaps.sort((a, b) => parseFloat(a.size) - parseFloat(b.size));
    }
  }

  async function installMap(mapId: string) {
    try {
      console.log('Installing map:', mapId);
      const map = mockMaps.find(m => m.id === mapId);
      if (map) map.installed = true;
      updateFilter();
    } catch (error) {
      console.error('Failed to install map:', error);
    }
  }

  async function uninstallMap(mapId: string) {
    try {
      console.log('Uninstalling map:', mapId);
      const map = mockMaps.find(m => m.id === mapId);
      if (map) map.installed = false;
      updateFilter();
    } catch (error) {
      console.error('Failed to uninstall map:', error);
    }
  }

  function getDifficultyColor(difficulty: string) {
    switch (difficulty.toLowerCase()) {
      case 'easy': return 'var(--success)';
      case 'medium': return 'var(--warning)';
      case 'hard': return 'var(--error)';
      default: return 'var(--text-muted)';
    }
  }

  // React to changes
  $: searchQuery, selectedCategory, sortBy, updateFilter();
</script>

<div class="maps-page">
  <div class="page-header">
    <h1>Maps</h1>
    <p>Discover amazing worlds and adventures created by the community</p>
  </div>

  {#if !$currentAccount}
    <div class="auth-required">
      <div class="warning-card">
        <div class="warning-icon">🔒</div>
        <div class="warning-content">
          <h3>Authentication Required</h3>
          <p>Sign in with Microsoft to download and install custom maps</p>
          <button on:click={() => AuthManager.signIn()} class="sign-in-btn">
            Sign in with Microsoft
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Info Banner -->
  <section class="info-banner">
    <div class="banner-content">
      <div class="banner-icon">💡</div>
      <div class="banner-text">
        <h3>Map Installation</h3>
        <p>Maps are automatically installed to your saves folder. Make sure to backup your existing worlds before installing new maps.</p>
      </div>
    </div>
  </section>

  <!-- Search and Filters -->
  <section class="search-section">
    <div class="search-bar">
      <div class="search-input-wrapper">
        <span class="search-icon">🔍</span>
        <input 
          type="text" 
          placeholder="Search maps..." 
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
          <option value="size">File Size</option>
        </select>
      </div>
    </div>
  </section>

  <!-- Maps Grid -->
  <section class="maps-section">
    <div class="section-header">
      <h2>Available Maps ({filteredMaps.length})</h2>
      <div class="view-options">
        <button class="view-btn active">Grid</button>
        <button class="view-btn">List</button>
      </div>
    </div>

    {#if filteredMaps.length > 0}
      <div class="maps-grid">
        {#each filteredMaps as map}
          <div class="map-card" class:installed={map.installed}>
            <div class="map-preview">
              <div class="preview-placeholder">
                <span class="preview-icon">🖼️</span>
                <span class="preview-text">Screenshot</span>
              </div>
              
              <div class="map-badges">
                {#if map.installed}
                  <span class="badge installed">✅ Installed</span>
                {/if}
                <span class="badge category">{categories.find(c => c.id === map.category)?.icon} {categories.find(c => c.id === map.category)?.name}</span>
              </div>
            </div>
            
            <div class="map-content">
              <div class="map-header">
                <h3 class="map-name">{map.name}</h3>
                <div class="difficulty-indicator">
                  <span 
                    class="difficulty-dot" 
                    style="background-color: {getDifficultyColor(map.difficulty)}"
                  ></span>
                  <span class="difficulty-text">{map.difficulty}</span>
                </div>
              </div>
              
              <p class="map-description">{map.description}</p>
              
              <div class="map-stats">
                <div class="stat-item">
                  <span class="stat-icon">👥</span>
                  <span class="stat-value">{map.players}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-icon">📊</span>
                  <span class="stat-value">{map.size}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-icon">⬇️</span>
                  <span class="stat-value">{map.downloads}</span>
                </div>
              </div>
              
              <div class="map-meta">
                <div class="meta-item">
                  <span class="meta-label">Author:</span>
                  <span class="meta-value">{map.author}</span>
                </div>
                <div class="meta-item">
                  <span class="meta-label">Version:</span>
                  <span class="meta-value">{map.version}</span>
                </div>
              </div>
            </div>
            
            <div class="map-actions">
              {#if map.installed}
                <button 
                  on:click={() => uninstallMap(map.id)}
                  class="action-btn uninstall-btn"
                  disabled={!$currentAccount}
                >
                  🗑️ Remove
                </button>
                <button class="action-btn play-btn">
                  ▶️ Play
                </button>
              {:else}
                <button 
                  on:click={() => installMap(map.id)}
                  class="action-btn install-btn"
                  disabled={!$currentAccount}
                >
                  ⬇️ Download
                </button>
                <button class="action-btn info-btn">
                  ℹ️ Details
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="no-results">
        <div class="empty-state">
          <div class="empty-icon">🗺️</div>
          <h3>No maps found</h3>
          <p>Try adjusting your search criteria or browse different categories.</p>
        </div>
      </div>
    {/if}
  </section>

  <!-- My Maps Section -->
  <section class="my-maps">
    <h2>My Installed Maps</h2>
    
    {#if mockMaps.some(map => map.installed)}
      <div class="installed-maps">
        {#each mockMaps.filter(map => map.installed) as map}
          <div class="installed-map-item">
            <div class="map-icon">
              {categories.find(c => c.id === map.category)?.icon || '🗺️'}
            </div>
            <div class="map-info">
              <h4>{map.name}</h4>
              <p>{map.category} • {map.size}</p>
            </div>
            <div class="map-quick-actions">
              <button class="quick-btn play">▶️</button>
              <button class="quick-btn edit">✏️</button>
              <button class="quick-btn delete" on:click={() => uninstallMap(map.id)}>🗑️</button>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="no-installed-maps">
        <p>No maps installed yet. Download some maps to get started!</p>
      </div>
    {/if}
  </section>
</div>

<style lang="scss">
  .maps-page {
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

  .info-banner {
    margin-bottom: 2rem;
    
    .banner-content {
      display: flex;
      align-items: center;
      gap: 1rem;
      padding: 1.5rem;
      background: var(--info-light);
      border: 1px solid var(--info);
      border-radius: 1rem;
      
      .banner-icon {
        font-size: 2rem;
      }
      
      .banner-text {
        flex: 1;
        
        h3 {
          margin: 0 0 0.5rem 0;
          color: var(--info);
        }
        
        p {
          margin: 0;
          color: var(--text-muted);
        }
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

  .maps-section {
    margin-bottom: 3rem;
    
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

  .maps-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: 1.5rem;
  }

  .map-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 1rem;
    overflow: hidden;
    transition: all 0.2s ease;
    
    &:hover {
      transform: translateY(-2px);
      box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
    }
    
    &.installed {
      border-color: var(--success);
    }
  }

  .map-preview {
    position: relative;
    height: 200px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    
    .preview-placeholder {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 0.5rem;
      color: white;
      opacity: 0.8;
      
      .preview-icon {
        font-size: 2rem;
      }
      
      .preview-text {
        font-size: 0.875rem;
        font-weight: 500;
      }
    }
    
    .map-badges {
      position: absolute;
      top: 1rem;
      right: 1rem;
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }
    
    .badge {
      padding: 0.25rem 0.75rem;
      border-radius: 1rem;
      font-size: 0.75rem;
      font-weight: 500;
      backdrop-filter: blur(10px);
      
      &.installed {
        background: rgba(34, 197, 94, 0.9);
        color: white;
      }
      
      &.category {
        background: rgba(0, 0, 0, 0.7);
        color: white;
      }
    }
  }

  .map-content {
    padding: 1.5rem;
    
    .map-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 0.5rem;
      
      .map-name {
        margin: 0;
        color: var(--text);
        font-size: 1.125rem;
        font-weight: 600;
      }
      
      .difficulty-indicator {
        display: flex;
        align-items: center;
        gap: 0.25rem;
        
        .difficulty-dot {
          width: 8px;
          height: 8px;
          border-radius: 50%;
        }
        
        .difficulty-text {
          font-size: 0.75rem;
          color: var(--text-muted);
        }
      }
    }
    
    .map-description {
      margin: 0 0 1rem 0;
      color: var(--text-muted);
      font-size: 0.875rem;
      line-height: 1.5;
      display: -webkit-box;
      -webkit-line-clamp: 2;
      -webkit-box-orient: vertical;
      overflow: hidden;
    }
    
    .map-stats {
      display: flex;
      justify-content: space-around;
      margin-bottom: 1rem;
      padding: 0.75rem;
      background: var(--background);
      border-radius: 0.5rem;
    }
    
    .stat-item {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 0.25rem;
      
      .stat-icon {
        font-size: 1rem;
      }
      
      .stat-value {
        font-size: 0.75rem;
        color: var(--text);
        font-weight: 500;
      }
    }
    
    .map-meta {
      display: flex;
      flex-direction: column;
      gap: 0.25rem;
      margin-bottom: 1rem;
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

  .map-actions {
    padding: 0 1.5rem 1.5rem;
    display: flex;
    gap: 0.5rem;
    
    .action-btn {
      flex: 1;
      padding: 0.75rem;
      border: none;
      border-radius: 0.5rem;
      font-size: 0.875rem;
      font-weight: 500;
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
      
      &.play-btn {
        background: var(--success);
        color: white;
        
        &:hover {
          background: var(--success-hover);
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

  .my-maps {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 1rem;
    padding: 2rem;
    
    h2 {
      margin: 0 0 1.5rem 0;
      color: var(--text);
      font-size: 1.5rem;
    }
  }

  .installed-maps {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .installed-map-item {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: var(--background);
    border: 1px solid var(--border);
    border-radius: 0.75rem;
    transition: all 0.2s ease;
    
    &:hover {
      background: var(--surface-hover);
    }
    
    .map-icon {
      font-size: 1.5rem;
      width: 40px;
      height: 40px;
      display: flex;
      align-items: center;
      justify-content: center;
      background: var(--primary);
      color: white;
      border-radius: 0.5rem;
    }
    
    .map-info {
      flex: 1;
      
      h4 {
        margin: 0 0 0.25rem 0;
        color: var(--text);
        font-size: 1rem;
      }
      
      p {
        margin: 0;
        color: var(--text-muted);
        font-size: 0.875rem;
      }
    }
    
    .map-quick-actions {
      display: flex;
      gap: 0.5rem;
    }
    
    .quick-btn {
      width: 32px;
      height: 32px;
      border: none;
      border-radius: 0.5rem;
      display: flex;
      align-items: center;
      justify-content: center;
      cursor: pointer;
      transition: all 0.2s ease;
      
      &.play {
        background: var(--success);
        color: white;
        
        &:hover {
          background: var(--success-hover);
        }
      }
      
      &.edit {
        background: var(--primary);
        color: white;
        
        &:hover {
          background: var(--primary-hover);
        }
      }
      
      &.delete {
        background: var(--error);
        color: white;
        
        &:hover {
          background: var(--error-hover);
        }
      }
    }
  }

  .no-installed-maps {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
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
