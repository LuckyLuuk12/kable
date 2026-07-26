import {
  type AppearanceSettings,
  type CategorizedLauncherSettings,
  type ContentSettings,
  type IconTemplate,
  type SoundSettings,
  api
} from "$lib";
import type { Service } from "./app.service";

type SoundpackListEntry = {
  name: string;
  displayName: string;
  type: "builtin" | "custom";
};

type SoundpackManifest = {
  name: string;
  version?: string;
  author?: string;
  sounds: Record<string, string>;
  music?: Record<string, string[]>;
};

/**
 * Service for managing customization settings, including appearance, content, and sound settings as defined in the CategorizedLauncherSettings type:
 * ```ts
 * export type CategorizedLauncherSettings = {
 *   general?: GeneralSettings;
 *   appearance?: AppearanceSettings;
 *   content?: ContentSettings;
 *   logging?: LoggingSettings;
 *   network?: NetworkSettings;
 *   advanced?: AdvancedSettings;
 *   misc?: MiscSettings;
 * };
 * ```
 */
export class CustomizationService implements Service {
  settings = $state<CategorizedLauncherSettings | null>(null);

  loading = $state(false);
  saving = $state(false);

  selectedIconTemplate = $state<string | null>(null);

  selectedSoundpack = $state("default");
  availableSoundpacks = $state<SoundpackListEntry[]>([]);
  isSoundsLoading = $state(false);
  soundError = $state<string | null>(null);
  isSoundEnabled = $state(true);
  isMusicEnabled = $state(true);

  private audioContext: AudioContext | null = null;
  private soundBuffers = new Map<string, AudioBuffer>();
  private currentSoundpack = "default";
  private soundpackMetadata: SoundpackManifest | null = null;
  private initialized = false;

  private masterVolume = 1.0;
  private soundVolume = 1.0;
  private musicVolume = 0.5;

  private currentMusicSource: AudioBufferSourceNode | null = null;
  private currentMusicGainNode: GainNode | null = null;
  private musicPlaylist: string[] = [];
  private currentMusicIndex = 0;
  private musicLoop = true;
  private musicShuffle = false;

  // TODO: Similar as with icons: move this into static/sounds/default.json and let backend handle loading it and such
  private readonly defaultSoundpack: SoundpackManifest = {
    name: "default",
    version: "1.0.0",
    author: "Kable",
    sounds: {
      click: "click.mp3",
      hover: "hover.mp3",
      success: "success.mp3",
      error: "error.mp3",
      notification: "notification.mp3",
      launch: "launch.mp3",
    },
    music: {},
  };

  /**
   * Load full settings bundle once
   */
  async init() {
    if (!this.settings) {
      this.loading = true;
      try {
        this.settings = await api.getSettings();
        this.initializeSoundSystem();
      } catch (error) {
        console.error("Failed to load customization settings:", error);
      } finally {
        this.loading = false;
      }
    }

    if (!this.initialized) {
      await this.initializeSoundSystem();
    }
  }

  /**
   * Persist full settings object
   */
  async save() {
    if (!this.settings) return;

    this.saving = true;
    try {
      await api.setSettings(this.settings);
    } finally {
      this.saving = false;
    }
  }

  /// A sync method that makes a async callback to save settings, used in setters to persist changes without blocking the UI
  private saveSettingsAsync() {
    (async () => await this.save())().catch((error) => {
      console.error("Failed to save customization settings:", error);
    });
  }


  // #region GENERAL

  // #endregion GENERAL



  // #region APPEARANCE
  get appearance(): AppearanceSettings | undefined {
    return this.settings?.appearance;
  }

  set appearance(value: AppearanceSettings | undefined) {
    if (!this.settings) return;

    // If selected sound or icon template is changed we need to update the selectedSoundpack and selectedIconTemplate states accordingly to make sure reactive UI updates trigger:
    this.selectedIconTemplate = value?.icon_template ?? null;
    this.selectedSoundpack = value?.sound_settings?.selected_soundpack ?? "default";

    this.settings = { ...this.settings, appearance: value };

    this.saveSettingsAsync();
  }

  // #region language
  setLanguage(language: AppearanceSettings["language"]) {
    if (!this.settings?.appearance) return;

    this.settings = {
      ...this.settings,
      appearance: {
        ...this.settings.appearance,
        language,
      },
    };
  }

  // #endregion language

