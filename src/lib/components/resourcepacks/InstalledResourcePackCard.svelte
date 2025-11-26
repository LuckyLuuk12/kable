<!-- @component
InstalledResourcePackCard - Advanced card for displaying installed resource packs

Shows pack icon, name, and provides actions:
- Enable/Disable button
- Remove button
- Visit page button

@prop {any} pack - The installed resource pack data
@prop {KableInstallation} installation - The installation this pack belongs to
@prop {any | null} extendedInfo - Extended pack information from provider
@prop {(() =► void) | undefined} onpackchanged - Callback invoked when pack is toggled or removed

@example
```svelte
◄InstalledResourcePackCard {pack} {installation} {extendedInfo} onpackchanged={handleChange} /►
```
-->
<script lang="ts">
import { Icon, NotificationService } from "$lib";
import type { KableInstallation } from "$lib";
import * as installationsApi from "$lib/api/installations";

export let pack: any; // ResourcePackInfo type (to be defined)
export let installation: KableInstallation;
export let extendedInfo: any | null = null; // ExtendedResourcePackInfo type (to be defined)
export let onpackchanged: (() => void) | undefined = undefined;

let loading = false;

$: isDisabled = pack.disabled || false;
$: displayName = decodeURIComponent(pack.name || pack.file_name.replace(/\.zip$/, ""));
$: iconUrl = extendedInfo?.icon_uri || null;

async function toggleDisabled(event: MouseEvent) {
  event.stopPropagation();

  if (loading) return;

  loading = true;
  try {
    const newDisabledState = await installationsApi.toggleResourcePackDisabled(
      installation,
      pack.file_name,
    );

    // Update local state
    pack.disabled = newDisabledState;

    NotificationService.success(
      newDisabledState
        ? `Disabled "${displayName}"`
        : `Enabled "${displayName}"`,
    );

    onpackchanged?.();
  } catch (error) {
    NotificationService.error(`Failed to toggle pack: ${error}`);
    console.error("Failed to toggle pack:", error);
  } finally {
    loading = false;
  }
}

async function handleRemove(event: MouseEvent) {
  event.stopPropagation();

  if (loading) return;

  const confirmed = confirm(
    `Remove "${displayName}"?\n\nThis will permanently delete the resource pack file.`,
  );
  if (!confirmed) return;

  loading = true;
  try {
    await installationsApi.deleteResourcePack(installation, pack.file_name);
    NotificationService.success(`Removed "${displayName}"`);
    onpackchanged?.();
  } catch (error) {
    NotificationService.error(`Failed to remove pack: ${error}`);
    console.error("Failed to remove pack:", error);
  } finally {
    loading = false;
  }
}

async function handleVisitPage(event: MouseEvent) {
  event.stopPropagation();

  if (!extendedInfo || !extendedInfo.page_uri) {
    NotificationService.warning(
      `No page URL available for "${displayName}"`,
    );
    return;
  }

  try {
    const { openUrl } = await import("$lib/api/system");
    await openUrl(extendedInfo.page_uri);
  } catch (error) {
    NotificationService.error(`Failed to open page: ${error}`);
    console.error("Failed to open page:", error);
  }
}
</script>

<div
  class="installed-pack-card"
  class:disabled={isDisabled}
  class:loading
