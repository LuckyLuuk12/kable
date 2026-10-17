
import {
  type AppearanceSettings,
  type CategorizedLauncherSettings,
  type CssTheme,
  type IconTemplate,
  type SoundSettings,
  api,
  app,
} from "$lib";
import { SvelteMap } from "svelte/reactivity";
import type { Service } from "./app.svelte";

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

export class CustomizationService implements Service {
  settings = $state<CategorizedLauncherSettings | null>(null);

  loading = $state(false);
  saving = $state(false);

  iconTemplates = $state<IconTemplate[]>([]);
  selectedIconTemplate = $state<string | null>(null);

  cssThemes = $state<CssTheme[]>([]);
  selectedCssTheme = $state<string | null>(null);
  isCssThemesLoading = $state(false);
  cssThemeError = $state<string | null>(null);

  selectedSoundpack = $state("default");
  availableSoundpacks = $state<SoundpackListEntry[]>([]);
  isSoundsLoading = $state(false);
  soundError = $state<string | null>(null);
  isSoundEnabled = $state(true);
  isMusicEnabled = $state(true);

  private audioContext: AudioContext | null = null;
  private soundBuffers = new SvelteMap<string, AudioBuffer>();
  private currentSoundpack = "default";
  private soundpackMetadata: SoundpackManifest | null = null;
  private soundInitialized = false;

  private masterVolume = 1.0;
  private soundVolume = 1.0;
  private musicVolume = 0.5;

  private currentMusicSource: AudioBufferSourceNode | null = null;
  private currentMusicGainNode: GainNode | null = null;
  private musicPlaylist: string[] = [];
  private currentMusicIndex = 0;
  private musicLoop = true;
  private musicShuffle = false;

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

  private saveTimer: ReturnType<typeof setTimeout> | undefined;

  private iconTemplatesRequest: Promise<IconTemplate[]> | null = null;
  private cssThemesRequest: Promise<CssTheme[]> | null = null;

  /**
   * Each new selection invalidates pending theme loads.
   * This prevents a slow response from an earlier selection from replacing
   * the CSS belonging to a newer selection.
   */
  private cssThemeRevision = 0;

  scheduleSave(delay = 300) {
    if (this.saveTimer) {
      clearTimeout(this.saveTimer);
    }

    this.saveTimer = setTimeout(() => {
      this.saveTimer = undefined;
      this.saveSettingsAsync();
    }, delay);
  }

  /**
   * Load settings and initialize the persisted appearance.
   */
  async init() {
    if (!this.settings) {
      this.loading = true;

      try {
        this.settings = await api.getSettings();
        console.log("[CustomizationService] Loaded customization settings");
      } catch (error) {
        console.error("[CustomizationService] Failed to load customization settings:", error);
        return;
      } finally {
        this.loading = false;
      }
    }

    await Promise.allSettled([
      this.initializeAppearance(),
      this.initializeSoundSystem(),
    ]);
  }

  private async initializeAppearance(): Promise<void> {
    await Promise.allSettled([
      this.getIconTemplates(),
      this.loadCssThemes(),
    ]);

    const appearance = this.settings?.appearance;
    if (!appearance) return;

    this.selectedIconTemplate = appearance.icon_template ?? null;

    // Prefer the new setting. Only accept legacy values if they correspond
    // to an actual discovered theme ID.
    const savedThemeId = appearance.theme ?? null;

    if (!savedThemeId) {
      this.selectedCssTheme = null;
      this.removeCustomCSS();
      return;
    }

    const themeExists = this.cssThemes.some((theme) => theme.id === savedThemeId);

    if (!themeExists) {
      // Old values such as "light", "dark", or "system" are not CSS theme IDs.
      this.selectedCssTheme = null;
      this.removeCustomCSS();

      this.settings = {
        ...this.settings!,
        appearance: {
          ...this.settings!.appearance,
          theme: null,
        },
      };

      this.scheduleSave();
      return;
    }

    try {
      await this.applyCssTheme(savedThemeId);
    } catch (error) {
      console.error("[CustomizationService] Failed to apply saved CSS theme:", error);
      this.selectedCssTheme = null;
      this.removeCustomCSS();
    }
  }

