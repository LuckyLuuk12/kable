<!-- @component
ContentSettingsUI - Content management settings panel

Configures automatic world backups, backup retention limits,
and other content-related settings.

@example
```svelte
◄ContentSettingsUI /►
```
-->
<script lang="ts">
import { app } from "$lib";
</script>

<div class="settings-tab">
  <h2>Content Settings</h2>
  <p>Configure content-related features and notifications.</p>

  {#if app.customizationService.settings?.content}
    <form>
      <div class="setting-item">
        <div class="setting-info">
          <label for="allow-adult-content">Allow Adult Content</label>
          <p class="setting-description">Allow content that may be intended for mature audiences.</p>
        </div>

        <div class="setting-control">
          <label class="toggle-switch">
            <input
              type="checkbox"
              id="allow-adult-content"
              bind:checked={app.customizationService.settings.content!.allow_adult_content}
              onchange={() => app.customizationService.scheduleSave()}
            />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label for="allow-ads">Allow Advertisements</label>
          <p class="setting-description">Allow advertisements to be displayed by the launcher.</p>
        </div>

        <div class="setting-control">
          <label class="toggle-switch">
            <input
              type="checkbox"
              id="allow-ads"
              bind:checked={app.customizationService.settings.content!.allow_ads}
              onchange={() => app.customizationService.scheduleSave()}
            />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label for="enable-recommendations">Recommendations</label>
          <p class="setting-description">Enable personalized content and feature recommendations.</p>
        </div>

        <div class="setting-control">
          <label class="toggle-switch">
            <input
              type="checkbox"
              id="enable-recommendations"
              bind:checked={app.customizationService.settings.content!.enable_recommendations}
              onchange={() => app.customizationService.scheduleSave()}
            />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label for="enable-notifications">Notifications</label>
          <p class="setting-description">Allow the launcher to display content-related notifications.</p>
        </div>

        <div class="setting-control">
          <label class="toggle-switch">
            <input
              type="checkbox"
              id="enable-notifications"
              bind:checked={app.customizationService.settings.content!.enable_notifications}
              onchange={() => app.customizationService.scheduleSave()}
            />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>
    </form>
  {:else if app.customizationService.loading}
    <p>Loading content settings...</p>
  {:else}
    <p>Content settings are unavailable.</p>
  {/if}
</div>

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

.toggle-switch {
  display: flex;
  align-items: center;
  gap: 0.65rem;

  min-height: 2rem;
}

.toggle-switch label {
  display: flex;
  align-items: center;
  gap: 0.65rem;

  color: $color-text;
  font-size: 0.9rem;

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
    box-shadow 0.2s ease,
    transform 0.15s ease;

  &:hover {
    border-color: $color-accent-muted;
    background: $color-surface-3;
  }

  &:active {
    transform: scale(0.94);
  }

  &:checked {
    border-color: $color-accent;
    background: $color-accent;
  }

  &:focus-visible {
    outline: none;

    border-color: $color-focus;
    box-shadow: 0 0 0 3px color-mix(in srgb, $color-focus 20%, transparent);
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
}

input[type="checkbox"]:checked::after {
  content: "";

  display: block;

  width: 0.35rem;
  height: 0.65rem;

  margin: 0.12rem auto 0;

  border-right: 2px solid $color-text;
  border-bottom: 2px solid $color-text;

  transform: rotate(45deg);
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
    min-width: 0;
  }
}
</style>
