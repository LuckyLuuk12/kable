<!-- @component
ModVersionModal - Modal for selecting mod versions

Displays available versions for a mod with filtering by loaders and game versions.
Allows users to browse, filter, and select specific versions to install.

@prop {ModInfoKind} mod - The mod to show versions for
@prop {KableInstallation | null} [currentInstallation=null] - Current installation context
@prop {boolean} [open=false] - Whether the modal is open
@prop {string | null} [installedVersion=null] - Currently installed version to filter out
@prop {(() =► void) | undefined} onclose - Callback when modal is closed
@prop {((event: { versionId: string; versionNumber: string }) =► void) | undefined} onselectversion - Callback when a version is selected

@example
```svelte
◄ModVersionModal {mod} {currentInstallation} {installedVersion} bind:open onclose={handleClose} onselectversion={handleVersionSelect} /►
```
-->
<script lang="ts">
import { Icon, ProviderKind as ProviderKindEnum, VersionUtils } from "$lib";
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
export let onclose: (() => void) | undefined = undefined;
export let onselectversion:
  | ((event: { versionId: string; versionNumber: string }) => void)
  | undefined = undefined;

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

// Track expanded patchnotes by version ID
let expandedPatchnotes = new Set<string>();
// Track expanded MC versions by version ID
let expandedMcVersions = new Set<string>();

function togglePatchnotes(versionId: string) {
  if (expandedPatchnotes.has(versionId)) {
    expandedPatchnotes.delete(versionId);
  } else {
    expandedPatchnotes.add(versionId);
  }
  expandedPatchnotes = expandedPatchnotes; // Trigger reactivity
}

function toggleMcVersions(versionId: string) {
  if (expandedMcVersions.has(versionId)) {
    expandedMcVersions.delete(versionId);
  } else {
    expandedMcVersions.add(versionId);
  }
  expandedMcVersions = expandedMcVersions; // Trigger reactivity
}

function truncateChangelog(changelog: string, maxLength: number = 150): string {
  if (changelog.length <= maxLength) return changelog;
  return changelog.substring(0, maxLength).trim() + "...";
}

function highlightVersionNumber(
  name: string,
  versionNumber: string,
): { before: string; version: string; after: string } {
  const index = name.indexOf(versionNumber);
  if (index === -1) {
    return { before: name, version: "", after: "" };
  }
  return {
    before: name.substring(0, index),
    version: versionNumber,
    after: name.substring(index + versionNumber.length),
  };
}

$: if (open && mod) {
  loadVersions();
}

// Reactively apply filters when any filter changes
$: filteredVersions = (() => {
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
  // Use installation MC version context if available
  const mcVersion = currentInstallation
    ? extractGameVersion(currentInstallation.version_id)
    : null;
  result.sort((a, b) =>
    VersionUtils.compareVersions(
      b.version_number,
      a.version_number,
      mcVersion || undefined,
    ),
  );

  return result;
})();

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
      // Don't pass MC version context here since we're sorting actual MC versions
      return VersionUtils.compareVersions(b, a);
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
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    console.error("[ModVersionModal] Failed to load versions:", e);
  } finally {
    loading = false;
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
  onclose?.();
}

