<script lang="ts">
import { Icon, Image, app, clickSound, successSound, type LoaderKind, type ProfileVersion } from "$lib";
import { MODAL_CONTEXT, type ModalContext } from "$lib/utils/modal";
import { open } from "@tauri-apps/plugin-dialog";
import { getContext } from "svelte";
import { SvelteSet } from "svelte/reactivity";

type CreationSource = "none" | "profile" | "zip" | "mrpack";

const modal = getContext<ModalContext>(MODAL_CONTEXT);

const LOADER_ORDER: LoaderKind[] = ["fabric", "iris_fabric", "quilt", "vanilla", "forge", "neo_forge"];

const ROW_HEIGHT = 36;
const VIRTUALIZATION_OVERSCAN = 8;

let selectedLoader = $state<LoaderKind>("fabric");
let selectedVersionId = $state("");
let searchQuery = $state("");

let creationSource = $state<CreationSource>("none");
let selectedProfileId = $state<string | null>(null);
let exportedZip = $state<string | null>(null);
let mrpack = $state<string | null>(null);

let isCreating = $state(false);
let error = $state<string | null>(null);

let versionListElement = $state<HTMLDivElement | null>(null);
let versionListHeight = $state(360);
let scrollTop = $state(0);

const availableVersions = $derived(app.launcherService.versions ?? []);
let isLoading = $state(false);
const availableProfiles = $derived(app.profilesService.profiles);

$effect(() => {
  if (availableVersions.length === 0) {
    loadVersions();
  }
});

async function loadVersions() {
  if (isLoading) return;

  isLoading = true;
  error = null;

  try {
    await app.launcherService.loadVersions();
  } catch (e) {
    error = e instanceof Error ? e.message : "Failed to load versions.";
  } finally {
    isLoading = false;
  }
}

const loaderOptions = $derived.by(() => {
  const loaders = new SvelteSet<LoaderKind>();

  for (const version of availableVersions) {
    loaders.add(version.loader);
  }

  return Array.from(loaders).sort((a, b) => {
    const aIndex = LOADER_ORDER.indexOf(a);
    const bIndex = LOADER_ORDER.indexOf(b);

    if (aIndex === -1 && bIndex === -1) {
      return a.localeCompare(b);
    }

    if (aIndex === -1) {
      return 1;
    }

    if (bIndex === -1) {
      return -1;
    }

    return aIndex - bIndex;
  });
});

const versionsForLoader = $derived(availableVersions.filter((version) => version.loader === selectedLoader));

const filteredVersions = $derived.by(() => {
  const query = searchQuery.trim().toLowerCase();

  if (!query) {
    return versionsForLoader;
  }

  return versionsForLoader.filter((version) => version.id.toLowerCase().includes(query));
});

const selectedProfile = $derived(selectedProfileId ? (availableProfiles.find((profile) => profile.id === selectedProfileId) ?? null) : null);

const totalVersionHeight = $derived(filteredVersions.length * ROW_HEIGHT);

const firstVisibleIndex = $derived(Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - VIRTUALIZATION_OVERSCAN));

const lastVisibleIndex = $derived(Math.min(filteredVersions.length, Math.ceil((scrollTop + versionListHeight) / ROW_HEIGHT) + VIRTUALIZATION_OVERSCAN));

const visibleVersions = $derived(filteredVersions.slice(firstVisibleIndex, lastVisibleIndex));

const visibleOffset = $derived(firstVisibleIndex * ROW_HEIGHT);

const canCreate = $derived(
  !isCreating &&
    selectedVersionId !== "" &&
    (creationSource !== "profile" || selectedProfile !== null) &&
    (creationSource !== "zip" || exportedZip !== null) &&
    (creationSource !== "mrpack" || mrpack !== null),
);

$effect(() => {
  if (loaderOptions.length === 0) {
    selectedLoader = "vanilla";
    return;
  }

  if (!loaderOptions.includes(selectedLoader)) {
    selectedLoader = loaderOptions[0];
  }
});

$effect(() => {
  const firstVersion = filteredVersions[0];

  if (!firstVersion || !filteredVersions.some((version) => version.id === selectedVersionId)) {
    selectedVersionId = firstVersion?.id ?? "";
  }
});

function selectLoader(loader: LoaderKind) {
  selectedLoader = loader;
  searchQuery = "";
  resetVersionScroll();
}

function selectVersion(version: ProfileVersion) {
  selectedVersionId = version.id;
}

function handleVersionScroll() {
  if (!versionListElement) {
    return;
  }

  scrollTop = versionListElement.scrollTop;
}

