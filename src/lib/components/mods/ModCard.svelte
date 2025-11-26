<!-- @component
ModCard - Displays mod information in various view modes

Reusable card component for showing mod details including title, description,
author, downloads, and installation status. Supports grid, list, and compact views.

@prop {ModInfoKind} mod - The mod data to display
@prop {'grid' | 'list' | 'compact'} [viewMode='grid'] - Display mode
@prop {KableInstallation | null} [currentInstallation=null] - Current installation context
@prop {boolean} [loading=false] - Whether mod is being downloaded
@prop {boolean} [isInstalled=false] - Whether mod is already installed
@prop {string | null} [installedVersion=null] - Version number of installed mod
@prop {((event: { mod: ModInfoKind }) => void) | undefined} ondownloadmod - Callback when download button is clicked
@prop {((event: { mod: ModInfoKind }) => void) | undefined} oninfomod - Callback when info button is clicked
@prop {((event: { mod: ModInfoKind; versionId: string; versionNumber: string }) => void) | undefined} ondownloadversion - Callback when specific version is selected

@example
```svelte
◄ModCard {mod} viewMode="grid" ondownloadmod={handleDownload} /►
```
-->
<script lang="ts">
import { Icon, ProviderKind as ProviderKindEnum } from "$lib";
import ModVersionModal from "./ModVersionModal.svelte";
import * as modsApi from "$lib/api/mods";
import type { KableInstallation, ModInfoKind, ProviderKind } from "$lib";

type ViewMode = "grid" | "list" | "compact";

export let mod: ModInfoKind;
export let viewMode: ViewMode = "grid";
export let currentInstallation: KableInstallation | null = null;
export let loading: boolean = false;
export let isInstalled: boolean = false;
export let installedVersion: string | null = null;
export let ondownloadmod: ((event: { mod: ModInfoKind }) => void) | undefined = undefined;
export let oninfomod: ((event: { mod: ModInfoKind }) => void) | undefined = undefined;
export let ondownloadversion: ((event: { mod: ModInfoKind; versionId: string; versionNumber: string }) => void) | undefined = undefined;

let showVersionModal = false;
let hasNewerVersion = false;
let hasOtherVersions = false;
let checkingVersions = false;

// Check for available updates when mod is installed
$: if (isInstalled && installedVersion && currentInstallation) {
  checkForUpdates();
}

async function checkForUpdates() {
  if (!currentInstallation || !installedVersion || checkingVersions) return;

  checkingVersions = true;
  hasNewerVersion = false;
  hasOtherVersions = false;

  try {
    const projectId = getProjectId(mod);
    const provider = getProvider(mod);

    if (!projectId) return;

    // Get loader and game version from installation
    const loader = extractLoader(currentInstallation.version_id);
    const gameVersion = extractGameVersion(currentInstallation.version_id);

    // Fetch compatible versions
    const versions = await modsApi.getProjectVersions(
      provider,
      projectId,
      loader ? [loader] : undefined,
      gameVersion ? [gameVersion] : undefined,
    );

    // Check if there are other versions besides the installed one
    const otherVersions = versions.filter(
      (v) => v.version_number !== installedVersion,
    );
    hasOtherVersions = otherVersions.length > 0;

    // Check if any version is newer (higher version number)
    if (hasOtherVersions) {
      hasNewerVersion = otherVersions.some(
        (v) => compareVersions(v.version_number, installedVersion) > 0,
      );
    }
  } catch (e) {
    console.error("[ModCard] Failed to check for updates:", e);
  } finally {
    checkingVersions = false;
  }
}

