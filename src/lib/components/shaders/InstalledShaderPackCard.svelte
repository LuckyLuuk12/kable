<!-- @component
InstalledShaderPackCard - Card for displaying installed shader packs

Shows shader icon, name, and provides actions:
- Enable/Disable button
- Remove button
- Visit page button (if extended info available)

@prop {any} shader - The installed shader pack data
@prop {KableInstallation} installation - The installation this shader belongs to
@prop {any | null} extendedInfo - Extended shader information from provider
@prop {(() => void) | undefined} onshaderchanged - Callback invoked when shader is toggled or removed
-->
<script lang="ts">
import { Icon, NotificationService } from "$lib";
import type { KableInstallation } from "$lib";
import * as installationsApi from "$lib/api/installations";

export let shader: any;
export let installation: KableInstallation;
export let extendedInfo: any | null = null;
export let onshaderchanged: (() => void) | undefined = undefined;

let loading = false;

$: isDisabled = shader.disabled || false;
$: displayName = decodeURIComponent(shader.name || shader.file_name.replace(/\.(zip|jar)$/, ""));
$: iconUrl = extendedInfo?.icon_uri || null;

async function toggleDisabled(event: MouseEvent) {
  event.stopPropagation();

  if (loading) return;

  loading = true;
  try {
    const newDisabledState = await installationsApi.toggleShaderDisabled(
      installation,
      shader.file_name,
    );

    shader.disabled = newDisabledState;

    NotificationService.success(
      newDisabledState
        ? `Disabled "${displayName}"`
        : `Enabled "${displayName}"`,
    );

    onshaderchanged?.();
  } catch (error) {
    NotificationService.error(`Failed to toggle shader: ${error}`);
    console.error("Failed to toggle shader:", error);
  } finally {
    loading = false;
  }
}

async function handleRemove(event: MouseEvent) {
  event.stopPropagation();

  if (loading) return;

  const confirmed = confirm(
    `Remove "${displayName}"?\n\nThis will permanently delete the shader pack file.`,
  );
  if (!confirmed) return;

  loading = true;
  try {
    await installationsApi.deleteShader(installation, shader.file_name);
    NotificationService.success(`Removed "${displayName}"`);
    onshaderchanged?.();
  } catch (error) {
    NotificationService.error(`Failed to remove shader: ${error}`);
    console.error("Failed to remove shader:", error);
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
  class="installed-shader-card"
  class:disabled={isDisabled}
  class:loading
>
  <!-- Shader Icon and Name -->
  <div class="shader-info">
    <div class="shader-icon-wrapper">
      {#if iconUrl}
        <img src={iconUrl} alt={displayName} class="shader-icon" />
      {:else}
        <div class="shader-icon-placeholder">
          <Icon name="image" size="md" />
        </div>
      {/if}

      {#if isDisabled}
        <div class="disabled-overlay">
          <Icon name="error" size="sm" />
        </div>
      {/if}
    </div>

    <div class="shader-details">
      <div class="shader-name" class:disabled-text={isDisabled}>
        {displayName}
      </div>
      {#if extendedInfo?.description}
        <div class="shader-description">
          {extendedInfo.description.substring(0, 50)}{extendedInfo.description.length > 50 ? "..." : ""}
        </div>
      {/if}
    </div>
  </div>

  <!-- Action Buttons -->
  <div class="shader-actions">
    <button
      class="action-btn toggle-btn"
      class:enabled={!isDisabled}
      on:click={toggleDisabled}
      title={isDisabled ? "Enable shader" : "Disable shader"}
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
      title="Remove shader"
      disabled={loading}
    >
      <Icon name="trash" size="sm" />
      <span>Remove</span>
    </button>
  </div>
</div>

<style lang="scss">
.installed-shader-card {
  background: var(--card);
  border: 1px solid color-mix(in srgb, var(--primary), 8%, transparent);
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

.shader-info {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  flex: 1;
  min-width: 0;
}

.shader-icon-wrapper {
  position: relative;
  flex-shrink: 0;
}

.shader-icon,
.shader-icon-placeholder {
  width: 40px;
  height: 40px;
  border-radius: 0.375rem;
  object-fit: cover;
}

.shader-icon-placeholder {
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
  color: var(--red-600);
}

.shader-details {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.shader-name {
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

.shader-description {
  font-size: 0.75rem;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.shader-actions {
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
  border: 1px solid color-mix(in srgb, var(--primary), 20%, transparent);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;

  &:hover:not(:disabled) {
    background: var(--bg-tertiary);
    border-color: color-mix(in srgb, var(--primary), 30%, transparent);
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
  border-color: color-mix(in srgb, var(--orange-500), 30%, transparent);

  &:hover:not(:disabled) {
    border-color: color-mix(in srgb, var(--orange-500), 50%, transparent);
    background: color-mix(in srgb, var(--orange-500), 10%, transparent);
  }

  &.enabled {
    border-color: color-mix(in srgb, var(--green-500), 30%, transparent);
    
    &:hover:not(:disabled) {
      border-color: color-mix(in srgb, var(--green-500), 50%, transparent);
      background: color-mix(in srgb, var(--green-500), 10%, transparent);
      color: var(--green-600);
    }
  }
}

.visit-btn {
  border-color: color-mix(in srgb, var(--blue-500), 30%, transparent);

  &:hover:not(:disabled) {
    border-color: color-mix(in srgb, var(--blue-500), 50%, transparent);
    background: color-mix(in srgb, var(--blue-500), 10%, transparent);
  }
}

.remove-btn {
  border-color: color-mix(in srgb, var(--red-500), 30%, transparent);

  &:hover:not(:disabled) {
    border-color: color-mix(in srgb, var(--red-500), 50%, transparent);
    background: color-mix(in srgb, var(--red-500), 10%, transparent);
    color: var(--red-600);
  }
}

@media (max-width: 768px) {
  .shader-icon,
  .shader-icon-placeholder {
    width: 32px;
    height: 32px;
  }

  .shader-name {
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
