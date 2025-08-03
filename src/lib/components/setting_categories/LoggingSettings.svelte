<script lang="ts">
  import { settings } from "$lib/stores";
  import { SettingsService } from "$lib/services/SettingsService";
  import { error } from "@sveltejs/kit";
  const logLevels = ['debug', 'info', 'warn', 'error'];
  function disableFileSizeLimit() {
    SettingsService.update('logging', {
      ...$settings.logging,
      log_file_size_limit_mb: 'disabled'
    });
  }
  function disableRetentionDays() {
    SettingsService.update('logging', {
      ...$settings.logging,
      log_retention_days: 'disabled'
    });
  }
</script>

<div class="settings-tab">
  <h2>Logging Settings</h2>
  <form>
    <div class="setting-item">
      <div class="setting-info">
        <label for="show-logs-page-nav">Show Logs Page in Navigation</label>
        <p class="setting-description">Display the logs page in the sidebar navigation</p>
      </div>
      <div class="setting-control">
        <label class="toggle-switch">
          <input type="checkbox" id="show-logs-page-nav" bind:checked={$settings.logging.show_logs_page_in_nav} />
          <span class="toggle-slider"></span>
        </label>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="enable-persistent-logging">Persistent Log Storage</label>
        <p class="setting-description">Save logs to disk for permanent storage and analysis</p>
      </div>
      <div class="setting-control">
        <label class="toggle-switch">
          <input type="checkbox" id="enable-persistent-logging" bind:checked={$settings.logging.enable_persistent_logging} />
          <span class="toggle-slider"></span>
        </label>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="enable-log-compression">Log Compression</label>
        <p class="setting-description">Automatically compress large log files to save disk space</p>
      </div>
      <div class="setting-control">
        <label class="toggle-switch">
          <input type="checkbox" id="enable-log-compression" bind:checked={$settings.logging.enable_log_compression} />
          <span class="toggle-slider"></span>
        </label>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="log-file-size-limit">Log File Size Limit (MB)</label>
        <p class="setting-description">Maximum size for individual log files before compression</p>
      </div>
      <div class="setting-control slider-control log-file-size-layout">
        <div class="log-file-size-inputs">
          <input type="range" id="log-file-size-limit-slider" min="1" max="1024"
            value={$settings.logging.log_file_size_limit_mb === 'disabled' ? 1 : $settings.logging.log_file_size_limit_mb}
            disabled={$settings.logging.log_file_size_limit_mb === 'disabled'}
            on:input={(e) => {
              if ($settings.logging.log_file_size_limit_mb !== 'disabled') {
                SettingsService.update('logging', {
                  ...$settings.logging,
                  log_file_size_limit_mb: Number((e.target as HTMLInputElement).value)
                });
              }
            }}
          />
          <input type="number" id="log-file-size-limit" min="1" max="1024"
            value={$settings.logging.log_file_size_limit_mb === 'disabled' ? '' : $settings.logging.log_file_size_limit_mb}
            disabled={$settings.logging.log_file_size_limit_mb === 'disabled'}
            on:input={(e) => {
              if ($settings.logging.log_file_size_limit_mb !== 'disabled') {
                SettingsService.update('logging', {
                  ...$settings.logging,
                  log_file_size_limit_mb: Number((e.target as HTMLInputElement).value)
                });
              }
            }}
          />
        </div>
        <div class="log-file-size-btn">
          <button type="button" on:click={disableFileSizeLimit}>Disable</button>
        </div>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="log-retention-days">Log Retention Period (days)</label>
        <p class="setting-description">How many days to keep log files before automatic cleanup</p>
      </div>
      <div class="setting-control slider-control log-retention-layout">
        <div class="log-retention-inputs">
          <input type="range" id="log-retention-days-slider" min="1" max="365"
            value={$settings.logging.log_retention_days === 'disabled' ? 1 : $settings.logging.log_retention_days}
            disabled={$settings.logging.log_retention_days === 'disabled'}
            on:input={(e) => {
              if ($settings.logging.log_retention_days !== 'disabled') {
                SettingsService.update('logging', {
                  ...$settings.logging,
                  log_retention_days: Number((e.target as HTMLInputElement).value)
                });
              }
            }}
          />
          <input type="number" id="log-retention-days" min="1" max="365"
            value={$settings.logging.log_retention_days === 'disabled' ? '' : $settings.logging.log_retention_days}
            disabled={$settings.logging.log_retention_days === 'disabled'}
            on:input={(e) => {
              if ($settings.logging.log_retention_days !== 'disabled') {
                SettingsService.update('logging', {
                  ...$settings.logging,
                  log_retention_days: Number((e.target as HTMLInputElement).value)
                });
              }
            }}
          />
        </div>
        <div class="log-retention-btn">
          <button type="button" on:click={disableRetentionDays}>Disable</button>
        </div>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="merge-log-tabs">Merge Log Tabs</label>
        <p class="setting-description">Try to merge log tabs into one if they are from the same game instance</p>
      </div>
      <div class="setting-control">
        <label class="toggle-switch">
          <input type="checkbox" id="merge-log-tabs" bind:checked={$settings.logging.merge_log_tabs} />
          <span class="toggle-slider"></span>
        </label>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label>Default Log Levels</label>
        <p class="setting-description">Which log levels are shown by default</p>
      </div>
      <div class="setting-control log-levels-control">
        {#each logLevels as level}
          <label
            class="log-level-label { $settings.logging.default_log_levels.includes(level as 'debug' | 'info' | 'warn' | 'error') ? 'selected' : 'unselected' }"
            role="none"
            tabindex="-1"
            on:click={(e) => {
              const typedLevel = level as 'debug' | 'info' | 'warn' | 'error';
              const idx = $settings.logging.default_log_levels.indexOf(typedLevel);
              if (idx === -1) {
                console.log(`Adding log level: ${typedLevel}`);
                SettingsService.update('logging', {
                  ...$settings.logging,
                  default_log_levels: [...$settings.logging.default_log_levels, typedLevel]
                });
              } else {
                console.log(`Removing log level: ${typedLevel}`);
                SettingsService.update('logging', {
                  ...$settings.logging,
                  default_log_levels: $settings.logging.default_log_levels.filter(l => l !== typedLevel)
                });
              }
              e.preventDefault();
            }}
            on:keydown={(e) => {
              if (e.key === ' ' || e.key === 'Enter') {
                const typedLevel = level as 'debug' | 'info' | 'warn' | 'error';
                const idx = $settings.logging.default_log_levels.indexOf(typedLevel);
                if (idx === -1) {
                  SettingsService.update('logging', {
                    ...$settings.logging,
                    default_log_levels: [...$settings.logging.default_log_levels, typedLevel]
                  });
                } else {
                  SettingsService.update('logging', {
                    ...$settings.logging,
                    default_log_levels: $settings.logging.default_log_levels.filter(l => l !== typedLevel)
                  });
                }
                e.preventDefault();
              }
            }}
          >
            <input type="checkbox" bind:group={$settings.logging.default_log_levels} value={level} class="visually-hidden" />
            {level}
          </label>
        {/each}
      </div>
    </div>
  </form>
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;

.settings-tab {
  background: $container;
  border-radius: $border-radius-large;
  box-shadow: 0 0.125rem 0.5rem rgba(0,0,0,0.08);
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
  background: linear-gradient(to right, $primary, $secondary);
  color: transparent;
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
  border-bottom: 1px solid $dark-200;
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
  color: $text;
  margin-bottom: 0.1rem;
}
.setting-description {
  font-size: 0.95rem;
  color: $placeholder;
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
.log-file-size-layout, .log-retention-layout {
  display: flex;
  align-items: flex-start;
  gap: 1.5rem;
}
.log-file-size-inputs, .log-retention-inputs {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  min-width: 12rem;
  width: 14rem;
}
.log-file-size-btn, .log-retention-btn {
  display: flex;
  align-items: center;
  height: 100%;
}
button[type="button"] {
  font-size: 0.95rem;
  padding: 0.35em 1em;
  border-radius: $border-radius;
  color: #fff;
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
  color: $text;
}
.log-levels-control {
  display: grid;
  gap: 1rem;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
}
.log-level-label {
  cursor: pointer;
  padding: 0.4rem 1.1rem;
  border-radius: $border-radius;
  font-weight: 600;
  user-select: none;
  transition: background 0.18s, color 0.18s;
  outline: none;
  border: 2px solid transparent;
}
.log-level-label.unselected {
  background: $input;
  color: $red-700;
}
.log-level-label.selected {
  background: $input;
  color: $green-600;
}
.log-level-label.unselected:hover {
  color: color-mix(in srgb, $red-700 70%, #fff 30%);
  border-color: $dark-200;
}
.log-level-label.selected:hover {
  color: color-mix(in srgb, $green-600 70%, #fff 30%);
  border-color: $dark-200;
}
.log-level-label:focus {
  border: 2px solid $primary;
}
.visually-hidden {
  position: absolute;
  opacity: 0;
  pointer-events: none;
  width: 0;
  height: 0;
}
</style>