  // #region themes
  async setTheme(theme: AppearanceSettings["theme"]) {
    if (!this.settings?.appearance) return;

    this.settings = {
      ...this.settings,
      appearance: {
        ...this.settings.appearance,
        theme,
      },
    };

    // Get the css content from backend and inject it or remove injected css if theme is null or empty
    if (theme) {
      await api.css.then((cssContent) => {
        this.injectCustomCSS(cssContent);
      });
    }
  }
  private injectCustomCSS(cssContent: string = "") {
    // First, remove any existing custom CSS
    this.removeCustomCSS();

    // Create a new style element
    const styleElement = document.createElement("style");
    styleElement.type = "text/css";
    styleElement.id = "user-custom-css";
    styleElement.innerHTML = cssContent;

    // Append to head (this ensures it comes after our compiled SCSS)
    document.head.appendChild(styleElement);

    // Force font loading and DOM reflow
    setTimeout(() => {
      // Trigger a reflow to ensure fonts are applied
      document.body.style.fontFamily =
        document.body.style.fontFamily ??
        '"Open Sans", Tahoma, Geneva, sans-serif';
      console.log("Font loading triggered: ", document.body.style.fontFamily);
    }, 100);
  }

  private removeCustomCSS() {
    // Remove existing custom CSS
    const existingStyle = document.getElementById("user-custom-css");
    if (existingStyle) {
      existingStyle.remove();
      // Trigger a reflow to ensure fonts are applied
      document.body.style.fontFamily =
        document.body.style.fontFamily ??
        '"Open Sans", Tahoma, Geneva, sans-serif';
      console.log(
        "Previous custom CSS removed: ",
        document.body.style.fontFamily,
      );
    }
  }

  // #endregion themes

  // #region icons
  async getIconTemplates(): Promise<IconTemplate[]> {
    try {
      return await api.getIconTemplates();
    } catch (error) {
      console.error("[CustomizationService] Failed to load icon templates:", error);
      throw error;
    }
  }

  async saveCustomIconTemplate(template: IconTemplate) {
    try {
      return await api.saveCustomIconTemplate(template);
    } catch (error) {
      console.error("[CustomizationService] Failed to save icon template:", error);
      throw error;
    }
  }

  async deleteCustomIconTemplate(templateName: string) {
    try {
      return await api.deleteCustomIconTemplate(templateName);
    } catch (error) {
      console.error("[CustomizationService] Failed to delete icon template:", error);
      throw error;
    }
  }
  async openIconsDirectory() {
    try {
      return await api.openIconsDirectory();
    } catch (error) {
      console.error("[CustomizationService] Failed to open icons directory:", error);
      throw error;
    }
  }

  // Handle getting icons by key/name in current template, if there is no selected template, we use "windows" by default with support of overriding icons with optional parameter:
  async getIcon(key: string, overrideTemplate?: string) {
    // If the type of the selected template is not "builtin" we NEVER use the override template, override template is only used if a builtin is used as builtins match my personal design opinions
    const selectedTemplateId = this.settings?.appearance?.icon_template ?? overrideTemplate ?? "windows";
    const selectedTemplate = (await this.getIconTemplates()).find((t) => t.id === selectedTemplateId);
    return selectedTemplate?.icons[key] ?? selectedTemplate?.icons[selectedTemplate?.fallback_icon];
  }

  // #endregion icons

  // #region sounds
  // We handle sound settings and soundpack management here, including loading, playing, and managing soundpacks and music tracks.
  private async initializeSoundSystem(): Promise<void> {
    if (this.initialized) return;

    this.isSoundsLoading = true;
    this.soundError = null;

    try {
      this.audioContext = new AudioContext();

      this.applySoundSettings(this.settings?.appearance?.sound_settings);

      await this.updateAvailableSoundpacks();

      try {
        await this.loadSoundpack(this.currentSoundpack);
      } catch (error) {
        console.warn(
          `[CustomizationService] Failed to load soundpack "${this.currentSoundpack}", falling back to default:`,
          error,
        );

        if (this.currentSoundpack !== "default") {
          await this.loadSoundpack("default");
        }
      }

      this.initialized = true;
      console.log(
        "[CustomizationService] Sound system initialized with soundpack:",
        this.currentSoundpack,
      );
    } catch (error) {
      console.error("[CustomizationService] Failed to initialize sounds:", error);
      this.soundError = `Failed to initialize sounds: ${error}`;
      this.selectedSoundpack = "default";
    } finally {
      this.isSoundsLoading = false;
    }
  }

