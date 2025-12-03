<!-- @component
AdvancedSettingsUI - Advanced configuration settings panel

Provides interface for advanced launcher settings including JVM overrides,
custom paths, experimental features, and key-value extra settings.

@example
```svelte
◄AdvancedSettingsUI /►
```
-->
<script lang="ts">
import { settings } from "$lib/stores";
import { get } from "svelte/store";
import Icon from "$lib/components/Icon.svelte";
let collapsed = false;
// Local state for editing extra settings
import { onMount } from "svelte";
let localExtra: Array<{ key: string; value: string }> = [];

// Initialize localExtra from store on mount
onMount(() => {
  const adv = get(settings).advanced;
  const entries = Object.entries(adv.extra || {});
  localExtra = entries.map(([key, value]) => ({
    key,
    value: typeof value === "string" ? value : JSON.stringify(value),
  }));
});

// Sync localExtra to store
function syncToStore() {
  const adv = { ...get(settings).advanced };
  adv.extra = {};
  for (const { key, value } of localExtra) {
    if (key) adv.extra[key] = parseValue(value);
  }
  settings.update((s) => ({ ...s, advanced: adv }));
}

function handleKeyChange(index: number, newKey: string) {
  if (!newKey) return;
  // Prevent duplicate keys
  if (localExtra.some((e, i) => i !== index && e.key === newKey)) return;
  localExtra[index].key = newKey;
  syncToStore();
}

function handleValueChange(index: number, newValue: string) {
  localExtra[index].value = newValue;
  syncToStore();
}

function removeExtra(index: number) {
  localExtra = [...localExtra.slice(0, index), ...localExtra.slice(index + 1)];
  syncToStore();
}

function addExtra() {
  let base = "key";
  let i = 1;
  const existing = new Set(localExtra.map((e) => e.key));
  while (existing.has(base + i)) i++;
  localExtra = [...localExtra, { key: base + i, value: "" }];
  syncToStore();
}

