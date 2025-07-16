<script lang="ts">
  import { AuthManager, currentAccount } from '$lib';
  import { onMount } from 'svelte';

  let searchQuery = '';
  let selectedCategory = 'all';
  let sortBy = 'popular';
  
  // Mock shader data
  const categories = [
    { id: 'all', name: 'All Shaders', icon: '✨' },
    { id: 'realistic', name: 'Realistic', icon: '🌅' },
    { id: 'fantasy', name: 'Fantasy', icon: '🔮' },
    { id: 'performance', name: 'Performance', icon: '⚡' },
    { id: 'cinematic', name: 'Cinematic', icon: '🎬' },
    { id: 'cartoon', name: 'Cartoon', icon: '🎨' }
  ];

  const mockShaders = [
    {
      id: 'seus-ptgi',
      name: 'SEUS PTGI',
      description: 'Sonic Ether\'s Unbelievable Shaders with Path Traced Global Illumination. The most advanced lighting system.',
      author: 'sonicether',
      downloads: '12M',
      version: 'E12',
      category: 'realistic',
      image: '/shader-preview-seus.jpg',
      performance: 'High-end',
      installed: false,
      premium: true
    },
    {
      id: 'bsl-shaders',
      name: 'BSL Shaders',
      description: 'A beautiful shader pack with customizable settings and excellent performance optimization.',
      author: 'capttatsu',
      downloads: '45M',
      version: 'v8.2.04',
      category: 'realistic',
      image: '/shader-preview-bsl.jpg',
      performance: 'Medium',
      installed: true,
      premium: false
    },
    {
      id: 'complementary',
      name: 'Complementary Shaders',
      description: 'Complementary Shaders is a shader pack for Minecraft that aims to provide great visuals while maintaining good performance.',
      author: 'EminGT',
      downloads: '23M',
      version: 'v4.7.2',
      category: 'performance',
      image: '/shader-preview-comp.jpg',
      performance: 'Good',
      installed: false,
      premium: false
    }
  ];

  let filteredShaders = mockShaders;

  onMount(async () => {
    await AuthManager.initialize();
    updateFilter();
  });

  function updateFilter() {
    filteredShaders = mockShaders.filter(shader => {
      const matchesSearch = shader.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                           shader.description.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesCategory = selectedCategory === 'all' || shader.category === selectedCategory;
      return matchesSearch && matchesCategory;
    });

    // Sort shaders
    if (sortBy === 'popular') {
      filteredShaders.sort((a, b) => parseFloat(b.downloads) - parseFloat(a.downloads));
    } else if (sortBy === 'name') {
      filteredShaders.sort((a, b) => a.name.localeCompare(b.name));
    } else if (sortBy === 'installed') {
      filteredShaders.sort((a, b) => (b.installed ? 1 : 0) - (a.installed ? 1 : 0));
    }
  }

  async function installShader(shaderId: string) {
    try {
      console.log('Installing shader:', shaderId);
      const shader = mockShaders.find(s => s.id === shaderId);
      if (shader) shader.installed = true;
      updateFilter();
    } catch (error) {
      console.error('Failed to install shader:', error);
    }
  }

  async function uninstallShader(shaderId: string) {
    try {
      console.log('Uninstalling shader:', shaderId);
      const shader = mockShaders.find(s => s.id === shaderId);
      if (shader) shader.installed = false;
      updateFilter();
    } catch (error) {
      console.error('Failed to uninstall shader:', error);
    }
  }

  function getPerformanceColor(performance: string) {
    switch (performance.toLowerCase()) {
      case 'high-end': return 'var(--error)';
      case 'medium': return 'var(--warning)';
      case 'good': return 'var(--success)';
      default: return 'var(--text-muted)';
    }
  }

  // React to changes
  $: searchQuery, selectedCategory, sortBy, updateFilter();
</script>

