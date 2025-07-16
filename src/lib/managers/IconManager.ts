import { writable, get } from 'svelte/store';
import type { CustomIconTemplate, IconSettings } from '../types';
import { SettingsManager } from './SettingsManager';

// Icon stores for reactive updates
export const selectedTemplate = writable<string>('emoji');
export const availableTemplates = writable<Array<{ name: string; displayName: string; type: 'builtin' | 'custom' }>>([]);
export const isIconsLoading = writable(false);
export const iconError = writable<string | null>(null);

export class IconManager {
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
      const settings = await SettingsManager.getSettings();
      const templateName = settings.selected_icon_template || 'emoji';
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
        hammer: '🔨',
        cube: '🧊',
        
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
        eye: '👁️'
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
      await SettingsManager.updateSetting('selected_icon_template', templateName);
      
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
}
