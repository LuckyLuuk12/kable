import { writable, get } from 'svelte/store';
import { type CustomIconTemplate, SettingsService } from '$lib';

// Icon stores for reactive updates
export const selectedTemplate = writable<string>('emoji');
export const availableTemplates = writable<Array<{ name: string; displayName: string; type: 'builtin' | 'custom' }>>([]);
export const isIconsLoading = writable(false);
export const iconError = writable<string | null>(null);

export class IconService {
  private static customTemplates = new Map<string, CustomIconTemplate>();
  private static builtinTemplates = new Map<string, CustomIconTemplate>();
  private static initialized = false;
  
  /**
   * Initialize the icon system
   */
  static async initialize(): Promise<void> {
    if (this.initialized) return;
    
    isIconsLoading.set(true);
    iconError.set(null);
    
    try {
      // Setup built-in templates
      this.setupBuiltinTemplates();
      
      // Load custom templates from backend
      await this.loadCustomTemplates();
      
      // Load the selected template from settings and set it
      const settings = await SettingsService.getSettings();
      const templateName = settings.appearance.selected_icon_template || 'emoji';
      selectedTemplate.set(templateName);
      
      // Update available templates store
      this.updateAvailableTemplates();
      
      this.initialized = true;
      console.log('IconManager initialized with template:', templateName);
      
    } catch (error) {
      console.error('Failed to initialize icon system:', error);
      iconError.set(`Failed to initialize icons: ${error}`);
      // Fallback to emoji
      selectedTemplate.set('emoji');
    } finally {
      isIconsLoading.set(false);
    }
  }
  
