<script lang="ts">
  import { settings } from "$lib/stores";
  import { SettingsManager } from "$lib/managers/SettingsManager";
  function setUnlimitedDownloadSpeed() {
    SettingsManager.update('network', {
      ...$settings.network,
      download_speed_limit: 'unlimited'
    });
  }
</script>

<div class="settings-tab">
  <h2>Network & Downloads Settings</h2>
  <form>
    <div class="setting-item">
      <div class="setting-info">
        <label for="parallel-downloads">Parallel Downloads</label>
        <p class="setting-description">Number of simultaneous downloads</p>
      </div>
      <div class="setting-control slider-control">
        <div class="slider-inputs">
          <input type="range" id="parallel-downloads-slider" min="1" max="64" bind:value={$settings.network.parallel_downloads} />
          <input type="number" id="parallel-downloads" min="1" max="64" bind:value={$settings.network.parallel_downloads} />
        </div>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="connection-timeout">Connection Timeout (seconds)</label>
        <p class="setting-description">Network timeout for downloads and API calls</p>
      </div>
      <div class="setting-control slider-control">
        <div class="slider-inputs">
          <input type="range" id="connection-timeout-slider" min="1" max="360" bind:value={$settings.network.connection_timeout} />
          <input type="number" id="connection-timeout" min="1" max="360" bind:value={$settings.network.connection_timeout} />
        </div>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="download-speed-limit">Download Speed Limit (MB/s)</label>
        <p class="setting-description">How much to throttle the download speed for parallel downloads</p>
      </div>
      <div class="setting-control slider-control download-speed-layout">
        <div class="download-speed-inputs">
          <input type="range" id="download-speed-limit-slider" min="0" max="20000"
            value={$settings.network.download_speed_limit === 'unlimited' ? 0 : $settings.network.download_speed_limit}
            disabled={$settings.network.download_speed_limit === 'unlimited'}
            on:input={(e) => {
              if ($settings.network.download_speed_limit !== 'unlimited') {
                SettingsManager.update('network', {
                  ...$settings.network,
                  download_speed_limit: Number((e.target as HTMLInputElement)?.value)
                });
              }
            }}
          />
          <input type="number" id="download-speed-limit" min="0" max="20000"
            value={$settings.network.download_speed_limit === 'unlimited' ? '' : $settings.network.download_speed_limit}
            disabled={$settings.network.download_speed_limit === 'unlimited'}
            on:input={(e) => {
              if ($settings.network.download_speed_limit !== 'unlimited') {
                SettingsManager.update('network', {
                  ...$settings.network,
                  download_speed_limit: Number((e.target as HTMLInputElement)?.value)
                });
              }
            }}
          />
        </div>
        <div class="download-speed-btn">
          {#if $settings.network.download_speed_limit !== 'unlimited'}
            <button type="button" class="primary" on:click={setUnlimitedDownloadSpeed}>Unlimited</button>
          {:else}
            <button type="button" class="primary" on:click={() => SettingsManager.update('network', {
              ...$settings.network,
              download_speed_limit: 100
            })}>Use limit</button>
          {/if}
        </div>
      </div>
    </div>
  </form>
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;

// px to rem conversion: 1rem = 16px
.settings-tab {
  background: $container;
  border-radius: $border-radius-large;
  box-shadow: 0 0.125rem 0.5rem rgba(0,0,0,0.08); // 2px 8px
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
  flex: 1 1 16.25rem; // 260px
  min-width: 13.75rem; // 220px
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
  flex: 1 1 11.25rem; // 180px
  display: flex;
  align-items: center;
  gap: 1rem;
  min-width: 10rem; // 160px
}
.slider-control {
  gap: 0.7rem;
}
.slider-inputs {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  min-width: 12rem;
  width: 14rem;
}
.download-speed-layout {
  align-items: flex-start;
  gap: 1.5rem;
}
.download-speed-inputs {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  min-width: 12rem;
  width: 14rem;
}
.download-speed-btn {
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
</style>