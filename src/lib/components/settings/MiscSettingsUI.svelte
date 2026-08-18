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
          <p class="setting-description">Enable additional fun and non-essential features in the launcher.</p>
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
.radio-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
select {
  min-width: 12rem;
  padding: 0.5rem 1rem;
  border-radius: var(--border-radius);
  background: var(--input);
  color: var(--text);
  border: 1px solid var(--dark-200);
  font-size: 1rem;
}
.toggle-switch {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
</style>
