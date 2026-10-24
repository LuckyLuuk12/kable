<script lang="ts">
import { app, Icon, type AccountCape } from "$lib";

const skinsService = app.skinsService;

let loading = $state(false);
let error = $state("");

let capes = $derived(skinsService.capes);
let activeCape = $derived(skinsService.activeCape);
let accountAvailable = $derived(Boolean(app.authService.activeAccount));

function capeName(cape: AccountCape): string {
  return cape.alias?.trim() || cape.id;
}

function isCapeActive(cape: AccountCape): boolean {
  return activeCape?.id === cape.id || cape.state.toUpperCase() === "ACTIVE";
}

function dismissError(): void {
  error = "";
  skinsService.error = null;
}

async function applyCape(capeId: string | null): Promise<void> {
  if (!accountAvailable) {
    error = "Sign in to a Minecraft account before changing your cape.";
    return;
  }

  if (loading) return;

  loading = true;
  error = "";

  try {
    const success = await skinsService.applyCape(capeId);

    if (!success) {
      throw new Error(skinsService.error ?? "Failed to apply cape.");
    }

    if (skinsService.error) error = skinsService.error;
  } catch (err) {
    error = `Failed to apply cape: ${err instanceof Error ? err.message : String(err)}`;
  } finally {
    loading = false;
  }
}
</script>

<section class="cape-selection" aria-label="Cape library">
  {#if error || skinsService.error}
    <div class="error-message" role="alert">
      <Icon name="alert" forceType="svg" size="sm" />
      <span>{error || skinsService.error}</span>
      <button type="button" class="icon-button" onclick={dismissError} aria-label="Dismiss error">
        <Icon name="close" forceType="svg" size="sm" />
      </button>
    </div>
  {/if}

  <header class="cape-header">
    <div>
      <h2>Available capes</h2>
      <span class="subtitle">Choose a cape to equip on your account.</span>
    </div>

    {#if activeCape}
      <span class="active-summary">
        Equipped: {capeName(activeCape)}
      </span>
    {/if}
  </header>

  {#if !accountAvailable}
    <div class="account-notice">
      <Icon name="info" forceType="svg" size="sm" />
      Sign in to a Minecraft account to change your equipped cape.
    </div>
  {/if}

  <div class="capes-grid">
    <button type="button" class="cape-card" class:active={!activeCape} onclick={() => applyCape(null)} disabled={loading || !accountAvailable} aria-pressed={!activeCape}>
      <div class="cape-image no-cape">
        <Icon name="ban" forceType="svg" size="xl" />
      </div>

      <div class="cape-details">
        <span class="cape-name">No cape</span>
        <span class="cape-description">Remove the equipped cape</span>
      </div>

      {#if !activeCape}
        <span class="cape-status">Equipped</span>
      {/if}
    </button>

    {#each capes as cape (cape.id)}
      <button
        type="button"
        class="cape-card"
        class:active={isCapeActive(cape)}
        onclick={() => applyCape(cape.id)}
        disabled={loading || !accountAvailable}
        aria-pressed={isCapeActive(cape)}>
        <div class="cape-image">
          {#if cape.url}
            <img src={cape.url} alt={capeName(cape)} loading="lazy" />
          {:else}
            <Icon name="image" forceType="svg" size="xl" />
          {/if}
        </div>

        <div class="cape-details">
          <span class="cape-name" title={capeName(cape)}>{capeName(cape)}</span>
          <span class="cape-description">
            {isCapeActive(cape) ? "Currently equipped" : "Available cape"}
          </span>
        </div>

        {#if isCapeActive(cape)}
          <span class="cape-status">Equipped</span>
        {/if}
      </button>
    {/each}
  </div>

  {#if capes.length === 0}
    <div class="empty-capes">
      <Icon name="image" forceType="svg" size="xl" />
      <p>No capes are available for this account.</p>
    </div>
  {/if}
</section>

<style lang="scss">
.cape-selection {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  color: $color-text;
}

.error-message,
.account-notice {
  display: flex;
  align-items: center;
  gap: $space-md;
  flex: 0 0 auto;
  margin: $space-md $space-lg 0;
  padding: $space-sm $space-md;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  font-size: 0.85rem;
}

.error-message {
  border-color: $color-error;
  background: $color-error-muted;
  color: $color-error;
}

.account-notice {
  border-color: $color-info;
  background: $color-info-muted;
  color: $color-text;
}

.cape-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-lg;
  flex: 0 0 auto;
  padding: $space-lg;
  border-bottom: 1px solid $color-border;

  h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }
}

.subtitle {
  display: block;
  margin-top: $space-xs;
  color: $color-text-muted;
  font-size: 0.8rem;
}

.active-summary {
  max-width: 45%;
  overflow: hidden;
  color: $color-success;
  font-size: 0.8rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.capes-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  align-content: start;
  gap: $space-md;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: $space-lg;
}

.cape-card {
  display: flex;
  flex-direction: column;
  min-width: 0;
  padding: 0;
  overflow: hidden;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  background: $color-surface-1;
  color: $color-text;
  text-align: left;
  cursor: pointer;

  &:hover:not(:disabled) {
    border-color: $color-accent;
    background: $color-surface-2;
  }

  &.active {
    border-color: $color-success;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
}

.cape-image {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  aspect-ratio: 4 / 3;
  overflow: hidden;
  background: $color-surface-0;
  color: $color-text-muted;

  img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }
}

.cape-details {
  display: flex;
  flex-direction: column;
  gap: $space-xs;
  padding: $space-md;
}

.cape-name {
  overflow: hidden;
  font-size: 0.85rem;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cape-description {
  color: $color-text-muted;
  font-size: 0.75rem;
}

.cape-status {
  align-self: flex-start;
  margin: 0 $space-md $space-md;
  padding: $space-1 $space-sm;
  border: 1px solid $color-success;
  border-radius: $radius-md;
  color: $color-success;
  font-size: 0.7rem;
}

.empty-capes {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: $space-md;
  padding: $space-2xl;
  color: $color-text-muted;
  text-align: center;

  p {
    margin: 0;
    font-size: 0.85rem;
  }
}

.icon-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  min-height: 32px;
  padding: $space-sm;
  border: 1px solid transparent;
  border-radius: $radius-md;
  background: transparent;
  color: inherit;
  cursor: pointer;

  &:hover {
    border-color: currentColor;
    background: $color-hover;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }
}

@media (max-width: 480px) {
  .cape-header {
    align-items: flex-start;
    flex-direction: column;
    gap: $space-sm;
    padding: $space-md;
  }

  .active-summary {
    max-width: 100%;
  }

  .capes-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: $space-sm;
    padding: $space-sm;
  }
}
</style>
