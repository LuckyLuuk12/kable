<script lang="ts">
  import '$lib/styles/global.scss';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { AuthManager, SettingsManager, GameManager } from '$lib';
  
  let isTauriReady = false;
  let initializationStatus = 'Initializing...';

  onMount(async () => {
    // Wait a bit for Tauri to fully initialize
    await new Promise(resolve => setTimeout(resolve, 100));
    
    try {
      // Test if Tauri is ready by making a simple call
      await GameManager.getDefaultMinecraftDirectory();
      isTauriReady = true;
      
      // Initialize all managers
      await Promise.all([
        AuthManager.initialize(),
        SettingsManager.initialize(),
        GameManager.initialize()
      ]);
      
      initializationStatus = 'Ready';
    } catch (error) {
      console.error('Tauri initialization error:', error);
      initializationStatus = `Initialization error: ${error}`;
      isTauriReady = false;
    }
  });

  // Navigation items
  const navItems = [
    { path: '/', label: 'Home', icon: '🏠' },
    { path: '/settings', label: 'Settings', icon: '⚙️' },
    { path: '/profile', label: 'Profile', icon: '👤' },
    { path: '/mods', label: 'Mods', icon: '🧩' },
    { path: '/shaders', label: 'Shaders', icon: '✨' },
    { path: '/maps', label: 'Maps', icon: '🗺️' }
  ];

  $: currentPath = $page.url.pathname;
</script>

<div class="app-layout">
  <nav class="sidebar">
    <div class="logo">
      <h1>Kable</h1>
      <span class="subtitle">Minecraft Launcher</span>
    </div>
    
    <div class="nav-items">
      {#each navItems as item}
        <a 
          href={item.path} 
          class="nav-item" 
          class:active={currentPath === item.path}
        >
          <span class="icon">{item.icon}</span>
          <span class="label">{item.label}</span>
        </a>
      {/each}
    </div>
    
    <div class="status">
      <div class="status-indicator" class:ready={isTauriReady}></div>
      <span class="status-text">{initializationStatus}</span>
    </div>
  </nav>

  <main class="content">
    <slot />
  </main>
</div>

<style lang="scss">
  .app-layout {
    display: flex;
    height: 100vh;
    background: var(--background);
    color: var(--text);
  }

  .sidebar {
    width: 250px;
    background: var(--surface);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    padding: 1rem;
  }

  .logo {
    margin-bottom: 2rem;
    
    h1 {
      margin: 0;
      font-size: 1.5rem;
      font-weight: 700;
      color: var(--primary);
    }
    
    .subtitle {
      font-size: 0.875rem;
      color: var(--text-muted);
    }
  }

  .nav-items {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    border-radius: 0.5rem;
    text-decoration: none;
    color: var(--text);
    transition: all 0.2s ease;
    
    &:hover {
      background: var(--surface-hover);
    }
    
    &.active {
      background: var(--primary);
      color: white;
    }
    
    .icon {
      font-size: 1.25rem;
    }
    
    .label {
      font-weight: 500;
    }
  }

  .status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    border-radius: 0.5rem;
    background: var(--surface-variant);
    
    .status-indicator {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: var(--warning);
      
      &.ready {
        background: var(--success);
      }
    }
    
    .status-text {
      font-size: 0.875rem;
      color: var(--text-muted);
    }
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
  }
</style>