function handleVersionResize() {
  if (!versionListElement) {
    return;
  }

  versionListHeight = versionListElement.clientHeight;
}

function resetVersionScroll() {
  scrollTop = 0;

  if (versionListElement) {
    versionListElement.scrollTop = 0;
  }
}

function selectSource(source: CreationSource) {
  creationSource = source;
  error = null;

  if (source !== "profile") {
    selectedProfileId = null;
  }

  if (source !== "zip") {
    exportedZip = null;
  }

  if (source !== "mrpack") {
    mrpack = null;
  }
}

async function selectZip() {
  error = null;

  try {
    const path = await open({
      multiple: false,
      directory: false,
      filters: [
        {
          name: "Exported Profile",
          extensions: ["zip"],
        },
      ],
    });

    if (typeof path === "string") {
      exportedZip = path;
    }
  } catch (e) {
    console.error("Failed to open file dialog:", e);
    error = e instanceof Error ? e.message : "Failed to select ZIP file.";
  }
}

async function selectMrpack() {
  error = null;

  try {
    const path = await open({
      multiple: false,
      directory: false,
      filters: [
        {
          name: "Modrinth Pack",
          extensions: ["mrpack"],
        },
      ],
    });

    if (typeof path === "string") {
      mrpack = path;
    }
  } catch (e) {
    error = e instanceof Error ? e.message : "Failed to select Modrinth pack.";
  }
}

async function createProfile(event: SubmitEvent) {
  event.preventDefault();

  if (!canCreate) {
    return;
  }

  isCreating = true;
  error = null;

  try {
    await app.profilesService.createProfile(
      selectedVersionId,
      creationSource === "profile" ? selectedProfile : null,
      creationSource === "zip" ? exportedZip : null,
      creationSource === "mrpack" ? mrpack : null,
    );

    modal.resolve();
  } catch (e) {
    error = e instanceof Error ? e.message : "Failed to create installation.";
  } finally {
    isCreating = false;
  }
}

function cancel() {
  if (!isCreating) {
    modal.dismiss();
  }
}
</script>

