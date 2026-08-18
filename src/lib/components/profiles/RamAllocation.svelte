<script lang="ts">
import { app } from "$lib";

let ramAllocation = $state(2048);
let ramInputValue = $state("2048");
let isEditingRam = $state(false);

let lastPlayedProfiles = $derived(app.profilesService.profiles);

let profile = $derived(lastPlayedProfiles[0] ?? null);

function formatRamDisplay(ram: number): string {
  if (ram >= 1024) {
    return `${(ram / 1024).toFixed(ram % 1024 === 0 ? 0 : 1)} GB`;
  }

  return `${ram} MB`;
}

function updateRamFromSlider() {
  ramInputValue = String(ramAllocation);
}

function updateRamFromInput() {
  const value = Number.parseInt(ramInputValue, 10);

  if (Number.isNaN(value)) {
    ramInputValue = String(ramAllocation);
    return;
  }

  ramAllocation = Math.min(32768, Math.max(512, value));
  ramInputValue = String(ramAllocation);
}

function commitRamChange(immediate = false) {
  if (!immediate) {
    isEditingRam = false;
    return;
  }

  isEditingRam = false;

  // TODO: Persist RAM allocation here.
}
</script>

<div class="ram-controls">
  <div class="ram-header">
    <span class="installation-name">
      {profile ? (profile.metadata.name !== "" ? profile.metadata.name : profile.version.id) : "No Profile"}
    </span>

    <span class="ram-display">{formatRamDisplay(ramAllocation)}</span>
  </div>

  <div class="ram-inputs">
    <div class="ram-slider-container">
      <input
        type="range"
        class="ram-slider"
        bind:value={ramAllocation}
        oninput={() => {
          updateRamFromSlider();
          isEditingRam = true;
        }}
        onchange={() => {
          commitRamChange();
        }}
        min="512"
        max="32768"
        step="256"
      />

      <div class="slider-labels">
        <span>512MB</span>
        <span>8GB</span>
        <span>16GB</span>
        <span>32GB</span>
      </div>
    </div>

    <div class="ram-text-container">
      <input
        type="text"
        class="ram-input"
        bind:value={ramInputValue}
        onfocus={() => {
          isEditingRam = true;
        }}
        onblur={() => {
          updateRamFromInput();
          commitRamChange(true);
        }}
        onkeydown={(e) => {
          if (e.key === "Enter") {
            updateRamFromInput();
            commitRamChange(true);
          }
        }}
        placeholder="2048"
      />

      <span class="ram-unit">MB</span>
    </div>
  </div>
</div>

<style lang="scss">
.ram-controls {
  position: absolute;
  right: $space-md;
  bottom: $space-md;

  max-width: 23.75rem;
  min-width: 20rem;
  margin: 0;
  padding: 1rem;

  background: $color-surface-3;
  border: 1px solid $color-border;
  border-radius: 0.5rem;

  .ram-header {
    display: flex;
    align-items: center;
    justify-content: space-between;

    margin-bottom: 0.75rem;

    color: var(--text);
    font-size: 0.875rem;
    font-weight: 500;

    .installation-name {
      flex: 1;
      overflow: hidden;
      margin-right: 0.5rem;

      color: var(--text);
      font-weight: 500;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .ram-display {
      flex-shrink: 0;

      color: $color-accent;
      font-weight: 600;
    }
  }

  .ram-inputs {
    display: flex;
    align-items: center;
    gap: 0.75rem;

    .ram-slider-container {
      flex: 1;

      .ram-slider {
        width: 100%;
        height: 0.25rem;

        appearance: none;
        border-radius: 0.125rem;
        outline: none;

        cursor: pointer;

        &::-webkit-slider-thumb {
          width: 1rem;
          height: 1rem;

          appearance: none;
          background: $color-accent;
          border-radius: 50%;

          cursor: pointer;
          transition: all 0.2s ease;

          &:hover {
            background: var(--primary-600);
            transform: scale(1.1);
          }
        }

        &::-moz-range-thumb {
          width: 1rem;
          height: 1rem;

          background: $color-accent;
          border: none;
          border-radius: 50%;

          cursor: pointer;
          transition: all 0.2s ease;

          &:hover {
            background: var(--primary-600);
            transform: scale(1.1);
          }
        }
      }

      .slider-labels {
        display: flex;
        justify-content: space-between;

        margin-top: 0.25rem;

        color: var(--placeholder);
        font-size: 0.625rem;
      }
    }

    .ram-text-container {
      display: flex;
      align-items: center;
      gap: 0.375rem;
      flex-shrink: 0;

      .ram-input {
        width: 3.75rem;
        padding: 0.375rem 0.5rem;

        background: var(--dark-600);
        border: 1px solid var(--dark-500);
        border-radius: 0.25rem;

        color: var(--text);
        font-size: 0.75rem;
        text-align: center;

        transition: border-color 0.2s ease;

        &:focus {
          outline: none;
          border-color: $color-accent;
        }

        &::placeholder {
          color: var(--placeholder);
        }
      }

      .ram-unit {
        color: var(--placeholder);
        font-size: 0.75rem;
        font-weight: 500;
      }
    }
  }
}
</style>
