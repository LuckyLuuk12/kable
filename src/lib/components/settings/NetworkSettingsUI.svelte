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
  background: var(--container);
  border-radius: var(--border-radius-large);
  box-shadow: 0 0.125rem 0.5rem rgba(0, 0, 0, 0.08);
  padding: 2rem 2.5rem;
  margin-bottom: 2rem;
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.settings-tab h2 {
  font-size: 1.5rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  background: linear-gradient(to right, var(--primary), var(--secondary));
  color: var(--text-transparent);
  background-clip: text;
  -webkit-background-clip: text;
  -moz-background-clip: text;
  letter-spacing: 0.02em;
}

form {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.setting-item {
  display: flex;
  align-items: flex-start;
  gap: 2rem;
  padding: 1rem 0;
  border-bottom: 1px solid var(--dark-200);
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
  font-size: 1.08rem;
  font-weight: 500;
  color: var(--text);
  margin-bottom: 0.1rem;
}

.setting-description {
  font-size: 0.95rem;
  color: var(--placeholder);
  margin-bottom: 0.2rem;
  line-height: 1.4;
}

.setting-control {
  flex: 1 1 11.25rem;
  display: flex;
  align-items: center;
  gap: 1rem;
  min-width: 10rem;
}

.slider-control {
  gap: 0.7rem;
}

.slider-inputs,
.download-speed-inputs,
.request-rate-inputs {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  min-width: 12rem;
  width: 14rem;
}

.download-speed-layout,
.request-rate-layout {
  align-items: flex-start;
  gap: 1.5rem;
}

.download-speed-btn,
.request-rate-btn {
  display: flex;
  align-items: center;
  height: 100%;
}

button[type="button"] {
  font-size: 0.95rem;
  padding: 0.35em 1em;
  border-radius: var(--border-radius);
  color: var(--text-white);
  border: none;
  cursor: pointer;
  transition: background 0.2s;
}
</style>
