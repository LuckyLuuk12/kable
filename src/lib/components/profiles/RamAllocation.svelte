<script lang="ts">
import { app, Icon, type KableProfile } from "$lib";
import { onMount } from "svelte";

const RAM_STEP = 256;
const RESERVED_SYSTEM_RAM = 4096;
const MIN_RAM = 512;
const MAX_RAM = 262144; // 256 GiB in MiB
const SNAP_THRESHOLD_PX = 10;
const SLIDER_THUMB_WIDTH_REM = 0.875;

const SNAP_POINTS = [
  512, // 512 MiB
  1024, // 1 GiB
  2048, // 2 GiB
  4096, // 4 GiB
  6144, // 6 GiB
  8192, // 8 GiB
  12288, // 12 GiB
  16384, // 16 GiB
  24576, // 24 GiB
  32768, // 32 GiB
  40960, // 40 GiB
  49152, // 48 GiB
  65536, // 64 GiB
  98304, // 96 GiB
  131072, // 128 GiB
  196608, // 192 GiB
  262144, // 256 GiB
];

let minRamAllocation = $state(512);
let maxRamAllocation = $state(4096);

let minRamInputValue = $state("512");
let maxRamInputValue = $state("4096");

let isMinExpanded = $state(false);
let isMaxExpanded = $state(true);

let isEditingMinRam = $state(false);
let isEditingMaxRam = $state(false);
let isSaving = $state(false);

let systemRam = $state(8192);
let systemRamLoaded = $state(false);
let initializedProfileId = $state<string | null>(null);

let profile = $derived(app.profilesService.profiles[0] ?? null);

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
 * Snaps to a predefined marker only when it is visually close to the
 * current thumb position. Otherwise, the value remains on the 256 MiB grid.
 *
 * Marker distance is measured in pixels so the interaction remains
 * consistent across different slider ranges and rendered widths.
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

/**
 * Aligns marker centers with the slider thumb's travel range.
 * The thumb occupies 0.875rem, so its center starts and ends half a
 * thumb-width inside the slider's bounds.
 */
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

  const updatedProfile = app.profilesService.memoryClone(currentProfile);

  updatedProfile.settings.java_args = [
    ...updatedProfile.settings.java_args.filter((arg) => !/^-Xm[sx](?:=|\d)/i.test(arg)),
    `-Xms${minRamAllocation}M`,
    `-Xmx${maxRamAllocation}M`,
  ];

  isSaving = true;

  try {
    await app.profilesService.modify(currentProfile, updatedProfile);
  } finally {
    isSaving = false;
  }
}
</script>