  private applySoundSettings(soundSettings?: SoundSettings): void {
    this.isSoundEnabled = soundSettings?.enabled ?? true;
    this.isMusicEnabled = soundSettings?.music_enabled ?? true;
    this.masterVolume = (soundSettings?.master_volume ?? 100) / 100;
    this.soundVolume = (soundSettings?.sound_volume ?? 100) / 100;
    this.musicVolume = (soundSettings?.music_volume ?? 50) / 100;
    this.currentSoundpack = soundSettings?.selected_soundpack ?? "default";
    this.selectedSoundpack = this.currentSoundpack;
  }

  private async updateAvailableSoundpacks(): Promise<void> {
    try {
      const packs = await api.listSoundpacks();
      this.availableSoundpacks = packs.map((pack) => ({
        name: pack,
        displayName: pack.charAt(0).toUpperCase() + pack.slice(1),
        type: pack === "default" ? "builtin" : "custom",
      }));
    } catch (error) {
      console.error("[CustomizationService] Failed to list soundpacks:", error);
      this.availableSoundpacks = [
        {
          name: "default",
          displayName: "Default",
          type: "builtin",
        },
      ];
    }
  }

  private async loadSoundpackManifest(packName: string): Promise<SoundpackManifest> {
    if (packName === "default") {
      return this.defaultSoundpack;
    }

    // Backend metadata exists, but it does not yet carry sound/music maps.
    // const metadata = await api.getSoundpackMetadata(packName);

    try {
      const fileData = await api.loadSoundpackFile(packName, "soundpack.json");
      const manifestText = new TextDecoder().decode(new Uint8Array(fileData));
      const parsed = JSON.parse(manifestText) as Partial<SoundpackManifest>;

      if (!parsed.sounds) {
        throw new Error(`soundpack.json missing sounds map for ${packName}`);
      }

      return {
        name: parsed.name ?? packName,
        version: parsed.version,
        author: parsed.author,
        sounds: parsed.sounds,
        music: parsed.music ?? {},
      };
    } catch (error) {
      console.error(
        `[CustomizationService] Failed to read soundpack manifest for ${packName}:`,
        error,
      );
      throw error;
    }
  }

  private async loadSound(
    packName: string,
    filename: string,
    key: string,
  ): Promise<void> {
    try {
      let audioData: Uint8Array;

      if (packName === "default") {
        const response = await fetch(`/sounds/${filename}`);
        if (!response.ok) {
          console.warn(
            `[CustomizationService] Default sound not found: ${filename}, skipping`,
          );
          return;
        }

        const arrayBuffer = await response.arrayBuffer();
        audioData = new Uint8Array(arrayBuffer);
      } else {
        const fileData = await api.loadSoundpackFile(packName, filename);
        audioData = new Uint8Array(fileData);
      }

      if (!this.audioContext) {
        throw new Error("AudioContext not initialized");
      }

      const buffer = new Uint8Array(audioData).buffer as ArrayBuffer;
      const audioBuffer = await this.audioContext.decodeAudioData(buffer);
      this.soundBuffers.set(key, audioBuffer);
    } catch (error) {
      console.error(
        `[CustomizationService] Failed to load sound file ${filename}:`,
        error,
      );
      throw error;
    }
  }

  private async playNextMusicTrack(volume?: number): Promise<void> {
    if (
      !this.isMusicEnabled ||
      !this.audioContext ||
      this.musicPlaylist.length === 0
    ) {
      return;
    }

    const trackPath = this.musicPlaylist[this.currentMusicIndex];

    try {
      const fileData = await api.loadSoundpackFile(this.currentSoundpack, trackPath);
      const audioData = new Uint8Array(fileData);
      const buffer = new Uint8Array(audioData).buffer as ArrayBuffer;
      const audioBuffer = await this.audioContext.decodeAudioData(buffer);

      const source = this.audioContext.createBufferSource();
      source.buffer = audioBuffer;

      const gainNode = this.audioContext.createGain();
      const trackVolume = volume ?? 1.0;
      gainNode.gain.value = trackVolume * this.musicVolume * this.masterVolume;

      this.currentMusicSource = source;
      this.currentMusicGainNode = gainNode;

      source.connect(gainNode);
      gainNode.connect(this.audioContext.destination);

      source.onended = () => {
        this.handleMusicTrackEnd();
      };

      source.start(0);

      console.log(`[CustomizationService] Playing music track: ${trackPath}`);
    } catch (error) {
      console.error(
        `[CustomizationService] Failed to play music track ${trackPath}:`,
        error,
      );
      this.handleMusicTrackEnd();
    }
  }

