<script lang="ts">
  import { settings } from "$lib/stores";
  function disableMaxWorldBackups() {
    $settings.content.max_world_backups = 'disabled';
  }
</script>

<div class="settings-tab">
  <h2>Content Management Settings</h2>
  <form>
    <div class="setting-item">
      <div class="setting-info">
        <label for="auto-backup-worlds">Auto-backup Worlds</label>
        <p class="setting-description">Automatically create backups before modifying worlds</p>
      </div>
      <div class="setting-control">
        <label class="toggle-switch">
          <input type="checkbox" id="auto-backup-worlds" bind:checked={$settings.content.auto_backup_worlds} />
        </label>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="max-world-backups">Maximum World Backups</label>
        <p class="setting-description">How many backups to keep per world (set to 0 or 'disabled' to turn off)</p>
      </div>
      <div class="setting-control">
        <input type="number" id="max-world-backups" min="0" bind:value={$settings.content.max_world_backups} />
        <button type="button" on:click={disableMaxWorldBackups}>Disable</button>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="per-installation-mods">Per-Installation Mods Folder</label>
        <p class="setting-description">Use a separate mods folder for each installation</p>
      </div>
      <div class="setting-control">
        <label class="toggle-switch">
          <input type="checkbox" id="per-installation-mods" bind:checked={$settings.content.use_per_installation_mods_folder} />
        </label>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="per-installation-resource-packs">Per-Installation Resource Packs</label>
        <p class="setting-description">Use a separate resource packs folder for each installation</p>
      </div>
      <div class="setting-control">
        <label class="toggle-switch">
          <input type="checkbox" id="per-installation-resource-packs" bind:checked={$settings.content.use_per_installation_resource_packs} />
        </label>
      </div>
    </div>
  </form>
  <!-- Save status and backend update logic handled in parent Settings component -->
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
input[type="number"] {
  font-size: 1rem;
  padding: 0.4em 0.8em;
  border-radius: $border-radius;
  border: 1px solid $dark-200;
  color: $text;
  width: 7rem;
}

  .toggle-switch {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
</style>