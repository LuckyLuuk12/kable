<!-- @component
MiscSettingsUI - Miscellaneous settings panel

Additional settings including titlebar preferences, auto-updates,
and other general launcher features.

@example
```svelte
◄MiscSettingsUI /►
```
-->
<script lang="ts">
import { app } from "$lib";

function setEnableFun(enabled: boolean) {
  const settings = app.customizationService.settings;
  if (!settings) return;

  app.customizationService.settings = {
    ...settings,
    misc: {
      ...settings.misc,
      enable_fun: enabled,
    },
  };

  app.customizationService.scheduleSave();
}
</script>

{#if app.customizationService.settings}
  <div class="settings-tab">
    <h2>Miscellaneous Settings</h2>
    <p>Configure miscellaneous launcher features and experimental options.</p>

    <form>
      <div class="setting-item">
        <div class="setting-info">
          <label for="enable-fun">Fun Features</label>
          <p class="setting-description">
            Enable additional fun and non-essential features in the launcher.<br /> I recommend you just leave this enabled, there are no negative consequences.
          </p>
        </div>

        <div class="setting-control">
          <label class="toggle-switch">
            <input
              type="checkbox"
              id="enable-fun"
              checked={app.customizationService.settings.misc?.enable_fun ?? false}
              onchange={(event) => setEnableFun((event.currentTarget as HTMLInputElement).checked)}
            />
            <span class="toggle-slider"></span>
          </label>
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
}
</style>
