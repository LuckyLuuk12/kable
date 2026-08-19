<!-- @component
AdvancedSettingsUI - Advanced configuration settings panel
-->
<script lang="ts">
import { Icon, app } from "$lib";

type LocalExtra = {
  key: string;
  value: string;
};

let localExtra: LocalExtra[] = $state([]);
let collapsed = $state(true);

const advanced = () => app.customizationService.settings!.advanced!;

// Initialize the UI representation of the extra settings.
// This component assumes the customization service has already been initialized.
$effect(() => {
  const extra = advanced().extra ?? {};

  localExtra = Object.entries(extra).map(([key, value]) => ({
    key,
    value,
  }));
});

function syncExtraToSettings() {
  const extra: Record<string, string> = {};

  for (const { key, value } of localExtra) {
    if (key.trim()) {
      extra[key] = value;
    }
  }

  advanced().extra = extra;
  app.customizationService.scheduleSave();
}

function handleKeyChange(index: number, newKey: string) {
  const trimmedKey = newKey.trim();

  // Don't allow duplicate keys.
  if (trimmedKey && localExtra.some((entry, i) => i !== index && entry.key === trimmedKey)) {
    return;
  }

  localExtra[index].key = newKey;
  syncExtraToSettings();
}

function handleValueChange(index: number, newValue: string) {
  localExtra[index].value = newValue;
  syncExtraToSettings();
}

function removeExtra(index: number) {
  localExtra = [...localExtra.slice(0, index), ...localExtra.slice(index + 1)];

  syncExtraToSettings();
}

function addExtra() {
  let key = "key1";
  let i = 1;

  const existing = new Set(localExtra.map((entry) => entry.key));

  while (existing.has(key)) {
    i++;
    key = `key${i}`;
  }

  localExtra = [
    ...localExtra,
    {
      key,
      value: "",
    },
  ];

  syncExtraToSettings();
}

function scheduleSave() {
  app.customizationService.scheduleSave();
}
</script>

