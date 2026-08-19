<script lang="ts">
import { app, SettingsUI } from "$lib";

let saveStatus = "";
</script>

<div class="settings-page">
  <div class="page-header">
    {#if saveStatus}
      <div class="warning-card" class:success={saveStatus.includes("success")} class:error={saveStatus.includes("Failed")}>
        {saveStatus}
      </div>
    {/if}
  </div>

  {#if app.customizationService.settings}
    <SettingsUI />
  {:else}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading settings...</p>
    </div>
  {/if}
</div>

<style lang="scss">
.settings-page {
  width: 100%;
  max-width: 100%;
  min-height: 100%;
  max-height: 100vh;
  overflow: hidden;
  margin: 0;
}

/* Save status */

.warning-card {
  position: fixed;
  right: 1.5rem;
  bottom: 1.5rem;
  z-index: 100;

  width: fit-content;
  max-width: min(28rem, calc(100vw - 3rem));

  @extend .card !optional;

  display: flex;
  align-items: center;

  padding: 0.8rem 1rem;

  border: 1px solid $color-border;
  border-radius: $radius-lg;

  color: $color-text;
  background: $color-surface-3;

  font-size: 0.875rem;
  line-height: 1.4;

  box-shadow:
    0 0.5rem 1.5rem rgba(0, 0, 0, 0.15),
    0 0 0 1px color-mix(in srgb, $color-border 40%, transparent);

  animation: status-enter 0.25s ease-out;

  &.success {
    border-color: color-mix(in srgb, $color-success 50%, $color-border);
    background: color-mix(in srgb, $color-success 12%, $color-surface-3);
  }

  &.error {
    border-color: color-mix(in srgb, $color-error 50%, $color-border);
    background: color-mix(in srgb, $color-error 12%, $color-surface-3);
  }
}

/* Loading */

.loading-state {
  @extend .empty-state !optional;

  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;

  min-height: 20rem;
  padding: 4rem;

  color: $color-text-muted;

  .spinner {
    width: 2.5rem;
    height: 2.5rem;
    margin-bottom: 1rem;

    border: 3px solid $color-surface-3;
    border-top-color: $color-accent;
    border-right-color: $color-accent-secondary;
    border-radius: 50%;

    box-shadow: 0 0 1rem color-mix(in srgb, $color-accent 15%, transparent);

    animation: spin 0.8s linear infinite;
  }

  p {
    margin: 0;

    font-size: 0.9rem;
    color: $color-text-muted;
  }
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@keyframes status-enter {
  from {
    opacity: 0;
    transform: translateY(0.5rem);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 700px) {
  .page-header {
    padding: 2rem 1.5rem 1rem;
  }

  .page-header h1 {
    font-size: 2rem;
  }

  .warning-card {
    right: 1rem;
    bottom: 1rem;
    max-width: calc(100vw - 2rem);
  }
}
</style>