function compareVersions(a: string, b: string): number {
  const parseVersion = (v: string) => {
    const parts = v.split(/[.-]+/).map((p) => parseInt(p) || 0);
    return parts;
  };

  const aParts = parseVersion(a);
  const bParts = parseVersion(b);
  const maxLength = Math.max(aParts.length, bParts.length);

  for (let i = 0; i < maxLength; i++) {
    const aPart = aParts[i] || 0;
    const bPart = bParts[i] || 0;

    if (aPart !== bPart) {
      return aPart - bPart;
    }
  }

  return 0;
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

function getProjectId(mod: ModInfoKind): string | null {
  if ("Modrinth" in mod) {
    return mod.Modrinth.project_id;
  } else if ("kind" in mod && mod.kind === "Modrinth") {
    return mod.data.project_id;
  }
  return null;
}

function getProvider(mod: ModInfoKind): ProviderKind {
  if ("Modrinth" in mod || ("kind" in mod && mod.kind === "Modrinth")) {
    return ProviderKindEnum.Modrinth;
  }
  return ProviderKindEnum.CurseForge;
}

// ModCard component for displaying mod information
function getModDisplayInfo(mod: ModInfoKind) {
  // Type guard for Rust enum format
  if ("Modrinth" in mod) {
    const modrinthData = mod.Modrinth;
    // Find the display version by matching latest_version ID with versions_obj
    let displayVersion = "Unknown";

    // Debug: log what data we have
    console.log("Mod data for", modrinthData.title, ":", {
      latest_version: modrinthData.latest_version,
      versions: modrinthData.versions,
      versions_obj: modrinthData.versions_obj,
      hasVersionsObj: !!modrinthData.versions_obj,
    });

    if (
      modrinthData.latest_version &&
      modrinthData.versions_obj &&
      modrinthData.versions_obj.length > 0
    ) {
      // Find the version object that matches the latest_version ID
      const latestVersionObj = modrinthData.versions_obj.find(
        (v) => v.id === modrinthData.latest_version,
      );
      if (latestVersionObj) {
        displayVersion = latestVersionObj.version_number;
        console.log(
          "Found matching version object:",
          latestVersionObj.version_number,
        );
      } else {
        // Fallback to last (newest) version's version_number if latest not found
        displayVersion =
          modrinthData.versions_obj[modrinthData.versions_obj.length - 1]
            .version_number;
        console.log("Using last version object:", displayVersion);
      }
    } else if (modrinthData.versions && modrinthData.versions.length > 0) {
      // Fallback to last (newest) version ID from versions array if versions_obj not available
      displayVersion = modrinthData.versions[modrinthData.versions.length - 1];
      console.log("Using last version ID (fallback):", displayVersion);
    } else if (modrinthData.latest_version) {
      // Last resort: use the latest_version ID directly
      displayVersion = modrinthData.latest_version;
      console.log("Using latest_version directly:", displayVersion);
    }

    console.log(
      "Final displayVersion for",
      modrinthData.title,
      ":",
      displayVersion,
    );

    return {
      title: modrinthData.title || "Unknown Mod",
      description: modrinthData.description || "No description available",
      author: modrinthData.author || "Unknown Author",
      icon_url: modrinthData.icon_url || null,
      downloads: modrinthData.downloads || 0,
      follows: modrinthData.follows || 0,
      updated: modrinthData.date_modified
        ? new Date(modrinthData.date_modified).toLocaleDateString()
        : "Unknown",
      categories: modrinthData.categories || [],
      client_side: modrinthData.client_side || "unknown",
      server_side: modrinthData.server_side || "unknown",
      project_type: modrinthData.project_type || "mod",
      latest_version: displayVersion,
    };
  }

  // Type guard for TypeScript discriminated union format
  if ("kind" in mod && mod.kind === "Modrinth") {
    const modrinthData = mod.data;
    // Find the display version by matching latest_version ID with versions_obj
    let displayVersion = "Unknown";

    // Debug: log what data we have
    console.log("Mod data for", modrinthData.title, ":", {
      latest_version: modrinthData.latest_version,
      versions: modrinthData.versions,
      versions_obj: modrinthData.versions_obj,
      hasVersionsObj: !!modrinthData.versions_obj,
    });

    if (
      modrinthData.latest_version &&
      modrinthData.versions_obj &&
      modrinthData.versions_obj.length > 0
    ) {
      // Find the version object that matches the latest_version ID
      const latestVersionObj = modrinthData.versions_obj.find(
        (v) => v.id === modrinthData.latest_version,
      );
      if (latestVersionObj) {
        displayVersion = latestVersionObj.version_number;
        console.log(
          "Found matching version object:",
          latestVersionObj.version_number,
        );
      } else {
        // Fallback to last (newest) version's version_number if latest not found
        displayVersion =
          modrinthData.versions_obj[modrinthData.versions_obj.length - 1]
            .version_number;
        console.log("Using last version object:", displayVersion);
      }
    } else if (modrinthData.versions && modrinthData.versions.length > 0) {
      // Fallback to last (newest) version ID from versions array if versions_obj not available
      displayVersion = modrinthData.versions[modrinthData.versions.length - 1];
      console.log("Using last version ID (fallback):", displayVersion);
    } else if (modrinthData.latest_version) {
      // Last resort: use the latest_version ID directly
      displayVersion = modrinthData.latest_version;
      console.log("Using latest_version directly:", displayVersion);
    }

    console.log(
      "Final displayVersion for",
      modrinthData.title,
      ":",
      displayVersion,
    );

    return {
      title: modrinthData.title || "Unknown Mod",
      description: modrinthData.description || "No description available",
      author: modrinthData.author || "Unknown Author",
      icon_url: modrinthData.icon_url || null,
      downloads: modrinthData.downloads || 0,
      follows: modrinthData.follows || 0,
      updated: modrinthData.date_modified
        ? new Date(modrinthData.date_modified).toLocaleDateString()
        : "Unknown",
      categories: modrinthData.categories || [],
      client_side: modrinthData.client_side || "unknown",
      server_side: modrinthData.server_side || "unknown",
      project_type: modrinthData.project_type || "mod",
      latest_version: displayVersion,
    };
  }

  return {
    title: "Unknown Mod",
    description: "No description available",
    author: "Unknown Author",
    icon_url: null,
    downloads: 0,
    follows: 0,
    updated: "Unknown",
    categories: [],
    client_side: "unknown",
    server_side: "unknown",
    project_type: "mod",
    latest_version: "Unknown",
  };
}

// Filter categories for display
function getDisplayCategories(categories: string[]): string[] {
  const filterOut = [
    "forge",
    "fabric",
    "quilt",
    "neoforge",
    "client",
    "server",
  ];
  return categories
    .filter((cat) => !filterOut.includes(cat.toLowerCase()))
    .slice(0, 4);
}

// Handle image errors
function handleImageError(event: Event) {
  const img = event.target as HTMLImageElement;
  const placeholder = img.nextElementSibling as HTMLElement;
  if (placeholder) {
    img.style.display = "none";
    placeholder.style.display = "flex";
  }
}

$: displayInfo = getModDisplayInfo(mod);

function handleDownload() {
  ondownloadmod?.({ mod });
}

function handleInfo() {
  oninfomod?.({ mod });
}

function handleVersions(event?: Event) {
  // Stop propagation to prevent card click
  if (event) {
    event.stopPropagation();
  }

  // Open version modal instead of external link
  showVersionModal = true;
}

function handleVersionSelect(
  event: { versionId: string; versionNumber: string },
) {
  const { versionId, versionNumber } = event;
  ondownloadversion?.({ mod, versionId, versionNumber });
}

function handleCardClick() {
  // Navigate to the mod's main page on Modrinth
  handleInfo();
}

function handleCardKeydown(event: KeyboardEvent) {
  if (event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    handleCardClick();
  }
}
</script>

<div
  class="mod-card"
  class:installed={isInstalled}
  class:compact={viewMode === "compact"}
  class:list={viewMode === "list"}
  class:grid={viewMode === "grid"}
>
  {#if viewMode === "compact"}
    <!-- !Compact Mode - Icon + Name/Description + Stacked Buttons -->
    <div class="compact-layout">
      <div class="compact-icon">
        {#if displayInfo.icon_url}
          <img
            src={displayInfo.icon_url}
            alt={displayInfo.title}
            class="compact-mod-icon"
            on:error={handleImageError}
          />
          <div class="compact-icon-placeholder" style="display: none;">
            <Icon name="package" size="sm" />
          </div>
        {:else}
          <div class="compact-icon-placeholder">
            <Icon name="package" size="sm" />
          </div>
        {/if}
      </div>

      <div class="compact-content">
        <h4 class="compact-title" title={displayInfo.title}>
          {displayInfo.title}
        </h4>
        <p class="compact-description" title={displayInfo.description}>
          {displayInfo.description}
        </p>
      </div>

      <div class="compact-actions">
        <button
          class="compact-versions-btn"
          on:click={handleVersions}
          title="View all versions"
        >
          <Icon name="list" size="sm" />
        </button>

        {#if isInstalled && currentInstallation}
          {#if hasNewerVersion || hasOtherVersions}
            <button
              class="compact-update-btn"
              on:click={handleVersions}
              title={hasNewerVersion ? "Update available" : "Change version"}
            >
              <Icon
                name={hasNewerVersion ? "arrow-up" : "edit"}
                size="sm"
                forceType="svg"
              />
            </button>
          {/if}
        {:else if currentInstallation}
          <button
            class="compact-download-btn"
            on:click={handleDownload}
            disabled={loading}
            title="Download latest version"
          >
            <Icon name="download" size="sm" forceType="svg" />
          </button>
        {:else}
          <div
            class="compact-no-installation"
            title="Select installation first"
          >
            <Icon name="info" size="sm" />
          </div>
        {/if}
      </div>
    </div>
  {:else}
    <!-- !Grid and List Mode -->
    {#if viewMode === "grid"}
      <!-- Clickable grid card -->
      <div
        class="mod-content-wrapper clickable"
        on:click={handleCardClick}
        on:keydown={handleCardKeydown}
        role="button"
        tabindex={0}
        aria-label={`View ${displayInfo.title} on Modrinth`}
      >
        <!-- New Flexbox Layout: Header + Data + Controls -->
        <div class="flex-layout">
          <!-- Header Section -->
          <div class="flex-header">
            <div class="flex-icon">
              {#if displayInfo.icon_url}
                <img
                  src={displayInfo.icon_url}
                  alt={displayInfo.title}
                  class="flex-icon-img"
                  on:error={handleImageError}
                />
                <div class="flex-icon-placeholder" style="display: none;">
                  <Icon name="package" size="md" />
                </div>
              {:else}
                <div class="flex-icon-placeholder">
                  <Icon name="package" size="md" />
                </div>
              {/if}
            </div>

            <div class="flex-header-info">
              <h3
                class="flex-title"
                title={`${displayInfo.title} - Latest: ${displayInfo.latest_version}`}
              >
                {displayInfo.title}
              </h3>
              <div class="flex-author">
                by {displayInfo.author}
              </div>
            </div>

            <div class="flex-controls">
              <button
                class="control-btn versions-btn"
                on:click|stopPropagation={handleVersions}
                title="View all versions"
              >
                <Icon name="list" size="sm" />
              </button>
              {#if isInstalled && currentInstallation}
                {#if hasNewerVersion || hasOtherVersions}
                  <button
                    class="control-btn update-btn"
                    on:click|stopPropagation={handleVersions}
                    title={hasNewerVersion
                      ? "Update available"
                      : "Change version"}
                  >
                    <Icon
                      name={hasNewerVersion ? "arrow-up" : "edit"}
                      size="sm"
                      forceType="svg"
                    />
                  </button>
                {/if}
              {:else if currentInstallation}
                <button
                  class="control-btn download-btn"
                  on:click|stopPropagation={handleDownload}
                  disabled={loading}
                  title="Download latest version"
                >
                  <Icon name="download" size="sm" forceType="svg" />
                </button>
              {:else}
                <button
                  class="control-btn disabled-btn"
                  disabled
                  title="Select installation first"
                >
                  <Icon name="info" size="sm" />
                </button>
              {/if}
            </div>
          </div>

          <!-- Data Section -->
          <div class="flex-data">
            <div class="flex-stats">
              <div class="flex-stat">
                <Icon name="download" size="sm" forceType="svg" />
                <span class="stat-value">
                  {displayInfo.downloads > 999
                    ? displayInfo.downloads / 1000000 > 1
                      ? `${(displayInfo.downloads / 1000000).toFixed(1)}M`
                      : `${(displayInfo.downloads / 1000).toFixed(0)}K`
                    : displayInfo.downloads.toLocaleString()}
                </span>
              </div>

              <div class="flex-stat">
                <Icon name="star" size="sm" forceType="svg" />
                <span class="stat-value"
                  >{displayInfo.follows.toLocaleString()}</span
                >
              </div>

              <div class="flex-stat">
                <Icon name="calendar" size="sm" forceType="svg" />
                <span class="stat-value">{displayInfo.updated}</span>
              </div>
            </div>

            <div class="flex-description">
              {displayInfo.description}
            </div>

            {#if getDisplayCategories(displayInfo.categories).length > 0}
              <div class="flex-tags">
                {#each getDisplayCategories(displayInfo.categories) as category}
                  <span class="flex-tag">{category}</span>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      </div>
    {:else}
      <!-- List Mode - Two-row layout -->
      <div class="mod-content-wrapper">
        <!-- List Mode Container -->
        <div class="list-layout">
          <!-- Icon Section (Left) -->
          <div class="list-icon">
            {#if displayInfo.icon_url}
              <img
                src={displayInfo.icon_url}
                alt={displayInfo.title}
                class="list-icon-img"
                on:error={handleImageError}
              />
              <div class="list-icon-placeholder" style="display: none;">
                <Icon name="package" size="lg" />
              </div>
            {:else}
              <div class="list-icon-placeholder">
                <Icon name="package" size="lg" />
              </div>
            {/if}
          </div>

          <!-- Content Section (Two Rows) -->
          <div class="list-content">
            <!-- Top Row: Name/Author + Stats + Versions Button -->
            <div class="list-top-row">
              <div class="list-name-author">
                <h3 class="list-title" title={displayInfo.title}>
                  {displayInfo.title}
                </h3>
                <div class="list-author">
                  by {displayInfo.author}
                </div>
              </div>

              <div class="list-stats">
                <div class="list-stat">
                  <Icon name="download" size="sm" forceType="svg" />
                  <span class="stat-value">
                    {displayInfo.downloads > 999
                      ? displayInfo.downloads / 1000000 > 1
                        ? `${(displayInfo.downloads / 1000000).toFixed(1)}M`
                        : `${(displayInfo.downloads / 1000).toFixed(0)}K`
                      : displayInfo.downloads.toLocaleString()}
                  </span>
                </div>

                <div class="list-stat">
                  <Icon name="star" size="sm" forceType="svg" />
                  <span class="stat-value"
                    >{displayInfo.follows.toLocaleString()}</span
                  >
                </div>

                <div class="list-stat">
                  <Icon name="calendar" size="sm" forceType="svg" />
                  <span class="stat-value">{displayInfo.updated}</span>
                </div>
              </div>

              <button
                class="list-versions-btn"
                on:click={handleVersions}
                title="View all versions"
              >
                <Icon name="list" size="sm" />
                Versions
              </button>
            </div>

            <!-- Bottom Row: Description + Tags + Download Button -->
            <div class="list-bottom-row">
              <div class="list-desc-tags">
                <p class="list-description">
                  {displayInfo.description}
                </p>

                {#if getDisplayCategories(displayInfo.categories).length > 0}
                  <div class="list-tags">
                    {#each getDisplayCategories(displayInfo.categories) as category}
                      <span class="list-tag">{category}</span>
                    {/each}
                  </div>
                {/if}
              </div>

              <div class="list-action">
                {#if isInstalled && currentInstallation}
                  {#if hasNewerVersion || hasOtherVersions}
                    <button
                      class="list-update-btn"
                      on:click={handleVersions}
                      title={hasNewerVersion
                        ? "Update available"
                        : "Change version"}
                    >
                      <Icon
                        name={hasNewerVersion ? "arrow-up" : "edit"}
                        size="sm"
                        forceType="svg"
                      />
                      {hasNewerVersion ? "Update" : "Change"}
                    </button>
                  {/if}
                {:else if currentInstallation}
                  <button
                    class="list-download-btn"
                    on:click={handleDownload}
                    disabled={loading}
                  >
                    <Icon name="download" size="sm" forceType="svg" />
                    Download
                  </button>
                {:else}
                  <div class="no-installation-warning">
                    <Icon name="info" size="sm" forceType="svg" />
                    <span>Select installation</span>
                  </div>
                {/if}
              </div>
            </div>
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>

<!-- Version Selection Modal -->
<ModVersionModal
  {mod}
  {currentInstallation}
  {installedVersion}
  bind:open={showVersionModal}
  onselectversion={handleVersionSelect}
/>

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;

.mod-card {
  background: var(--card);
  border: 1px solid rgba($primary, 0.08);
  border-radius: 0.5rem;
  overflow: hidden;
  transition: all 0.2s ease;
  display: flex;
  position: relative;

  &.installed {
    background: linear-gradient(
      135deg,
      rgba($green, 0.05) 0%,
      rgba($green, 0.02) 100%
    );
    border-color: rgba($green, 0.3);
  }

  &.grid {
    flex-direction: column;
    width: 100%;
    height: auto;
    overflow: hidden;

    &:hover {
      transform: translateY(-2px);
      box-shadow: 0 8px 25px rgba($dark-900, 0.15);
      background: rgba($primary, 0.02);
      border-color: rgba($primary, 0.15);
    }
  }

  &.list {
    width: 100%;
    max-width: none;
    min-height: fit-content;

    &:hover {
      background: rgba($primary, 0.02);
      border-color: rgba($primary, 0.12);
      transform: none;
      box-shadow: 0 2px 8px rgba($dark-900, 0.08);
    }
  }

  // New List Layout - Ultra-compact two-row design with centered icon
  .list-layout {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem;
    width: 100%;
    min-width: 0;
    max-width: 100%;
    overflow: hidden;
  }

  .list-icon {
    width: 40px;
    height: 40px;
    border-radius: 0.25rem;
    overflow: hidden;
    background: rgba($placeholder, 0.1);
    border: 1px solid rgba($placeholder, 0.2);
    flex-shrink: 0;
    position: relative;

    .list-icon-img {
      width: 100%;
      height: 100%;
      object-fit: cover;
    }

    .list-icon-placeholder {
      width: 100%;
      height: 100%;
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--placeholder);
      background: rgba($placeholder, 0.05);
    }
  }

  .list-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
    overflow: hidden;
    width: 100%;
  }

  .list-top-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    min-width: 0;
    overflow: hidden;
  }

  .list-name-author {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    flex: 0 0 auto;
    min-width: 0;
    width: 180px;
    max-width: 400px;
  }

  .list-title {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .list-author {
    color: var(--placeholder);
    font-size: 0.7rem;
    font-weight: 500;
    margin: 0;
  }

  .list-stats {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex: 1;
    justify-content: center;
    min-width: 0;
    overflow: hidden;
  }

  .list-stat {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    color: var(--placeholder);
    font-size: 0.7rem;
    white-space: nowrap;

    .stat-value {
      font-weight: 500;
      color: var(--text);
    }
  }

  .list-versions-btn {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.375rem 0.5rem;
    background: rgba($secondary, 0.1);
    border: 1px solid rgba($secondary, 0.3);
    border-radius: 0.25rem;
    color: var(--secondary);
    font-size: 0.7rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    flex-shrink: 0;
    height: 1.75rem;
    min-width: 5.75rem;
    max-width: 5.5rem;
    margin-left: auto;

    &:hover {
      background: rgba($secondary, 0.15);
      border-color: rgba($secondary, 0.4);
    }
  }

  .list-bottom-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    min-width: 0;
    overflow: hidden;
  }

  .list-desc-tags {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .list-description {
    margin: 0;
    color: var(--placeholder);
    font-size: 0.75rem;
    line-height: 1.2;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }

  .list-tags {
    display: flex;
    flex-wrap: nowrap;
    gap: 0.125rem;
    flex-shrink: 0;
    margin-right: 1.5rem;
  }

  .list-tag {
    padding: 0.0625rem 0.25rem;
    background: rgba($primary, 0.1);
    border: 1px solid rgba($primary, 0.2);
    border-radius: 0.5rem;
    color: var(--primary);
    font-size: 0.6rem;
    font-weight: 500;
    white-space: nowrap;
  }

  // Responsive adjustments for list view
  @media (max-width: 1024px) {
    .list-name-author {
      width: 150px;
      max-width: 160px;
    }

    .list-stats {
      gap: 0.25rem;

      .list-stat {
        font-size: 0.65rem;
      }
    }

    .list-versions-btn,
    .list-download-btn,
    .no-installation-warning {
      min-width: 6rem;
      max-width: 7rem;
      font-size: 0.65rem;
      padding: 0.25rem 0.375rem;
    }
  }

  @media (max-width: 768px) {
    .list-layout {
      gap: 0.375rem;
      padding: 0.375rem;
    }

    .list-name-author {
      width: 120px;
      max-width: 130px;
    }

    .list-stats {
      gap: 0.25rem;

      .list-stat {
        font-size: 0.6rem;
        gap: 0.125rem;
      }
    }

    .list-versions-btn,
    .list-download-btn,
    .no-installation-warning {
      min-width: 5rem;
      max-width: 6rem;
      font-size: 0.6rem;
      gap: 0.125rem;
    }

    .list-description {
      font-size: 0.7rem;
    }

    .list-tag {
      font-size: 0.55rem;
      padding: 0.0625rem 0.1875rem;
    }
  }

  .list-action {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    min-width: 70px;
    max-width: 80px;
    justify-content: flex-end;
    margin-left: auto;
  }

  .list-download-btn,
  .list-update-btn {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.375rem 0.5rem;
    background: var(--tertiary);
    border: none;
    border-radius: 0.25rem;
    color: white;
    font-size: 0.7rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    height: 1.75rem;
    min-width: 5.75rem;
    max-width: 10rem;
    margin-left: auto;

    &:hover:not(:disabled) {
      background: rgba($tertiary, 0.9);
      transform: translateY(-1px);
      box-shadow: 0 4px 12px rgba($tertiary, 0.3);
    }

    &:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }
  }

  .list-update-btn {
    background: var(--secondary);

    &:hover:not(:disabled) {
      background: rgba($secondary, 0.9);
      box-shadow: 0 4px 12px rgba($secondary, 0.3);
    }
  }

  .no-installation-warning {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.375rem 0.5rem;
    background: rgba($yellow, 0.1);
    border: 1px solid rgba($yellow, 0.3);
    border-radius: 0.25rem;
    color: var(--yellow);
    font-size: 0.7rem;
    font-weight: 500;
    height: 1.75rem;
    min-width: 70px;
    max-width: 80px;
    justify-content: center;
  }

  &.compact {
    border-radius: 0.375rem;
    height: fit-content;
    flex: 1 1 220px;
    min-width: 220px;
    max-width: 300px;

    &:hover {
      background: rgba($primary, 0.03);
      border-color: rgba($primary, 0.2);
      transform: none;
      box-shadow: 0 2px 6px rgba($dark-900, 0.1);
    }
  }

  .mod-content-wrapper {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-width: 100%;
  }
}

// New Flexbox Layout Styles
.flex-layout {
  display: flex;
  flex-direction: column;
  padding: 0.75rem;
  height: 100%;
  gap: 0.5rem;

  .flex-header {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;

    .flex-icon {
      width: 48px;
      height: 48px;
      border-radius: 0.375rem;
      overflow: hidden;
      background: rgba($placeholder, 0.1);
      border: 1px solid rgba($placeholder, 0.2);
      flex-shrink: 0;
      position: relative;

      .flex-icon-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
      }

      .flex-icon-placeholder {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--placeholder);
        background: rgba($placeholder, 0.05);
      }
    }

    .flex-header-info {
      flex: 1;
      min-width: 0;

      .flex-title {
        margin: 0 0 0.25rem 0;
        font-size: 0.9rem;
        font-weight: 600;
        color: var(--text);
        line-height: 1.3;
        word-wrap: break-word;
        overflow-wrap: break-word;
        hyphens: auto;
        white-space: normal;
      }

      .flex-author {
        color: var(--placeholder);
        font-size: 0.75em;
        font-weight: 500;
      }
    }

    .flex-controls {
      display: flex;
      flex-direction: column;
      gap: 0.375rem;
      align-items: center;
      flex-shrink: 0;

      .control-btn {
        width: 32px;
        height: 32px;
        padding: unset;
        border: none;
        border-radius: 0.375rem;
        cursor: pointer;
        transition: all 0.2s ease;
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;

        &.download-btn,
        &.update-btn {
          background: var(--tertiary);
          color: white;

          &:hover:not(:disabled) {
            background: rgba($tertiary, 0.8);
            transform: translateY(-1px);
          }

          &:disabled {
            opacity: 0.6;
            cursor: not-allowed;
            transform: none;
          }
        }

        &.update-btn {
          background: var(--secondary);

          &:hover:not(:disabled) {
            background: rgba($secondary, 0.8);
          }
        }

        &.versions-btn {
          background: rgba($secondary, 0.1);
          color: var(--secondary);
          border: 1px solid rgba($secondary, 0.3);

          &:hover {
            background: rgba($secondary, 0.2);
            border-color: rgba($secondary, 0.5);
          }
        }

        &.disabled-btn {
          background: rgba($placeholder, 0.05);
          color: var(--placeholder);
          border: 1px solid rgba($placeholder, 0.2);
          cursor: not-allowed;
          opacity: 0.6;
        }
      }
    }
  }

  .flex-data {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;

    .flex-stats {
      display: flex;
      gap: 0.5rem;

      .flex-stat {
        display: flex;
        align-items: center;
        gap: 0.25rem;
        color: var(--placeholder);
        font-size: 0.65em;

        .stat-value {
          font-weight: 600;
          color: var(--text);
        }
      }
    }

    .flex-description {
      color: var(--placeholder);
      font-size: 0.75em;
      line-height: 1.4;
      display: -webkit-box;
      -webkit-line-clamp: 2;
      line-clamp: 2;
      -webkit-box-orient: vertical;
      overflow: hidden;
      word-wrap: break-word;
      overflow-wrap: break-word;
    }

    .flex-tags {
      display: flex;
      flex-wrap: wrap;
      gap: 0.25rem;

      .flex-tag {
        background: rgba($primary, 0.08);
        color: var(--primary);
        padding: 0.125rem 0.25rem;
        border-radius: 0.1875rem;
        font-size: 0.65em;
        font-weight: 500;
        text-transform: capitalize;
      }
    }
  }
}

.clickable {
  cursor: pointer;
}

// Compact layout specific styles
.compact-layout {
  display: flex;
  align-items: center;
  width: 100%;
  gap: 0.5rem;
  padding: 0.5rem;
  position: relative;

  .compact-icon {
    width: 32px;
    height: 32px;
    border-radius: 0.25rem;
    overflow: hidden;
    background: rgba($placeholder, 0.1);
    border: 1px solid rgba($placeholder, 0.2);
    flex-shrink: 0;
    position: relative;

    .compact-mod-icon {
      width: 100%;
      height: 100%;
      object-fit: cover;
    }

    .compact-icon-placeholder {
      width: 100%;
      height: 100%;
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--placeholder);
      background: rgba($placeholder, 0.05);
    }
  }

  .compact-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 0.125rem;
    padding-right: 0.5rem;
  }

  .compact-title {
    margin: 0;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text);
    line-height: 1.2;
    word-wrap: break-word;
    overflow-wrap: break-word;
    hyphens: auto;
    display: -webkit-box;
    -webkit-line-clamp: 1;
    line-clamp: 1;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .compact-description {
    margin: 0;
    font-size: 0.65rem;
    font-weight: 400;
    color: var(--placeholder);
    line-height: 1.3;
    word-wrap: break-word;
    overflow-wrap: break-word;
    hyphens: auto;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .compact-actions {
    margin-left: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    flex-shrink: 0;

    .compact-versions-btn {
      background: var(--secondary);
      color: white;
      border: none;
      border-radius: 0.25rem;
      padding: 0.25rem;
      cursor: pointer;
      transition: all 0.2s ease;
      display: flex;
      align-items: center;
      justify-content: center;
      width: 28px;
      height: 28px;

      &:hover:not(:disabled) {
        background: rgba($secondary, 0.8);
        transform: translateY(-1px);
      }

      &:disabled {
        opacity: 0.6;
        cursor: not-allowed;
        transform: none;
      }
    }

    .compact-download-btn,
    .compact-update-btn {
      background: var(--tertiary);
      color: white;
      border: none;
      border-radius: 0.25rem;
      padding: 0.25rem;
      cursor: pointer;
      transition: all 0.2s ease;
      display: flex;
      align-items: center;
      justify-content: center;
      width: 28px;
      height: 28px;

      &:hover:not(:disabled) {
        background: rgba($tertiary, 0.8);
        transform: translateY(-1px);
      }

      &:disabled {
        opacity: 0.6;
        cursor: not-allowed;
        transform: none;
      }
    }

    .compact-update-btn {
      background: var(--secondary);

      &:hover:not(:disabled) {
        background: rgba($secondary, 0.8);
      }
    }

    .compact-no-installation {
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--placeholder);
      font-size: 0.65rem;
      opacity: 0.6;
      width: 28px;
      height: 28px;
    }
  }
}
</style>
