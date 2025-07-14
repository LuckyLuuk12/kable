import { writable } from 'svelte/store';
import { SettingsManager } from './settings';
import { GameManager } from './game';
import { MapsManager } from './managers/MapsManager';

// Global initialization state
export const isAppInitialized = writable(false);
export const initializationStatus = writable('Starting...');
export const initializationError = writable<string | null>(null);

export class DataManager {
  private static isInitialized = false;

  /**
   * Initialize all application data on startup
   * This ensures all necessary data is loaded consistently
   */
  static async initialize(): Promise<void> {
    if (this.isInitialized) {
      return; // Already initialized
    }

    initializationStatus.set('Initializing settings...');
    
    try {
      // Step 1: Initialize settings first (required for other managers)
      await SettingsManager.initialize();
      initializationStatus.set('Settings loaded');

      // Step 2: Initialize game data (installations, Java check)
      initializationStatus.set('Loading Minecraft installations...');
      await GameManager.initialize();
      initializationStatus.set('Installations loaded');

      // Step 3: Load maps/worlds data
      initializationStatus.set('Loading worlds data...');
      await MapsManager.loadWorlds();
      initializationStatus.set('Worlds loaded');

      // Mark as complete
      this.isInitialized = true;
      isAppInitialized.set(true);
      initializationStatus.set('Ready');
      initializationError.set(null);

      console.log('✅ DataManager: All application data initialized successfully');

    } catch (error) {
      console.error('❌ DataManager: Initialization failed:', error);
      initializationError.set(`Initialization failed: ${error}`);
      initializationStatus.set('Failed');
      
      // Don't throw here - let the app continue with partial functionality
    }
  }

  /**
   * Refresh all data (useful after settings changes)
   */
  static async refresh(): Promise<void> {
    initializationStatus.set('Refreshing data...');
    
    try {
      // Refresh in dependency order
      await Promise.all([
        GameManager.loadInstallations(),
        MapsManager.loadWorlds()
      ]);

      initializationStatus.set('Ready');
      console.log('✅ DataManager: Data refreshed successfully');

    } catch (error) {
      console.error('❌ DataManager: Refresh failed:', error);
      initializationError.set(`Refresh failed: ${error}`);
    }
  }

  /**
   * Reset initialization state (useful for testing)
   */
  static reset(): void {
    this.isInitialized = false;
    isAppInitialized.set(false);
    initializationStatus.set('Starting...');
    initializationError.set(null);
  }
}
