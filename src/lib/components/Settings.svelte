<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { GeneralSettings, AppearanceSettings, ContentSettings, LoggingSettings, AdvancedSettings, MiscSettings, NetworkSettings } from './setting_categories/';
import { settings, SettingsService } from '$lib';
  import { writable } from 'svelte/store';

  const sections = ['general', 'appearance', 'logging', 'content', 'network', 'advanced', 'misc'];
  let currentSection = writable('general');
  currentSection.subscribe(val => $currentSection = val);

  function updateCurrentSection() {
    let found = false;
    for (const id of sections) {
      const el = document.getElementById(id);
      if (el) {
        const rect = el.getBoundingClientRect();
        if (rect.top <= 200 && rect.bottom > 200) {
          currentSection.set(id);
          found = true;
          break;
        }
      }
    }
    if (!found) currentSection.set(sections[0]);
  }

  // Local validation functions
  export function validateMemory(value: string): number | null {
    const num = parseInt(value);
    if (isNaN(num) || num < 512 || num > 262144) return null;
    return Math.floor(num / 512) * 512; // Round to nearest 512MB
  }
  export function validateNumber(value: string, min: number, max: number): number | null {
    const num = parseInt(value);
    if (isNaN(num) || num < min || num > max) return null;
    return num;
  }
  export function validatePath(value: string): string {
    return value.trim();
  }

  // Periodic save logic
  import { get } from 'svelte/store';
  let lastSettings: any = null;

  // Validate memory settings before saving
  function getValidatedSettings() {
    const snapshot = get(settings);
    snapshot.advanced.default_memory = validateMemory(snapshot.advanced.default_memory.toString()) || 1024;
    return snapshot;
  }

  let saveInterval: ReturnType<typeof setInterval> | null = null;
  let unsubscribeSettings: (() => void) | null = null;

  if (typeof window !== 'undefined') {
    // Save settings on page unload
    window.addEventListener('beforeunload', () => {
      lastSettings = getValidatedSettings();
      SettingsService.saveSettings(lastSettings);
    });
  }

  let sectionInterval: ReturnType<typeof setInterval> | null = null;
  onMount(() => {
    sectionInterval = setInterval(updateCurrentSection, 1000);

    // Auto-save interval logic
    if (typeof window !== 'undefined') {
      let prevIntervalSetting: number | null = null;
      unsubscribeSettings = settings.subscribe(($settings) => {
        let intervalSetting = $settings.advanced.auto_save_interval;
        const isEnabled = typeof intervalSetting === 'number' && intervalSetting > 0;
        intervalSetting = validateNumber(intervalSetting.toString(), 5000, 3600000) || 30000; // Default to 30 seconds if invalid
        if (isEnabled) {
          if (saveInterval) {
            clearInterval(saveInterval);
            saveInterval = null;
          }
          saveInterval = setInterval(() => {
            lastSettings = getValidatedSettings();
            SettingsService.saveSettings(lastSettings);
          }, intervalSetting);
          prevIntervalSetting = intervalSetting;
        } else {
          if (saveInterval) {
            clearInterval(saveInterval);
            saveInterval = null;
          }
        }
      });
    }
  });
  onDestroy(() => {
    if (saveInterval) {
      clearInterval(saveInterval);
      saveInterval = null;
    }
    if (sectionInterval) {
      clearInterval(sectionInterval);
      sectionInterval = null;
    }
    if (unsubscribeSettings) {
      unsubscribeSettings();
      unsubscribeSettings = null;
    }
    window.removeEventListener('beforeunload', () => {
      lastSettings = getValidatedSettings();
      SettingsService.saveSettings(lastSettings);
    });
    SettingsService.saveSettings(getValidatedSettings());
  });
</script>

<!-- a small fixed nav on the left side with links to specific tabs -->
<div class="settings-content">
  <div class="mini-nav">
    {#each sections as section}
      <a href={`#${section}`} class:active={$currentSection === section} on:click={() => currentSection.set(section)}>{section.charAt(0).toUpperCase() + section.slice(1)}</a>
    {/each}
  </div>
  <div class="settings">
    <div id="general"><GeneralSettings /></div>
    <div id="appearance"><AppearanceSettings /></div>
    <div id="logging"><LoggingSettings /></div>
    <div id="content"><ContentSettings /></div>
    <div id="network"><NetworkSettings /></div>
    <div id="advanced"><AdvancedSettings /></div>
    <div id="misc"><MiscSettings /></div>
  </div>
</div>
<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;

  .settings-content {
    display: flex;
    flex-direction: row;
    align-items: flex-start;
    gap: 2rem;
    width: 100%;
    max-height: 80vh;
    min-height: 0;
  }
  .mini-nav {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    flex: 0 0 auto;
    min-width: 120px;
    align-self: center;
  }
  .mini-nav a {
    color: $tertiary;
    &.active {
      color: $primary;
    }
    text-decoration: none;
    position: relative;
    padding-bottom: 2px;
    transition: all 0.4s ease;
    &:hover {
      transform: scale(1.15) translateY(-0.15rem) translateX(0.15rem);
    }
    &::before {
      content: '';
      position: absolute;
      left: 0;
      bottom: 0;
      width: 100%;
      height: 2px;
      background: $tertiary;
      border-radius: 2px;
      transform: scaleX(0);
      transform-origin: left;
      transition: transform 0.25s cubic-bezier(0.4,0,0.2,1);
      z-index: 1;
    }
    &:hover::before {
      transform: scaleX(1);
    }
    &.active::before {
      background: $primary;
    }
  }
  .settings {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    flex: 1 1 0;
    min-width: 0;
    max-height: 80vh;
    overflow-y: auto;
  }
</style>