<div class="shaders-page">
  <div class="page-header">
    <h1>Shaders</h1>
    <p>Transform your Minecraft world with beautiful lighting and effects</p>
  </div>

  {#if !$currentAccount}
    <div class="auth-required">
      <div class="warning-card">
        <div class="warning-icon">🔒</div>
        <div class="warning-content">
          <h3>Authentication Required</h3>
          <p>Sign in with Microsoft to download and manage shader packs</p>
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
        <h3>OptiFine Required</h3>
        <p>Most shader packs require OptiFine or Iris to work properly. Make sure you have one installed before downloading shaders.</p>
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
          placeholder="Search shader packs..." 
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

  <!-- Shaders Grid -->
  <section class="shaders-section">
    <div class="section-header">
      <h2>Shader Packs ({filteredShaders.length})</h2>
      <div class="view-options">
        <button class="view-btn active">Grid</button>
        <button class="view-btn">List</button>
      </div>
    </div>

    {#if filteredShaders.length > 0}
      <div class="shaders-grid">
        {#each filteredShaders as shader}
          <div class="shader-card" class:installed={shader.installed} class:premium={shader.premium}>
            <div class="shader-preview">
              <div class="preview-placeholder">
                <span class="preview-icon">🖼️</span>
                <span class="preview-text">Preview</span>
              </div>
              
              <div class="shader-badges">
                {#if shader.premium}
                  <span class="badge premium">👑 Premium</span>
                {/if}
                {#if shader.installed}
                  <span class="badge installed">✅ Installed</span>
                {/if}
              </div>
            </div>
            
            <div class="shader-content">
              <div class="shader-header">
                <h3 class="shader-name">{shader.name}</h3>
                <div class="performance-indicator">
                  <span 
                    class="performance-dot" 
                    style="background-color: {getPerformanceColor(shader.performance)}"
                  ></span>
                  <span class="performance-text">{shader.performance}</span>
                </div>
              </div>
              
              <p class="shader-description">{shader.description}</p>
              
              <div class="shader-meta">
                <div class="meta-item">
                  <span class="meta-label">Author:</span>
                  <span class="meta-value">{shader.author}</span>
                </div>
                <div class="meta-item">
                  <span class="meta-label">Downloads:</span>
                  <span class="meta-value">{shader.downloads}</span>
                </div>
                <div class="meta-item">
                  <span class="meta-label">Version:</span>
                  <span class="meta-value">{shader.version}</span>
                </div>
              </div>
            </div>
            
            <div class="shader-actions">
              {#if shader.installed}
                <button 
                  on:click={() => uninstallShader(shader.id)}
                  class="action-btn uninstall-btn"
                  disabled={!$currentAccount}
                >
                  🗑️ Uninstall
                </button>
              {:else}
                <button 
                  on:click={() => installShader(shader.id)}
                  class="action-btn install-btn"
                  disabled={!$currentAccount}
                >
                  ⬇️ {shader.premium ? 'Purchase' : 'Download'}
                </button>
              {/if}
              
              <button class="action-btn preview-btn">
                👁️ Preview
              </button>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="no-results">
        <div class="empty-state">
          <div class="empty-icon">✨</div>
          <h3>No shader packs found</h3>
          <p>Try adjusting your search criteria or browse different categories.</p>
        </div>
      </div>
    {/if}
  </section>

  <!-- Settings Section -->
  <section class="shader-settings">
    <h2>Shader Settings</h2>
    <div class="settings-grid">
      <div class="setting-card">
        <h3>🔧 Shader Quality</h3>
        <p>Adjust shader quality based on your hardware</p>
        <select class="setting-select">
          <option>Ultra (RTX 4080+)</option>
          <option>High (RTX 3070+)</option>
          <option selected>Medium (GTX 1660+)</option>
          <option>Low (GTX 1050+)</option>
        </select>
      </div>
      
      <div class="setting-card">
        <h3>🌊 Water Quality</h3>
        <p>Control water reflection and transparency</p>
        <select class="setting-select">
          <option>Ultra</option>
          <option selected>High</option>
          <option>Medium</option>
          <option>Low</option>
        </select>
      </div>
      
      <div class="setting-card">
        <h3>☀️ Shadow Quality</h3>
        <p>Configure shadow resolution and distance</p>
        <select class="setting-select">
          <option>4096x4096</option>
          <option selected>2048x2048</option>
          <option>1024x1024</option>
          <option>512x512</option>
        </select>
      </div>
    </div>
  </section>
</div>

<style lang="scss">
  .shaders-page {
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

  .shaders-section {
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

  .shaders-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: 1.5rem;
  }

  .shader-card {
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
    
    &.premium {
      border-color: var(--accent);
    }
  }

  .shader-preview {
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
    
    .shader-badges {
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
      
      &.premium {
        background: rgba(255, 215, 0, 0.9);
        color: #333;
      }
      
      &.installed {
        background: rgba(34, 197, 94, 0.9);
        color: white;
      }
    }
  }

  .shader-content {
    padding: 1.5rem;
    
    .shader-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 0.5rem;
      
      .shader-name {
        margin: 0;
        color: var(--text);
        font-size: 1.125rem;
        font-weight: 600;
      }
      
      .performance-indicator {
        display: flex;
        align-items: center;
        gap: 0.25rem;
        
        .performance-dot {
          width: 8px;
          height: 8px;
          border-radius: 50%;
        }
        
        .performance-text {
          font-size: 0.75rem;
          color: var(--text-muted);
        }
      }
    }
    
    .shader-description {
      margin: 0 0 1rem 0;
      color: var(--text-muted);
      font-size: 0.875rem;
      line-height: 1.5;
      display: -webkit-box;
      line-clamp: 3;
      -webkit-line-clamp: 3;
      -webkit-box-orient: vertical;
      overflow: hidden;
    }
    
    .shader-meta {
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

  .shader-actions {
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
      
      &.preview-btn {
        background: var(--surface-variant);
        color: var(--text);
        
        &:hover {
          background: var(--surface-hover);
        }
      }
    }
  }

  .shader-settings {
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

  .settings-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1rem;
  }

  .setting-card {
    padding: 1.5rem;
    background: var(--background);
    border: 1px solid var(--border);
    border-radius: 0.75rem;
    
    h3 {
      margin: 0 0 0.5rem 0;
      color: var(--text);
      font-size: 1rem;
    }
    
    p {
      margin: 0 0 1rem 0;
      color: var(--text-muted);
      font-size: 0.875rem;
    }
    
    .setting-select {
      width: 100%;
      padding: 0.75rem;
      border: 1px solid var(--border);
      border-radius: 0.5rem;
      background: var(--surface);
      color: var(--text);
      cursor: pointer;
      
      &:focus {
        outline: none;
        border-color: var(--primary);
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
