<!-- @component
LoggingSettingsUI - Logging configuration settings panel

Manages logging levels, file size limits, retention policies,
and log visibility in the navigation.

@example
```svelte
◄LoggingSettingsUI /►
```
-->
<script lang="ts">
import { app, type LoggingSettings } from "$lib";

const logLevels = ["debug", "info", "warn", "error"] as const;

function updateLogging<K extends keyof LoggingSettings>(field: K, value: LoggingSettings[K]) {
  const settings = app.customizationService.settings;
  if (!settings) return;

  app.customizationService.settings = {
    ...settings,
    logging: {
      ...settings.logging,
      [field]: value,
    },
  };

  app.customizationService.scheduleSave();
}

function updateNumber(event: Event, field: keyof LoggingSettings, min: number, max: number) {
  const value = Number((event.currentTarget as HTMLInputElement).value);

  if (!Number.isFinite(value)) return;

  updateLogging(field, Math.max(min, Math.min(max, value)) as never);
}

// function toggleLogLevel(level: (typeof logLevels)[number]) {
//   const settings = app.customizationService.settings;
//   const currentLevels = settings?.logging?.default_log_levels;

//   if (!settings || !currentLevels) return;

//   const levels = currentLevels.includes(level) ? currentLevels.filter((current) => current !== level) : [...currentLevels, level];

//   updateLogging("default_log_levels" as never, levels as never);
// }
</script>

