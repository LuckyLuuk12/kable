<!-- @component
ModVersionModal - Modal for selecting mod versions

Displays available versions for a mod with filtering by loaders and game versions.
Allows users to browse, filter, and select specific versions to install.

@prop {ModInfoKind} mod - The mod to show versions for
@prop {KableInstallation | null} [currentInstallation=null] - Current installation context
@prop {boolean} [open=false] - Whether the modal is open
@prop {string | null} [installedVersion=null] - Currently installed version to filter out

@event close - Fires when modal is closed
@event selectVersion - Fires when a version is selected for download

@example
```svelte
◄ModVersionModal {mod} {currentInstallation} {installedVersion} bind:open on:selectVersion={handleVersionSelect} /►
```
-->
<script lang="ts">
import { createEventDispatcher } from "svelte";
import { Icon, ProviderKind as ProviderKindEnum } from "$lib";
import * as modsApi from "$lib/api/mods";
import type {
  KableInstallation,
  ModInfoKind,
  ModrinthVersion,
  ProviderKind,
} from "$lib";

export let mod: ModInfoKind;
export let currentInstallation: KableInstallation | null = null;
export let open = false;
export let installedVersion: string | null = null;

const dispatch = createEventDispatcher<{
  close: void;
  selectVersion: { versionId: string; versionNumber: string };
}>();

let versions: ModrinthVersion[] = [];
let filteredVersions: ModrinthVersion[] = [];
let loading = false;
let error: string | null = null;

// Filter state
let selectedLoader: string | null = null;
let selectedGameVersion: string | null = null;
let searchQuery = "";

// Available filter options (extracted from versions)
let availableLoaders: string[] = [];
let availableGameVersions: string[] = [];

$: if (open && mod) {
  loadVersions();
}

$: {
  // Apply filters when they change or installedVersion changes
  applyFilters();
}

async function loadVersions() {
  loading = true;
  error = null;

  try {
    const projectId = getProjectId(mod);
    const provider = getProvider(mod);

    if (!projectId) {
      error = "Could not determine project ID";
      return;
    }

    // Fetch all versions (we'll filter client-side for better UX)
    versions = await modsApi.getProjectVersions(provider, projectId);

    // Extract available loaders and game versions
    const loaderSet = new Set<string>();
    const gameVersionSet = new Set<string>();

    versions.forEach((v) => {
      v.loaders.forEach((l) => loaderSet.add(l));
      v.game_versions.forEach((gv) => gameVersionSet.add(gv));
    });

    availableLoaders = Array.from(loaderSet).sort();
    availableGameVersions = Array.from(gameVersionSet).sort((a, b) => {
      // Sort game versions in descending order (newest first)
      return compareVersions(b, a);
    });

    // Auto-select filters based on current installation
    if (currentInstallation) {
      const loader = extractLoader(currentInstallation.version_id);
      if (loader && availableLoaders.includes(loader)) {
        selectedLoader = loader;
      }

      const gameVersion = extractGameVersion(currentInstallation.version_id);
      if (gameVersion && availableGameVersions.includes(gameVersion)) {
        selectedGameVersion = gameVersion;
      }
    }

    applyFilters();
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    console.error("[ModVersionModal] Failed to load versions:", e);
  } finally {
    loading = false;
  }
}