function parseValue(val: string): any {
  try {
    return JSON.parse(val);
  } catch {
    return val;
  }
}
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
          <input
            type="checkbox"
            id="enable-experimental-features"
            bind:checked={$settings.advanced.enable_experimental_features}
          />
          <span class="toggle-slider"></span>
        </label>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="default-memory">Default Memory Allocation (MB)</label>
        <p class="setting-description">
          Default RAM allocated to new installations
        </p>
      </div>
      <div class="setting-control slider-control">
        <input
          type="range"
          id="default-memory-slider"
          min="512"
          max="131072"
          step="256"
          bind:value={$settings.advanced.default_memory}
        />
        <input
          type="number"
          id="default-memory"
          min="512"
          max="131072"
          step="256"
          bind:value={$settings.advanced.default_memory}
        />
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="separate-logs-window">Separate Logs Window</label>
        <p class="setting-description">
          Show logs in a separate window (experimental)
        </p>
      </div>
      <div class="setting-control">
        <label class="toggle-switch">
          <input
            type="checkbox"
            id="separate-logs-window"
            bind:checked={$settings.advanced.separate_logs_window}
          />
          <span class="toggle-slider"></span>
        </label>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="show-advanced-page">Show Advanced Page</label>
        <p class="setting-description">
          Display the Advanced page in the navigation bar
        </p>
      </div>
      <div class="setting-control">
        <label class="toggle-switch">
          <input
            type="checkbox"
            id="show-advanced-page"
            bind:checked={$settings.advanced.show_advanced_page}
          />
          <span class="toggle-slider"></span>
        </label>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="check-nightly-updates">Check Nightly Updates</label>
        <p class="setting-description">
          Enable automatic checks for nightly/prerelease builds (unstable, for
          testing)
        </p>
      </div>
      <div class="setting-control">
        <label class="toggle-switch">
          <input
            type="checkbox"
            id="check-nightly-updates"
            bind:checked={$settings.advanced.check_nightly_updates}
          />
          <span class="toggle-slider"></span>
        </label>
      </div>
    </div>

    <div class="setting-item advanced-extra-item">
      <div class="setting-info">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label id="advanced-extra-label">Advanced Extra Settings</label>
        <p class="setting-description">
          Add, edit, or remove advanced key-value pairs. Values can be JSON,
          strings, numbers, or hex (e.g. <code>0x1234</code>), and are used by
          advanced features.
        </p>
      </div>
      <div class="setting-control advanced-extra-control">
        <div class="extra-table">
          <div class="extra-table-header">
            <span>Key</span>
            <span>Value (JSON)</span>
            <span class="collapse-toggle">
              <button
                type="button"
                class="collapse-btn"
                on:click={() => (collapsed = !collapsed)}
                title={collapsed ? "Expand all" : "Collapse all"}
              >
                <Icon
                  name={collapsed ? "chevron-down" : "chevron-up"}
                  forceType="svg"
                />
              </button>
            </span>
          </div>
          {#if !collapsed}
            {#each localExtra as entry, i (entry.key)}
              <div class="extra-row">
                <input
                  class="extra-key"
                  type="text"
                  aria-labelledby="advanced-extra-label"
                  bind:value={entry.key}
                  on:input={(e) =>
                    handleKeyChange(i, (e.target as HTMLInputElement).value)}
                  placeholder="Key"
                  autocomplete="off"
                />
                <textarea
                  class="extra-value"
                  aria-label="Value for extra setting"
                  bind:value={entry.value}
                  on:input={(e) =>
                    handleValueChange(
                      i,
                      (e.target as HTMLTextAreaElement).value,
                    )}
                  placeholder="Value (JSON)"
                  autocomplete="off"
                  rows="1"
                  style="resize:vertical; min-height:2.1em; max-height:12em; width:100%;"
                ></textarea>
                <button
                  type="button"
                  class="remove-btn"
                  on:click={() => removeExtra(i)}
                  title="Remove"
                >
                  <Icon name="delete" forceType="svg" />
                </button>
              </div>
            {/each}
          {/if}
        </div>
        <button type="button" class="add-btn" on:click={addExtra}
          >Add Extra Setting</button
        >
      </div>
    </div>
  </form>
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;

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
  background: linear-gradient(to right, $primary, $secondary);
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

.advanced-extra-item {
  align-items: flex-start;
}
.advanced-extra-control {
  flex-direction: column;
  align-items: flex-start;
  gap: 0.7rem;
  min-width: 18rem;
  width: 100%;
}
.extra-table {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}
.extra-table-header {
  display: flex;
  align-items: stretch;
  gap: 0.5rem;
  font-size: 0.98rem;
  color: var(--placeholder);
  font-weight: 500;
  margin-bottom: 0.2rem;
}
.extra-table-header span:first-child {
  flex: 0 1 8rem;
  max-width: 10rem;
  min-width: 5rem;
  text-align: left;
}
.extra-table-header span:nth-child(2) {
  flex: 2 1 60%;
  text-align: left;
}
.extra-table-header span:last-child {
  flex: 0 0 auto;
}
.extra-row {
  display: flex;
  align-items: stretch;
  gap: 0.5rem;
  margin-bottom: 0.1rem;
}
.extra-key {
  flex: 0 1 8rem;
  max-width: 10rem;
  min-width: 5rem;
  border: 1px solid var(--dark-200);
  border-radius: 4px;
  background: var(--background);
  color: var(--text);
  font-size: 1rem;
}
.extra-value {
  flex: 2 1 60%;
  border: 1px solid var(--dark-200);
  border-radius: 4px;
  background: var(--background);
  color: var(--text);
  font-size: 1rem;
  font-family: inherit;
  resize: vertical;
  line-height: 1.2;
  width: 100%;
  box-sizing: border-box;
}
.remove-btn {
  flex: 0 0 auto;
  background: none;
  border: none;
  color: var(--red-600);
  font-size: 1.1rem;
  cursor: pointer;
  padding: 0 0.3rem;
  transition: color 0.2s;
  align-self: center;
}
.remove-btn:hover {
  color: var(--red-700);
}

.add-btn {
  margin-top: 0.5rem;
  border: none;
  border-radius: 4px;
  padding: 0.3rem 0.8rem;
  font-size: 1rem;
  cursor: pointer;

  transition: background 0.2s;
}

.collapse-toggle {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  height: 100%;
}
.collapse-btn {
  background: none;
  border: none;
  color: var(--text);
  font-size: 1.1rem;
  cursor: pointer;
  padding: 0 0.2rem;
  display: flex;
  align-items: center;
  transition: color 0.2s;
}
</style>