{#if app.customizationService.settings}
  <div class="settings-tab">
    <h2>Logging Settings</h2>
    <p>Configure application logging, storage, performance, and log filtering.</p>

    <form>
      <div class="setting-item">
        <div class="setting-info">
          <label for="logging-enabled">Enable Logging</label>
          <p class="setting-description">Enable logging throughout the launcher.</p>
        </div>

        <div class="setting-control">
          <label class="toggle-switch">
            <input
              type="checkbox"
              id="logging-enabled"
              checked={app.customizationService.settings.logging?.enabled ?? true}
              onchange={(event) => updateLogging("enabled", (event.currentTarget as HTMLInputElement).checked)}
            />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label for="persistent-logging">Persistent Log Storage</label>
          <p class="setting-description">Save logs to disk so they remain available after restarting the launcher.</p>
        </div>

        <div class="setting-control">
          <label class="toggle-switch">
            <input
              type="checkbox"
              id="persistent-logging"
              checked={app.customizationService.settings.logging?.persistent ?? true}
              onchange={(event) => updateLogging("persistent", (event.currentTarget as HTMLInputElement).checked)}
            />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label for="log-compression">Log Compression</label>
          <p class="setting-description">Compress stored log files to reduce disk usage.</p>
        </div>

        <div class="setting-control">
          <label class="toggle-switch">
            <input
              type="checkbox"
              id="log-compression"
              checked={app.customizationService.settings.logging?.compression ?? true}
              onchange={(event) => updateLogging("compression", (event.currentTarget as HTMLInputElement).checked)}
            />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label for="retention-days">Log Retention Period</label>
          <p class="setting-description">Number of days log files are retained before automatic cleanup.</p>
        </div>

        <div class="setting-control slider-control">
          <input
            type="range"
            id="retention-days-slider"
            min="1"
            max="365"
            value={app.customizationService.settings.logging?.retention_days ?? 30}
            onchange={(event) => updateNumber(event, "retention_days", 1, 365)}
          />

          <input
            type="number"
            id="retention-days"
            min="1"
            max="365"
            value={app.customizationService.settings.logging?.retention_days ?? 30}
            onchange={(event) => updateNumber(event, "retention_days", 1, 365)}
          />

          <span>days</span>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label for="max-file-size">Maximum Log File Size</label>
          <p class="setting-description">Maximum size of an individual log file before it is rotated or compressed.</p>
        </div>

        <div class="setting-control slider-control">
          <input
            type="range"
            id="max-file-size-slider"
            min="1"
            max="1024"
            value={app.customizationService.settings.logging?.max_file_size_mb ?? 50}
            onchange={(event) => updateNumber(event, "max_file_size_mb", 1, 1024)}
          />

          <input
            type="number"
            id="max-file-size"
            min="1"
            max="1024"
            value={app.customizationService.settings.logging?.max_file_size_mb ?? 50}
            onchange={(event) => updateNumber(event, "max_file_size_mb", 1, 1024)}
          />

          <span>MB</span>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label for="frontend-batch-size">Frontend Batch Size</label>
          <p class="setting-description">Maximum number of frontend log entries sent to the backend in one batch.</p>
        </div>

        <div class="setting-control slider-control">
          <input
            type="range"
            id="frontend-batch-size-slider"
            min="1"
            max="1000"
            step="1"
            value={app.customizationService.settings.logging?.frontend_batch_size ?? 100}
            onchange={(event) => updateNumber(event, "frontend_batch_size", 1, 1000)}
          />

          <input
            type="number"
            id="frontend-batch-size"
            min="1"
            max="1000"
            value={app.customizationService.settings.logging?.frontend_batch_size ?? 100}
            onchange={(event) => updateNumber(event, "frontend_batch_size", 1, 1000)}
          />
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label for="frontend-batch-interval">Frontend Batch Interval</label>
          <p class="setting-description">Maximum time in milliseconds between frontend log batches.</p>
        </div>

        <div class="setting-control slider-control">
          <input
            type="range"
            id="frontend-batch-interval-slider"
            min="10"
            max="5000"
            step="10"
            value={app.customizationService.settings.logging?.frontend_batch_interval_ms ?? 250}
            onchange={(event) => updateNumber(event, "frontend_batch_interval_ms", 10, 5000)}
          />

          <input
            type="number"
            id="frontend-batch-interval"
            min="10"
            max="5000"
            step="10"
            value={app.customizationService.settings.logging?.frontend_batch_interval_ms ?? 250}
            onchange={(event) => updateNumber(event, "frontend_batch_interval_ms", 10, 5000)}
          />

          <span>ms</span>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label for="frontend-max-per-second">Frontend Log Rate Limit</label>
          <p class="setting-description">Maximum number of frontend log entries allowed per second.</p>
        </div>

        <div class="setting-control slider-control">
          <input
            type="range"
            id="frontend-max-per-second-slider"
            min="1"
            max="10000"
            step="10"
            value={app.customizationService.settings.logging?.frontend_max_per_second ?? 1000}
            onchange={(event) => updateNumber(event, "frontend_max_per_second", 1, 10000)}
          />

          <input
            type="number"
            id="frontend-max-per-second"
            min="1"
            max="10000"
            step="10"
            value={app.customizationService.settings.logging?.frontend_max_per_second ?? 1000}
            onchange={(event) => updateNumber(event, "frontend_max_per_second", 1, 10000)}
          />

          <span>/sec</span>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label for="max-memory-logs">Maximum Logs in Memory</label>
          <p class="setting-description">Maximum number of log entries retained in memory per instance.</p>
        </div>

        <div class="setting-control slider-control">
          <input
            type="range"
            id="max-memory-logs-slider"
            min="1000"
            max="20000"
            step="500"
            value={app.customizationService.settings.logging?.max_memory_logs ?? 5000}
            onchange={(event) => updateNumber(event, "max_memory_logs", 1000, 20000)}
          />

          <input
            type="number"
            id="max-memory-logs"
            min="1000"
            max="20000"
            step="500"
            value={app.customizationService.settings.logging?.max_memory_logs ?? 5000}
            onchange={(event) => updateNumber(event, "max_memory_logs", 1000, 20000)}
          />
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label for="dedupe-enabled">Log Deduplication</label>
          <p class="setting-description">Filter duplicate log messages to reduce memory usage and unnecessary processing.</p>
        </div>

        <div class="setting-control">
          <label class="toggle-switch">
            <input
              type="checkbox"
              id="dedupe-enabled"
              checked={app.customizationService.settings.logging?.dedupe_enabled ?? true}
              onchange={(event) => updateLogging("dedupe_enabled", (event.currentTarget as HTMLInputElement).checked)}
            />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label for="dedupe-window-size">Deduplication Window Size</label>
          <p class="setting-description">Number of recent messages checked for duplicates.</p>
        </div>

        <div class="setting-control slider-control">
          <input
            type="range"
            id="dedupe-window-size-slider"
            min="10"
            max="200"
            step="10"
            value={app.customizationService.settings.logging?.dedupe_window_size ?? 50}
            disabled={!(app.customizationService.settings.logging?.dedupe_enabled ?? true)}
            onchange={(event) => updateNumber(event, "dedupe_window_size", 10, 200)}
          />

          <input
            type="number"
            id="dedupe-window-size"
            min="10"
            max="200"
            step="10"
            value={app.customizationService.settings.logging?.dedupe_window_size ?? 50}
            disabled={!(app.customizationService.settings.logging?.dedupe_enabled ?? true)}
            onchange={(event) => updateNumber(event, "dedupe_window_size", 10, 200)}
          />
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label>Default Log Levels</label>
          <p class="setting-description">Select which log levels are enabled by default.</p>
        </div>

        <!-- <div class="setting-control log-levels-control">
          {#each logLevels as level}
            <label
              class:selected={app.customizationService.settings.logging?.default_log_levels?.includes(level)}
              class:unselected={!app.customizationService.settings.logging?.default_log_levels?.includes(level)}
              class="log-level-label">
              <input
                type="checkbox"
                checked={app.customizationService.settings.logging?.default_log_levels?.includes(level) ?? true}
                onchange={() => toggleLogLevel(level)} />
              {level}
            </label>
          {/each}
        </div> -->
      </div>
    </form>
  </div>
{/if}

<style lang="scss">
.settings-tab {
  width: 100%;

  display: flex;
  flex-direction: column;
  gap: 1.75rem;

  padding: 2rem 2.5rem;
  margin-bottom: 2rem;

  background: $color-surface-1;
  border: 1px solid $color-border-muted;
  border-radius: $radius-2xl;
  box-shadow: 0 0.125rem 0.5rem rgba(0, 0, 0, 0.08);
}

.settings-tab h2 {
  margin: 0;

  color: $color-text;
  font-size: 1.5rem;
  font-weight: 600;
  line-height: 1.3;
  letter-spacing: 0.02em;

  background: linear-gradient(to right, $color-accent, $color-accent-secondary);
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.settings-tab > p {
  margin: -1rem 0 0;

  color: $color-text-muted;
  font-size: 0.95rem;
  line-height: 1.5;
}

form {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.setting-item {
  display: flex;
  align-items: center;
  gap: 2.5rem;

  min-width: 0;
  padding: 1.25rem 0;

  border-bottom: 1px solid $color-border-muted;

  &:last-child {
    border-bottom: none;
  }
}

.setting-info {
  flex: 1 1 16rem;
  min-width: 13rem;

  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.setting-info label {
  margin: 0;

  color: $color-text;
  font-size: 1rem;
  font-weight: 500;
  line-height: 1.4;
}

.setting-description {
  margin: 0;

  color: $color-text-muted;
  font-size: 0.875rem;
  line-height: 1.45;
}

.setting-control {
  flex: 1 1 18rem;
  min-width: 12rem;

  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0.75rem;
}

.slider-control {
  gap: 0.7rem;
}

/* Log file size / retention */

.log-file-size-layout,
.log-retention-layout {
  display: flex;
  align-items: center;
  gap: 0.75rem;

  width: 100%;
}

.log-file-size-inputs,
.log-retention-inputs {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;

  width: 14rem;
  min-width: 12rem;
}

.log-file-size-btn,
.log-retention-btn {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

/* Inputs */

input[type="text"],
input[type="number"],
select {
  width: 100%;
  max-width: 20rem;

  padding: 0.55rem 0.8rem;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  background: $color-surface-2;
  color: $color-text;

  font: inherit;
  font-size: 0.9rem;

  outline: none;

  transition:
    border-color 0.2s ease,
    background 0.2s ease,
    box-shadow 0.2s ease;

  &:hover {
    background: $color-surface-3;
    border-color: $color-accent-muted;
  }

  &:focus {
    border-color: $color-focus;
    box-shadow: 0 0 0 3px color-mix(in srgb, $color-focus 20%, transparent);
  }
}

select {
  cursor: pointer;
}

/* Buttons */

button[type="button"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;

  min-height: 2.25rem;

  padding: 0.5rem 0.9rem;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  background: $color-surface-2;
  color: $color-text;

  font: inherit;
  font-size: 0.875rem;
  font-weight: 500;

  cursor: pointer;

  transition:
    background 0.2s ease,
    border-color 0.2s ease,
    color 0.2s ease,
    transform 0.15s ease;

  &:hover {
    background: $color-surface-3;
    border-color: $color-accent;
    color: $color-accent;
  }

  &:active {
    transform: translateY(1px);
  }

  &:focus-visible {
    outline: none;
    border-color: $color-focus;
    box-shadow: 0 0 0 3px color-mix(in srgb, $color-focus 20%, transparent);
  }
}

/* Checkbox */

input[type="checkbox"] {
  appearance: none;

  width: 1.15rem;
  height: 1.15rem;
  flex: 0 0 1.15rem;

  margin: 0;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  background: $color-surface-2;

  cursor: pointer;

  transition:
    background 0.2s ease,
    border-color 0.2s ease,
    box-shadow 0.2s ease;

  &:hover {
    border-color: $color-accent-muted;
    background: $color-surface-3;
  }

  &:checked {
    border-color: $color-accent;
    background: $color-accent;
    box-shadow: inset 0 0 0 2px $color-accent;
  }

  &:focus-visible {
    outline: none;
    border-color: $color-focus;
    box-shadow: 0 0 0 3px color-mix(in srgb, $color-focus 20%, transparent);
  }
}

input[type="checkbox"]:checked::after {
  content: "";

  display: block;

  width: 0.35rem;
  height: 0.65rem;

  margin: 0.15rem auto 0;

  border-right: 2px solid $color-text;
  border-bottom: 2px solid $color-text;

  transform: rotate(45deg);
}

/* Range sliders */

input[type="range"] {
  --range-progress: 50%;

  width: 100%;
  max-width: 20rem;
  height: 0.35rem;

  margin: 0;

  appearance: none;

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

/* Toggle */

.toggle-switch {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.setting-control label {
  display: flex;
  align-items: center;
  gap: 0.6rem;

  color: $color-text;
  font-size: 0.9rem;
  cursor: pointer;
}

/* Log levels */

.log-levels-control {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(7.5rem, 1fr));
  gap: 0.6rem;

  width: 100%;
  max-width: 32rem;
}

.log-level-label {
  display: flex;
  align-items: center;
  justify-content: center;

  min-height: 2.5rem;
  padding: 0.5rem 0.85rem;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  background: $color-surface-2;
  color: $color-text-muted;

  font-size: 0.875rem;
  font-weight: 600;
  text-align: center;
  user-select: none;
  cursor: pointer;

  transition:
    background 0.2s ease,
    border-color 0.2s ease,
    color 0.2s ease,
    box-shadow 0.2s ease,
    transform 0.15s ease;

  outline: none;

  &:hover {
    background: $color-surface-3;
    border-color: $color-accent-muted;
    color: $color-text;
  }

  &:active {
    transform: translateY(1px);
  }

  &:focus-visible {
    border-color: $color-focus;
    box-shadow: 0 0 0 3px color-mix(in srgb, $color-focus 20%, transparent);
  }
}

.log-level-label.selected {
  background: $color-selected;
  border-color: $color-accent;
  color: $color-accent;
  box-shadow: inset 0 0 0 1px color-mix(in srgb, $color-accent 15%, transparent);
}

.log-level-label.selected:hover {
  background: color-mix(in srgb, $color-accent 22%, $color-surface-2);
  border-color: $color-accent-hover;
  color: $color-accent-hover;
}

.log-level-label.unselected {
  background: $color-surface-2;
  color: $color-text-muted;
}

.log-level-label.unselected:hover {
  color: $color-text;
}

/* Hidden radio/checkbox inputs */

.visually-hidden {
  position: absolute;

  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;

  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;

  border: 0;
}

/* Responsive */

@media (max-width: 700px) {
  .settings-tab {
    padding: 1.5rem;
  }

  .setting-item {
    flex-direction: column;
    align-items: stretch;
    gap: 1rem;
  }

  .setting-info {
    min-width: 0;
  }

  .setting-control {
    width: 100%;
    min-width: 0;
  }

  .log-file-size-layout,
  .log-retention-layout {
    align-items: stretch;
  }

  .log-file-size-inputs,
  .log-retention-inputs {
    flex: 1;
    width: auto;
    min-width: 0;
  }

  .log-levels-control {
    max-width: none;
  }

  input[type="text"],
  input[type="number"],
  select {
    max-width: none;
  }
}
</style>
