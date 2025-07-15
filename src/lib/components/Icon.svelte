<script lang="ts">
  import { onMount } from 'svelte';
  import { IconManager, selectedTemplate } from '../managers/IconManager';

  export let name: string;
  export let provider: 'emoji' | 'fontawesome' | 'system' | 'auto' = 'auto';
  export let size: 'sm' | 'md' | 'lg' | 'xl' = 'md';
  export let className: string = '';

  let iconData: { icon: string; type: string; fallback: string } = { icon: '❓', type: 'emoji', fallback: '❓' };

  // Size mappings
  const sizeClasses = {
    sm: 'icon-sm',
    md: 'icon-md', 
    lg: 'icon-lg',
    xl: 'icon-xl'
  };

  // Initialize icon system on mount
  onMount(async () => {
    await IconManager.initialize();
    updateIcon();
  });

  // Update icon when name, provider changes, or when selected template changes
  $: if (name || $selectedTemplate) {
    updateIcon();
  }

  function updateIcon() {
    if (provider === 'auto') {
      // Use IconManager for dynamic template selection
      iconData = IconManager.getIcon(name);
    } else {
      // Use legacy static mappings for specific providers
      iconData = getLegacyIcon(name, provider);
    }
  }

  function getLegacyIcon(iconName: string, iconProvider: string): { icon: string; type: string; fallback: string } {
  // Define icon map types with index signatures
  type EmojiIconMap = { [key: string]: string };
  type FontAwesomeIconMap = { [key: string]: string };
  type SystemIconMap = { [key: string]: string };

  const iconMaps: {
    emoji: EmojiIconMap;
    fontawesome: FontAwesomeIconMap;
    system: SystemIconMap;
  } = {
    emoji: {
      // Navigation & UI
      home: '🏠',
      settings: '⚙️',
      profile: '👤',
      mods: '🧩',
      shaders: '✨',
      maps: '🗺️',
      
      // Actions
      search: '🔍',
      refresh: '🔄',
      download: '⬇️',
      upload: '⬆️',
      install: '📦',
      uninstall: '🗑️',
      edit: '✏️',
      duplicate: '📋',
      delete: '🗑️',
      trash: '🗑️',
      play: '▶️',
      launch: '▶️',
      info: 'ℹ️',
      preview: '👁️',
      backup: '💾',
      more: '•••',
      'more-horizontal': '•••',
      menu: '☰',
      hamburger: '☰',
      
      // Status & Indicators
      success: '✅',
      error: '❌',
      warning: '⚠️',
      alert: '⚠️',
      loading: '⏳',
      loader: '⏳',
      authenticated: '✅',
      lock: '🔒',
      unlock: '🔓',
      
      // Content Types
      folder: '📂',
      'folder-open': '📂',
      file: '📄',
      image: '🖼️',
      code: '💻',
      package: '📦',
      mod: '🧩',
      shader: '✨',
      map: '🗺️',
      
      // Categories
      technology: '⚙️',
      magic: '✨',
      adventure: '⚔️',
      decoration: '🎨',
      utility: '🔧',
      'world-gen': '🌍',
      survival: '🏠',
      creative: '🎨',
      parkour: '🏃',
      puzzle: '🧩',
      horror: '👻',
      minigame: '🎮',
      realistic: '🌅',
      fantasy: '🔮',
      performance: '⚡',
      cinematic: '🎬',
      cartoon: '🎨',
      
      // Players & Social
      player: '👤',
      players: '👥',
      avatar: '👤',
      crown: '👑',
      user: '👤',
      'user-plus': '👤➕',
      'qr-code': '📱',
      logout: '🚪',
      
      // System & Technical
      memory: '💾',
      java: '☕',
      terminal: '💻',
      bug: '🐛',
      analytics: '📊',
      debug: '🔧',
      coffee: '☕',
      database: '🗄️',
      wifi: '📶',
      target: '🎯',
      zap: '⚡',
      wrench: '🔧',
      
      // Minecraft Specific
      minecraft: '🎮',
      block: '🧱',
      pickaxe: '⛏️',
      sword: '⚔️',
      diamond: '💎',
      emerald: '💚',
      gold: '🟨',
      iron: '⚪',
      redstone: '🔴',
      world: '🌍',
      skull: '💀',
      fabric: '🧵',
      hammer: '🔨',
      cube: '🧊',
      
      // General UI
      close: '✖️',
      minimize: '📦',
      hide: '👻',
      browse: '📁',
      save: '💾',
      load: '📂',
      export: '📤',
      import: '📥',
      copy: '📋',
      paste: '📄',
      cut: '✂️',
      grid: '▦',
      list: '☰',
      layout: '🎛️',
      
      // Arrows & Navigation
      'arrow-up': '⬆️',
      'arrow-down': '⬇️',
      'arrow-left': '⬅️',
      'arrow-right': '➡️',
      'arrow-back': '🔙',
      'arrow-forward': '🔜',
      
      // Media & Effects
      volume: '🔊',
      mute: '🔇',
      brightness: '☀️',
      contrast: '🌓',
      color: '🎨',
      filter: '🎚️',
      palette: '🎨',
      
      // Networking
      online: '🌐',
      offline: '📴',
      sync: '🔄',
      cloud: '☁️',
      server: '🖥️',
      microsoft: 'Ⓜ️',
      
      // Time & Calendar
      time: '⏰',
      date: '📅',
      clock: '🕐',
      timer: '⏱️',
      calendar: '📅',
      chart: '📊',
      
      // Geography & Navigation
      compass: '🧭',
      eye: '👁️',
      
      // Weather & Environment
      sun: '☀️',
      moon: '🌙',
      star: '⭐',
      fire: '🔥',
      water: '💧',
      earth: '🌍',
      air: '💨'
    },
    
    fontawesome: {
      // Navigation & UI
      home: 'fas fa-home',
      settings: 'fas fa-cog',
      profile: 'fas fa-user',
      mods: 'fas fa-puzzle-piece',
      shaders: 'fas fa-magic',
      maps: 'fas fa-map',
      
      // Actions
      search: 'fas fa-search',
      refresh: 'fas fa-sync-alt',
      download: 'fas fa-download',
      upload: 'fas fa-upload',
      install: 'fas fa-box',
      uninstall: 'fas fa-trash',
      edit: 'fas fa-edit',
      duplicate: 'fas fa-copy',
      delete: 'fas fa-trash',
      trash: 'fas fa-trash',
      play: 'fas fa-play',
      launch: 'fas fa-rocket',
      info: 'fas fa-info-circle',
      preview: 'fas fa-eye',
      backup: 'fas fa-save',
      more: 'fas fa-ellipsis-h',
      'more-horizontal': 'fas fa-ellipsis-h',
      
      // Status & Indicators
      success: 'fas fa-check-circle',
      error: 'fas fa-times-circle',
      warning: 'fas fa-exclamation-triangle',
      alert: 'fas fa-exclamation-triangle',
      loading: 'fas fa-spinner fa-spin',
      loader: 'fas fa-circle-notch fa-spin',
      authenticated: 'fas fa-shield-check',
      lock: 'fas fa-lock',
      unlock: 'fas fa-unlock',
      
      // Content Types
      folder: 'fas fa-folder',
      'folder-open': 'fas fa-folder-open',
      file: 'fas fa-file',
      image: 'fas fa-image',
      code: 'fas fa-code',
      package: 'fas fa-box',
      
      // System & Technical
      memory: 'fas fa-memory',
      java: 'fab fa-java',
      terminal: 'fas fa-terminal',
      bug: 'fas fa-bug',
      analytics: 'fas fa-chart-bar',
      debug: 'fas fa-wrench',
      coffee: 'fas fa-coffee',
      database: 'fas fa-database',
      wifi: 'fas fa-wifi',
      target: 'fas fa-bullseye',
      zap: 'fas fa-bolt',
      wrench: 'fas fa-wrench',
      
      // Players & Social
      user: 'fas fa-user',
      'user-plus': 'fas fa-user-plus',
      'qr-code': 'fas fa-qrcode',
      logout: 'fas fa-sign-out-alt',
      
      // Minecraft Specific
      world: 'fas fa-globe',
      skull: 'fas fa-skull',
      fabric: 'fas fa-thread',
      hammer: 'fas fa-hammer',
      cube: 'fas fa-cube',
      
      // General UI
      grid: 'fas fa-th',
      list: 'fas fa-list',
      layout: 'fas fa-th-large',
      palette: 'fas fa-palette',
      
      // Geography & Navigation
      compass: 'fas fa-compass',
      eye: 'fas fa-eye',
      
      // Time & Calendar
      clock: 'fas fa-clock',
      calendar: 'fas fa-calendar-alt',
      chart: 'fas fa-chart-line',
      
      // Microsoft
      microsoft: 'fab fa-microsoft'
    },
    
    system: {
      // System-specific icons would be handled differently
      // This is a placeholder for system icons that might be loaded dynamically
      home: 'system-home',
      settings: 'system-settings',
      profile: 'system-user',
      folder: 'system-folder',
      file: 'system-file'
    }
  };

  // Size mappings
  const sizeClasses = {
    sm: 'icon-sm',
    md: 'icon-md', 
    lg: 'icon-lg',
    xl: 'icon-xl'
  };

  // Define allowed icon providers as a type
  type IconProvider = 'emoji' | 'fontawesome' | 'system';

  // Get the icon based on provider and name
  const icon =
    iconMaps[iconProvider as IconProvider]?.[iconName] ||
    iconMaps.emoji[iconName] ||
    '❓';

  return {
    icon,
    type: iconProvider,
    fallback: '❓'
  };
}

