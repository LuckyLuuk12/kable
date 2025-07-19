<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { GeneralSettings, AppearanceSettings, ContentSettings, LoggingSettings, AdvancedSettings, MiscSettings, NetworkSettings } from './setting_categories/';
  import { settings, SettingsManager } from '$lib';
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
      SettingsManager.save(lastSettings);
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
            SettingsManager.save(lastSettings);
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
      SettingsManager.save(lastSettings);
    });
    SettingsManager.save(getValidatedSettings());
  });
</script>

<!-- a small fixed nav on the left side with links to specific tabs -->
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

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;

  .settings {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    width: 100%;
  }
  .mini-nav {
    position: fixed;
    top: 1rem;
    left: 15%;
    display: flex;
    flex-direction: column;
    justify-content: center;
    height: 100%;
    gap: 0.75rem;
    a {
      color: $tertiary;
      &.active {
        color: $primary;
      }
      text-decoration: none;
      position: relative;
      padding-bottom: 2px;
      transition: all 0.4s ease;
      &:hover {
        transform: scale(1.25) translateY(-0.25rem) translateX(0.25rem);
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
  }
</style>