import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { SettingsService } from "$lib";

// Sound stores for reactive updates
export const selectedSoundpack = writable<string>("default");
export const availableSoundpacks = writable<
  Array<{ name: string; displayName: string; type: "builtin" | "custom" }>
>([]);
export const isSoundsLoading = writable(false);
export const soundError = writable<string | null>(null);
export const isSoundEnabled = writable(true);
export const isMusicEnabled = writable(true);

interface SoundpackMetadata {
  name: string;
  version: string;
  author: string;
  sounds: Record<string, string>;
  music?: Record<string, string[]>;
}

interface PlaybackOptions {
  volume?: number;
  loop?: boolean;
}

interface MusicOptions {
  shuffle?: boolean;
  loop?: boolean;
  volume?: number;
}

export class SoundService {
  private static audioContext: AudioContext | null = null;
  private static soundBuffers = new Map<string, AudioBuffer>();
  private static currentSoundpack: string = "default";
  private static soundpackMetadata: SoundpackMetadata | null = null;
  private static initialized = false;

  // Volume settings (0-1)
  private static masterVolume = 1.0;
  private static soundVolume = 1.0;
  private static musicVolume = 0.5;
  private static soundEnabled = true;
  private static musicEnabled = true;

  // Background music state
  private static currentMusicSource: AudioBufferSourceNode | null = null;
  private static currentMusicGainNode: GainNode | null = null;
  private static musicPlaylist: string[] = [];
  private static currentMusicIndex = 0;
  private static musicLoop = true;
  private static musicShuffle = false;

  /**
   * Initialize the sound system
   */
  static async initialize(): Promise<void> {
    if (this.initialized) return;

    isSoundsLoading.set(true);
    soundError.set(null);

    try {
      // Initialize Web Audio API
      this.audioContext = new AudioContext();

      // Load settings
      const settings = await SettingsService.getSettings();
      this.soundEnabled = settings.appearance?.sound?.enabled ?? true;
      this.musicEnabled = settings.appearance?.sound?.music_enabled ?? true;
      this.masterVolume =
        (settings.appearance?.sound?.master_volume ?? 100) / 100;
      this.soundVolume =
        (settings.appearance?.sound?.sound_volume ?? 100) / 100;
      this.musicVolume = (settings.appearance?.sound?.music_volume ?? 50) / 100;
      this.currentSoundpack =
        settings.appearance?.sound?.selected_soundpack ?? "default";

      // Update stores
      isSoundEnabled.set(this.soundEnabled);
      isMusicEnabled.set(this.musicEnabled);
      selectedSoundpack.set(this.currentSoundpack);

      // Load the selected soundpack
      await this.loadSoundpack(this.currentSoundpack);

      // Update available soundpacks
      await this.updateAvailableSoundpacks();

      this.initialized = true;
      console.log(
        "[SoundService] Initialized with soundpack:",
        this.currentSoundpack,
      );
    } catch (error) {
      console.error("[SoundService] Failed to initialize:", error);
      soundError.set(`Failed to initialize sounds: ${error}`);
      // Fallback to default
      selectedSoundpack.set("default");
    } finally {
      isSoundsLoading.set(false);
    }
  }

  /**
   * Update the list of available soundpacks
   */
  private static async updateAvailableSoundpacks(): Promise<void> {
    try {
      const packs = await invoke<string[]>("list_soundpacks");
      const packsWithType = packs.map((pack) => ({
        name: pack,
        displayName: pack.charAt(0).toUpperCase() + pack.slice(1),
        type: pack === "default" ? ("builtin" as const) : ("custom" as const),
      }));
      availableSoundpacks.set(packsWithType);
    } catch (error) {
      console.error("[SoundService] Failed to list soundpacks:", error);
    }
  }

  /**
   * Load a soundpack by name
   */
  static async loadSoundpack(packName: string): Promise<void> {
    try {
      console.log(`[SoundService] Loading soundpack: ${packName}`);

      // Load metadata
      const metadata = await invoke<SoundpackMetadata>(
        "get_soundpack_metadata",
        {
          pack: packName,
        },
      );
      this.soundpackMetadata = metadata;

      // Clear existing buffers
      this.soundBuffers.clear();

      // Pre-load all sound effects (not music)
      for (const [key, filename] of Object.entries(metadata.sounds)) {
        try {
          await this.loadSound(packName, filename, key);
        } catch (error) {
          console.warn(`[SoundService] Failed to load sound ${key}: ${error}`);
        }
      }

      this.currentSoundpack = packName;
      selectedSoundpack.set(packName);

      console.log(
        `[SoundService] Loaded ${this.soundBuffers.size} sounds from ${packName}`,
      );
    } catch (error) {
      console.error(
        `[SoundService] Failed to load soundpack ${packName}:`,
        error,
      );
      throw error;
    }
  }