  private handleMusicTrackEnd(): void {
    if (this.currentMusicSource) {
      try {
        this.currentMusicSource.disconnect();
      } catch {
        // Ignore disconnect errors.
      }
      this.currentMusicSource = null;
    }

    if (this.currentMusicGainNode) {
      try {
        this.currentMusicGainNode.disconnect();
      } catch {
        // Ignore disconnect errors.
      }
      this.currentMusicGainNode = null;
    }

    this.currentMusicIndex++;

    if (this.currentMusicIndex >= this.musicPlaylist.length) {
      if (this.musicLoop) {
        this.currentMusicIndex = 0;
        if (this.musicShuffle) {
          this.shufflePlaylist();
        }

        void this.playNextMusicTrack();
      } else {
        this.musicPlaylist = [];
        this.currentMusicIndex = 0;
      }

      return;
    }

    void this.playNextMusicTrack();
  }

  private shufflePlaylist(): void {
    for (let index = this.musicPlaylist.length - 1; index > 0; index--) {
      const randomIndex = Math.floor(Math.random() * (index + 1));
      [this.musicPlaylist[index], this.musicPlaylist[randomIndex]] = [
        this.musicPlaylist[randomIndex],
        this.musicPlaylist[index],
      ];
    }
  }

  get sound(): SoundSettings | undefined {
    return this.settings?.appearance?.sound_settings;
  }

  set sound(value: SoundSettings | undefined) {
    if (!this.settings) return;

    this.settings = {
      ...this.settings,
      appearance: {
        ...this.settings.appearance,
        sound_settings: value,
      },
    };

    this.saveSettingsAsync();
  }

  async listSoundpacks() {
    await this.updateAvailableSoundpacks();
    return this.availableSoundpacks.map((pack) => pack.name);
  }

  getSoundpackMetadata(): SoundpackManifest | null {
    return this.soundpackMetadata;
  }

  async loadSoundpack(packName: string): Promise<void> {
    try {
      console.log(`[CustomizationService] Loading soundpack: ${packName}`);

      const metadata = await this.loadSoundpackManifest(packName);
      this.soundpackMetadata = metadata;

      this.soundBuffers.clear();

      for (const [key, filename] of Object.entries(metadata.sounds)) {
        try {
          await this.loadSound(packName, filename, key);
        } catch (error) {
          console.warn(
            `[CustomizationService] Failed to load sound ${key}: ${error}`,
          );
        }
      }

      this.currentSoundpack = packName;
      this.selectedSoundpack = packName;

      console.log(
        `[CustomizationService] Loaded ${this.soundBuffers.size} sounds from ${packName}`,
      );
    } catch (error) {
      console.error(
        `[CustomizationService] Failed to load soundpack ${packName}:`,
        error,
      );
      throw error;
    }
  }

  playSound(key: string, options: { volume?: number; loop?: boolean } = {}): void {
    if (!this.isSoundEnabled || !this.audioContext) return;

    const buffer = this.soundBuffers.get(key);
    if (!buffer) {
      console.warn(`[CustomizationService] Sound not found: ${key}`);
      return;
    }

    try {
      const source = this.audioContext.createBufferSource();
      source.buffer = buffer;

      const gainNode = this.audioContext.createGain();
      const volume = options.volume ?? 1.0;
      gainNode.gain.value = volume * this.soundVolume * this.masterVolume;

      source.connect(gainNode);
      gainNode.connect(this.audioContext.destination);

      if (options.loop) {
        source.loop = true;
      }

      source.start(0);
      console.log(`[CustomizationService] Playing sound: ${key}`);

      if (!options.loop) {
        source.onended = () => {
          source.disconnect();
          gainNode.disconnect();
        };
      }
    } catch (error) {
      console.error(`[CustomizationService] Failed to play sound ${key}:`, error);
    }
  }