  /**
   * Persist the full settings object.
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

  private saveSettingsAsync() {
    void this.save().catch((error) => {
      console.error("[CustomizationService] Failed to save customization settings:", error);
    });
  }

  // #region APPEARANCE

  setLanguage(language: AppearanceSettings["language"]) {
    if (!this.settings?.appearance) return;

    this.settings = {
      ...this.settings,
      appearance: {
        ...this.settings.appearance,
        language,
      },
    };

    this.scheduleSave();
  }

  // #region CSS themes

  /**
   * Retrieve built-in and custom CSS theme metadata.
   * Concurrent requests share a single promise.
   */
  async loadCssThemes(): Promise<CssTheme[]> {
    if (this.cssThemesRequest) {
      return this.cssThemesRequest;
    }

    this.isCssThemesLoading = true;
    this.cssThemeError = null;

    this.cssThemesRequest = (async () => {
      try {
        const themes = await api.listCssThemes();
        this.cssThemes = themes;
        return themes;
      } catch (error) {
        this.cssThemeError = String(error);
        console.error("[CustomizationService] Failed to load CSS themes:", error);
        throw error;
      } finally {
        this.isCssThemesLoading = false;
        this.cssThemesRequest = null;
      }
    })();

    return this.cssThemesRequest;
  }

  /**
   * Alias for callers that use the previous load* naming convention.
   */
  async getCssThemes(): Promise<CssTheme[]> {
    return this.loadCssThemes();
  }

  /**
   * Apply a theme by stable ID, or clear the custom stylesheet when null.
   *
   * Selection is persisted only after a valid ID is supplied. A null
   * selection means Kable's default styling, not a CSS file.
   */
  async setTheme(themeId: string | null): Promise<void> {
    if (!this.settings?.appearance) return;

    const revision = ++this.cssThemeRevision;

    if (themeId === null || themeId.trim() === "") {
      this.selectedCssTheme = null;
      this.removeCustomCSS();

      this.settings = {
        ...this.settings,
        appearance: {
          ...this.settings.appearance,
          theme: null,
        },
      };

      this.scheduleSave();
      return;
    }

    const themes = this.cssThemes.length > 0
      ? this.cssThemes
      : await this.loadCssThemes();

    if (revision !== this.cssThemeRevision) return;

    const theme = themes.find((entry) => entry.id === themeId);

    if (!theme) {
      throw new Error(`Unknown CSS theme ID: ${themeId}`);
    }

    // Keep the old stylesheet until the new CSS is successfully fetched.
    const cssContent = await api.loadCss(themeId);

    if (revision !== this.cssThemeRevision) return;

    this.injectCustomCSS(cssContent);
    this.selectedCssTheme = themeId;

    this.settings = {
      ...this.settings,
      appearance: {
        ...this.settings.appearance,
        theme: themeId,
      },
    };

    this.scheduleSave();
  }

  /**
   * Load and apply a CSS theme without changing the selected ID.
   * Used when restoring persisted settings during initialization.
   */
  private async applyCssTheme(themeId: string): Promise<void> {
    const revision = ++this.cssThemeRevision;
    const cssContent = await api.loadCss(themeId);

    if (revision !== this.cssThemeRevision) return;

    this.injectCustomCSS(cssContent);
    this.selectedCssTheme = themeId;
  }

  /**
   * Save a custom theme and refresh the available theme list.
   * Returns the stable ID produced by the backend.
   */
  async saveCssTheme(name: string, css: string): Promise<string> {
    const themeId = await api.saveCssTheme(name, css);
    await this.loadCssThemes();
    return themeId;
  }

  /**
   * Delete a custom theme by stable ID.
   * If it is currently selected, return to the default appearance first.
   */
  async deleteCssTheme(themeId: string): Promise<void> {
    const theme = this.cssThemes.find((entry) => entry.id === themeId);

    if (theme?.type === "builtin" || themeId.startsWith("builtin:")) {
      throw new Error("Built-in CSS themes cannot be deleted.");
    }

    if (this.selectedCssTheme === themeId) {
      await this.setTheme(null);
    }

    try {
      await api.deleteCssTheme(themeId);
      await this.loadCssThemes();
    } catch (e) {
      app.notificationService.error(`Failed to delete CSS theme: ${e}`);
    }
  }

  async openCssThemesDirectory(): Promise<void> {
    try {
      await api.openCssThemesDirectory();
    } catch (e) {
      app.notificationService.error(`Failed to open CSS themes directory: ${e}`);
    }
  }

  /**
   * Inject CSS after removing the previously injected stylesheet.
   */
  private injectCustomCSS(cssContent: string): void {
    this.removeCustomCSS();

    const styleElement = document.createElement("style");
    styleElement.type = "text/css";
    styleElement.id = "user-custom-css";
    styleElement.textContent = cssContent;

    document.head.appendChild(styleElement);
  }

  /**
   * Remove the injected stylesheet and restore the application's default CSS.
   */
  private removeCustomCSS(): void {
    document.getElementById("user-custom-css")?.remove();
  }

  // #endregion CSS themes

  // #region icons