function handleSelectVersion(version: ModrinthVersion) {
  onselectversion?.({
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
              {@const highlighted = highlightVersionNumber(
                version.name,
                version.version_number,
              )}
              {@const showAllMcVersions = expandedMcVersions.has(version.id)}
              {@const displayMcVersions = showAllMcVersions
                ? version.game_versions
                : version.game_versions.slice(0, 5)}
              <div class="version-item">
                <div class="version-info">
                  <div class="version-header">
                    <div class="version-title-group">
                      <span class="version-name">
                        {highlighted.before}<span class="version-highlight"
                          >{highlighted.version}</span
                        >{highlighted.after}
                      </span>
                    </div>
                    <button
                      class="select-btn"
                      on:click={() => handleSelectVersion(version)}
                    >
                      <Icon name="download" size="md" forceType="svg" />
                      Install
                    </button>
                  </div>
                  <div class="version-meta">
                    <span class="version-loaders">
                      {#each version.loaders as loader}
                        <span class="loader-badge">{loader}</span>
                      {/each}
                    </span>
                    {#if version.files.length > 0}
                      <span class="file-info">
                        <Icon name="file" size="sm" />
                        {version.files[0].filename}
                        <span class="file-size"
                          >({formatFileSize(version.files[0].size)})</span
                        >
                      </span>
                    {/if}
                    <span class="version-game-versions">
                      {#each displayMcVersions as gameVersion}
                        <span class="mc-version-badge">{gameVersion}</span>
                      {/each}
                      {#if version.game_versions.length > 5}
                        <button
                          class="more-versions-btn"
                          on:click|stopPropagation={() =>
                            toggleMcVersions(version.id)}
                        >
                          {showAllMcVersions
                            ? "show less"
                            : `+${version.game_versions.length - 5} more`}
                        </button>
                      {/if}
                    </span>
                  </div>
                  {#if version.changelog}
                    <div class="version-changelog">
                      <p
                        class="changelog-text"
                        class:expanded={expandedPatchnotes.has(version.id)}
                      >
                        {expandedPatchnotes.has(version.id)
                          ? version.changelog
                          : truncateChangelog(version.changelog)}
                      </p>
                      <button
                        class="toggle-changelog-btn"
                        on:click|stopPropagation={() =>
                          togglePatchnotes(version.id)}
                      >
                        {expandedPatchnotes.has(version.id)
                          ? "hide"
                          : "read more"}
                      </button>
                    </div>
                  {/if}
                </div>
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
  align-items: stretch;
  justify-content: space-between;
  gap: 1rem;
  padding: 1rem;
  background: var(--container);
  border: 1px solid rgba($primary, 0.1);
  border-radius: 0.5rem;
  transition: all 0.2s ease;

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
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;

    .version-title-group {
      flex: 1;
    }

    .version-name {
      color: var(--text);
      font-size: 1rem;
      font-weight: 600;
      line-height: 1.4;

      .version-highlight {
        color: var(--primary);
        font-weight: 700;
      }
    }

    .select-btn {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      padding: 0.5rem 1rem;
      background: var(--tertiary);
      border: none;
      border-radius: 0.375rem;
      color: white;
      font-weight: 600;
      font-size: 0.875rem;
      cursor: pointer;
      transition: all 0.2s ease;
      flex-shrink: 0;
      height: fit-content;

      &:hover {
        background: color-mix(in srgb, var(--tertiary) 85%, black);
        transform: scale(1.05);
      }
    }
  }

  .version-meta {
    display: flex;
    align-items: center;
    gap: 1rem;
    font-size: 0.75rem;
    color: var(--placeholder);
    flex-wrap: wrap;

    .version-loaders {
      display: flex;
      gap: 0.25rem;
      flex-wrap: wrap;
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

    .file-info {
      display: flex;
      align-items: center;
      gap: 0.25rem;
      font-size: 0.7rem;

      .file-size {
        opacity: 0.7;
      }
    }

    .version-game-versions {
      display: flex;
      align-items: center;
      gap: 0.25rem;
      flex-wrap: wrap;
    }

    .mc-version-badge {
      padding: 0.125rem 0.5rem;
      background: rgba(#10b981, 0.1);
      border: 1px solid rgba(#10b981, 0.3);
      border-radius: 0.25rem;
      color: #10b981;
      font-weight: 600;
      font-size: 0.7rem;
    }

    .more-versions-btn {
      padding: 0.125rem 0.5rem;
      background: rgba($primary, 0.1);
      border: 1px solid rgba($primary, 0.3);
      border-radius: 0.25rem;
      color: var(--primary);
      font-weight: 600;
      font-size: 0.7rem;
      cursor: pointer;
      transition: all 0.2s ease;

      &:hover {
        background: rgba($primary, 0.15);
        border-color: rgba($primary, 0.4);
      }
    }
  }

  .version-changelog {
    margin-top: 0.25rem;
    padding-top: 0.5rem;
    border-top: 1px solid rgba($primary, 0.05);

    .changelog-text {
      margin: 0 0 0.5rem 0;
      font-size: 0.8rem;
      line-height: 1.4;
      color: var(--text);
      white-space: pre-wrap;
      word-break: break-word;

      &:not(.expanded) {
        display: -webkit-box;
        -webkit-line-clamp: 1;
        line-clamp: 1;
        -webkit-box-orient: vertical;
        overflow: hidden;
      }
    }

    .toggle-changelog-btn {
      background: none;
      border: none;
      color: var(--primary);
      font-size: 0.75rem;
      font-weight: 600;
      cursor: pointer;
      padding: 0;
      text-decoration: underline;
      transition: opacity 0.2s ease;

      &:hover {
        opacity: 0.7;
      }
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
