import {
  type AppearanceSettings,
  type CategorizedLauncherSettings,
  type ContentSettings,
  type CustomIconTemplate,
  type SoundSettings,
  type SoundpackMetadata,
  api,
} from "$lib";
import type { Service } from "./app.service";

export class CustomizationService implements Service {
  settings = $state<CategorizedLauncherSettings | null>(null);

  loading = $state(false);
  saving = $state(false);

  /**
   * Load full settings bundle once
   */
  async init() {
    if (this.settings) return;

    this.loading = true;
    try {
      this.settings = await api.getSettings();
    } finally {
      this.loading = false;
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

  /**
   * Appearance helpers
   */
  get appearance(): AppearanceSettings | undefined {
    return this.settings?.appearance;
  }

  set appearance(value: AppearanceSettings | undefined) {
    if (!this.settings) return;
    this.settings = { ...this.settings, appearance: value };
  }

  /**
   * Sound helpers
   */
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
  }

  /**
   * Content helpers
   */
  get content(): ContentSettings | undefined {
    return this.settings?.content;
  }

  set content(value: ContentSettings | undefined) {
    if (!this.settings) return;
    this.settings = { ...this.settings, content: value };
  }

  /**
   * Theme shortcut
   */
  setTheme(theme: AppearanceSettings["theme"]) {
    if (!this.settings?.appearance) return;

    this.settings = {
      ...this.settings,
      appearance: {
        ...this.settings.appearance,
        theme,
      },
    };
  }

  /**
   * Language shortcut
   */
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

  /**
   * Icon templates
   */
  async getIconTemplates() {
    return await api.getIconTemplates();
  }

  async saveCustomIconTemplate(template: CustomIconTemplate) {
    return await api.saveCustomIconTemplate(template);
  }

  async deleteCustomIconTemplate(templateName: string) {
    return await api.deleteCustomIconTemplate(templateName);
  }

  /**
   * Soundpacks
   */
  async listSoundpacks() {
    return await api.listSoundpacks();
  }

  async getSoundpackMetadata(pack: string): Promise<SoundpackMetadata> {
    return await api.getSoundpackMetadata(pack);
  }

  async importSoundpackZip(path: string) {
    return await api.importSoundpackZip(path);
  }

  /**
   * Direct directory actions
   */
  async openIconsDirectory() {
    return await api.openIconsDirectory();
  }

  async openSoundsDirectory() {
    return await api.openSoundsDirectory();
  }

  /**
   * Cleanup
   */
  async destroy() {
    this.settings = null;
    this.loading = false;
    this.saving = false;
  }
}
