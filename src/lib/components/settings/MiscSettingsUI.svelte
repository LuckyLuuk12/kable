<script lang="ts">
  import { settings } from "$lib/stores";
  import { onMount } from 'svelte';
  import AutoUpdater from '../AutoUpdater.svelte';
  
  let isWideScreen = true;
  function checkScreen() {
    isWideScreen = window.innerWidth >= 700;
  }
  onMount(() => {
    checkScreen();
    window.addEventListener('resize', checkScreen);
    return () => window.removeEventListener('resize', checkScreen);
  });
</script>

<div class="settings-tab">
  <h2>Miscellaneous Settings</h2>
  <form>
    <div class="setting-item">
      <div class="setting-info">
        <label for="use-titlebar">Use Titlebar</label>
        <p class="setting-description">Enable a custom titlebar for the application</p>
      </div>
      <div class="setting-control">
        <label class="toggle-switch">
          <input type="checkbox" id="use-titlebar" bind:checked={$settings.misc.use_titlebar} />
          <span class="toggle-slider"></span>
        </label>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label>Authentication Preference</label>
        <p class="setting-description">Choose your preferred authentication flow</p>
      </div>
      <div class="setting-control">
        {#if isWideScreen}
          <div class="radio-group">
            <label><input type="radio" name="auth-preference" value="code" bind:group={$settings.misc.auth_preference} /> Code Flow (Recommended)</label>
            <label><input type="radio" name="auth-preference" value="device_code" bind:group={$settings.misc.auth_preference} /> Device Code Flow</label>
          </div>
        {:else}
          <select id="auth-preference" bind:value={$settings.misc.auth_preference}>
            <option value="code">Code Flow (Recommended)</option>
            <option value="device_code">Device Code Flow</option>
          </select>
        {/if}
      </div>
    </div>
  </form>
  
  <!-- Auto-updater section -->
  <AutoUpdater />
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;

.settings-tab {
  background: var(--container);
  border-radius: var(--border-radius-large);
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
  background: linear-gradient(to right, var(--primary), var(--secondary));
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