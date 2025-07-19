<script lang="ts">
  import { settings } from "$lib/stores";
</script>

<div class="settings-tab">
  <h2>Advanced Settings</h2>
  <form>
    <div class="setting-item">
      <div class="setting-info">
        <label for="enable-experimental-features">Experimental Features</label>
        <p class="setting-description">Enable experimental launcher features</p>
      </div>
      <div class="setting-control">
        <label class="toggle-switch">
          <input type="checkbox" id="enable-experimental-features" bind:checked={$settings.advanced.enable_experimental_features} />
          <span class="toggle-slider"></span>
        </label>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="default-memory">Default Memory Allocation (MB)</label>
        <p class="setting-description">Default RAM allocated to new installations</p>
      </div>
      <div class="setting-control slider-control">
        <input type="range" id="default-memory-slider" min="512" max="131072" step="256" bind:value={$settings.advanced.default_memory} />
        <input type="number" id="default-memory" min="512" max="131072" step="256" bind:value={$settings.advanced.default_memory} />
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="separate-logs-window">Separate Logs Window</label>
        <p class="setting-description">Show logs in a separate window (experimental)</p>
      </div>
      <div class="setting-control">
        <label class="toggle-switch">
          <input type="checkbox" id="separate-logs-window" bind:checked={$settings.advanced.separate_logs_window} />
          <span class="toggle-slider"></span>
        </label>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="auto-save-interval">Auto Save Interval (seconds)</label>
        <p class="setting-description">How often to auto save settings (0 to disable)</p>
      </div>
      <div class="setting-control">
        <input type="number" id="auto-save-interval" min="5000" bind:value={$settings.advanced.auto_save_interval} />
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
.slider-control > input[type="range"],
.slider-control > input[type="number"] {
  display: block;
  width: 14rem;
  margin-bottom: 0.5rem;
}
.toggle-switch {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
</style>

