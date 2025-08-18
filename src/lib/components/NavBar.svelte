<script lang="ts">
  import '$lib/styles/global.scss';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { currentAccount, AuthService, SettingsService, InstallationService, Icon, logsService, LogsService, IconService, WindowStateService, settings } from '$lib';
  import type { NavigationEventPayload, BehaviorChoiceEventPayload, GameRestartEventPayload } from '$lib/types';
  
  let isTauriReady = false;
  let initializationStatus = 'Initializing...';

  // Here we initialize all the required managers and services
  onMount(async () => {
    console.log('Starting layout initialization...');
    
    // Wait a bit for Tauri to fully initialize
    await new Promise(resolve => setTimeout(resolve, 50));
    
    try {
      // Test if Tauri is ready by making a simple call
      await InstallationService.loadInstallations();
      isTauriReady = true;
      
      // Initialize logs service first
      await logsService.initialize();
      LogsService.emitLauncherEvent('Kable launcher starting up...', 'info');
      
      // Initialize all services
      LogsService.emitLauncherEvent('Initializing launcher components...', 'info');
      
      await Promise.all([
        WindowStateService.initialize(), // Initialize window state first
        SettingsService.initialize(),
        AuthService.initialize(),
        IconService.initialize()
      ]);
      
      LogsService.emitLauncherEvent('All components initialized successfully', 'info');
      initializationStatus = 'Ready';
      
      console.log('Layout initialization complete');

      // Show the window now that initialization is complete
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('show_main_window');
      } catch (error) {
        console.error('Failed to show main window:', error);
      }

      // Set up settings behavior event listeners
      await setupSettingsEventListeners();
      
    } catch (error) {
      console.error('Tauri initialization error:', error);
      LogsService.emitLauncherEvent(`Initialization error: ${error}`, 'error');
      initializationStatus = `Initialization error: ${error}`;
      isTauriReady = false;
    }
  });

  // Set up event listeners for settings behavior
  async function setupSettingsEventListeners() {
    try {
      const { listen } = await import('@tauri-apps/api/event');
      
      // Navigation events
      await listen<NavigationEventPayload>('navigate-to-logs', (event) => {
        console.log('Navigating to logs due to settings:', event.payload);
        LogsService.emitLauncherEvent('Navigating to logs page due to game settings', 'info');
        goto('/logs');
      });

      await listen<NavigationEventPayload>('navigate-to-home', (event) => {
        console.log('Navigating to home due to settings:', event.payload);
        LogsService.emitLauncherEvent('Navigating to home page due to game settings', 'info');
        goto('/');
      });

      // User choice dialogs
      await listen<BehaviorChoiceEventPayload>('ask-launch-behavior', async (event) => {
        console.log('User choice requested for launch behavior:', event.payload);
        const choice = await showBehaviorDialog('Launch Behavior', 
          'What should happen when the game launches?', 
          event.payload.options);
        if (choice) {
          await handleUserChoice('on_game_launch', choice);
        }
      });

      await listen<BehaviorChoiceEventPayload>('ask-close-behavior', async (event) => {
        console.log('User choice requested for close behavior:', event.payload);
        const choice = await showBehaviorDialog('Close Behavior', 
          `What should happen now? (Game exited with code ${event.payload.exit_code})`, 
          event.payload.options);
        if (choice) {
          await handleUserChoice('on_game_close', choice);
        }
      });

      await listen<BehaviorChoiceEventPayload>('ask-crash-behavior', async (event) => {
        console.log('User choice requested for crash behavior:', event.payload);
        const choice = await showBehaviorDialog('Game Crashed', 
          `The game crashed (exit code ${event.payload.exit_code}). What should we do?`, 
          event.payload.options);
        if (choice) {
          await handleUserChoice('on_game_crash', choice);
        }
      });

      await listen<GameRestartEventPayload>('game-restart-requested', (event) => {
        console.log('Game restart requested:', event.payload);
        LogsService.emitLauncherEvent(`Game restart requested due to crash (exit code: ${event.payload.exit_code})`, 'warn');
        // TODO: Implement game restart functionality
        alert('Game restart feature is not implemented yet. Please launch manually.');
      });

      LogsService.emitLauncherEvent('Settings behavior event listeners initialized', 'info');
    } catch (error) {
      console.error('Failed to set up settings event listeners:', error);
      LogsService.emitLauncherEvent(`Failed to set up settings event listeners: ${error}`, 'error');
    }
  }

  // Show a dialog for user behavior choice
  async function showBehaviorDialog(title: string, message: string, options: string[]): Promise<string | null> {
    const optionLabels: Record<string, string> = {
      'keep_open': 'Keep Launcher Open',
      'exit': 'Close Launcher',
      'minimize': 'Minimize Launcher',
      'open_logs': 'Open Logs Page',
      'open_home': 'Go to Home Page',
      'restart': 'Restart Game',
      'close': 'Close Launcher',
      'ask': 'Ask Me Each Time'
    };

    const buttons = options.map(opt => optionLabels[opt] || opt);
    
    // Use browser's confirm for now - could be replaced with a custom modal
    if (options.length === 2) {
      const result = confirm(`${title}\n\n${message}\n\nClick OK for "${buttons[0]}" or Cancel for "${buttons[1]}"`);
      return result ? options[0] : options[1];
    } else {
      // For multiple options, show a simple prompt
      let promptMessage = `${title}\n\n${message}\n\nOptions:\n`;
      buttons.forEach((label, index) => {
        promptMessage += `${index + 1}. ${label}\n`;
      });
      promptMessage += '\nEnter the number of your choice:';
      
      const choice = prompt(promptMessage);
      const choiceIndex = parseInt(choice || '0') - 1;
      
      if (choiceIndex >= 0 && choiceIndex < options.length) {
        return options[choiceIndex];
      }
    }
    
    return null;
  }

  // Handle user's choice by executing the action
  async function handleUserChoice(settingType: string, choice: string) {
    LogsService.emitLauncherEvent(`User chose "${choice}" for ${settingType}`, 'info');
    
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const window = getCurrentWindow();
      
      switch (choice) {
        case 'exit':
        case 'close':
          await window.close();
          break;
        case 'minimize':
          await window.minimize();
          break;
        case 'open_logs':
          goto('/logs');
          break;
        case 'open_home':
          goto('/');
          break;
        case 'restart':
          LogsService.emitLauncherEvent('Game restart requested by user', 'info');
          alert('Game restart feature is not implemented yet. Please launch manually.');
          break;
        case 'keep_open':
          // Do nothing - keep launcher open
          LogsService.emitLauncherEvent('Keeping launcher open as requested', 'info');
          break;
        default:
          console.warn(`Unknown choice: ${choice}`);
      }
    } catch (error) {
      console.error('Error handling user choice:', error);
      LogsService.emitLauncherEvent(`Error handling user choice: ${error}`, 'error');
    }
  }

  // Navigation items - conditionally include logs based on settings
  $: navItems = [
    { path: '/', label: 'Home', icon: 'home' },
    { path: '/installations', label: 'Installations', icon: 'minecraft' },
    { path: '/mods', label: 'Mods', icon: 'mods' },
    { path: '/shaders', label: 'Shaders', icon: 'shaders' },
    { path: '/maps', label: 'Worlds', icon: 'maps' },
    { path: '/skins', label: 'Skins', icon: 'palette' },
    // Only show logs if enabled in settings (default: true for developers)
    ...($settings?.logging.show_logs_page_in_nav !== false ? [{ path: '/logs', label: 'Logs', icon: 'terminal' }] : [])
  ];

  // State for navigation collapse
  let isNavCollapsed = true;

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
          {#if $currentAccount?.avatar}
            <img src={$currentAccount.avatar} alt="User Avatar" class="avatar-image" />
          {:else}
            <Icon name="user" size="lg" />
          {/if}
        </div>
        {#if !isNavCollapsed}
          <div class="header-content">
            <h1 class="app-title">{$currentAccount?.username}</h1>
            <span class="app-subtitle">{!!($currentAccount?.access_token) ? 'Logged in' : 'Not logged in'}</span>
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
        <Icon name={isNavCollapsed ? 'arrow-right' : 'arrow-left'} size="lg" forceType="svg" />
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
    height: 100%;
    overflow: scroll;
    background: $background;
    color: $text;
  }

  .sidebar {
    min-width: calc(fit-content + 2rem);
    background: $container;
    border-right: 1px solid $dark-600;
    display: flex;
    flex-direction: column;
    padding: 0.25rem;
    transition: width 0.3s ease;
    resize: horizontal;
    
    &.collapsed {
      width: 3.5rem;
      
      .hamburger-btn {
        margin: 0 -0.25rem;
      }
      .header-section>.user-profile {
        margin: 0 0.25rem;
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
        & .user-avatar{
          background: $primary;
        }
      }
      
      .user-avatar {
        margin-top: 0.5rem;
        width: 2.5rem;
        height: 2.5rem;
        border-radius: 40%;
        background: rgba($primary, 0.1);
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        flex-shrink: 0;
        overflow: hidden;
        position: relative;
      }
      .user-avatar .avatar-image {
        width: 100%;
        height: 100%;
        object-fit: cover;
        border-radius: 40%;
        display: block;
        background: rgba($primary, 0.1);
      }
      
      .header-content {
        display: flex;
        flex-direction: column;
        min-width: 0;
        
        .app-title {
          margin: 0;
          font-size: 1rem;
          font-weight: 800;
          color: $primary;
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }
        
        .app-subtitle {
          font-size: 0.65rem;
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
      padding: 0 0.75rem;
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
        z-index: 9999;
      }
      
      &:hover::after {
        opacity: 1;
      }
    }
  }

  .bottom-section {
    margin-top: auto;
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
        z-index: 10;
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

