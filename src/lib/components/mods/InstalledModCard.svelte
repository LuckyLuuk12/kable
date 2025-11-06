<!-- @component
InstalledModCard - Advanced card for displaying installed mods

Shows mod icon, name, version, and provides actions:
- Click card to toggle enable/disable
- Uninstall/remove button
- Manage versions button (similar to ModBrowser)

@prop {ModJarInfo} mod - The installed mod data
@prop {KableInstallation} installation - The installation this mod belongs to
@prop {ExtendedModInfo | null} extendedInfo - Extended mod information from provider

@event modChanged - Fires when mod is toggled, removed, or updated
@event openVersions - Fires when user wants to manage versions

@example
```svelte
◄InstalledModCard {mod} {installation} {extendedInfo} on:modChanged on:openVersions /►
```
-->
<script lang="ts">
import { createEventDispatcher } from "svelte";
import { Icon, NotificationService, ProviderKind } from "$lib";
import type {
  ModJarInfo,
  KableInstallation,
  ExtendedModInfo,
  ModInfoKind,
} from "$lib";
import * as installationsApi from "$lib/api/installations";
import * as modsApi from "$lib/api/mods";
import ModVersionModal from "./ModVersionModal.svelte";

export let mod: ModJarInfo;
export let installation: KableInstallation;
export let extendedInfo: ExtendedModInfo | null = null;

const dispatch = createEventDispatcher<{
  modChanged: void;
  openVersions: { mod: ModJarInfo };
}>();

let loading = false;
let showVersionModal = false;
let modInfoKind: ModInfoKind | null = null;
let loadingVersions = false;

$: isDisabled = mod.disabled || false;
$: displayName = mod.mod_name || mod.file_name.replace(/\.jar$/, "");
$: version = mod.mod_version || "Unknown";
$: iconUrl = extendedInfo?.icon_uri || null;

async function toggleDisabled(event?: MouseEvent) {
  if (event) {
    event.stopPropagation();
  }

  if (loading) return;

  loading = true;
  try {
    const newDisabledState = await installationsApi.toggleModDisabled(
      installation,
      mod.file_name,
    );

    // Update local state
    mod.disabled = newDisabledState;

    NotificationService.success(
      newDisabledState
        ? `Disabled "${displayName}"`
        : `Enabled "${displayName}"`,
    );

    dispatch("modChanged");
  } catch (error) {
    NotificationService.error(`Failed to toggle mod: ${error}`);
    console.error("Failed to toggle mod:", error);
  } finally {
    loading = false;
  }
}

async function handleRemove(event: MouseEvent) {
  event.stopPropagation();

  if (loading) return;

  // Confirm deletion
  const confirmed = confirm(
    `Remove "${displayName}"?\n\nThis will permanently delete the mod file.`,
  );
  if (!confirmed) return;

  loading = true;
  try {
    await installationsApi.deleteMod(installation, mod.file_name);
    NotificationService.success(`Removed "${displayName}"`);
    dispatch("modChanged");
  } catch (error) {
    NotificationService.error(`Failed to remove mod: ${error}`);
    console.error("Failed to remove mod:", error);
  } finally {
    loading = false;
  }
}

async function handleManageVersions(event: MouseEvent) {
  event.stopPropagation();

  if (loadingVersions) return;

  // If we don't have extended info, we can't fetch versions
  if (!extendedInfo || !extendedInfo.page_uri) {
    NotificationService.warning(
      `Could not find mod information for "${displayName}". Version management requires mod metadata.`,
    );
    return;
  }

  loadingVersions = true;

  try {
    // Extract project ID from page_uri (e.g., "https://modrinth.com/mod/sodium" -> "sodium")
    const projectId = extendedInfo.page_uri.split("/").pop();

    if (!projectId) {
      NotificationService.error(
        `Could not determine project ID for "${displayName}"`,
      );
      return;
    }

    // Convert ExtendedModInfo to ModInfoKind format for the modal
    // We'll use Modrinth format since that's what we have
    modInfoKind = {
      Modrinth: {
        project_id: projectId,
        slug: projectId,
        title: displayName,
        description: extendedInfo.description || "",
        author: extendedInfo.authors?.[0] || "Unknown",
        icon_url: extendedInfo.icon_uri || undefined,
        downloads: 0,
        follows: 0,
        updated: "",
        categories: [],
        display_categories: [],
        client_side: "unknown",
        server_side: "unknown",
        project_type: "mod",
        versions: [],
        versions_obj: [],
        latest_version: mod.mod_version || undefined,
      },
    };

    // Fetch available versions from backend
    const loader = extractLoader(installation.version_id);
    const gameVersion = extractGameVersion(installation.version_id);

    const versions = await modsApi.getProjectVersions(
      ProviderKind.Modrinth,
      projectId,
      loader ? [loader] : undefined,
      gameVersion ? [gameVersion] : undefined,
    );

    if (!versions || versions.length === 0) {
      NotificationService.warning(
        `No compatible versions found for "${displayName}" (${loader || "unknown loader"}, ${gameVersion || "unknown version"})`,
      );
      return;
    }

    // Update the ModInfoKind with version data
    if (modInfoKind && "Modrinth" in modInfoKind) {
      modInfoKind.Modrinth.versions_obj = versions;
      modInfoKind.Modrinth.versions = versions.map((v) => v.id);
    }

    showVersionModal = true;
    dispatch("openVersions", { mod });
  } catch (error) {
    NotificationService.error(`Failed to load versions: ${error}`);
    console.error("Failed to load versions:", error);
  } finally {
    loadingVersions = false;
  }
}