<div class="create-profile-modal">
  <header class="modal-header">
    <h2>Create New Installation</h2>

    <button use:clickSound type="button" class="close-button" aria-label="Close" onclick={cancel} disabled={isCreating}>
      <Icon name="x" size="sm" />
    </button>
  </header>

  {#if error}
    <div class="error-message">
      <Icon name="alert-circle" size="sm" />
      <span>{error}</span>
    </div>
  {/if}

  <form onsubmit={createProfile}>
    <section class="section">
      <div class="section-label">Mod Loader</div>

      <div class="loader-select-row">
        {#each loaderOptions as loader (loader)}
          <button
            type="button"
            class:selected={selectedLoader === loader}
            class="loader-button"
            style:background={`${app.profilesService.getLoaderColor(loader)}20`}
            style:color={app.profilesService.getLoaderColor(loader)}
            onclick={() => selectLoader(loader)}>
            <span class="loader-icon">
              <Image key={loader} />
            </span>

            <span class="loader-label">
              {loader.replace(/_/g, " ").replace(/(^|\s)([a-z])/g, (_, prefix, character) => prefix + character.toUpperCase())}
            </span>
          </button>
        {/each}
      </div>
    </section>

    <section class="section">
      <div class="section-label">Minecraft Version</div>

      <input
        class="version-search"
        type="text"
        bind:value={searchQuery}
        placeholder="Search versions..."
        aria-label="Search Minecraft versions"
        oninput={resetVersionScroll} />

      {#if filteredVersions.length > 0}
        <div
          bind:this={versionListElement}
          class="version-list"
          role="listbox"
          tabindex="0"
          aria-label="Minecraft versions"
          onscroll={handleVersionScroll}
          onresize={handleVersionResize}>
          <div class="version-list-content" style:height={`${totalVersionHeight}px`}>
            <div class="version-list-items" style:transform={`translateY(${visibleOffset}px)`}>
              {#each visibleVersions as version (version.id)}
                <button
                  type="button"
                  role="option"
                  aria-selected={selectedVersionId === version.id}
                  class:selected={selectedVersionId === version.id}
                  class="version-option"
                  onclick={() => selectVersion(version)}>
                  {version.id}
                </button>
              {/each}
            </div>
          </div>
        </div>

        <div class="version-count">
          {filteredVersions.length}
          version{filteredVersions.length === 1 ? "" : "s"}
        </div>
      {:else}
        <div class="no-results">
          No versions found{searchQuery ? ` matching "${searchQuery}"` : ""}.
        </div>
      {/if}
    </section>

    <section class="section">
      <div class="section-label">Creation Source</div>

      <div class="source-tabs" role="tablist">
        <button type="button" role="tab" aria-selected={creationSource === "none"} class:active={creationSource === "none"} onclick={() => selectSource("none")}>
          <Icon name="plus" size="sm" />
          Empty
        </button>

        <button type="button" role="tab" aria-selected={creationSource === "profile"} class:active={creationSource === "profile"} onclick={() => selectSource("profile")}>
          <Icon name="copy" size="sm" />
          Existing Profile
        </button>

        <button type="button" role="tab" aria-selected={creationSource === "zip"} class:active={creationSource === "zip"} onclick={() => selectSource("zip")}>
          <Icon name="archive" size="sm" />
          Exported ZIP
        </button>

        <button type="button" role="tab" aria-selected={creationSource === "mrpack"} class:active={creationSource === "mrpack"} onclick={() => selectSource("mrpack")}>
          <Icon name="package" size="sm" />
          Modrinth Pack
        </button>
      </div>

      <div class="source-content">
        {#if creationSource === "none"}
          <div class="source-description">Create a fresh installation for the selected Minecraft version.</div>
        {:else if creationSource === "profile"}
          <label for="base-profile">Base Profile</label>

          <select id="base-profile" class="source-select" bind:value={selectedProfileId}>
            <option value={null}>Select a profile...</option>

            {#each availableProfiles as profile (profile.id)}
              <option value={profile.id}>
                {profile.metadata.name} ({profile.version.id})
              </option>
            {/each}
          </select>

          <div class="source-description">Use an existing installation as the base for the new profile.</div>
        {:else if creationSource === "zip"}
          <div class="file-picker">
            <div class:file-selected={exportedZip} class="file-name">
              {exportedZip ?? "No exported profile selected"}
            </div>

            <button use:clickSound type="button" class="btn btn-secondary" onclick={selectZip} disabled={isCreating}>
              <Icon name="folder" size="sm" />
              Choose ZIP
            </button>
          </div>

          <div class="source-description">Import an exported Kable profile from a ZIP file.</div>
        {:else if creationSource === "mrpack"}
          <div class="file-picker">
            <div class:file-selected={mrpack} class="file-name">
              {mrpack ?? "No Modrinth pack selected"}
            </div>

            <button use:clickSound type="button" class="btn btn-secondary" onclick={selectMrpack} disabled={isCreating}>
              <Icon name="folder" size="sm" />
              Choose Pack
            </button>
          </div>

          <div class="source-description">Import a Modrinth .mrpack file.</div>
        {/if}
      </div>
    </section>

    <footer class="actions">
      <button use:clickSound type="button" class="btn btn-secondary" onclick={cancel} disabled={isCreating}> Cancel </button>

      <button use:successSound type="submit" class="btn btn-primary" disabled={!canCreate}>
        {#if isCreating}
          <Icon name="refresh" size="sm" className="spin" />
          Creating...
        {:else}
          <Icon name="plus" size="sm" />
          Create
        {/if}
      </button>
    </footer>
  </form>
</div>

<style lang="scss">
.create-profile-modal {
  width: min(900px, 90vw);
  max-height: 90vh;
  overflow-y: auto;

  padding: 1.5rem;

  background: $color-surface-1;
  color: $color-text;

  border: 1px solid $color-border;
  border-radius: $radius-lg;
  box-shadow: 0 0.75rem 3rem rgba(0, 0, 0, 0.35);

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1.5rem;

    h2 {
      margin: 0;
      color: $color-text;
    }

    .close-button {
      display: flex;
      align-items: center;
      justify-content: center;

      width: 2rem;
      height: 2rem;
      padding: 0;

      border: none;
      border-radius: $radius-lg;
      background: transparent;
      color: $color-text-muted;

      cursor: pointer;

      &:hover {
        background: $color-surface-2;
        color: $color-text;
      }
    }
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: 0.5rem;

    margin-bottom: 1rem;
    padding: 0.75rem 1rem;

    border-radius: $radius-lg;
    background: color-mix(in srgb, $color-error 15%, transparent);
    color: $color-error;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .section {
    display: flex;
    flex-direction: column;
  }

  .section-label {
    margin-bottom: 0.5rem;

    color: $color-text-muted;
    font-size: 0.9rem;
    font-weight: 600;
  }

  .loader-select-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
  }

  .loader-button {
    display: flex;
    align-items: center;
    gap: 0.5rem;

    padding: 0.65rem 1rem;

    border: 2px solid transparent;
    border-radius: $radius-lg;

    cursor: pointer;

    transition:
      border-color 0.15s,
      box-shadow 0.15s;

    &.selected {
      border-color: $color-accent;
    }

    .loader-icon {
      display: flex;
      align-items: center;
      justify-content: center;

      width: 1.5rem;
      height: 1.5rem;
    }

    .loader-label {
      font-weight: 500;
    }
  }

  .version-search {
    width: 100%;
    box-sizing: border-box;

    padding: 0.65rem 0.75rem;
    margin-bottom: 0.5rem;

    border: 1px solid $color-border;
    border-radius: $radius-lg;

    background: $color-surface-2;
    color: $color-text;

    font-size: 0.95rem;

    &:focus {
      outline: none;
      border-color: $color-accent;
    }

    &::placeholder {
      color: $color-text-muted;
    }
  }

  .version-list {
    height: 360px;
    overflow-y: auto;

    border: 1px solid $color-border;
    border-radius: $radius-lg;

    background: $color-surface-2;

    &:focus {
      outline: none;
      border-color: $color-accent;
    }
  }

  .version-list-content {
    position: relative;
    width: 100%;
  }

  .version-list-items {
    position: absolute;
    inset-inline: 0;
    top: 0;
  }

  .version-option {
    display: block;

    width: 100%;
    height: 36px;
    padding: 0 0.75rem;

    border: none;
    border-bottom: 1px solid color-mix(in srgb, $color-border 50%, transparent);

    background: transparent;
    color: $color-text;

    text-align: left;
    font-size: 0.9rem;

    cursor: pointer;

    &:hover {
      background: color-mix(in srgb, $color-accent 10%, transparent);
    }

    &.selected {
      background: color-mix(in srgb, $color-accent 20%, transparent);
      color: $color-accent;
      font-weight: 600;
    }
  }

  .version-count {
    margin-top: 0.4rem;

    color: $color-text-muted;
    font-size: 0.8rem;
    text-align: right;
  }

  .no-results {
    padding: 2rem;

    border: 1px solid $color-border;
    border-radius: $radius-lg;

    background: $color-surface-2;
    color: $color-text-muted;

    text-align: center;
  }

  .source-tabs {
    display: grid;
    grid-template-columns: repeat(4, 1fr);

    gap: 0.25rem;
    padding: 0.25rem;

    border-radius: $radius-lg;
    background: $color-surface-2;

    button {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.4rem;

      min-width: 0;
      padding: 0.65rem 0.75rem;

      border: none;
      border-radius: $radius-lg;

      background: transparent;
      color: $color-text-muted;

      cursor: pointer;
      font-size: 0.9rem;

      &.active {
        background: $color-surface-1;
        color: $color-text;

        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
      }

      &:hover:not(.active) {
        color: $color-text;
      }
    }
  }

  .source-content {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;

    margin-top: 1rem;
  }

  .source-content label {
    color: $color-text;
    font-size: 0.9rem;
    font-weight: 500;
  }

  .source-select {
    width: 100%;
    box-sizing: border-box;

    padding: 0.65rem 0.75rem;

    border: 1px solid $color-border;
    border-radius: $radius-lg;

    background: $color-surface-2;
    color: $color-text;

    &:focus {
      outline: none;
      border-color: $color-accent;
    }
  }

  .source-description {
    color: $color-text-muted;
    font-size: 0.8rem;
  }

  .file-picker {
    display: flex;
    align-items: center;
    gap: 0.75rem;

    .file-name {
      min-width: 0;
      flex: 1;

      overflow: hidden;

      padding: 0.65rem 0.75rem;

      border: 1px solid $color-border;
      border-radius: $radius-lg;

      background: $color-surface-2;
      color: $color-text-muted;

      text-overflow: ellipsis;
      white-space: nowrap;

      &.file-selected {
        color: $color-text;
      }
    }
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;

    padding-top: 0.5rem;

    .btn {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.4rem;

      padding: 0.6rem 1.25rem;

      border: none;
      border-radius: $radius-lg;

      cursor: pointer;
      font-size: 0.95rem;

      &:disabled {
        cursor: default;
        opacity: 0.5;
      }
    }

    .btn-primary {
      background: $color-accent;
      color: $color-text;
    }

    .btn-secondary {
      background: $color-surface-2;
      color: $color-text;
    }
  }
}
</style>
