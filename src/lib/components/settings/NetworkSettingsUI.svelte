<!-- @component
NetworkSettingsUI - Network and download settings panel

Configures download behavior including parallel downloads,
speed limits, retry policies, and timeout settings.

@example
```svelte
◄NetworkSettingsUI /►
```
-->
<script lang="ts">
import { app, type NetworkSettings } from "$lib";

function onChange(event: Event, field: keyof NetworkSettings) {
  const target = event.currentTarget as HTMLInputElement;

  const networkSettings = app.customizationService.settings?.network;
  if (!networkSettings) return;

  const value = target.type === "number" ? Number(target.value) : target.value;

  app.customizationService.settings = {
    ...app.customizationService.settings,
    network: {
      ...networkSettings,
      [field]: value,
    },
  };

  app.customizationService.scheduleSave();
}

function setUnlimited(field: keyof NetworkSettings) {
  const networkSettings = app.customizationService.settings?.network;
  if (!networkSettings) return;

  app.customizationService.settings = {
    ...app.customizationService.settings,
    network: {
      ...networkSettings,
      [field]: null,
    },
  };

  app.customizationService.scheduleSave();
}
</script>

<div class="settings-tab">
  <h2>Network & Downloads Settings</h2>
  <p>Configure download concurrency, bandwidth limits, and request throttling.</p>

  <form>
    <div class="setting-item">
      <div class="setting-info">
        <label for="max-download-threads">Maximum Download Threads</label>
        <p class="setting-description">
          Maximum number of downloads that can run simultaneously. Higher values can improve download speed but use more network and system resources.
        </p>
      </div>
      <div class="setting-control slider-control">
        <div class="slider-inputs">
          <input
            type="range"
            id="max-download-threads-slider"
            min="1"
            max="32"
            value={app.customizationService.settings?.network?.max_download_threads ?? 8}
            onchange={(event) => onChange(event, "max_download_threads")}
          />
          <input
            type="number"
            id="max-download-threads"
            min="1"
            max="32"
            value={app.customizationService.settings?.network?.max_download_threads ?? 8}
            onchange={(event) => onChange(event, "max_download_threads")}
          />
        </div>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="max-download-speed">Maximum Download Speed (KB/s)</label>
        <p class="setting-description">Limit the total download bandwidth used by the launcher. Disable the limit to allow downloads to use the available bandwidth.</p>
      </div>
      <div class="setting-control slider-control download-speed-layout">
        <div class="download-speed-inputs">
          <input
            type="range"
            id="max-download-speed-slider"
            min="64"
            max="100000"
            step="64"
            value={app.customizationService.settings?.network?.max_download_speed_kbps ?? 1024}
            disabled={app.customizationService.settings?.network?.max_download_speed_kbps == null}
            onchange={(event) => onChange(event, "max_download_speed_kbps")}
          />
          <input
            type="number"
            id="max-download-speed"
            min="64"
            max="100000"
            step="64"
            value={app.customizationService.settings?.network?.max_download_speed_kbps ?? 1024}
            disabled={app.customizationService.settings?.network?.max_download_speed_kbps == null}
            onchange={(event) => onChange(event, "max_download_speed_kbps")}
          />
        </div>

        <div class="download-speed-btn">
          {#if app.customizationService.settings?.network?.max_download_speed_kbps == null}
            <button
              type="button"
              onclick={() => {
                const networkSettings = app.customizationService.settings?.network;
                if (!networkSettings) return;

                app.customizationService.settings = {
                  ...app.customizationService.settings,
                  network: {
                    ...networkSettings,
                    max_download_speed_kbps: 1024,
                  },
                };

                app.customizationService.scheduleSave();
              }}
            >
              Use Limit
            </button>
          {:else}
            <button type="button" onclick={() => setUnlimited("max_download_speed_kbps")}> Unlimited </button>
          {/if}
        </div>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="max-requests-per-second">Maximum Requests per Second</label>
        <p class="setting-description">Limit how many network requests the launcher can make per second. Disable the limit to allow unrestricted requests.</p>
      </div>
      <div class="setting-control slider-control request-rate-layout">
        <div class="request-rate-inputs">
          <input
            type="range"
            id="max-requests-per-second-slider"
            min="1"
            max="100"
            value={app.customizationService.settings?.network?.max_requests_per_second ?? 10}
            disabled={app.customizationService.settings?.network?.max_requests_per_second == null}
            onchange={(event) => onChange(event, "max_requests_per_second")}
          />
          <input
            type="number"
            id="max-requests-per-second"
            min="1"
            max="100"
            value={app.customizationService.settings?.network?.max_requests_per_second ?? 10}
            disabled={app.customizationService.settings?.network?.max_requests_per_second == null}
            onchange={(event) => onChange(event, "max_requests_per_second")}
          />
        </div>

        <div class="request-rate-btn">
          {#if app.customizationService.settings?.network?.max_requests_per_second == null}
            <button
              type="button"
              onclick={() => {
                const networkSettings = app.customizationService.settings?.network;
                if (!networkSettings) return;

                app.customizationService.settings = {
                  ...app.customizationService.settings,
                  network: {
                    ...networkSettings,
                    max_requests_per_second: 10,
                  },
                };

                app.customizationService.scheduleSave();
              }}
            >
              Use Limit
            </button>
          {:else}
            <button type="button" onclick={() => setUnlimited("max_requests_per_second")}> Unlimited </button>
          {/if}
        </div>
      </div>
    </div>
  </form>
</div>

<style lang="scss">
.settings-tab {
  width: 100%;
  margin-bottom: 2rem;
  padding: 2rem 2.5rem;

  display: flex;
  flex-direction: column;
  gap: 1.5rem;

  background: $color-surface-1;
  border: 1px solid $color-border-muted;
  border-radius: $radius-2xl;
  box-shadow: 0 0.125rem 0.5rem rgba(0, 0, 0, 0.12);
}

.settings-tab h2 {
  margin: 0 0 0.25rem;

  font-size: 1.5rem;
  font-weight: 600;
  letter-spacing: 0.02em;

  background: linear-gradient(to right, $color-accent, $color-accent-secondary);
  color: transparent;
  background-clip: text;
  -webkit-background-clip: text;
}

form {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.setting-item {
  display: flex;
  align-items: center;
  gap: 2rem;

  padding: 1.25rem 0;

  border-bottom: 1px solid $color-border-muted;
}

.setting-item:last-child {
  border-bottom: none;
}

.setting-info {
  flex: 1 1 16.25rem;
  min-width: 13.75rem;

  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.setting-info label {
  margin: 0;

  font-size: 1.08rem;
  font-weight: 500;
  color: $color-text;
}

.setting-description {
  margin: 0;

  font-size: 0.95rem;
  line-height: 1.4;
  color: $color-text-muted;
}

.setting-control {
  flex: 1 1 22rem;
  min-width: 18rem;

  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.slider-control {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  min-width: 0;
}

.slider-inputs,
.download-speed-inputs,
.request-rate-inputs {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 0.75rem;

  width: 100%;
  min-width: 0;
}

.download-speed-layout,
.request-rate-layout {
  display: flex;
  align-items: center;
  gap: 0.75rem;

  width: 100%;
  min-width: 0;
}

.download-speed-btn,
.request-rate-btn {
  display: flex;
  align-items: center;
  justify-content: center;

  flex: 0 0 auto;
}

/* Inputs */

input[type="number"],
input[type="text"] {
  box-sizing: border-box;

  width: 7rem;
  min-width: 7rem;
  height: 2.25rem;

  padding: 0 0.75rem;

  font-size: 0.95rem;
  line-height: 1;

  color: $color-text;
  background: $color-surface-2;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  transition:
    border-color 0.2s ease,
    box-shadow 0.2s ease,
    background 0.2s ease;

  &:hover {
    border-color: $color-accent-muted;
  }

  &:focus {
    outline: none;
    border-color: $color-accent;
    background: $color-surface-2;
    box-shadow: 0 0 0 3px $color-highlight;
  }
}

/* Range sliders */

input[type="range"] {
  --range-progress: 50%;

  flex: 1 1 auto;
  width: 100%;
  min-width: 8rem;
  max-width: none;
  height: 0.35rem;
  margin: 0;

  appearance: none;
  -webkit-appearance: none;

  border-radius: 999px;

  background: linear-gradient(to right, $color-accent 0%, $color-accent var(--range-progress), $color-surface-3 var(--range-progress), $color-surface-3 100%);

  cursor: pointer;
  outline: none;
}

/* Chromium / WebKit */

input[type="range"]::-webkit-slider-runnable-track {
  height: 0.35rem;
  border-radius: 999px;
  background: transparent;
}

input[type="range"]::-webkit-slider-thumb {
  appearance: none;
  -webkit-appearance: none;

  width: 1rem;
  height: 1rem;

  margin-top: -0.325rem;

  border: 2px solid $color-surface-1;
  border-radius: 50%;

  background: $color-accent;
  box-shadow: 0 0 0 1px $color-accent;

  transition:
    transform 0.15s ease,
    box-shadow 0.2s ease;
}

input[type="range"]:hover::-webkit-slider-thumb {
  transform: scale(1.15);

  box-shadow:
    0 0 0 1px $color-accent,
    0 0 0 4px color-mix(in srgb, $color-accent 15%, transparent);
}

input[type="range"]:focus-visible::-webkit-slider-thumb {
  box-shadow:
    0 0 0 1px $color-focus,
    0 0 0 4px color-mix(in srgb, $color-focus 20%, transparent);
}

/* Firefox */

input[type="range"]::-moz-range-track {
  height: 0.35rem;
  border-radius: 999px;
  background: $color-surface-3;
}

input[type="range"]::-moz-range-progress {
  height: 0.35rem;
  border-radius: 999px;
  background: $color-accent;
}

input[type="range"]::-moz-range-thumb {
  width: 1rem;
  height: 1rem;

  border: 2px solid $color-surface-1;
  border-radius: 50%;

  background: $color-accent;
  box-shadow: 0 0 0 1px $color-accent;

  transition:
    transform 0.15s ease,
    box-shadow 0.2s ease;
}

input[type="range"]:hover::-moz-range-thumb {
  transform: scale(1.15);

  box-shadow:
    0 0 0 1px $color-accent,
    0 0 0 4px color-mix(in srgb, $color-accent 15%, transparent);
}

input[type="range"]:focus-visible::-moz-range-thumb {
  box-shadow:
    0 0 0 1px $color-focus,
    0 0 0 4px color-mix(in srgb, $color-focus 20%, transparent);
}

/* Buttons */

button[type="button"] {
  box-sizing: border-box;

  display: inline-flex;
  align-items: center;
  justify-content: center;

  flex: 0 0 auto;

  height: 2.25rem;
  padding: 0 1rem;

  font-size: 0.95rem;
  font-weight: 500;
  line-height: 1;

  color: $color-text;
  background: $color-surface-3;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  cursor: pointer;

  transition:
    background 0.2s ease,
    border-color 0.2s ease,
    color 0.2s ease,
    transform 0.15s ease;

  &:hover {
    color: $color-accent;
    background: $color-surface-2;
    border-color: $color-accent;
  }

  &:active {
    transform: translateY(1px);
  }

  &:focus-visible {
    outline: none;
    border-color: $color-accent;
    box-shadow: 0 0 0 3px $color-highlight;
  }
}

/* Toggle */

.toggle-switch {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

/* Responsive */

@media (max-width: 700px) {
  .settings-tab {
    padding: 1.5rem;
  }

  .setting-item {
    flex-direction: column;
    align-items: stretch;
    gap: 0.75rem;
  }

  .setting-info {
    min-width: 0;
  }

  .setting-control {
    width: 100%;
    min-width: 0;
  }
}

@media (max-width: 500px) {
  .slider-inputs,
  .download-speed-inputs,
  .request-rate-inputs,
  .download-speed-layout,
  .request-rate-layout {
    flex-wrap: wrap;
  }

  input[type="range"] {
    flex: 1 1 100%;
    order: -1;
  }

  input[type="number"],
  input[type="text"] {
    width: 6.5rem;
    min-width: 6.5rem;
  }
}
</style>