  /**
   * Setup built-in icon templates
   */
  private static setupBuiltinTemplates(): void {
    // Emoji template with all icons from the component
    this.builtinTemplates.set('emoji', {
      name: 'emoji',
      displayName: 'Emoji Icons',
      version: '1.0.0',
      author: 'Kable Team',
      description: 'Default emoji-based icons',
      iconType: 'emoji',
      fallbackIcon: '❓',
      icons: {
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
        'eye-off': '🙈',
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
        irisfabric: '<svg viewBox="0 0 24 24" fill="none" stroke="#8e24aa" stroke-width="2" preserveAspectRatio="xMidYMid meet"><path d="M 12 3 L 13.53 5.12 L 15.85 4.62 L 15.38 7.09 L 17.39 8.09 L 15.85 10.15 L 17.39 12 L 15.85 13.85 L 17.39 15.91 L 15.38 16.91 L 15.85 19.38 L 13.53 18.88 L 12 21 L 10.47 18.88 L 8.15 19.38 L 8.62 16.91 L 6.61 15.91 L 8.15 13.85 L 6.61 12 L 8.15 10.15 L 6.61 8.09 L 8.62 7.09 L 8.15 4.62 L 10.47 5.12 Z" fill="none"/></svg>', // Purple circle for Iris Fabric
        hammer: '🔨',
        forge: '⚒️', // Forge/Anvil emoji
        neoforge: '🦊', // NeoForge, fox emoji
        cube: '🧊',
        iris: '🧶',
        
        // Players & Social
        player: '👤',
        players: '👥',
        avatar: '👤',
        crown: '👑',
        user: '👤',
        'user-plus': '👤➕',
        'qr-code': '📱',
        logout: '🚪',
        
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
        clipboard: '📋',
        archive: '🗃️',
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
        'chevron-up': '🔼',
        'chevron-down': '🔽',
        'chevron-left': '◀️',
        'chevron-right': '▶️',
        
        // Media & Effects
        volume: '🔊',
        mute: '🔇',
        brightness: '☀️',
        contrast: '🌓',
        color: '🎨',
        filter: '🎚️',
        
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
        timer: '⏱️',
        
        // Weather & Environment
        sun: '☀️',
        moon: '🌙',
        star: '⭐',
        fire: '🔥',
        water: '💧',
        air: '💨',
        
        // Additional common icons
        rocket: '🚀',
        check: '✅',
        stop: '⏹️',
        help: '❓',
        activity: '📊',
        globe: '🌐',
        gamepad: '🎮',
        
        // Additional icons
        plus: '➕',
        x: '✖️',
        clock: '🕐',
        calendar: '📅',
        chart: '📊',
        palette: '🎨',
        compass: '🧭',
        eye: '👁️',
        link: '🔗'
      }
    });
    
    // FontAwesome template
    this.builtinTemplates.set('fontawesome', {
      name: 'fontawesome',
      displayName: 'Font Awesome Icons',
      version: '6.0.0',
      author: 'Font Awesome Team',
      description: 'Professional icon set',
      iconType: 'fontawesome',
      fallbackIcon: 'fas fa-question-circle',
      icons: {
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
        forge: 'fas fa-anvil', // Forge/Anvil icon (FontAwesome 6+), fallback to hammer if not available
        neoforge: 'fas fa-fox', // NeoForge, fox icon (FontAwesome 6+), fallback to something similar if not available
        iris: 'fas fa-eye', // Use eye icon for Iris Fabric
        hammer: 'fas fa-hammer',
        cube: 'fas fa-cube',
        
        // General UI
        grid: 'fas fa-th',
        list: 'fas fa-list',
        layout: 'fas fa-th-large',
        palette: 'fas fa-palette',
        
        // Missing common icons
        clipboard: 'fas fa-clipboard',
        archive: 'fas fa-archive',
        'arrow-up': 'fas fa-arrow-up',
        'arrow-down': 'fas fa-arrow-down',
        'arrow-left': 'fas fa-arrow-left',
        'arrow-right': 'fas fa-arrow-right',
        'arrow-back': 'fas fa-arrow-left',
        'arrow-forward': 'fas fa-arrow-right',
        'chevron-up': 'fas fa-chevron-up',
        'chevron-down': 'fas fa-chevron-down',
        'chevron-left': 'fas fa-chevron-left',
        'chevron-right': 'fas fa-chevron-right',
        volume: 'fas fa-volume-up',
        mute: 'fas fa-volume-mute',
        brightness: 'fas fa-sun',
        contrast: 'fas fa-adjust',
        color: 'fas fa-palette',
        filter: 'fas fa-filter',
        online: 'fas fa-wifi',
        offline: 'fas fa-wifi-slash',
        sync: 'fas fa-sync',
        cloud: 'fas fa-cloud',
        server: 'fas fa-server',
        time: 'fas fa-clock',
        date: 'fas fa-calendar',
        timer: 'fas fa-stopwatch',
        sun: 'fas fa-sun',
        moon: 'fas fa-moon',
        star: 'fas fa-star',
        fire: 'fas fa-fire',
        water: 'fas fa-tint',
        air: 'fas fa-wind',
        rocket: 'fas fa-rocket',
        check: 'fas fa-check',
        stop: 'fas fa-stop',
        help: 'fas fa-question',
        activity: 'fas fa-chart-line',
        globe: 'fas fa-globe',
        gamepad: 'fas fa-gamepad',
        
        // Additional icons
        plus: 'fas fa-plus',
        x: 'fas fa-times',
        clock: 'fas fa-clock',
        calendar: 'fas fa-calendar-alt',
        chart: 'fas fa-chart-line',
        compass: 'fas fa-compass',
        eye: 'fas fa-eye',
        microsoft: 'fab fa-microsoft'
      }
    });
    
    // SVG template with clean, minimal icons
    this.builtinTemplates.set('svg', {
      name: 'svg',
      displayName: 'SVG Icons',
      version: '1.0.0',
      author: 'Kable Team',
      description: 'Clean SVG-based icons',
      iconType: 'svg',
      fallbackIcon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M9,9h6v6H9z"/></svg>',
      icons: {
        // Navigation & UI
        home: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><polyline points="9,22 9,12 15,12 15,22"/></svg>',
        settings: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M12 1v6m0 6v6m11-7h-6m-6 0H1m15.5-7.5L19 7l-1.5 1.5m-11 11L5 17l1.5-1.5M6.5 6.5 5 7l1.5 1.5"/></svg>',
        profile: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>',
        mods: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2v6.5m0 7V22M8 8l4-4 4 4m-8 8l4 4 4-4"/></svg>',
        shaders: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="12,2 22,8.5 22,15.5 12,22 2,15.5 2,8.5"/><path d="M12 8.5v7"/></svg>',
        maps: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="1,6 1,22 8,18 16,22 23,18 23,2 16,6 8,2"/><line x1="8" y1="2" x2="8" y2="18"/><line x1="16" y1="6" x2="16" y2="22"/></svg>',
        
        // Actions
        search: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/></svg>',
        refresh: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M3 21v-5h5"/></svg>',
        download: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7,10 12,15 17,10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>',
        upload: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17,8 12,3 7,8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>',
        install: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 16h2a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"/><polyline points="8,12 12,16 16,12"/><line x1="12" y1="16" x2="12" y2="8"/></svg>',
        uninstall: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3,6 5,6 21,6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>',
        edit: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>',
        duplicate: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>',
        delete: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3,6 5,6 21,6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></svg>',
        trash: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3,6 5,6 21,6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></svg>',
        play: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5,3 19,12 5,21"/></svg>',
        launch: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 3h6v6"/><path d="m10 14 11-11"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/></svg>',
        info: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>',
        preview: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>',
        'eye-off': '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.48 18.48 0 0 1 5.06-5.94"/><path d="M1 1l22 22"/><path d="M14.12 14.12a3 3 0 0 1-4.24-4.24"/></svg>',
        backup: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17,21 17,13 7,13 7,21"/><polyline points="7,3 7,8 15,8"/></svg>',
        more: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/></svg>',
        'more-horizontal': '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/></svg>',
        menu: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="3" y1="6" x2="21" y2="6"/><line x1="3" y1="12" x2="21" y2="12"/><line x1="3" y1="18" x2="21" y2="18"/></svg>',
        hamburger: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="3" y1="6" x2="21" y2="6"/><line x1="3" y1="12" x2="21" y2="12"/><line x1="3" y1="18" x2="21" y2="18"/></svg>',
        
        // Status & Indicators
        success: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22,4 12,14.01 9,11.01"/></svg>',
        error: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>',
        warning: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>',
        alert: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>',
        loading: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg>',
        loader: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg>',
        authenticated: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 12l2 2 4-4"/><path d="M21 12c-1 0-3-1-3-3s2-3 3-3 3 1 3 3-2 3-3 3"/><path d="M3 12c1 0 3-1 3-3s-2-3-3-3-3 1-3 3 2 3 3 3"/><path d="M3 12h6m6 0h6"/></svg>',
        lock: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><circle cx="12" cy="16" r="1"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>',
        unlock: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><circle cx="12" cy="16" r="1"/><path d="M7 11V7a5 5 0 0 1 9.9-1"/></svg>',
        
        // Arrows & Navigation
        'arrow-up': '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="19" x2="12" y2="5"/><polyline points="5,12 12,5 19,12"/></svg>',
        'arrow-down': '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><polyline points="19,12 12,19 5,12"/></svg>',
        'arrow-left': '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="19" y1="12" x2="5" y2="12"/><polyline points="12,19 5,12 12,5"/></svg>',
        'arrow-right': '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"/><polyline points="12,5 19,12 12,19"/></svg>',
        'arrow-back': '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="19" y1="12" x2="5" y2="12"/><polyline points="12,19 5,12 12,5"/></svg>',
        'arrow-forward': '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"/><polyline points="12,5 19,12 12,19"/></svg>',
        'chevron-up': '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="18,15 12,9 6,15"/></svg>',
        'chevron-down': '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6,9 12,15 18,9"/></svg>',
        'chevron-left': '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15,18 9,12 15,6"/></svg>',
        'chevron-right': '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9,18 15,12 9,6"/></svg>',
        
        // Content Types
        folder: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>',
        'folder-open': '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 14l1.45-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.55 6a2 2 0 0 1-1.94 1.5H4a2 2 0 0 1-2-2V5c0-1.1.9-2 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H18a2 2 0 0 1 2 2v2"/></svg>',
        file: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14,2 14,8 20,8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10,9 9,9 8,9"/></svg>',
        image: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21,15 16,10 5,21"/></svg>',
        code: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16,18 22,12 16,6"/><polyline points="8,6 2,12 8,18"/></svg>',
        package: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="16.5" y1="9.4" x2="7.5" y2="4.21"/><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27,6.96 12,12.01 20.73,6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>',
        mod: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2v6.5m0 7V22M8 8l4-4 4 4m-8 8l4 4 4-4"/></svg>',
        shader: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="12,2 22,8.5 22,15.5 12,22 2,15.5 2,8.5"/><path d="M12 8.5v7"/></svg>',
        map: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="1,6 1,22 8,18 16,22 23,18 23,2 16,6 8,2"/><line x1="8" y1="2" x2="8" y2="18"/><line x1="16" y1="6" x2="16" y2="22"/></svg>',
        
        // System & Technical
        memory: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>',
        java: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 2v6h4V2"/><path d="M6 8v8a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V8"/><circle cx="12" cy="12" r="2"/></svg>',
        terminal: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="4,17 10,11 4,5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>',
        bug: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 2v4"/><path d="M16 2v4"/><path d="M21 9.5c0 .28-.22.5-.5.5H20v6c0 2.76-2.24 5-5 5h-6c-2.76 0-5-2.24-5-5v-6H3.5c-.28 0-.5-.22-.5-.5S3.22 9 3.5 9H4V7c0-1.1.9-2 2-2h12c1.1 0 2 .9 2 2v2h.5c.09.37.13.73.13 1.1h0c.09.37.24.72.44 1.03l.15.24c.13.21.28.4.45.57l.05.04c.18.17.38.31.6.42l.3.14c.28.12.49.3.49.52C21.5 6.5 18.09 4 14 4s-7.5 2.5-7.5 6c0 .22.21.4.49.52l.3.14c.22.11.42.25.6.42l.05.04c.17.17.32.36.45.57l.15.24c.2.31.35.66.44 1.03h0c.09.37.13.75.13 1.1Z"/><circle cx="9" cy="12" r="1"/><circle cx="15" cy="12" r="1"/></svg>',
        analytics: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 3v18h18"/><path d="m19 9-5 5-4-4-3 3"/></svg>',
        debug: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/></svg>',
        coffee: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 8h1a4 4 0 0 1 0 8h-1"/><path d="M2 8h16v9a4 4 0 0 1-4 4H6a4 4 0 0 1-4-4V8z"/><line x1="6" y1="1" x2="6" y2="4"/><line x1="10" y1="1" x2="10" y2="4"/><line x1="14" y1="1" x2="14" y2="4"/></svg>',
        database: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14c0 1.66 4.03 3 9 3s9-1.34 9-3V5"/><path d="M3 12c0 1.66 4.03 3 9 3s9-1.34 9-3"/></svg>',
        wifi: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12.55a11 11 0 0 1 14.08 0"/><path d="M1.42 9a16 16 0 0 1 21.16 0"/><path d="M8.53 16.11a6 6 0 0 1 6.95 0"/><line x1="12" y1="20" x2="12.01" y2="20"/></svg>',
        target: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="6"/><circle cx="12" cy="12" r="2"/></svg>',
        zap: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="13,2 3,14 12,14 11,22 21,10 12,10"/></svg>',
        wrench: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/></svg>',
        
        // Minecraft Specific
        minecraft: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="2" ry="2"/><rect x="9" y="9" width="6" height="6"/><line x1="9" y1="1" x2="15" y2="1"/><line x1="9" y1="23" x2="15" y2="23"/><line x1="1" y1="9" x2="1" y2="15"/><line x1="23" y1="9" x2="23" y2="15"/></svg>',
        block: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><rect x="7" y="7" width="10" height="10"/></svg>',
        pickaxe: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 14l8-8"/><path d="M18 6l4 4"/><path d="M8 10l-4 4 6 6 4-4"/></svg>',
        sword: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="14.5,17.5 3,6 3,3 6,3 17.5,14.5"/><line x1="13" y1="19" x2="19" y2="13"/><line x1="16" y1="16" x2="20" y2="20"/><line x1="19" y1="21" x2="21" y2="19"/></svg>',
        diamond: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 3h12l4 6-10 12L2 9z"/><path d="M11 3 8 9l4 12 4-12-3-6"/><path d="M2 9h20"/></svg>',
        emerald: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 3h12l4 6-10 12L2 9z"/><path d="M11 3 8 9l4 12 4-12-3-6"/><path d="M2 9h20"/></svg>',
        gold: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="8"/><circle cx="12" cy="12" r="3"/></svg>',
        iron: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/></svg>',
        redstone: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="7.5,4.21 12,6.81 16.5,4.21"/><polyline points="7.5,19.79 7.5,14.6 3,12"/><polyline points="21,12 16.5,14.6 16.5,19.79"/><polyline points="3.27,6.96 12,12.01 20.73,6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>',
        world: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>',
        skull: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 14v.5c0 .75.25 1.46.7 2.05l.54.67c.07.09.15.18.24.24l.07.05c.12.08.25.14.38.17l2.81.9c.36.1.75.17 1.14.14.39-.03.76-.14 1.08-.33l.04-.02c.24-.13.44-.31.57-.52l.67-.84c.45-.59.7-1.3.7-2.05v-.06c0-.39.04-.77.13-1.14h0c.09-.37.24-.72.44-1.03l.15-.24c.13-.21.28-.4.45-.57l.05-.04c.18-.17.38-.31.6-.42l.3-.14c.28-.12.49-.3.49-.52C21.5 6.5 18.09 4 14 4s-7.5 2.5-7.5 6c0 .22.21.4.49.52l.3.14c.22.11.42.25.6.42l.05.04c.17.17.32.36.45.57l.15.24c.2.31.35.66.44 1.03h0c.09.37.13.75.13 1.14Z"/><circle cx="9" cy="12" r="1"/><circle cx="15" cy="12" r="1"/></svg>',
        fabric: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2L2 7v10l10 5 10-5V7z"/><polyline points="2,7 12,12 22,7"/><polyline points="12,22 12,12"/></svg>',
        // Hollow circle with zigzag/accordion-bellows-like edges for Iris Fabric
        iris: '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><ellipse cx="12" cy="12" rx="10" ry="10"/><ellipse cx="12" cy="12" rx="5" ry="5"/><ellipse cx="12" cy="12" rx="2" ry="2"/></svg>',
        hammer: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 12l-8.5 8.5c-.83.83-2.17.83-3 0 0 0 0 0 0 0-.83-.83-.83-2.17 0-3L12 9"/><path d="M17.64 15L22 10.64"/><path d="M20.05 11.38L12.67 4l-2.05 2.05 7.38 7.38z"/></svg>',
        cube: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="7.5,4.21 12,6.81 16.5,4.21"/><polyline points="7.5,19.79 7.5,14.6 3,12"/><polyline points="21,12 16.5,14.6 16.5,19.79"/><polyline points="3.27,6.96 12,12.01 20.73,6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>',
        forge: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="17" width="18" height="4" rx="1"/><rect x="7" y="13" width="10" height="4" rx="1"/><rect x="10" y="9" width="4" height="4" rx="1"/></svg>', // Anvil SVG
        neoforge: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 20c0-4 4-8 8-8s8 4 8 8"/><path d="M8 20c0-2 2-4 4-4s4 2 4 4"/><path d="M12 12c-2-2-2-6 0-8 2 2 2 6 0 8z"/><path d="M12 12c2-2 2-6 0-8"/><circle cx="12" cy="16" r="1"/><circle cx="9" cy="18" r="1"/><circle cx="15" cy="18" r="1"/></svg>', // Fox SVG

        // Players & Social
        player: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>',
        players: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>',
        avatar: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>',
        crown: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M2 18h20l-2-12-3 7-5-7-5 7-3-7z"/><path d="M7 18v2"/><path d="M12 18v2"/><path d="M17 18v2"/></svg>',
        user: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>',
        'user-plus': '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><line x1="20" y1="8" x2="20" y2="14"/><line x1="23" y1="11" x2="17" y2="11"/></svg>',
        'qr-code': '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="5" height="5"/><rect x="16" y="3" width="5" height="5"/><rect x="3" y="16" width="5" height="5"/><line x1="21" y1="16" x2="16" y2="21"/><line x1="21" y1="21" x2="16" y2="16"/><line x1="14" y1="14" x2="15" y2="15"/><line x1="10" y1="14" x2="15" y2="14"/><line x1="12" y1="16" x2="13" y2="17"/><line x1="10" y1="16" x2="11" y2="17"/><line x1="12" y1="18" x2="12" y2="19"/></svg>',
        logout: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16,17 21,12 16,7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>',
        
        // General UI
        close: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>',
        minimize: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"/></svg>',
        hide: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9.88 9.88 3 5.73 3-5.73"/><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>',
        browse: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>',
        save: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17,21 17,13 7,13 7,21"/><polyline points="7,3 7,8 15,8"/></svg>',
        load: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>',
        export: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17,8 12,3 7,8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>',
        import: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7,10 12,15 17,10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>',
        copy: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>',
        paste: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="8" y="2" width="8" height="4" rx="1" ry="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/></svg>',
        cut: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="6" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><line x1="20" y1="4" x2="8.12" y2="15.88"/><line x1="14.47" y1="14.48" x2="20" y2="20"/><line x1="8.12" y1="8.12" x2="12" y2="12"/></svg>',
        clipboard: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="8" y="2" width="8" height="4" rx="1" ry="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/></svg>',
        archive: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="21,8 21,21 3,21 3,8"/><rect x="1" y="3" width="22" height="5"/><line x1="10" y1="12" x2="14" y2="12"/></svg>',
        grid: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>',
        list: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>',
        layout: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><line x1="9" y1="9" x2="21" y2="9"/><line x1="9" y1="21" x2="9" y2="9"/></svg>',
        plus: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>',
        x: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>',
        check: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20,6 9,17 4,12"/></svg>',
        clock: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12,6 12,12 16,14"/></svg>',
        calendar: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>',
        chart: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 3v18h18"/><path d="m19 9-5 5-4-4-3 3"/></svg>',
        palette: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="13.5" cy="6.5" r=".5"/><circle cx="17.5" cy="10.5" r=".5"/><circle cx="8.5" cy="7.5" r=".5"/><circle cx="6.5" cy="12.5" r=".5"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"/></svg>',
        compass: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polygon points="16.24,7.76 14.12,14.12 7.76,16.24 9.88,9.88"/></svg>',
        help: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>',
        eye: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>',
        
        // Media & Effects
        volume: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="11,5 6,9 2,9 2,15 6,15 11,19"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"/></svg>',
        mute: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="11,5 6,9 2,9 2,15 6,15 11,19"/><line x1="23" y1="9" x2="17" y2="15"/><line x1="17" y1="9" x2="23" y2="15"/></svg>',
        brightness: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/></svg>',
        contrast: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 2v20"/></svg>',
        color: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="13.5" cy="6.5" r=".5"/><circle cx="17.5" cy="10.5" r=".5"/><circle cx="8.5" cy="7.5" r=".5"/><circle cx="6.5" cy="12.5" r=".5"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"/></svg>',
        filter: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="22,3 2,3 10,12.46 10,19 14,21 14,12.46"/></svg>',
        
        // Networking
        online: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12.55a11 11 0 0 1 14.08 0"/><path d="M1.42 9a16 16 0 0 1 21.16 0"/><path d="M8.53 16.11a6 6 0 0 1 6.95 0"/><line x1="12" y1="20" x2="12.01" y2="20"/></svg>',
        offline: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="1" y1="1" x2="23" y2="23"/><path d="M16.72 11.06A10.94 10.94 0 0 1 19 12.55"/><path d="M5 12.55a10.94 10.94 0 0 1 5.17-2.39"/><path d="M10.71 5.05A16 16 0 0 1 22.58 9"/><path d="M1.42 9a15.91 15.91 0 0 1 4.7-2.88"/><path d="M8.53 16.11a6 6 0 0 1 6.95 0"/><line x1="12" y1="20" x2="12.01" y2="20"/></svg>',
        sync: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M3 21v-5h5"/></svg>',
        cloud: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 10h1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z"/></svg>',
        server: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"/><rect x="2" y="14" width="20" height="8" rx="2" ry="2"/><line x1="6" y1="6" x2="6.01" y2="6"/><line x1="6" y1="18" x2="6.01" y2="18"/></svg>',
        microsoft: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="8" height="8"/><rect x="13" y="3" width="8" height="8"/><rect x="3" y="13" width="8" height="8"/><rect x="13" y="13" width="8" height="8"/></svg>',
        
        // Time & Calendar
        time: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12,6 12,12 16,14"/></svg>',
        date: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>',
        timer: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="13" r="8"/><path d="M12 9v4l2 2"/><path d="M5 3 2 6"/><path d="m22 6-3-3"/><path d="M6.38 18.7 4 21"/><path d="M17.64 18.67 20 21"/></svg>',
        
        // Weather & Environment
        sun: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/></svg>',
        moon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>',
        star: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="12,2 15.09,8.26 22,9.27 17,14.14 18.18,21.02 12,17.77 5.82,21.02 7,14.14 2,9.27 8.91,8.26"/></svg>',
        fire: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z"/></svg>',
        water: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z"/></svg>',
        air: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17.7 7.7a2.5 2.5 0 1 1 1.8 4.3H2"/><path d="M9.6 4.6A2 2 0 1 1 11 8H2"/><path d="M12.6 19.4A2 2 0 1 0 14 16H2"/></svg>',
        
        // Additional common icons
        rocket: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/><path d="M12 15l-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"/><path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/></svg>',
        stop: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="6" y="6" width="12" height="12" rx="2" ry="2"/></svg>',
        activity: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="22,12 18,12 15,21 9,3 6,12 2,12"/></svg>',
        globe: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>',
        gamepad: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="6" y1="11" x2="10" y2="11"/><line x1="8" y1="9" x2="8" y2="13"/><line x1="15" y1="12" x2="15.01" y2="12"/><line x1="18" y1="10" x2="18.01" y2="10"/><path d="M17.32 5H6.68a4 4 0 0 0-3.978 3.59c-.006.052-.01.101-.017.152C2.604 9.416 2 14.456 2 16a3 3 0 0 0 3 3c1 0 1.5-.5 2-1l1.414-1.414A2 2 0 0 1 9.828 16h4.344a2 2 0 0 1 1.414.586L17 18c.5.5 1 1 2 1a3 3 0 0 0 3-3c0-1.544-.604-6.584-.685-7.258-.007-.05-.011-.1-.017-.151A4 4 0 0 0 17.32 5z"/></svg>'
      }
    });
  }
  