// Reactive statements
$: isEmoji = iconData.type === 'emoji';
$: isFontAwesome = iconData.type === 'fontawesome';
$: isSystem = iconData.type === 'system';
</script>

{#if isEmoji}
  <span class="icon icon-emoji {sizeClasses[size]} {className}" role="img" aria-label={name}>
    {iconData.icon}
  </span>
{:else if isFontAwesome}
  <i class="icon icon-fontawesome {iconData.icon} {sizeClasses[size]} {className}" aria-label={name}></i>
{:else if isSystem}
  <span class="icon icon-system {sizeClasses[size]} {className}" data-icon={iconData.icon} aria-label={name}>
    <!-- System icon placeholder - fallback to emoji -->
    {iconData.fallback}
  </span>
{:else}
  <!-- Custom template or unknown type - render as span -->
  <span class="icon icon-custom {sizeClasses[size]} {className}" role="img" aria-label={name}>
    {iconData.icon}
  </span>
{/if}

<style lang="scss">
  .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    
    &.icon-sm {
      font-size: 0.875rem;
      width: 1rem;
      height: 1rem;
    }
    
    &.icon-md {
      font-size: 1.25rem;
      width: 1.5rem;
      height: 1.5rem;
    }
    
    &.icon-lg {
      font-size: 1.75rem;
      width: 2rem;
      height: 2rem;
    }
    
    &.icon-xl {
      font-size: 2.5rem;
      width: 3rem;
      height: 3rem;
    }
    
    &.icon-emoji {
      font-family: 'Apple Color Emoji', 'Segoe UI Emoji', 'Noto Color Emoji', sans-serif;
    }
    
    &.icon-fontawesome {
      font-family: 'Font Awesome 6 Free', 'Font Awesome 6 Brands';
    }
    
    &.icon-system {
      // System icon specific styles
      color: currentColor;
    }
    
    &.icon-custom {
      // Custom template icon styles
      color: currentColor;
      font-family: inherit;
    }
  }
</style>