  /**
   * Load a single sound file into a buffer
   */
  private static async loadSound(
    packName: string,
    filename: string,
    key: string,
  ): Promise<void> {
    try {
      let audioData: Uint8Array;

      // Default pack loads from static directory
      if (packName === "default") {
        try {
          const response = await fetch(`/sounds/${filename}`);
          if (!response.ok) {
            console.warn(
              `[SoundService] Default sound not found: ${filename}, skipping`,
            );
            return;
          }
          const arrayBuffer = await response.arrayBuffer();
          audioData = new Uint8Array(arrayBuffer);
        } catch (err) {
          console.warn(
            `[SoundService] Failed to load default sound ${filename}, skipping:`,
            err,
          );
          return;
        }
      } else {
        // Load file from backend for custom soundpacks
        const fileData = await invoke<number[]>("load_soundpack_file", {
          pack: packName,
          file: filename,
        });
        audioData = new Uint8Array(fileData);
      }

      // Decode audio data
      if (!this.audioContext) {
        throw new Error("AudioContext not initialized");
      }

      // Create a proper ArrayBuffer copy to satisfy TypeScript
      const buffer = new Uint8Array(audioData).buffer as ArrayBuffer;
      const audioBuffer = await this.audioContext.decodeAudioData(buffer);
      this.soundBuffers.set(key, audioBuffer);
    } catch (error) {
      console.error(
        `[SoundService] Failed to load sound file ${filename}:`,
        error,
      );
      throw error;
    }
  }

  /**
   * Play a sound effect by key
   */
  static playSound(key: string, options: PlaybackOptions = {}): void {
    if (!this.soundEnabled || !this.audioContext) return;

    const buffer = this.soundBuffers.get(key);
    if (!buffer) {
      console.warn(`[SoundService] Sound not found: ${key}`);
      return;
    }

    try {
      const source = this.audioContext.createBufferSource();
      source.buffer = buffer;

      // Create gain node for volume control
      const gainNode = this.audioContext.createGain();
      const volume = options.volume ?? 1.0;
      gainNode.gain.value = volume * this.soundVolume * this.masterVolume;

      // Connect nodes
      source.connect(gainNode);
      gainNode.connect(this.audioContext.destination);

      // Set loop if needed
      if (options.loop) {
        source.loop = true;
      }

      // Play
      source.start(0);
      console.log(`[SoundService] Playing sound: ${key}`);

      // Clean up when done (if not looping)
      if (!options.loop) {
        source.onended = () => {
          source.disconnect();
          gainNode.disconnect();
        };
      }
    } catch (error) {
      console.error(`[SoundService] Failed to play sound ${key}:`, error);
    }
  }

  /**
   * Play background music by playlist key
   */
  static async playBackgroundMusic(
    playlistKey: string,
    options: MusicOptions = {},
  ): Promise<void> {
    if (!this.musicEnabled || !this.audioContext || !this.soundpackMetadata) {
      return;
    }

    // Stop current music if playing
    this.stopBackgroundMusic();

    // Get playlist tracks
    const playlist = this.soundpackMetadata.music?.[playlistKey];
    if (!playlist || playlist.length === 0) {
      console.warn(`[SoundService] Music playlist not found: ${playlistKey}`);
      return;
    }

    this.musicPlaylist = [...playlist];
    this.musicShuffle = options.shuffle ?? false;
    this.musicLoop = options.loop ?? true;
    this.currentMusicIndex = 0;

    // Shuffle if needed
    if (this.musicShuffle) {
      this.shufflePlaylist();
    }

    // Start playing
    await this.playNextMusicTrack(options.volume);
  }

  /**
   * Play the next track in the music playlist
   */
  private static async playNextMusicTrack(volume?: number): Promise<void> {
    if (
      !this.musicEnabled ||
      !this.audioContext ||
      this.musicPlaylist.length === 0
    ) {
      return;
    }

    const trackPath = this.musicPlaylist[this.currentMusicIndex];

    try {
      // Load music file (not pre-cached like sound effects)
      const fileData = await invoke<number[]>("load_soundpack_file", {
        pack: this.currentSoundpack,
        file: trackPath,
      });

      const audioData = new Uint8Array(fileData);
      const buffer = new Uint8Array(audioData).buffer as ArrayBuffer;
      const audioBuffer = await this.audioContext.decodeAudioData(buffer);

      // Create source and gain nodes
      const source = this.audioContext.createBufferSource();
      source.buffer = audioBuffer;

      const gainNode = this.audioContext.createGain();
      const trackVolume = volume ?? 1.0;
      gainNode.gain.value = trackVolume * this.musicVolume * this.masterVolume;

      // Store references
      this.currentMusicSource = source;
      this.currentMusicGainNode = gainNode;

      // Connect nodes
      source.connect(gainNode);
      gainNode.connect(this.audioContext.destination);

      // Handle track end
      source.onended = () => {
        this.handleMusicTrackEnd();
      };

      // Play
      source.start(0);

      console.log(`[SoundService] Playing music track: ${trackPath}`);
    } catch (error) {
      console.error(
        `[SoundService] Failed to play music track ${trackPath}:`,
        error,
      );
      // Try next track on error
      this.handleMusicTrackEnd();
    }
  }