{#if app.customizationService.settings?.advanced}
  <div class="settings-tab">
    <h2>Advanced Settings</h2>

    <form>
      <div class="setting-item">
        <div class="setting-info">
          <label for="show-advanced-page">Show Advanced Page</label>
          <p class="setting-description">Display the Advanced page in the navigation bar</p>
        </div>

        <div class="setting-control">
          <label class="toggle-switch">
            <input type="checkbox" id="show-advanced-page" bind:checked={app.customizationService.settings!.advanced!.enable_advanced_features} onchange={scheduleSave} />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label for="check-nightly-updates">Check Nightly Updates</label>
          <p class="setting-description">Enable automatic checks for nightly/prerelease builds (unstable, for testing)</p>
        </div>

        <div class="setting-control">
          <label class="toggle-switch">
            <input
              type="checkbox"
              id="check-nightly-updates"
              bind:checked={app.customizationService.settings!.advanced!.enable_nightly_updates}
              onchange={scheduleSave}
            />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label for="developer-mode">Developer Mode</label>
          <p class="setting-description">Enable additional developer and debugging functionality</p>
        </div>

        <div class="setting-control">
          <label class="toggle-switch">
            <input type="checkbox" id="developer-mode" bind:checked={app.customizationService.settings!.advanced!.developer_mode} onchange={scheduleSave} />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item advanced-extra-item">
        <div class="setting-info">
          <!-- svelte-ignore a11y_label_has_associated_control -->
          <label id="advanced-extra-label">Advanced Extra Settings</label>

          <p class="setting-description">Add, edit, or remove advanced key-value pairs. Values are stored as strings and are used by advanced features.</p>
        </div>

        <div class="setting-control advanced-extra-control">
          <div class="extra-table">
            <div class="extra-table-header">
              <span>Key</span>
              <span>Value</span>

              <span class="collapse-toggle">
                <button type="button" class="collapse-btn" onclick={() => (collapsed = !collapsed)} title={collapsed ? "Expand all" : "Collapse all"}>
                  <Icon name={collapsed ? "chevron-down" : "chevron-up"} forceType="svg" />
                </button>
              </span>
            </div>

            {#if !collapsed}
              {#each localExtra as entry, i (i)}
                <div class="extra-row">
                  <input
                    class="extra-key"
                    type="text"
                    aria-labelledby="advanced-extra-label"
                    value={entry.key}
                    oninput={(event) => handleKeyChange(i, (event.currentTarget as HTMLInputElement).value)}
                    placeholder="Key"
                    autocomplete="off"
                  />

                  <textarea
                    class="extra-value"
                    aria-label="Value for extra setting"
                    value={entry.value}
                    oninput={(event) => handleValueChange(i, (event.currentTarget as HTMLTextAreaElement).value)}
                    placeholder="Value"
                    autocomplete="off"
                    rows="1"
                  ></textarea>

                  <button type="button" class="remove-btn" onclick={() => removeExtra(i)} title="Remove">
                    <Icon name="delete" forceType="svg" />
                  </button>
                </div>
              {/each}
            {/if}
          </div>

          <button type="button" class="add-btn" onclick={addExtra}> Add Extra Setting </button>
        </div>
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

.slider-control > input[type="range"] {
  width: 100%;
  max-width: 20rem;
}

.slider-control > input[type="number"] {
  width: 7rem;
  flex: 0 0 7rem;
}

/* Inputs */

input[type="text"],
input[type="number"],
select,
.extra-key,
.extra-value {
  border: 1px solid $color-border;
  border-radius: $radius-md;
  background: $color-surface-2;
  color: $color-text;
  font: inherit;
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

input[type="text"],
input[type="number"],
select {
  width: 100%;
  max-width: 20rem;
  padding: 0.55rem 0.8rem;
  font-size: 0.9rem;
}

select {
  cursor: pointer;
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

/* Advanced extra/list editor */

.advanced-extra-item {
  align-items: flex-start;
}

.advanced-extra-control {
  flex-direction: column;
  align-items: stretch;
  gap: 0.75rem;

  min-width: 18rem;
  width: 100%;
}

/* Table-like editor */

.extra-table {
  width: 100%;

  display: flex;
  flex-direction: column;
  gap: 0.5rem;

  padding: 0.75rem;

  background: $color-surface-2;
  border: 1px solid $color-border-muted;
  border-radius: $radius-lg;
}

.extra-table-header {
  display: grid;
  grid-template-columns: minmax(5rem, 10rem) minmax(0, 1fr) 2rem;
  gap: 0.5rem;

  padding: 0 0.25rem 0.25rem;

  color: $color-text-muted;
  font-size: 0.75rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.extra-table-header span {
  min-width: 0;
}

.extra-row {
  display: grid;
  grid-template-columns: minmax(5rem, 10rem) minmax(0, 1fr) 2rem;
  gap: 0.5rem;

  align-items: stretch;
}

.extra-key {
  min-width: 0;
  width: 100%;

  padding: 0.55rem 0.7rem;

  font-size: 0.9rem;
  box-sizing: border-box;
}

.extra-value {
  min-width: 0;
  width: 100%;

  min-height: 2.35rem;
  padding: 0.55rem 0.7rem;

  font-size: 0.9rem;
  line-height: 1.4;

  resize: vertical;
  box-sizing: border-box;
}

.remove-btn {
  display: flex;
  align-items: center;
  justify-content: center;

  width: 2rem;
  min-width: 2rem;
  height: 2.35rem;
  padding: 0;

  border: 1px solid transparent;
  border-radius: $radius-md;

  background: transparent;
  color: $color-error;

  font-size: 1rem;
  cursor: pointer;

  transition:
    background 0.2s ease,
    border-color 0.2s ease,
    color 0.2s ease;

  &:hover {
    background: color-mix(in srgb, $color-error 12%, transparent);
    border-color: color-mix(in srgb, $color-error 35%, transparent);
    color: $color-error;
  }

  &:focus-visible {
    outline: none;
    border-color: $color-focus;
    box-shadow: 0 0 0 3px color-mix(in srgb, $color-focus 20%, transparent);
  }
}

.add-btn {
  align-self: flex-start;

  display: inline-flex;
  align-items: center;
  justify-content: center;

  min-height: 2.25rem;
  margin-top: 0.25rem;
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

/* Collapse */

.collapse-toggle {
  display: flex;
  align-items: center;
  justify-content: flex-end;

  height: 100%;
}

.collapse-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;

  width: 2.25rem;
  height: 2.25rem;
  padding: 0;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  background: $color-surface-2;
  color: $color-text-muted;

  font-size: 1rem;
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

  .advanced-extra-control {
    min-width: 0;
  }

  .extra-table {
    padding: 0.5rem;
  }

  .extra-table-header {
    display: none;
  }

  .extra-row {
    grid-template-columns: minmax(0, 1fr) 2rem;
    gap: 0.5rem;
  }

  .extra-key {
    grid-column: 1;
  }

  .extra-value {
    grid-column: 1;
  }

  .remove-btn {
    grid-column: 2;
    grid-row: 1 / span 2;
    align-self: center;
  }

  input[type="text"],
  input[type="number"],
  select {
    max-width: none;
  }

  input[type="range"] {
    max-width: none;
  }
}
</style>