  async playBackgroundMusic(
    playlistKey: string,
    options: { shuffle?: boolean; loop?: boolean; volume?: number } = {},
  ): Promise<void> {
    if (!this.isMusicEnabled || !this.audioContext || !this.soundpackMetadata) {
      return;
    }

    this.stopBackgroundMusic();

    const playlist = this.soundpackMetadata.music?.[playlistKey];
    if (!playlist || playlist.length === 0) {
      console.warn(`[CustomizationService] Music playlist not found: ${playlistKey}`);
      return;
    }

    this.musicPlaylist = [...playlist];
    this.musicShuffle = options.shuffle ?? false;
    this.musicLoop = options.loop ?? true;
    this.currentMusicIndex = 0;

    if (this.musicShuffle) {
      this.shufflePlaylist();
    }

    await this.playNextMusicTrack(options.volume);
  }

  stopBackgroundMusic(): void {
    if (this.currentMusicSource) {
      try {
        this.currentMusicSource.stop();
        this.currentMusicSource.disconnect();
      } catch {
        // Ignore stop/disconnect errors.
      }
      this.currentMusicSource = null;
    }

    if (this.currentMusicGainNode) {
      try {
        this.currentMusicGainNode.disconnect();
      } catch {
        // Ignore disconnect errors.
      }
      this.currentMusicGainNode = null;
    }

    this.musicPlaylist = [];
    this.currentMusicIndex = 0;
  }

  setMasterVolume(volume: number): void {
    this.masterVolume = Math.max(0, Math.min(100, volume)) / 100;

    if (this.currentMusicGainNode) {
      this.currentMusicGainNode.gain.value =
        this.musicVolume * this.masterVolume;
    }
  }

  setSoundVolume(volume: number): void {
    this.soundVolume = Math.max(0, Math.min(100, volume)) / 100;
  }

  setMusicVolume(volume: number): void {
    this.musicVolume = Math.max(0, Math.min(100, volume)) / 100;

    if (this.currentMusicGainNode) {
      this.currentMusicGainNode.gain.value =
        this.musicVolume * this.masterVolume;
    }
  }

  setSoundEnabled(enabled: boolean): void {
    this.isSoundEnabled = enabled;

    if (this.settings?.appearance?.sound_settings) {
      this.settings = {
        ...this.settings,
        appearance: {
          ...this.settings.appearance,
          sound_settings: {
            ...this.settings.appearance.sound_settings,
            enabled,
          },
        },
      };
    }
  }

  setMusicEnabled(enabled: boolean): void {
    this.isMusicEnabled = enabled;

    if (this.settings?.appearance?.sound_settings) {
      this.settings = {
        ...this.settings,
        appearance: {
          ...this.settings.appearance,
          sound_settings: {
            ...this.settings.appearance.sound_settings,
            music_enabled: enabled,
          },
        },
      };
    }

    if (!enabled) {
      this.stopBackgroundMusic();
    }
  }

  async importSoundpackZip(path: string) {
    try {
      const packName = await api.importSoundpackZip(path);
      await this.updateAvailableSoundpacks();
      return packName;
    } catch (error) {
      console.error("[CustomizationService] Failed to import soundpack:", error);
      throw error;
    }
  }

  async openSoundsDirectory() {
    try {
      return await api.openSoundsDirectory();
    } catch (error) {
      console.error("[CustomizationService] Failed to open sounds directory:", error);
      throw error;
    }
  }
  // #endregion sounds

  // #endregion APPEARANCE



  // #region CONTENT
  get content(): ContentSettings | undefined {
    return this.settings?.content;
  }

  set content(value: ContentSettings | undefined) {
    if (!this.settings) return;
    this.settings = { ...this.settings, content: value };

    this.saveSettingsAsync();
  }

  // #endregion CONTENT



  // #region LOGGING

  // #endregion LOGGING



  // #region NETWORK

  // #endregion NETWORK



  // #region ADVANCED



  // #endregion ADVANCED



  // #region MISC

  // #endregion MISC


  /**
   * ! Cleanup
   */
  async destroy() {
    this.stopBackgroundMusic();

    if (this.audioContext) {
      try {
        await this.audioContext.close();
      } catch (error) {
        console.warn("[CustomizationService] Error closing AudioContext:", error);
      }
      this.audioContext = null;
    }

    this.soundBuffers.clear();
    this.soundpackMetadata = null;
    this.initialized = false;
    this.currentSoundpack = "default";
    this.selectedSoundpack = "default";
    this.availableSoundpacks = [];
    this.isSoundsLoading = false;
    this.soundError = null;
    this.isSoundEnabled = true;
    this.isMusicEnabled = true;

    this.settings = null;
    this.loading = false;
    this.saving = false;
  }
}
