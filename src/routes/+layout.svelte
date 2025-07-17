<script lang="ts">
  import '$lib/styles/global.scss';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { AuthManager, SettingsManager, GameManager, Icon, logsService, LogsManager, IconManager, WindowStateManager, settings } from '$lib';

  let isTauriReady = false;
  let initializationStatus = 'Initializing...';

  onMount(async () => {
    console.log('Starting layout initialization...');
    
    // Wait a bit for Tauri to fully initialize
    await new Promise(resolve => setTimeout(resolve, 100));
    
    try {
      // Test if Tauri is ready by making a simple call
      await GameManager.getDefaultMinecraftDirectory();
      isTauriReady = true;
      
      // Initialize logs service first
      await logsService.initialize();
      LogsManager.emitLauncherEvent('Kable launcher starting up...', 'info');
      
      // Initialize all managers
      LogsManager.emitLauncherEvent('Initializing launcher components...', 'info');
      
      await Promise.all([
        WindowStateManager.initialize(), // Initialize window state first
        AuthManager.initialize(),
        SettingsManager.initialize(),
        GameManager.initialize(),
        IconManager.initialize()
      ]);
      
      LogsManager.emitLauncherEvent('All components initialized successfully', 'info');
      initializationStatus = 'Ready';
      
      console.log('Layout initialization complete');

      // Show the window now that initialization is complete
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('show_main_window');
      } catch (error) {
        console.error('Failed to show main window:', error);
      }
    } catch (error) {
      console.error('Tauri initialization error:', error);
      LogsManager.emitLauncherEvent(`Initialization error: ${error}`, 'error');
      initializationStatus = `Initialization error: ${error}`;
      isTauriReady = false;
    }
  });

  // Navigation items - conditionally include logs based on settings
  $: navItems = [
    { path: '/', label: 'Home', icon: 'home' },
    { path: '/installations', label: 'Installations', icon: 'minecraft' },
    { path: '/mods', label: 'Mods', icon: 'mods' },
    { path: '/shaders', label: 'Shaders', icon: 'shaders' },
    { path: '/maps', label: 'Maps', icon: 'maps' },
    { path: '/skins', label: 'Skins', icon: 'palette' },
    // Only show logs if enabled in settings (default: true for developers)
    ...($settings?.show_logs_page_in_nav !== false ? [{ path: '/logs', label: 'Logs', icon: 'terminal' }] : [])
  ];

  // State for navigation collapse
  let isNavCollapsed = false;

  function toggleNavigation() {
    isNavCollapsed = !isNavCollapsed;
  }

  // Handle keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    // Ctrl/Cmd + B to toggle navigation
    if ((event.ctrlKey || event.metaKey) && event.key === 'b') {
      event.preventDefault();
      toggleNavigation();
    }
  }

  $: currentPath = $page.url.pathname;
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="app-layout" class:nav-open={!isNavCollapsed} on:keydown={handleKeydown} role="application" tabindex="-1">
  <nav class="sidebar" class:collapsed={isNavCollapsed}>
    <!-- Header Section with Profile -->
    <div class="header-section">
      <a href="/profile" class="user-profile" class:active={currentPath === '/profile'}>
        <div class="user-avatar">
          <Icon name="user" size="lg" />
        </div>
        {#if !isNavCollapsed}
          <div class="header-content">
            <h1 class="app-title">Kable</h1>
            <span class="app-subtitle">A Minecraft Launcher</span>
          </div>
        {/if}
      </a>
    </div>

    <!-- Hamburger Toggle -->
    <div class="hamburger-section">
      <button 
        class="hamburger-btn" 
        on:click={toggleNavigation} 
        aria-label={isNavCollapsed ? 'Expand navigation' : 'Collapse navigation'}
        title={isNavCollapsed ? 'Expand navigation (Ctrl+B)' : 'Collapse navigation (Ctrl+B)'}
      >
        <Icon name="menu" size="lg" />
      </button>
    </div>
    
    <!-- Main Navigation -->
    <div class="nav-items">
      {#each navItems as item}
        <a 
          href={item.path} 
          class="nav-item" 
          class:active={currentPath === item.path}
          title={item.label}
        >
          <Icon name={item.icon} size="md" className="nav-icon" />
          {#if !isNavCollapsed}
            <span class="label">{item.label}</span>
          {/if}
        </a>
      {/each}
    </div>
    
    <!-- Settings at Bottom -->
    <div class="bottom-section">
      <a 
        href="/settings" 
        class="nav-item settings-item" 
        class:active={currentPath === '/settings'}
        title="Settings"
      >
        <Icon name="settings" size="md" className="nav-icon" />
        {#if !isNavCollapsed}
          <span class="label">Settings</span>
        {/if}
      </a>
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
    transition: width 0.3s ease;
    
    &.collapsed {
      width: 80px;
      
      .hamburger-btn {
        margin: 0 auto;
      }
    }
  }

  .header-section {
    margin-bottom: 1rem;
    
    .user-profile {
      margin-left: 0.5rem;
      display: flex;
      align-items: center;
      gap: 0.75rem;
      padding: 0;
      border-radius: $border-radius;
      text-decoration: none;
      color: $text;
      transition: all 0.2s ease;
      cursor: pointer;
      
      &:hover, &.active {
        
        padding: 0.75rem;
        margin: -0.75rem -0.25rem;
        & .user-avatar{
          background: $primary;
        }
      }
      
      .user-avatar {
        width: 40px;
        height: 40px;
        border-radius: 50%;
        background: rgba($primary, 0.1);
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        flex-shrink: 0;
      }
      
      .header-content {
        display: flex;
        flex-direction: column;
        min-width: 0;
        
        .app-title {
          margin: 0;
          font-size: 1.5rem;
          font-weight: 700;
          color: $primary;
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }
        
        .app-subtitle {
          font-size: 0.875rem;
          color: $placeholder;
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }
      }
    }
  }

  .hamburger-section {
    margin-bottom: 1.5rem;
    
    .hamburger-btn {
      background: transparent;
      border: none;
      border-radius: $border-radius;
      padding: 0.75rem;
      color: $text;
      cursor: pointer;
      transition: all 0.2s ease;
      display: flex;
      align-items: center;
      justify-content: center;
      
      &:hover {
        color: $primary;
        border-color: $primary;
      }
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
    position: relative;
    
    &:hover {
      background: $button-hover;
    }
    
    &.active {
      background: linear-gradient(155deg, rgba($primary, 0.15), rgba($primary, 0.01));
      backdrop-filter: blur(15px);
      color: white;
    }
    
    :global(.nav-icon) {
      flex-shrink: 0;
    }
    
    .label {
      font-weight: 500;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }
    
    // Collapsed state - center icons and show tooltips
    .sidebar.collapsed & {
      justify-content: center;
      padding: 0.75rem;
      
      .label {
        display: none;
      }
      
      // Tooltip on hover for collapsed state
      &::after {
        content: attr(title);
        position: absolute;
        left: 100%;
        top: 50%;
        transform: translateY(-50%);
        background: $container;
        color: $text;
        padding: 0.5rem 0.75rem;
        border-radius: $border-radius;
        font-size: 0.875rem;
        white-space: nowrap;
        opacity: 0;
        pointer-events: none;
        transition: opacity 0.2s ease;
        margin-left: 0.5rem;
        border: 1px solid $dark-600;
        z-index: 1000;
      }
      
      &:hover::after {
        opacity: 1;
      }
    }
  }

  .bottom-section {
    margin-top: auto;
    padding-top: 1rem;
    border-top: 1px solid $dark-600;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
    transition: margin-left 0.3s ease;
  }

  // Mobile responsive behavior
  @media (max-width: 768px) {
    .sidebar {
      position: fixed;
      left: 0;
      top: 0;
      height: 100vh;
      z-index: 1000;
      transform: translateX(-100%);
      transition: transform 0.3s ease;
      
      &:not(.collapsed) {
        transform: translateX(0);
      }
      
      &.collapsed {
        transform: translateX(0);
        width: 60px;
      }
    }
    
    .content {
      margin-left: 0;
      padding: 1rem;
    }
    
    .app-layout {
      &::before {
        content: '';
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.5);
        z-index: 999;
        opacity: 0;
        pointer-events: none;
        transition: opacity 0.3s ease;
      }
      
      &.nav-open::before {
        opacity: 1;
        pointer-events: auto;
      }
    }
  }
</style>

