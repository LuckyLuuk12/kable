import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { WindowState } from '../types';

export class WindowStateManager {
  private static instance: WindowStateManager;
  private initialized = false;
  private window = getCurrentWindow();

  private constructor() {}

  static getInstance(): WindowStateManager {
    if (!WindowStateManager.instance) {
      WindowStateManager.instance = new WindowStateManager();
    }
    return WindowStateManager.instance;
  }

  /**
   * Static initialize method for consistency with other managers
   */
  static async initialize(): Promise<void> {
    const instance = WindowStateManager.getInstance();
    return await instance.initialize();
  }

  async initialize(): Promise<void> {
    if (this.initialized) return;

    try {
      console.log('Initializing window state manager...');
      
      // Load saved window state
      const savedState = await this.loadWindowState();
      console.log('Loaded window state:', savedState);
      
      // Apply the saved state to the window
      await this.applyWindowState(savedState);
      console.log('Applied window state');
      
      // Set up automatic state saving
      this.setupAutoSave();
      
      this.initialized = true;
      console.log('Window state manager initialized successfully');
    } catch (error) {
      console.error('Failed to initialize window state manager:', error);
      
      // Fallback: apply default state and center window
      try {
        const defaultState = {
          width: 1080,
          height: 720,
          x: -1,
          y: -1,
          maximized: false,
          fullscreen: false
        };
        await this.applyWindowState(defaultState);
        console.log('Applied default window state as fallback');
      } catch (fallbackError) {
        console.error('Failed to apply fallback state:', fallbackError);
        // Final fallback: just center and show
        await this.window.center();
      }
    }

    // Always ensure the window is visible at the end
    try {
      await this.window.show();
      console.log('Window made visible');
    } catch (showError) {
      console.error('Failed to show window:', showError);
    }
  }

  async loadWindowState(): Promise<WindowState> {
    try {
      return await invoke<WindowState>('load_window_state');
    } catch (error) {
      console.error('Failed to load window state:', error);
      // Return default state if loading fails
      return {
        width: 1080,
        height: 720,
        x: -1, // -1 means center
        y: -1, // -1 means center
        maximized: false,
        fullscreen: false,
        monitor_name: undefined,
        monitor_position: undefined,
        monitor_size: undefined
      };
    }
  }

  async saveWindowState(state?: WindowState): Promise<void> {
    try {
      let currentState = state;
      if (!currentState) {
        currentState = await this.getCurrentWindowState();
      }
      await invoke('save_window_state', { state: currentState });
    } catch (error) {
      console.error('Failed to save window state:', error);
    }
  }

  async getCurrentWindowState(): Promise<WindowState> {
    try {
      return await invoke<WindowState>('get_current_window_state');
    } catch (error) {
      console.error('Failed to get current window state:', error);
      throw error;
    }
  }

  async applyWindowState(state: WindowState): Promise<void> {
    try {
      await invoke('apply_window_state', { state });
    } catch (error) {
      console.error('Failed to apply window state:', error);
      throw error;
    }
  }

  private setupAutoSave(): void {
    // Save state when the window is about to close
    window.addEventListener('beforeunload', async () => {
      try {
        await this.saveCurrentState();
      } catch (error) {
        console.error('Failed to save window state on close:', error);
      }
    });

    // Save state periodically (every 5 seconds) when window properties change
    let saveTimeout: number | null = null;
    
    const debouncedSave = () => {
      if (saveTimeout) {
        clearTimeout(saveTimeout);
      }
      saveTimeout = setTimeout(async () => {
        try {
          await this.saveCurrentState();
        } catch (error) {
          console.error('Failed to auto-save window state:', error);
        }
      }, 1000); // Save 1 second after the last change
    };

    // Listen for resize and move events
    this.window.onResized(() => {
      debouncedSave();
    });

    this.window.onMoved(() => {
      debouncedSave();
    });
  }

  private async saveCurrentState(): Promise<void> {
    try {
      const currentState = await this.getCurrentWindowState();
      await this.saveWindowState(currentState);
    } catch (error) {
      console.error('Failed to save current window state:', error);
    }
  }

  // Manual methods for advanced usage
  async centerWindow(): Promise<void> {
    try {
      await this.window.center();
      // Save the new centered position
      setTimeout(async () => {
        await this.saveCurrentState();
      }, 100); // Small delay to ensure position is updated
    } catch (error) {
      console.error('Failed to center window:', error);
    }
  }

  async resetWindowState(): Promise<void> {
    try {
      const defaultState: WindowState = {
        width: 1080,
        height: 720,
        x: -1,
        y: -1,
        maximized: false,
        fullscreen: false,
        monitor_name: undefined,
        monitor_position: undefined,
        monitor_size: undefined
      };
      
      await this.applyWindowState(defaultState);
      await this.saveWindowState(defaultState);
    } catch (error) {
      console.error('Failed to reset window state:', error);
    }
  }

  async getMonitorInfo(): Promise<any[]> {
    try {
      return await invoke<any[]>('get_monitor_info');
    } catch (error) {
      console.error('Failed to get monitor info:', error);
      return [];
    }
  }
}

// Export singleton instance
export const windowStateManager = WindowStateManager.getInstance();
