<script lang="ts">
import { app, Icon, launchSound } from "$lib";

let isLaunching = $derived(app.launcherService.launching);

let launchStatus = $state("");

async function launch() {
  launchStatus = "Preparing to launch...";

  try {
    await app.launcherService.launchLatest();

    launchStatus = "Launched Minecraft!";
  } catch (error) {
    console.error("Launch error:", error);
    launchStatus = `Launch failed: ${error}`;
  } finally {
    setTimeout(() => {
      launchStatus = "";
    }, 2000);
  }
}
</script>

<div class="play-section">
  {#if !app.profilesService.profiles.length}
    <p class="no-installations">No installations found. Please check your Minecraft directory in settings.</p>
  {/if}

  {#if launchStatus}
    <p class="launch-status" class:error={launchStatus.includes("fail") || launchStatus.includes("error")}>
      {launchStatus}
    </p>
  {/if}
  <button class="play-button" onclick={launch} use:launchSound disabled={isLaunching}>
    {#if isLaunching}
      <Icon name="refresh" size="md" forceType="svg" className="spin" />
      <span>Launching...</span>
    {:else}
      <Icon name="play" size="md" forceType="svg" />
      <span>Play Minecraft</span>
    {/if}
  </button>
</div>

<style lang="scss">
.play-section {
  position: absolute;
  left: 50%;
  bottom: $space-md;
  transform: translateX(-50%);

  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
}

.play-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;

  width: 20rem;
  padding: 1rem 2rem;

  background: $color-accent;
  color: $color-text;

  border: none;
  border-radius: 0.75rem;

  font-size: 1.1rem;
  font-weight: 600;

  cursor: pointer;
  transition: all 0.2s ease;

  &:hover:not(:disabled) {
    background: $color-accent-hover;
    transform: translateY(-0.125rem);
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    transform: none;
  }
}

.no-installations {
  margin: 1rem 0 0;

  color: $color-placeholder;
  font-size: 0.875rem;
}

.launch-status {
  margin: 1rem 0 0;
  padding: 0.75rem 1rem;

  border-radius: 8px;
  border: 1px solid color-mix(in srgb, var(--color-success), 30%, transparent);

  background: color-mix(in srgb, var(--color-success), 10%, transparent);
  color: $color-success;

  font-size: 0.875rem;

  &.error {
    background: color-mix(in srgb, var(--color-error), 10%, transparent);
    color: $color-error;
    border-color: color-mix(in srgb, var(--color-error), 30%, transparent);
  }
}

@media (max-width: 768px) {
  .play-section {
    padding-left: 1rem;
    padding-right: 1rem;
  }

  .play-button {
    min-width: auto;
    width: 100%;
    max-width: 300px;
  }
}
</style>