  /**
   * Load custom templates from backend
   */
  private static async loadCustomTemplates(): Promise<void> {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const templates = await invoke<CustomIconTemplate[]>('get_custom_icon_templates');
      
      this.customTemplates.clear();
      templates.forEach(template => {
        this.customTemplates.set(template.name, template);
      });
      
    } catch (error) {
      console.warn('Failed to load custom icon templates:', error);
    }
  }
  
  /**
   * Update available templates store
   */
  private static updateAvailableTemplates(): void {
    const templates = [];
    
    // Add built-in templates
    for (const [name, template] of this.builtinTemplates) {
      templates.push({
        name,
        displayName: template.displayName,
        type: 'builtin' as const
      });
    }
    
    // Add custom templates
    for (const [name, template] of this.customTemplates) {
      templates.push({
        name,
        displayName: template.displayName,
        type: 'custom' as const
      });
    }
    
    availableTemplates.set(templates);
  }
  
  /**
   * Get the currently active template
   */
  static getActiveTemplate(): CustomIconTemplate | null {
    const templateName = get(selectedTemplate);
    
    // Check custom templates first
    if (this.customTemplates.has(templateName)) {
      return this.customTemplates.get(templateName)!;
    }
    
    // Check built-in templates
    if (this.builtinTemplates.has(templateName)) {
      return this.builtinTemplates.get(templateName)!;
    }
    
    // Fallback to emoji
    return this.builtinTemplates.get('emoji')!;
  }
  
  /**
   * Get icon from active template
   */
  static getIcon(iconName: string): { icon: string; type: string; fallback: string } {
    const template = this.getActiveTemplate();
    if (!template) {
      return { icon: '❓', type: 'emoji', fallback: '❓' };
    }
    
    const icon = template.icons[iconName] || template.fallbackIcon;
    return {
      icon,
      type: template.iconType,
      fallback: template.fallbackIcon
    };
  }

  /**
   * Get icon with priority: custom template first, then forced type for missing icons
   */
  static getIconWithFallback(iconName: string, forceType: 'emoji' | 'fontawesome' | 'svg' | 'system' | 'css' | null = null): { icon: string; type: string; fallback: string } {
    const activeTemplate = this.getActiveTemplate();
    const templateName = get(selectedTemplate);
    
    // Check if we have a CUSTOM template (not built-in) with the specific icon
    const isCustomTemplate = this.customTemplates.has(templateName);
    if (isCustomTemplate && activeTemplate && activeTemplate.icons[iconName]) {
      // Icon found in custom template - this always takes highest priority
      return {
        icon: activeTemplate.icons[iconName],
        type: activeTemplate.iconType,
        fallback: activeTemplate.fallbackIcon
      };
    }
    
    // forceType takes priority over built-in templates
    if (forceType) {
      const forcedIcon = this.getDefaultIcon(iconName, forceType);
      return {
        icon: forcedIcon,
        type: forceType,
        fallback: this.getDefaultIcon(iconName, 'emoji')
      };
    }
    
    // No forceType specified - use active template (built-in or custom)
    if (activeTemplate) {
      const icon = activeTemplate.icons[iconName] || activeTemplate.fallbackIcon;
      return {
        icon,
        type: activeTemplate.iconType,
        fallback: activeTemplate.fallbackIcon
      };
    }
    
    // No template - default to SVG
    const svgIcon = this.getDefaultIcon(iconName, 'svg');
    return {
      icon: svgIcon,
      type: 'svg',
      fallback: this.getDefaultIcon(iconName, 'emoji')
    };
  }
  
  /**
   * Set active template with persistence and reactive updates
   */
  static async setActiveTemplate(templateName: string): Promise<void> {
    try {
      // Validate template exists
      if (!this.builtinTemplates.has(templateName) && !this.customTemplates.has(templateName)) {
        throw new Error(`Template '${templateName}' not found`);
      }
      
      // Update reactive store immediately for hot-updating
      selectedTemplate.set(templateName);
      
      // Persist to settings
      await SettingsService.update('appearance', {
        ...(await SettingsService.getSettings()).appearance,
        selected_icon_template: templateName
      });
      
      console.log('Set active icon template to:', templateName);
      
    } catch (error) {
      console.error('Failed to set active template:', error);
      throw error;
    }
  }
  
  /**
   * Install a custom template from file
   */
  static async installCustomTemplate(templateData: CustomIconTemplate): Promise<void> {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Save template to backend
      await invoke('save_custom_icon_template', { template: templateData });
      
      // Add to local cache
      this.customTemplates.set(templateData.name, templateData);
      
      // Update available templates
      this.updateAvailableTemplates();
      
      console.log('Installed custom template:', templateData.displayName);
      
    } catch (error) {
      throw new Error(`Failed to install template: ${error}`);
    }
  }
  
  /**
   * Remove a custom template
   */
  static async removeCustomTemplate(templateName: string): Promise<void> {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Delete from backend
      await invoke('delete_custom_icon_template', { templateName });
      
      // Remove from local cache
      this.customTemplates.delete(templateName);
      
      // If the removed template was active, switch to emoji
      const currentTemplate = get(selectedTemplate);
      if (currentTemplate === templateName) {
        await this.setActiveTemplate('emoji');
      }
      
      // Update available templates
      this.updateAvailableTemplates();
      
      console.log('Removed custom template:', templateName);
      
    } catch (error) {
      throw new Error(`Failed to remove template: ${error}`);
    }
  }
  
  /**
   * Validate a template file content
   */
  static async validateTemplate(content: string, format: 'json' | 'yaml'): Promise<CustomIconTemplate> {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke('validate_icon_template', { 
        templateContent: content, 
        format 
      });
    } catch (error) {
      throw new Error(`Template validation failed: ${error}`);
    }
  }
  
  /**
   * Get the icons directory path
   */
  static async getIconsDirectoryPath(): Promise<string> {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke('get_icons_directory_path');
    } catch (error) {
      throw new Error(`Failed to get icons directory: ${error}`);
    }
  }
  
  /**
   * Open the icons directory in file explorer
   */
  static async openIconsDirectory(): Promise<void> {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('open_icons_directory');
    } catch (error) {
      throw new Error(`Failed to open icons directory: ${error}`);
    }
  }

  static getDefaultIcon(name: string, type: 'svg' | 'emoji' | 'fontawesome' | 'system' | 'css' | null = 'emoji'): string {
    const template = this.builtinTemplates.get(type as string) || this.customTemplates.get(type as string);
    if (template) {
      return template.icons[name] || template.fallbackIcon;
    }
    return type === 'emoji' ? '❓' : '<svg viewBox="0 0 24 24"><text x="50%" y="50%" dominant-baseline="middle" text-anchor="middle">?</text></svg>';
  }
}