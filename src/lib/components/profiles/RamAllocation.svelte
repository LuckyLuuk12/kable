<script lang="ts">
import { app, Icon, type KableProfile } from "$lib";
import { onMount } from "svelte";

let {
  profile: profileProp = null,
  embedded = false,
  saveToService = true,
  onJavaArgsChange,
}: {
  profile?: KableProfile | null;
  embedded?: boolean;
  saveToService?: boolean;
  onJavaArgsChange?: (args: string[]) => void;
} = $props();

const RAM_STEP = 256;
const RESERVED_SYSTEM_RAM = 4096;
const MIN_RAM = 512;
const MAX_RAM = 262144;
const SNAP_THRESHOLD_PX = 10;
const SLIDER_THUMB_WIDTH_REM = 0.875;

const SNAP_POINTS = [512, 1024, 2048, 4096, 6144, 8192, 12288, 16384, 24576, 32768, 40960, 49152, 65536, 98304, 131072, 196608, 262144];

let minRamAllocation = $state(512);
let maxRamAllocation = $state(4096);

let minRamInputValue = $state("512");
let maxRamInputValue = $state("4096");

let isMinExpanded = $state(false);
let isMaxExpanded = $state(true);
let isRamPanelExpanded = $state(false);

let isEditingMinRam = $state(false);
let isEditingMaxRam = $state(false);
let isSaving = $state(false);

let systemRam = $state(8192);
let systemRamLoaded = $state(false);
let initializedProfileId = $state<string | null>(null);

let profile = $derived(profileProp ?? app.profilesService.profiles[0] ?? null);

let maxRamLimit = $derived(Math.max(MIN_RAM, Math.min(MAX_RAM, Math.floor(Math.max(MIN_RAM, systemRam - RESERVED_SYSTEM_RAM) / RAM_STEP) * RAM_STEP)));

let availableSnapPoints = $derived(SNAP_POINTS.filter((value) => value >= MIN_RAM && value <= maxRamLimit));

onMount(() => {
  let cancelled = false;

  async function loadSystemRam() {
    try {
      const result = await Promise.resolve(app.getMaxRam());

      if (!cancelled && Number.isFinite(result) && result >= MIN_RAM) {
        systemRam = result;
      }
    } catch (error) {
      console.error("[RamAllocation] Failed to detect system RAM", error);
    } finally {
      if (!cancelled) {
        systemRamLoaded = true;
      }
    }
  }

  void loadSystemRam();

  return () => {
    cancelled = true;
  };
});