>
  <!-- Pack Icon and Name -->
  <div class="pack-info">
    <div class="pack-icon-wrapper">
      {#if iconUrl}
        <img src={iconUrl} alt={displayName} class="pack-icon" />
      {:else}
        <div class="pack-icon-placeholder">
          <Icon name="image" size="md" />
        </div>
      {/if}

      {#if isDisabled}
        <div class="disabled-overlay">
          <Icon name="error" size="sm" />
        </div>
      {/if}
    </div>

    <div class="pack-details">
      <div class="pack-name" class:disabled-text={isDisabled}>
        {displayName}
      </div>
      {#if extendedInfo?.description}
        <div class="pack-description">
          {extendedInfo.description.substring(0, 50)}{extendedInfo.description.length > 50 ? "..." : ""}
        </div>
      {/if}
    </div>
  </div>

  <!-- Action Buttons -->
  <div class="pack-actions">
    <button
      class="action-btn toggle-btn"
      class:enabled={!isDisabled}
      on:click={toggleDisabled}
      title={isDisabled ? "Enable pack" : "Disable pack"}
      disabled={loading}
    >
      <Icon name={isDisabled ? "eye-off" : "eye"} size="sm" />
      <span>{isDisabled ? "Enable" : "Disable"}</span>
    </button>

    {#if extendedInfo?.page_uri}
      <button
        class="action-btn visit-btn"
        on:click={handleVisitPage}
        title="Visit page"
        disabled={loading}
      >
        <Icon name="external-link" size="sm" />
        <span>Visit</span>
      </button>
    {/if}

    <button
      class="action-btn remove-btn"
      on:click={handleRemove}
      title="Remove pack"
      disabled={loading}
    >
      <Icon name="trash" size="sm" />
      <span>Remove</span>
    </button>
  </div>
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;
@use "sass:color";
@use "sass:map";

.installed-pack-card {
  background: var(--card);
  border: 1px solid rgba($primary, 0.08);
  border-radius: 0.5rem;
  padding: 0.25rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  transition: all 0.2s ease;
  position: relative;
  user-select: none;

  &.disabled {
    opacity: 0.6;
    background: var(--bg-tertiary);
  }

  &.loading {
    pointer-events: none;
    opacity: 0.7;
  }
}

.pack-info {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  flex: 1;
  min-width: 0;
}

.pack-icon-wrapper {
  position: relative;
  flex-shrink: 0;
}

.pack-icon,
.pack-icon-placeholder {
  width: 40px;
  height: 40px;
  border-radius: 0.375rem;
  object-fit: cover;
}

.pack-icon-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}

.disabled-overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.6);
  border-radius: 0.375rem;
  display: flex;
  align-items: center;
  justify-content: center;
  color: map.get($variables, "red-600");
}

.pack-details {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.pack-name {
  font-weight: 600;
  font-size: 0.875rem;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;

  &.disabled-text {
    text-decoration: line-through;
  }
}

.pack-description {
  font-size: 0.75rem;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pack-actions {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  flex-shrink: 0;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
  border: 1px solid rgba($primary, 0.2);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;

  &:hover:not(:disabled) {
    background: var(--bg-tertiary);
    border-color: rgba($primary, 0.3);
    transform: translateY(-1px);
  }

  &:active:not(:disabled) {
    transform: translateY(0);
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  span {
    white-space: nowrap;
  }
}

.toggle-btn {
  border-color: color-mix(in srgb, map.get($variables, "orange-500"), 30%, transparent);

  &:hover:not(:disabled) {
    border-color: color-mix(in srgb, map.get($variables, "orange-500"), 50%, transparent);
    background: color-mix(in srgb, map.get($variables, "orange-500"), 10%, transparent);
  }

  &.enabled {
    border-color: color-mix(in srgb, map.get($variables, "green-500"), 30%, transparent);
    
    &:hover:not(:disabled) {
      border-color: color-mix(in srgb, map.get($variables, "green-500"), 50%, transparent);
      background: color-mix(in srgb, map.get($variables, "green-500"), 10%, transparent);
      color: map.get($variables, "green-600");
    }
  }
}

.visit-btn {
  border-color: rgba(map.get($variables, "blue-500"), 0.3);

  &:hover:not(:disabled) {
    border-color: rgba(map.get($variables, "blue-500"), 0.5);
    background: rgba(map.get($variables, "blue-500"), 0.1);
  }
}

.remove-btn {
  border-color: rgba(map.get($variables, "red-500"), 0.3);

  &:hover:not(:disabled) {
    border-color: rgba(map.get($variables, "red-500"), 0.5);
    background: rgba(map.get($variables, "red-500"), 0.1);
    color: map.get($variables, "red-600");
  }
}

@media (max-width: 768px) {
  .pack-icon,
  .pack-icon-placeholder {
    width: 32px;
    height: 32px;
  }

  .pack-name {
    font-size: 0.8125rem;
  }

  .action-btn {
    padding: 0.25rem 0.375rem;

    span {
      display: none;
    }
  }
}
</style>
