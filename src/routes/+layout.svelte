<script lang="ts">
  import '$lib/styles/global.scss';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { AuthManager, SettingsManager, GameManager, Icon } from '$lib';
  
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
    { path: '/', label: 'Home', icon: 'home' },
    { path: '/settings', label: 'Settings', icon: 'settings' },
    { path: '/profile', label: 'Profile', icon: 'profile' },
    { path: '/mods', label: 'Mods', icon: 'mods' },
    { path: '/shaders', label: 'Shaders', icon: 'shaders' },
    { path: '/maps', label: 'Maps', icon: 'maps' }
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
          <Icon name={item.icon} size="md" className="nav-icon" />
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
  @use '@kablan/clean-ui/scss/variables' as *;

  .app-layout {
    display: flex;
    height: 100vh;
    background: $background;
    color: $text;
  }

  .sidebar {
    width: 250px;
    background: $container;
    border-right: 1px solid $dark-600;
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
      color: $primary;
    }
    
    .subtitle {
      font-size: 0.875rem;
      color: $placeholder;
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
    border-radius: $border-radius;
    text-decoration: none;
    color: $text;
    transition: all 0.2s ease;
    
    &:hover {
      background: $button-hover;
    }
    
    &.active {
      background: $primary;
      color: white;
    }
    
    :global(.nav-icon) {
      flex-shrink: 0;
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
    border-radius: $border-radius;
    background: $input;
    
    .status-indicator {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: $yellow;
      
      &.ready {
        background: $green;
      }
    }
    
    .status-text {
      font-size: 0.875rem;
      color: $placeholder;
    }
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
  }
</style>