function extractLoader(versionId: string): string | null {
  const lower = versionId.toLowerCase();
  if (lower.includes("fabric")) return "fabric";
  if (lower.includes("neoforge")) return "neoforge";
  if (lower.includes("forge")) return "forge";
  if (lower.includes("quilt")) return "quilt";
  return null;
}

function extractGameVersion(versionId: string): string | null {
  const match = versionId.match(/\b(1\.\d+(?:\.\d+)?)\b/);
  return match ? match[1] : null;
}

async function handleVersionSelect(
  event: CustomEvent<{ versionId: string; versionNumber: string }>,
) {
  const { versionId, versionNumber } = event.detail;

  if (!modInfoKind || !("Modrinth" in modInfoKind)) return;

  try {
    await modsApi.downloadMod(
      ProviderKind.Modrinth,
      modInfoKind.Modrinth.project_id,
      versionId,
      installation,
    );

    NotificationService.success(
      `Downloading "${displayName}" v${versionNumber}`,
    );
    showVersionModal = false;
    dispatch("modChanged");
  } catch (error) {
    NotificationService.error(`Failed to download version: ${error}`);
    console.error("Failed to download version:", error);
  }
}

function handleCardClick() {
  toggleDisabled();
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    toggleDisabled();
  }
}
</script>

<div
  class="installed-mod-card"
  class:disabled={isDisabled}
  class:loading
  on:click={handleCardClick}
  on:keydown={handleKeydown}
  role="button"
  tabindex="0"
  title={isDisabled ? "Click to enable" : "Click to disable"}
>
  <!-- Mod Icon and Name -->
  <div class="mod-info">
    <div class="mod-icon-wrapper">
      {#if iconUrl}
        <img src={iconUrl} alt={displayName} class="mod-icon" />
      {:else}
        <div class="mod-icon-placeholder">
          <Icon name="cube" size="md" />
        </div>
      {/if}

      {#if isDisabled}
        <div class="disabled-overlay">
          <Icon name="error" size="sm" />
        </div>
      {/if}
    </div>

    <div class="mod-details">
      <div class="mod-name" class:disabled-text={isDisabled}>
        {displayName}
      </div>
      <div class="mod-version">
        v{version}
      </div>
    </div>
  </div>

  <!-- Action Buttons -->
  <div class="mod-actions">
    <button
      class="action-btn manage-btn"
      on:click={handleManageVersions}
      title="Manage versions"
      disabled={loading || loadingVersions}
    >
      <Icon name="settings" size="sm" />
      <span>Versions</span>
    </button>

    <button
      class="action-btn remove-btn"
      on:click={handleRemove}
      title="Remove mod"
      disabled={loading}
    >
      <Icon name="trash" size="sm" />
      <span>Remove</span>
    </button>
  </div>
</div>

<!-- Version Selection Modal -->
{#if modInfoKind && showVersionModal}
  <ModVersionModal
    mod={modInfoKind}
    currentInstallation={installation}
    installedVersion={version}
    bind:open={showVersionModal}
    on:selectVersion={handleVersionSelect}
  />
{/if}

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;
@use "sass:color";
@use "sass:map";

.installed-mod-card {
  background: var(--card);
  border: 1px solid rgba($primary, 0.08);
  border-radius: 0.5rem;
  padding: 0.25rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;

  &:hover {
    border-color: rgba($primary, 0.2);
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba($primary, 0.1);
  }

  &:active {
    transform: translateY(0);
  }

  &.disabled {
    opacity: 0.6;
    background: var(--bg-tertiary);

    &:hover {
      opacity: 0.8;
    }
  }

  &.loading {
    pointer-events: none;
    opacity: 0.7;
  }

  &:focus {
    outline: 2px solid rgba($primary, 0.4);
    outline-offset: 2px;
  }
}

.mod-info {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  flex: 1;
  min-width: 0; // Allow text truncation
}

.mod-icon-wrapper {
  position: relative;
  flex-shrink: 0;
}

.mod-icon,
.mod-icon-placeholder {
  width: 40px;
  height: 40px;
  border-radius: 0.375rem;
  object-fit: cover;
}

.mod-icon-placeholder {
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

.mod-details {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.mod-name {
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

.mod-version {
  font-size: 0.75rem;
  color: var(--text-secondary);
  font-family: monospace;
}

.mod-actions {
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

.manage-btn {
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

// Responsive adjustments
@media (max-width: 768px) {
  .mod-icon,
  .mod-icon-placeholder {
    width: 32px;
    height: 32px;
  }

  .mod-name {
    font-size: 0.8125rem;
  }

  .action-btn {
    padding: 0.25rem 0.375rem;

    span {
      display: none; // Hide text on mobile, show only icons
    }
  }
}
</style>