  async getIconTemplates(): Promise<IconTemplate[]> {
    if (this.iconTemplatesRequest) {
      return this.iconTemplatesRequest;
    }

    this.iconTemplatesRequest = (async () => {
      console.log("[CustomizationService] Fetching icon templates...");

      try {
        this.iconTemplates = await api.getIconTemplates();
        return this.iconTemplates;
      } catch (error) {
        console.error("[CustomizationService] Failed to load icon templates:", error);
        throw error;
      } finally {
        this.iconTemplatesRequest = null;
      }
    })();

    return this.iconTemplatesRequest;
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

  /**
   * Resolve an icon from the selected template.
   *
   * When no template is explicitly selected, the default template is "emoji".
   * The optional override selects a built-in template only when the current
   * selection is absent or is itself a built-in template.
   */
  async getIcon(key: string, overrideTemplate?: string) {
    if (this.iconTemplates.length === 0) {
      await this.getIconTemplates();
    }

    const selectedTemplateId = this.settings?.appearance?.icon_template ?? null;
    const selectedTemplate = this.iconTemplates.find((template) => template.id === selectedTemplateId);

    const isBuiltin = selectedTemplate?.type === "builtin";
    const templateId = selectedTemplateId
      ? (isBuiltin && overrideTemplate ? overrideTemplate : selectedTemplateId)
      : (overrideTemplate ?? "emoji");

    const template = this.iconTemplates.find((entry) => entry.id === templateId)
      ?? this.iconTemplates.find((entry) => entry.id === "emoji");

    if (!template) return undefined;

    return template.icons[key] ?? template.icons[template.fallback_icon];
  }

  // #endregion icons

  // #endregion APPEARANCE

  // #region SOUNDS

  private async initializeSoundSystem(): Promise<void> {
    if (this.soundInitialized || this.isSoundsLoading) return;

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

      this.soundInitialized = true;
      console.log("[CustomizationService] Sound system initialized with soundpack:", this.currentSoundpack);
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
      console.error(`[CustomizationService] Failed to read soundpack manifest for ${packName}:`, error);
      throw error;
    }
  }

  private async loadSound(packName: string, filename: string, key: string): Promise<void> {
    try {
      let audioData: Uint8Array;

      if (packName === "default") {
        const response = await fetch(`/sounds/${filename}`);

        if (!response.ok) {
          console.warn(`[CustomizationService] Default sound not found: ${filename}, skipping`);
          return;
        }

        audioData = new Uint8Array(await response.arrayBuffer());
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
      console.error(`[CustomizationService] Failed to load sound file ${filename}:`, error);
      throw error;
    }
  }

  private async playNextMusicTrack(volume?: number): Promise<void> {
    if (!this.isMusicEnabled || !this.audioContext || this.musicPlaylist.length === 0) {
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
      console.error(`[CustomizationService] Failed to play music track ${trackPath}:`, error);
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

    this.scheduleSave();
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
          console.warn(`[CustomizationService] Failed to load sound ${key}: ${error}`);
        }
      }

      this.currentSoundpack = packName;
      this.selectedSoundpack = packName;

      console.log(`[CustomizationService] Loaded ${this.soundBuffers.size} sounds from ${packName}`);
    } catch (error) {
      console.error(`[CustomizationService] Failed to load soundpack ${packName}:`, error);
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
      this.currentMusicGainNode.gain.value = this.musicVolume * this.masterVolume;
    }
  }

  setSoundVolume(volume: number): void {
    this.soundVolume = Math.max(0, Math.min(100, volume)) / 100;
  }

  setMusicVolume(volume: number): void {
    this.musicVolume = Math.max(0, Math.min(100, volume)) / 100;

    if (this.currentMusicGainNode) {
      this.currentMusicGainNode.gain.value = this.musicVolume * this.masterVolume;
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

      this.scheduleSave();
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

      this.scheduleSave();
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

  // #endregion SOUNDS

  /**
   * Clean up service-owned resources.
   */
  async destroy() {
    if (this.saveTimer) {
      clearTimeout(this.saveTimer);
      this.saveTimer = undefined;
    }

    // Invalidate outstanding CSS requests.
    this.cssThemeRevision++;
    this.removeCustomCSS();

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
    this.soundInitialized = false;
    this.currentSoundpack = "default";
    this.selectedSoundpack = "default";
    this.availableSoundpacks = [];
    this.isSoundsLoading = false;
    this.soundError = null;
    this.isSoundEnabled = true;
    this.isMusicEnabled = true;

    this.iconTemplates = [];
    this.selectedIconTemplate = null;
    this.cssThemes = [];
    this.selectedCssTheme = null;
    this.cssThemeError = null;

    this.settings = null;
    this.loading = false;
    this.saving = false;
  }
}
