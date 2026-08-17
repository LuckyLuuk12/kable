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
  background: linear-gradient(to right, $color-accent, $color-accent-secondary);
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
.log-file-size-layout,
.log-retention-layout {
  display: flex;
  align-items: flex-start;
  gap: 1.5rem;
}
.log-file-size-inputs,
.log-retention-inputs {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  min-width: 12rem;
  width: 14rem;
}
.log-file-size-btn,
.log-retention-btn {
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
.toggle-switch {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.setting-control label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 1rem;
  color: var(--text);
}
.log-levels-control {
  display: grid;
  gap: 1rem;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
}
.log-level-label {
  cursor: pointer;
  padding: 0.4rem 1.1rem;
  border-radius: var(--border-radius);
  font-weight: 600;
  user-select: none;
  transition:
    background 0.18s,
    color 0.18s;
  outline: none;
  border: 2px solid transparent;
}
.log-level-label.unselected {
  background: var(--input);
  color: var(--red-700);
}
.log-level-label.selected {
  background: var(--input);
  color: var(--green-600);
}
.log-level-label.unselected:hover {
  color: color-mix(in srgb, var(--red-700) 70%, var(--text-white) 30%);
  border-color: var(--dark-200);
}
.log-level-label.selected:hover {
  color: color-mix(in srgb, var(--green-600) 70%, var(--text-white) 30%);
  border-color: var(--dark-200);
}
.log-level-label:focus {
  border: 2px solid var(--primary);
}
.visually-hidden {
  position: absolute;
  opacity: 0;
  pointer-events: none;
  width: 0;
  height: 0;
}
</style>