function applyFilters() {
  let result = [...versions];

  // Filter out the currently installed version
  if (installedVersion) {
    result = result.filter((v) => v.version_number !== installedVersion);
  }

  // Filter by loader
  if (selectedLoader) {
    result = result.filter((v) =>
      v.loaders.some((l) => l.toLowerCase() === selectedLoader!.toLowerCase()),
    );
  }

  // Filter by game version
  if (selectedGameVersion) {
    result = result.filter((v) =>
      v.game_versions.includes(selectedGameVersion!),
    );
  }

  // Filter by search query
  if (searchQuery.trim()) {
    const query = searchQuery.toLowerCase();
    result = result.filter(
      (v) =>
        v.version_number.toLowerCase().includes(query) ||
        v.name.toLowerCase().includes(query),
    );
  }

  // Sort by version number (descending - newest first)
  result.sort((a, b) => compareVersions(b.version_number, a.version_number));

  filteredVersions = result;
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
  // Match Minecraft version pattern (e.g., 1.20.1)
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

function getModTitle(mod: ModInfoKind): string {
  if ("Modrinth" in mod) {
    return mod.Modrinth.title;
  } else if ("kind" in mod && mod.kind === "Modrinth") {
    return mod.data.title;
  }
  return "Unknown Mod";
}

function handleClose() {
  open = false;
  dispatch("close");
}

function handleSelectVersion(version: ModrinthVersion) {
  dispatch("selectVersion", {
    versionId: version.id,
    versionNumber: version.version_number,
  });
  handleClose();
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatDate(dateString: string): string {
  try {
    return new Date(dateString).toLocaleDateString();
  } catch {
    return dateString;
  }
}
</script>

{#if open}
  <div
    class="modal-backdrop"
    on:click={handleClose}
    on:keydown={(e) => e.key === "Escape" && handleClose()}
    role="button"
    tabindex="-1"
  >
    <div
      class="modal-content"
      on:click|stopPropagation
      on:keydown|stopPropagation
      role="dialog"
      tabindex="-1"
    >
      <div class="modal-header">
        <div class="modal-title">
          <h2>Select Version - {getModTitle(mod)}</h2>
          {#if installedVersion}
            <span class="installed-badge">
              <Icon name="check" size="sm" />
              Currently: {installedVersion}
            </span>
          {/if}
        </div>
        <button
          class="close-btn"
          on:click={handleClose}
          aria-label="Close modal"
        >
          <Icon name="x" size="lg" forceType="svg" />
        </button>
      </div>

      <div class="modal-filters">
        <div class="filter-group">
          <label for="loader-select">Loader:</label>
          <select id="loader-select" bind:value={selectedLoader}>
            <option value={null}>All Loaders</option>
            {#each availableLoaders as loader}
              <option value={loader}>{loader}</option>
            {/each}
          </select>
        </div>

        <div class="filter-group">
          <label for="game-version-select">Game Version:</label>
          <select id="game-version-select" bind:value={selectedGameVersion}>
            <option value={null}>All Versions</option>
            {#each availableGameVersions as gameVersion}
              <option value={gameVersion}>{gameVersion}</option>
            {/each}
          </select>
        </div>

        <div class="filter-group search-group">
          <label for="version-search">Search:</label>
          <input
            id="version-search"
            type="text"
            bind:value={searchQuery}
            placeholder="Filter versions..."
          />
        </div>
      </div>

      <div class="modal-body">
        {#if loading}
          <div class="loading-state">
            <Icon name="loader" size="md" />
            <p>Loading versions...</p>
          </div>
        {:else if error}
          <div class="error-state">
            <Icon name="alert-circle" size="md" />
            <p>{error}</p>
          </div>
        {:else if filteredVersions.length === 0}
          <div class="empty-state">
            <Icon name="inbox" size="md" />
            <p>No versions match your filters</p>
          </div>
        {:else}
          <div class="versions-list">
            {#each filteredVersions as version}
              <div
                class="version-item"
                on:click={() => handleSelectVersion(version)}
                on:keydown={(e) =>
                  e.key === "Enter" && handleSelectVersion(version)}
                role="button"
                tabindex="0"
              >
                <div class="version-info">
                  <div class="version-header">
                    <span class="version-number">{version.version_number}</span>
                    <span class="version-name">{version.name}</span>
                  </div>
                  <div class="version-meta">
                    <span class="version-loaders">
                      {#each version.loaders as loader}
                        <span class="loader-badge">{loader}</span>
                      {/each}
                    </span>
                    <span class="version-game-versions">
                      MC {version.game_versions.slice(0, 3).join(", ")}
                      {#if version.game_versions.length > 3}
                        <span class="more-versions"
                          >+{version.game_versions.length - 3}</span
                        >
                      {/if}
                    </span>
                  </div>
                  {#if version.files.length > 0}
                    <div class="version-files">
                      {#each version.files.slice(0, 1) as file}
                        <span class="file-info">
                          <Icon name="file" size="sm" />
                          {file.filename} ({formatFileSize(file.size)})
                        </span>
                      {/each}
                    </div>
                  {/if}
                </div>
                <button
                  class="select-btn"
                  on:click={() => handleSelectVersion(version)}
                >
                  <Icon name="download" size="md" forceType="svg" />
                  Install
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <div class="modal-footer">
        <p class="version-count">
          {filteredVersions.length} of {versions.length} version{versions.length !==
          1
            ? "s"
            : ""}
        </p>
        <button class="cancel-btn" on:click={handleClose}>Close</button>
      </div>
    </div>
  </div>
{/if}

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;

.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba($dark-900, 0.8);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 2rem;
}

.modal-content {
  background: var(--card);
  border-radius: 0.75rem;
  border: 1px solid rgba($primary, 0.15);
  box-shadow: 0 20px 60px rgba($dark-900, 0.5);
  max-width: 800px;
  width: 100%;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.5rem;
  border-bottom: 1px solid rgba($primary, 0.1);

  .modal-title {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    flex: 1;

    h2 {
      margin: 0;
      font-size: 1.25rem;
      font-weight: 600;
      color: var(--text);
    }

    .installed-badge {
      display: inline-flex;
      align-items: center;
      gap: 0.25rem;
      padding: 0.25rem 0.5rem;
      background: rgba($secondary, 0.1);
      color: var(--secondary);
      border-radius: 4px;
      font-size: 0.75rem;
      font-weight: 500;
      width: fit-content;
    }
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--placeholder);
    cursor: pointer;
    padding: 0.5rem;
    border-radius: 0.375rem;
    transition: all 0.2s ease;

    &:hover {
      background: rgba($primary, 0.1);
      color: var(--text);
    }
  }
}

.modal-filters {
  display: flex;
  gap: 1rem;
  padding: 1rem 1.5rem;
  background: rgba($primary, 0.03);
  border-bottom: 1px solid rgba($primary, 0.1);
  flex-wrap: wrap;

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    flex: 1;
    min-width: 150px;

    label {
      font-size: 0.75rem;
      font-weight: 600;
      color: var(--placeholder);
      text-transform: uppercase;
      letter-spacing: 0.05em;
    }

    select,
    input {
      padding: 0.5rem;
      background: var(--card);
      border: 1px solid rgba($primary, 0.2);
      border-radius: 0.375rem;
      color: var(--text);
      font-size: 0.875rem;
      transition: all 0.2s ease;

      &:focus {
        outline: none;
        border-color: var(--primary);
        box-shadow: 0 0 0 3px rgba($primary, 0.1);
      }
    }
  }

  .search-group {
    flex: 2;
    min-width: 200px;
  }
}

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  min-height: 300px;
}

.loading-state,
.error-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  padding: 3rem;
  color: var(--placeholder);
  text-align: center;
}

.versions-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.version-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 1rem;
  background: var(--container);
  border: 1px solid rgba($primary, 0.1);
  border-radius: 0.5rem;
  transition: all 0.2s ease;
  cursor: pointer;

  &:hover {
    background: rgba($primary, 0.05);
    border-color: rgba($primary, 0.2);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba($dark-900, 0.1);
  }

  .version-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .version-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;

    .version-number {
      font-weight: 700;
      color: var(--primary);
      font-size: 1rem;
    }

    .version-name {
      color: var(--text);
      font-size: 0.875rem;
    }
  }

  .version-meta {
    display: flex;
    align-items: center;
    gap: 1rem;
    font-size: 0.75rem;
    color: var(--placeholder);

    .version-loaders {
      display: flex;
      gap: 0.25rem;
    }

    .loader-badge {
      padding: 0.125rem 0.5rem;
      background: rgba($secondary, 0.1);
      border: 1px solid rgba($secondary, 0.3);
      border-radius: 0.25rem;
      color: var(--secondary);
      font-weight: 600;
      font-size: 0.7rem;
      text-transform: capitalize;
    }

    .more-versions {
      color: var(--primary);
      font-weight: 600;
    }
  }

  .version-files {
    display: flex;
    gap: 0.5rem;
    font-size: 0.7rem;
    color: var(--placeholder);

    .file-info {
      display: flex;
      align-items: center;
      gap: 0.25rem;
    }
  }

  .select-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1rem;
    background: var(--tertiary);
    border: none;
    border-radius: 0.375rem;
    color: white;
    font-weight: 600;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.2s ease;
    flex-shrink: 0;

    &:hover {
      background: color-mix(in srgb, var(--tertiary) 85%, black);
      transform: scale(1.05);
    }
  }
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.5rem;
  border-top: 1px solid rgba($primary, 0.1);

  .version-count {
    margin: 0;
    font-size: 0.875rem;
    color: var(--placeholder);
  }

  .cancel-btn {
    padding: 0.625rem 1.25rem;
    background: rgba($placeholder, 0.1);
    border: 1px solid rgba($placeholder, 0.2);
    border-radius: 0.375rem;
    color: var(--text);
    font-weight: 600;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.2s ease;

    &:hover {
      background: rgba($placeholder, 0.15);
      border-color: rgba($placeholder, 0.3);
    }
  }
}

@media (max-width: 768px) {
  .modal-content {
    max-width: 100%;
    max-height: 95vh;
    margin: 0.5rem;
  }

  .modal-filters {
    flex-direction: column;
  }

  .version-item {
    flex-direction: column;
    align-items: stretch;

    .select-btn {
      width: 100%;
      justify-content: center;
    }
  }
}
</style>
