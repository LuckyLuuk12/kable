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
input[type="number"] {
  font-size: 1rem;
  padding: 0.4em 0.8em;
  border-radius: var(--border-radius);
  border: 1px solid var(--dark-200);
  color: var(--text);
  width: 7rem;
}

.toggle-switch {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
</style>