  /**
   * Handle music track ending
   */
  private static handleMusicTrackEnd(): void {
    // Clean up current track
    if (this.currentMusicSource) {
      try {
        this.currentMusicSource.disconnect();
      } catch (e) {
        // Ignore disconnect errors
      }
      this.currentMusicSource = null;
    }

    if (this.currentMusicGainNode) {
      try {
        this.currentMusicGainNode.disconnect();
      } catch (e) {
        // Ignore disconnect errors
      }
      this.currentMusicGainNode = null;
    }

    // Move to next track
    this.currentMusicIndex++;

    // Check if we've reached the end of the playlist
    if (this.currentMusicIndex >= this.musicPlaylist.length) {
      if (this.musicLoop) {
        // Loop: restart from beginning
        this.currentMusicIndex = 0;
        if (this.musicShuffle) {
          this.shufflePlaylist();
        }
        this.playNextMusicTrack();
      } else {
        // No loop: stop
        this.musicPlaylist = [];
        this.currentMusicIndex = 0;
      }
    } else {
      // Play next track
      this.playNextMusicTrack();
    }
  }

  /**
   * Stop background music
   */
  static stopBackgroundMusic(): void {
    if (this.currentMusicSource) {
      try {
        this.currentMusicSource.stop();
        this.currentMusicSource.disconnect();
      } catch (e) {
        // Ignore stop/disconnect errors
      }
      this.currentMusicSource = null;
    }

    if (this.currentMusicGainNode) {
      try {
        this.currentMusicGainNode.disconnect();
      } catch (e) {
        // Ignore disconnect errors
      }
      this.currentMusicGainNode = null;
    }

    this.musicPlaylist = [];
    this.currentMusicIndex = 0;
  }

  /**
   * Shuffle the music playlist
   */
  private static shufflePlaylist(): void {
    for (let i = this.musicPlaylist.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [this.musicPlaylist[i], this.musicPlaylist[j]] = [
        this.musicPlaylist[j],
        this.musicPlaylist[i],
      ];
    }
  }

  /**
   * Set master volume (0-100)
   */
  static setMasterVolume(volume: number): void {
    this.masterVolume = Math.max(0, Math.min(100, volume)) / 100;
  }

  /**
   * Set sound effects volume (0-100)
   */
  static setSoundVolume(volume: number): void {
    this.soundVolume = Math.max(0, Math.min(100, volume)) / 100;
  }

  /**
   * Set music volume (0-100)
   */
  static setMusicVolume(volume: number): void {
    this.musicVolume = Math.max(0, Math.min(100, volume)) / 100;

    // Update current music gain if playing
    if (this.currentMusicGainNode) {
      this.currentMusicGainNode.gain.value =
        this.musicVolume * this.masterVolume;
    }
  }

  /**
   * Enable or disable sound effects
   */
  static setSoundEnabled(enabled: boolean): void {
    this.soundEnabled = enabled;
    isSoundEnabled.set(enabled);
  }

  /**
   * Enable or disable background music
   */
  static setMusicEnabled(enabled: boolean): void {
    this.musicEnabled = enabled;
    isMusicEnabled.set(enabled);

    // Stop music if disabled
    if (!enabled) {
      this.stopBackgroundMusic();
    }
  }

  /**
   * Import a soundpack from a ZIP file
   */
  static async importSoundpackZip(path: string): Promise<string> {
    try {
      const packName = await invoke<string>("import_soundpack_zip", { path });
      await this.updateAvailableSoundpacks();
      return packName;
    } catch (error) {
      console.error("[SoundService] Failed to import soundpack:", error);
      throw error;
    }
  }

  /**
   * List all available soundpacks
   */
  static async listAvailableSoundpacks(): Promise<string[]> {
    try {
      return await invoke<string[]>("list_soundpacks");
    } catch (error) {
      console.error("[SoundService] Failed to list soundpacks:", error);
      return [];
    }
  }

  /**
   * Get the current soundpack metadata
   */
  static getSoundpackMetadata(): SoundpackMetadata | null {
    return this.soundpackMetadata;
  }

  /**
   * Check if the service is initialized
   */
  static isInitialized(): boolean {
    return this.initialized;
  }

  /**
   * Destroy the sound service and clean up resources
   */
  static async destroy(): Promise<void> {
    console.log("[SoundService] Destroying sound service...");

    // Stop any playing music
    this.stopBackgroundMusic();

    // Close AudioContext
    if (this.audioContext) {
      try {
        await this.audioContext.close();
      } catch (error) {
        console.warn("[SoundService] Error closing AudioContext:", error);
      }
      this.audioContext = null;
    }

    // Clear buffers
    this.soundBuffers.clear();

    // Reset state
    this.soundpackMetadata = null;
    this.initialized = false;
    this.currentSoundpack = "default";

    console.log("[SoundService] Sound service destroyed");
  }
}

// Export singleton instance
export const soundService = SoundService;