<div class="ram-controls">
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

    <span class="ram-display">
      {formatRamDisplay(minRamAllocation)} -
      {formatRamDisplay(maxRamAllocation)}
    </span>
  </div>

  <section class="ram-setting">
    <button type="button" class="setting-toggle" aria-expanded={isMinExpanded} onclick={() => (isMinExpanded = !isMinExpanded)}>
      <span class="setting-title">
        <span class="setting-name">Minimum heap</span>
        <span class="setting-summary">
          {formatRamDisplay(minRamAllocation)}
        </span>
      </span>

      <Icon name={isMinExpanded ? "chevron-up" : "chevron-down"} forceType="svg" />
    </button>

    {#if isMinExpanded}
      <div class="setting-content">
        <div class="setting-header">
          <label for="min-ram-slider">Initial heap allocation</label>

          <div class="ram-input-group">
            <input
              id="min-ram-input"
              type="number"
              class="ram-input"
              bind:value={minRamInputValue}
              onfocus={() => (isEditingMinRam = true)}
              onblur={() => {
                if (updateMinRamFromInput()) {
                  void commitRamChange();
                } else {
                  isEditingMinRam = false;
                }
              }}
              onkeydown={(event) => {
                if (event.key === "Enter") {
                  if (updateMinRamFromInput()) {
                    void commitRamChange();
                  } else {
                    isEditingMinRam = false;
                  }

                  event.currentTarget.blur();
                } else if (event.key === "Escape") {
                  minRamInputValue = String(minRamAllocation);
                  isEditingMinRam = false;
                  event.currentTarget.blur();
                }
              }}
              min={MIN_RAM}
              max={maxRamAllocation}
              step={RAM_STEP}
              disabled={!profile || isSaving}
              aria-label="Minimum heap in megabytes" />

            <div class="input-arrows">
              <button
                type="button"
                aria-label="Increase minimum heap by 256 MB"
                disabled={!profile || isSaving || minRamAllocation >= maxRamAllocation}
                onclick={() => adjustMinRam(RAM_STEP)}>
                <Icon name="chevron-up" forceType="svg" />
              </button>

              <button
                type="button"
                aria-label="Decrease minimum heap by 256 MB"
                disabled={!profile || isSaving || minRamAllocation <= MIN_RAM}
                onclick={() => adjustMinRam(-RAM_STEP)}>
                <Icon name="chevron-down" forceType="svg" />
              </button>
            </div>

            <span class="ram-unit">MB</span>
          </div>
        </div>

        <div class="ram-slider-container">
          <input
            id="min-ram-slider"
            type="range"
            class="ram-slider"
            bind:value={minRamAllocation}
            oninput={updateMinRamFromSlider}
            onchange={(event) => {
              snapMinRam(event.currentTarget);
              void commitRamChange();
            }}
            min={MIN_RAM}
            max={maxRamAllocation}
            step={RAM_STEP}
            disabled={!profile || isSaving} />

          <div class="slider-markers">
            {#each availableSnapPoints.filter((point) => point >= MIN_RAM && point <= maxRamAllocation) as point (point)}
              <span class="slider-marker" style:left={getMarkerPosition(point, MIN_RAM, maxRamAllocation)}>
                <span class="marker-label">{formatSnapLabel(point)}</span>
              </span>
            {/each}
          </div>
        </div>

        <p class="setting-description">Memory allocated when Java starts. Keeping this lower than the maximum lets Java grow its heap when needed.</p>
      </div>
    {/if}
  </section>

  <section class="ram-setting">
    <button type="button" class="setting-toggle" aria-expanded={isMaxExpanded} onclick={() => (isMaxExpanded = !isMaxExpanded)}>
      <span class="setting-title">
        <span class="setting-name">Maximum heap</span>
        <span class="setting-summary">
          {formatRamDisplay(maxRamAllocation)}
        </span>
      </span>

      <Icon name={isMaxExpanded ? "chevron-up" : "chevron-down"} forceType="svg" />
    </button>

    {#if isMaxExpanded}
      <div class="setting-content">
        <div class="setting-header">
          <label for="max-ram-slider">Maximum allocation</label>

          <div class="ram-input-group">
            <input
              id="max-ram-input"
              type="number"
              class="ram-input"
              bind:value={maxRamInputValue}
              onfocus={() => (isEditingMaxRam = true)}
              onblur={() => {
                if (updateMaxRamFromInput()) {
                  void commitRamChange();
                } else {
                  isEditingMaxRam = false;
                }
              }}
              onkeydown={(event) => {
                if (event.key === "Enter") {
                  if (updateMaxRamFromInput()) {
                    void commitRamChange();
                  } else {
                    isEditingMaxRam = false;
                  }

                  event.currentTarget.blur();
                } else if (event.key === "Escape") {
                  maxRamInputValue = String(maxRamAllocation);
                  isEditingMaxRam = false;
                  event.currentTarget.blur();
                }
              }}
              min={minRamAllocation}
              max={maxRamLimit}
              step={RAM_STEP}
              disabled={!profile || isSaving}
              aria-label="Maximum heap in megabytes" />

            <div class="input-arrows">
              <button
                type="button"
                aria-label="Increase maximum heap by 256 MB"
                disabled={!profile || isSaving || maxRamAllocation >= maxRamLimit}
                onclick={() => adjustMaxRam(RAM_STEP)}>
                <Icon name="chevron-up" forceType="svg" />
              </button>

              <button
                type="button"
                aria-label="Decrease maximum heap by 256 MB"
                disabled={!profile || isSaving || maxRamAllocation <= minRamAllocation}
                onclick={() => adjustMaxRam(-RAM_STEP)}>
                <Icon name="chevron-down" forceType="svg" />
              </button>
            </div>

            <span class="ram-unit">MB</span>
          </div>
        </div>

        <div class="ram-slider-container">
          <input
            id="max-ram-slider"
            type="range"
            class="ram-slider"
            bind:value={maxRamAllocation}
            oninput={updateMaxRamFromSlider}
            onchange={(event) => {
              snapMaxRam(event.currentTarget);
              void commitRamChange();
            }}
            min={minRamAllocation}
            max={maxRamLimit}
            step={RAM_STEP}
            disabled={!profile || isSaving} />

          <div class="slider-markers">
            {#each availableSnapPoints.filter((point) => point >= minRamAllocation) as point (point)}
              <span class="slider-marker" style:left={getMarkerPosition(point, minRamAllocation, maxRamLimit)}>
                <span class="marker-label">{formatSnapLabel(point)}</span>
              </span>
            {/each}
          </div>
        </div>

        <p class="setting-description">
          Maximum Java heap allocation. Up to ~{formatRamDisplay(maxRamLimit)} is available after reserving system memory.
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