function clampRam(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function roundRam(value: number): number {
  return Math.round(value / RAM_STEP) * RAM_STEP;
}

/**
 * Snap to a marker only when the thumb is within 10 rendered pixels.
 * Values otherwise remain on the 256 MiB grid.
 */
function snapRam(value: number, min: number, max: number, slider: HTMLInputElement): number {
  const normalizedValue = clampRam(roundRam(value), min, max);

  if (max <= min) {
    return min;
  }

  const sliderWidth = slider.getBoundingClientRect().width;
  const rootFontSize = Number.parseFloat(getComputedStyle(document.documentElement).fontSize) || 16;
  const thumbWidth = rootFontSize * SLIDER_THUMB_WIDTH_REM;
  const thumbTravel = sliderWidth - thumbWidth;

  if (thumbTravel <= 0) {
    return normalizedValue;
  }

  const currentPosition = ((normalizedValue - min) / (max - min)) * thumbTravel;

  let nearestPoint: number | null = null;
  let nearestDistance = Number.POSITIVE_INFINITY;

  for (const point of SNAP_POINTS) {
    if (point < min || point > max) {
      continue;
    }

    const markerPosition = ((point - min) / (max - min)) * thumbTravel;
    const distance = Math.abs(markerPosition - currentPosition);

    if (distance < nearestDistance) {
      nearestPoint = point;
      nearestDistance = distance;
    }
  }

  if (nearestPoint !== null && nearestDistance <= SNAP_THRESHOLD_PX) {
    return nearestPoint;
  }

  return normalizedValue;
}

function formatRamDisplay(ram: number): string {
  if (ram >= 1024) {
    return `${(ram / 1024).toFixed(ram % 1024 === 0 ? 0 : 1)} GB`;
  }

  return `${ram} MB`;
}

function formatSnapLabel(ram: number): string {
  return ram >= 1024 ? `${ram / 1024}G` : `${ram}M`;
}

function parseRamFlag(args: string[], flag: "Xms" | "Xmx"): number | null {
  const expression = new RegExp(`^-${flag}(?:=)?(\\d+(?:\\.\\d+)?)([kKmMgG]?)$`, "i");

  for (const arg of args) {
    const match = arg.match(expression);

    if (!match) {
      continue;
    }

    const amount = Number(match[1]);
    const unit = match[2].toUpperCase();

    if (!Number.isFinite(amount) || amount <= 0) {
      continue;
    }

    const multiplier: Record<string, number> = {
      "": 1 / (1024 * 1024),
      K: 1 / 1024,
      M: 1,
      G: 1024,
    };

    const megabytes = Math.round(amount * multiplier[unit]);

    if (megabytes > 0) {
      return megabytes;
    }
  }

  return null;
}

function readRamSettings(currentProfile: KableProfile) {
  let min = clampRam(parseRamFlag(currentProfile.settings.java_args, "Xms") ?? 512, MIN_RAM, maxRamLimit);

  let max = clampRam(parseRamFlag(currentProfile.settings.java_args, "Xmx") ?? 4096, MIN_RAM, maxRamLimit);

  if (min > max) {
    min = max;
  }

  return { min, max };
}

$effect(() => {
  const currentProfile = profile;

  if (!currentProfile || !systemRamLoaded || isEditingMinRam || isEditingMaxRam || isSaving) {
    return;
  }

  if (initializedProfileId !== currentProfile.id) {
    initializedProfileId = currentProfile.id;

    const settings = readRamSettings(currentProfile);

    minRamAllocation = settings.min;
    maxRamAllocation = settings.max;

    minRamInputValue = String(settings.min);
    maxRamInputValue = String(settings.max);
  }
});

function updateMinRamFromSlider() {
  minRamAllocation = clampRam(minRamAllocation, MIN_RAM, maxRamAllocation);
  minRamInputValue = String(minRamAllocation);
}

function updateMaxRamFromSlider() {
  maxRamAllocation = clampRam(maxRamAllocation, minRamAllocation, maxRamLimit);
  maxRamInputValue = String(maxRamAllocation);
}

function updateMinRamFromInput(): boolean {
  const value = Number(minRamInputValue);

  if (!Number.isFinite(value) || !Number.isInteger(value)) {
    minRamInputValue = String(minRamAllocation);
    return false;
  }

  minRamAllocation = clampRam(roundRam(value), MIN_RAM, maxRamAllocation);
  minRamInputValue = String(minRamAllocation);

  return true;
}

function updateMaxRamFromInput(): boolean {
  const value = Number(maxRamInputValue);

  if (!Number.isFinite(value) || !Number.isInteger(value)) {
    maxRamInputValue = String(maxRamAllocation);
    return false;
  }

  maxRamAllocation = clampRam(roundRam(value), minRamAllocation, maxRamLimit);
  maxRamInputValue = String(maxRamAllocation);

  return true;
}

function adjustMinRam(amount: number) {
  minRamAllocation = clampRam(minRamAllocation + amount, MIN_RAM, maxRamAllocation);

  minRamInputValue = String(minRamAllocation);
  void commitRamChange();
}

function adjustMaxRam(amount: number) {
  maxRamAllocation = clampRam(maxRamAllocation + amount, minRamAllocation, maxRamLimit);

  maxRamInputValue = String(maxRamAllocation);
  void commitRamChange();
}

function snapMinRam(slider: HTMLInputElement) {
  minRamAllocation = clampRam(snapRam(minRamAllocation, MIN_RAM, maxRamAllocation, slider), MIN_RAM, maxRamAllocation);

  minRamInputValue = String(minRamAllocation);
}

function snapMaxRam(slider: HTMLInputElement) {
  maxRamAllocation = clampRam(snapRam(maxRamAllocation, minRamAllocation, maxRamLimit, slider), minRamAllocation, maxRamLimit);

  maxRamInputValue = String(maxRamAllocation);
}

function getMarkerPosition(ram: number, min: number, max: number): string {
  if (max <= min) {
    return "50%";
  }

  const ratio = clampRam((ram - min) / (max - min), 0, 1);
  const percentage = ratio * 100;
  const remOffset = SLIDER_THUMB_WIDTH_REM / 2 - ratio * SLIDER_THUMB_WIDTH_REM;

  return `calc(${percentage}% + ${remOffset}rem)`;
}

async function commitRamChange() {
  isEditingMinRam = false;
  isEditingMaxRam = false;

  const currentProfile = profile;

  if (!currentProfile || isSaving) {
    return;
  }

  const args = currentProfile.settings.java_args;
  const existingMin = parseRamFlag(args, "Xms");
  const existingMax = parseRamFlag(args, "Xmx");

  if (existingMin === minRamAllocation && existingMax === maxRamAllocation) {
    return;
  }

  const updatedArgs = [...args.filter((arg) => !/^-Xm[sx](?:=|\d)/i.test(arg)), `-Xms${minRamAllocation}M`, `-Xmx${maxRamAllocation}M`];

  if (!saveToService) {
    onJavaArgsChange?.(updatedArgs);
    return;
  }

  const updatedProfile = app.profilesService.memoryClone(currentProfile);
  updatedProfile.settings.java_args = updatedArgs;

  isSaving = true;

  try {
    await app.profilesService.modify(currentProfile, updatedProfile);
  } finally {
    isSaving = false;
  }
}

function handleMinInputBlur() {
  if (updateMinRamFromInput()) {
    void commitRamChange();
  }
}

function handleMaxInputBlur() {
  if (updateMaxRamFromInput()) {
    void commitRamChange();
  }
}

function handleMinInputKeydown(event: KeyboardEvent) {
  if (event.key === "Enter") {
    (event.currentTarget as HTMLInputElement).blur();
  } else if (event.key === "Escape") {
    minRamInputValue = String(minRamAllocation);
    isEditingMinRam = false;
    (event.currentTarget as HTMLInputElement).blur();
  }
}

function handleMaxInputKeydown(event: KeyboardEvent) {
  if (event.key === "Enter") {
    (event.currentTarget as HTMLInputElement).blur();
  } else if (event.key === "Escape") {
    maxRamInputValue = String(maxRamAllocation);
    isEditingMaxRam = false;
    (event.currentTarget as HTMLInputElement).blur();
  }
}
</script>

<div class="ram-controls" class:embedded class:collapsed={!isRamPanelExpanded}>
  {#if isRamPanelExpanded}
    <div class="ram-header">
      <div class="profile-info">
        <span class="installation-name">
          {profile ? (profile.metadata.name !== "" ? profile.metadata.name : profile.version.id) : "No Profile"}
        </span>

        <span class="system-ram">
          <Icon name="memory" forceType="svg" />
          {formatRamDisplay(systemRam)} system RAM
        </span>
      </div>

      <div class="ram-header-right">
        <span class="ram-display">
          {formatRamDisplay(minRamAllocation)} -
          {formatRamDisplay(maxRamAllocation)}
        </span>

        <button
          type="button"
          class="panel-toggle"
          aria-label="Collapse RAM settings"
          title="Collapse RAM settings"
          aria-expanded={isRamPanelExpanded}
          onclick={() => (isRamPanelExpanded = false)}>
          <Icon name="chevron-down" forceType="svg" />
        </button>
      </div>
    </div>

    <!-- Minimum RAM -->
    <section class="ram-setting">
      <button type="button" class="setting-toggle" aria-expanded={isMinExpanded} onclick={() => (isMinExpanded = !isMinExpanded)}>
        <span class="setting-title">
          <Icon name={isMinExpanded ? "chevron-down" : "chevron-right"} forceType="svg" />
          <span class="setting-name">Minimum RAM</span>
        </span>

        <span class="setting-summary">
          {formatRamDisplay(minRamAllocation)}
        </span>
      </button>

      {#if isMinExpanded}
        <div class="setting-content">
          <div class="setting-header">
            <label for="min-ram-input">Minimum memory allocation</label>

            <div class="ram-input-group">
              <input
                id="min-ram-input"
                class="ram-input"
                type="number"
                min={MIN_RAM}
                max={maxRamAllocation}
                step={RAM_STEP}
                value={minRamInputValue}
                disabled={isSaving}
                onfocus={() => (isEditingMinRam = true)}
                oninput={(event) => {
                  minRamInputValue = event.currentTarget.value;
                }}
                onblur={handleMinInputBlur}
                onkeydown={handleMinInputKeydown} />

              <div class="input-arrows">
                <button
                  type="button"
                  aria-label="Increase minimum RAM"
                  title="Increase minimum RAM"
                  disabled={isSaving || minRamAllocation + RAM_STEP > maxRamAllocation}
                  onclick={() => adjustMinRam(RAM_STEP)}>
                  <Icon name="chevron-up" forceType="svg" />
                </button>
                <button
                  type="button"
                  aria-label="Decrease minimum RAM"
                  title="Decrease minimum RAM"
                  disabled={isSaving || minRamAllocation - RAM_STEP < MIN_RAM}
                  onclick={() => adjustMinRam(-RAM_STEP)}>
                  <Icon name="chevron-down" forceType="svg" />
                </button>
              </div>

              <span class="ram-unit">MiB</span>
            </div>
          </div>

          <div class="ram-slider-container">
            <input
              class="ram-slider"
              type="range"
              min={MIN_RAM}
              max={maxRamAllocation}
              step={RAM_STEP}
              value={minRamAllocation}
              disabled={isSaving || maxRamAllocation <= MIN_RAM}
              aria-label="Minimum RAM allocation"
              oninput={(event) => {
                minRamAllocation = Number(event.currentTarget.value);
                updateMinRamFromSlider();
              }}
              onchange={(event) => {
                snapMinRam(event.currentTarget);
                void commitRamChange();
              }} />

            <div class="slider-markers">
              {#each availableSnapPoints.filter((point) => point <= maxRamAllocation) as point (point)}
                <div class="slider-marker" style:left={getMarkerPosition(point, MIN_RAM, maxRamAllocation)}>
                  <span class="marker-label">{formatSnapLabel(point)}</span>
                </div>
              {/each}
            </div>
          </div>

          <p class="setting-description">
            The minimum heap size allocated to Java. Higher values can reduce heap resizing, but reserve more memory even when it is not needed.
          </p>
        </div>
      {/if}
    </section>

    <!-- Maximum RAM -->
    <section class="ram-setting">
      <button type="button" class="setting-toggle" aria-expanded={isMaxExpanded} onclick={() => (isMaxExpanded = !isMaxExpanded)}>
        <span class="setting-title">
          <Icon name={isMaxExpanded ? "chevron-down" : "chevron-right"} forceType="svg" />
          <span class="setting-name">Maximum RAM</span>
        </span>

        <span class="setting-summary">
          {formatRamDisplay(maxRamAllocation)}
        </span>
      </button>

      {#if isMaxExpanded}
        <div class="setting-content">
          <div class="setting-header">
            <label for="max-ram-input">Maximum memory allocation</label>

            <div class="ram-input-group">
              <input
                id="max-ram-input"
                class="ram-input"
                type="number"
                min={minRamAllocation}
                max={maxRamLimit}
                step={RAM_STEP}
                value={maxRamInputValue}
                disabled={isSaving}
                onfocus={() => (isEditingMaxRam = true)}
                oninput={(event) => {
                  maxRamInputValue = event.currentTarget.value;
                }}
                onblur={handleMaxInputBlur}
                onkeydown={handleMaxInputKeydown} />

              <div class="input-arrows">
                <button
                  type="button"
                  aria-label="Increase maximum RAM"
                  title="Increase maximum RAM"
                  disabled={isSaving || maxRamAllocation + RAM_STEP > maxRamLimit}
                  onclick={() => adjustMaxRam(RAM_STEP)}>
                  <Icon name="chevron-up" forceType="svg" />
                </button>
                <button
                  type="button"
                  aria-label="Decrease maximum RAM"
                  title="Decrease maximum RAM"
                  disabled={isSaving || maxRamAllocation - RAM_STEP < minRamAllocation}
                  onclick={() => adjustMaxRam(-RAM_STEP)}>
                  <Icon name="chevron-down" forceType="svg" />
                </button>
              </div>

              <span class="ram-unit">MiB</span>
            </div>
          </div>

          <div class="ram-slider-container">
            <input
              class="ram-slider"
              type="range"
              min={minRamAllocation}
              max={maxRamLimit}
              step={RAM_STEP}
              value={maxRamAllocation}
              disabled={isSaving || maxRamLimit <= minRamAllocation}
              aria-label="Maximum RAM allocation"
              oninput={(event) => {
                maxRamAllocation = Number(event.currentTarget.value);
                updateMaxRamFromSlider();
              }}
              onchange={(event) => {
                snapMaxRam(event.currentTarget);
                void commitRamChange();
              }} />

            <div class="slider-markers">
              {#each availableSnapPoints.filter((point) => point >= minRamAllocation) as point (point)}
                <div class="slider-marker" style:left={getMarkerPosition(point, minRamAllocation, maxRamLimit)}>
                  <span class="marker-label">{formatSnapLabel(point)}</span>
                </div>
              {/each}
            </div>
          </div>

          <p class="setting-description">
            The maximum Java heap size. Leave enough memory for Windows and other applications; this limit reserves memory for Java to use, not memory that is always
            consumed.
          </p>
        </div>
      {/if}
    </section>

    {#if isSaving}
      <div class="saving-indicator">
        <span class="saving-spinner"></span>
        Saving RAM settings...
      </div>
    {/if}
  {:else}
    <button
      type="button"
      class="collapsed-toggle"
      aria-label="Expand RAM settings"
      title="Expand RAM settings"
      aria-expanded={isRamPanelExpanded}
      onclick={() => (isRamPanelExpanded = true)}>
      <Icon name="memory" forceType="svg" />
      <span>RAM</span>
      <span class="collapsed-ram-value">
        {formatRamDisplay(maxRamAllocation)}
      </span>
    </button>
  {/if}
</div>

<style lang="scss">
.ram-controls {
  position: absolute;
  right: $space-md;
  bottom: $space-md;

  width: min(35rem, calc(100% - 2rem));
  min-width: 20rem;
  box-sizing: border-box;
  margin: 0;
  padding: 0.875rem;

  background: $color-surface-2;
  border: 1px solid $color-border;
  border-radius: 0.625rem;

  &.collapsed {
    width: auto;
    min-width: 0;
    padding: 0;
    overflow: hidden;
  }

  &.embedded.collapsed {
    width: fit-content;
    max-width: 100%;
  }

  .ram-header-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .panel-toggle,
  .collapsed-toggle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;

    background: transparent;
    border: none;
    border-radius: $radius-sm;
    color: $color-text-muted;
    cursor: pointer;
    transition:
      background 0.15s ease,
      color 0.15s ease;

    &:hover {
      background: $color-surface-3;
      color: $color-text;
    }

    &:focus-visible {
      outline: 1px solid $color-accent;
      outline-offset: 2px;
    }

    :global(svg) {
      width: 1rem;
      height: 1rem;
      flex-shrink: 0;
    }
  }

  .panel-toggle {
    width: 1.75rem;
    height: 1.75rem;
    flex-shrink: 0;
  }

  .collapsed-toggle {
    min-height: 2.5rem;
    padding: 0.625rem 0.75rem;
    color: $color-text;
    font-size: 0.8125rem;
    font-weight: 600;
  }

  .collapsed-ram-value {
    padding-left: 0.5rem;
    border-left: 1px solid $color-border;
    color: $color-accent;
    font-size: 0.75rem;
    font-variant-numeric: tabular-nums;
  }

  &.embedded {
    position: relative;
    inset: auto;

    width: 100%;
    min-width: 0;
    max-width: 100%;
    margin: 1rem 0 0;
  }

  .ram-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;

    margin-bottom: 0.625rem;

    .profile-info {
      display: flex;
      flex: 1;
      flex-direction: column;
      min-width: 0;
      gap: 0.25rem;
    }

    .installation-name {
      overflow: hidden;

      color: $color-text;
      font-size: 0.8125rem;
      font-weight: 600;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .system-ram {
      display: flex;
      align-items: center;
      gap: 0.25rem;

      color: $color-text-muted;
      font-size: 0.6875rem;

      :global(svg) {
        width: 0.8125rem;
        height: 0.8125rem;
      }
    }

    .ram-display {
      flex-shrink: 0;

      color: $color-accent;
      font-size: 0.75rem;
      font-weight: 600;
      white-space: nowrap;
    }
  }

  .ram-setting {
    & + .ram-setting {
      margin-top: 0.25rem;
      border-top: 1px solid $color-border;
    }

    .setting-toggle {
      display: flex;
      width: 100%;
      align-items: center;
      justify-content: space-between;
      gap: 0.75rem;

      padding: 0.625rem 0.125rem;

      background: transparent;
      border: none;
      border-radius: 0.25rem;

      color: $color-text;
      text-align: left;

      cursor: pointer;
      transition: color 0.15s ease;

      &:hover {
        color: $color-accent;
      }

      &:focus-visible {
        outline: 1px solid $color-accent;
        outline-offset: 2px;
      }

      :global(svg) {
        width: 1rem;
        height: 1rem;
        flex-shrink: 0;
      }

      .setting-title {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        min-width: 0;
      }

      .setting-name {
        font-size: 0.8125rem;
        font-weight: 500;
      }

      .setting-summary {
        padding: 0.125rem 0.375rem;

        background: $color-surface-3;
        border: 1px solid $color-border;
        border-radius: 0.25rem;

        color: $color-text-muted;
        font-size: 0.6875rem;
        font-weight: 500;
        white-space: nowrap;
      }
    }

    .setting-content {
      padding: 0.25rem 0.125rem 0.75rem;
    }

    .setting-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 0.5rem;
      margin-bottom: 0.75rem;

      label {
        color: $color-text-muted;
        font-size: 0.75rem;
        font-weight: 500;
      }
    }

    .ram-input-group {
      display: flex;
      align-items: center;
      gap: 0.375rem;
      flex-shrink: 0;

      .ram-input {
        box-sizing: border-box;
        width: 4.25rem;
        height: 2rem;
        padding: 0 0.375rem;

        appearance: textfield;
        background: $color-surface-3;
        border: 1px solid $color-border;
        border-radius: 0.3125rem;

        color: $color-text;
        font-family: inherit;
        font-size: 0.75rem;
        font-variant-numeric: tabular-nums;
        text-align: right;

        transition:
          border-color 0.15s ease,
          box-shadow 0.15s ease,
          background 0.15s ease;

        &::-webkit-inner-spin-button,
        &::-webkit-outer-spin-button {
          margin: 0;
          appearance: none;
        }

        &:hover {
          border-color: $color-accent;
        }

        &:focus {
          outline: none;
          border-color: $color-accent;
          box-shadow: 0 0 0 2px rgba($color-accent, 0.15);
        }

        &:disabled {
          cursor: not-allowed;
          opacity: 0.5;
        }
      }

      .input-arrows {
        display: flex;
        flex-direction: column;
        width: 1.25rem;
        height: 2rem;
        overflow: hidden;

        background: $color-surface-3;
        border: 1px solid $color-border;
        border-radius: 0.3125rem;

        button {
          display: flex;
          flex: 1;
          align-items: center;
          justify-content: center;

          min-height: 0;
          padding: 0;

          background: transparent;
          border: none;
          color: $color-text-muted;

          cursor: pointer;
          transition:
            background 0.12s ease,
            color 0.12s ease;

          & + button {
            border-top: 1px solid $color-border;
          }

          &:hover:not(:disabled) {
            background: $color-surface-3;
            color: $color-accent;
          }

          &:active:not(:disabled) {
            background: $color-border;
          }

          &:focus-visible {
            outline: 1px solid $color-accent;
            outline-offset: -1px;
          }

          &:disabled {
            cursor: not-allowed;
            opacity: 0.35;
          }

          :global(svg) {
            width: 0.6875rem;
            height: 0.6875rem;
            stroke-width: 2.5;
          }
        }
      }

      .ram-unit {
        color: $color-text-muted;
        font-size: 0.6875rem;
        font-weight: 500;
      }
    }

    .ram-slider-container {
      position: relative;
      padding-bottom: 1.5rem;

      .ram-slider {
        display: block;
        width: 100%;
        height: 0.25rem;
        margin: 0.5rem 0;
        padding: 0;

        appearance: none;
        background: $color-border;
        border: none;
        border-radius: 0.125rem;
        outline: none;

        cursor: pointer;

        &:disabled {
          cursor: not-allowed;
          opacity: 0.5;
        }

        &:focus-visible {
          outline: 2px solid $color-accent;
          outline-offset: 4px;
        }

        &::-webkit-slider-thumb {
          width: 0.875rem;
          height: 0.875rem;

          appearance: none;
          background: $color-accent;
          border: 2px solid $color-surface-3;
          border-radius: 50%;
          box-shadow: 0 0 0 1px $color-accent;

          cursor: grab;
          transition: transform 0.15s ease;

          &:hover {
            transform: scale(1.12);
          }

          &:active {
            cursor: grabbing;
          }
        }

        &::-moz-range-thumb {
          box-sizing: border-box;
          width: 0.875rem;
          height: 0.875rem;

          background: $color-accent;
          border: 2px solid $color-surface-3;
          border-radius: 50%;
          box-shadow: 0 0 0 1px $color-accent;

          cursor: grab;
          transition: transform 0.15s ease;
        }
      }

      .slider-markers {
        position: absolute;
        right: 0;
        bottom: 0;
        left: 0;
        height: 1rem;
        pointer-events: none;

        .slider-marker {
          position: absolute;
          transform: translateX(-50%);

          &::before {
            position: absolute;
            top: -1.5rem;
            left: 50%;
            width: 1px;
            height: 0.375rem;

            background: $color-text-muted;
            content: "";
            opacity: 0.6;
          }

          .marker-label {
            display: block;

            color: $color-text-muted;
            font-size: 0.5625rem;
            font-variant-numeric: tabular-nums;
            white-space: nowrap;
          }

          &:first-child .marker-label {
            transform: translateX(0);
          }

          &:last-child .marker-label {
            transform: translateX(-100%);
          }
        }
      }
    }

    .setting-description {
      margin: 0.125rem 0 0;

      color: $color-text-muted;
      font-size: 0.6875rem;
      line-height: 1.45;
    }
  }

  .saving-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;

    margin-top: 0.5rem;
    padding-top: 0.625rem;
    border-top: 1px solid $color-border;

    color: $color-text-muted;
    font-size: 0.6875rem;

    .saving-spinner {
      width: 0.75rem;
      height: 0.75rem;
      flex-shrink: 0;

      border: 2px solid $color-border;
      border-top-color: $color-accent;
      border-radius: 50%;

      animation: ram-spin 0.7s linear infinite;
    }
  }
}

@keyframes ram-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
