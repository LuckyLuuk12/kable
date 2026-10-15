<script lang="ts">
import { Icon, Image, app, clickSound, successSound, type CreateProfileRequest, type LoaderKind, type ProfileSource, type ProfileVersion } from "$lib";
import { MODAL_CONTEXT, type ModalContext } from "$lib/utils/modal";
import { open } from "@tauri-apps/plugin-dialog";
import { getContext } from "svelte";
import { SvelteSet } from "svelte/reactivity";

type SourceKind = "profile" | "zip" | "mrpack";

interface SourceDraft {
  id: number;
  type: SourceKind;
  profileId: string;
  path: string | null;
}

const modal = getContext<ModalContext>(MODAL_CONTEXT);

const LOADER_ORDER: LoaderKind[] = ["fabric", "iris_fabric", "quilt", "vanilla", "forge", "neo_forge"];

const ROW_HEIGHT = 36;
const VIRTUALIZATION_OVERSCAN = 8;

let selectedLoader = $state<LoaderKind>("fabric");
let selectedVersionId = $state("");
let searchQuery = $state("");

let sources = $state<SourceDraft[]>([]);
let nextSourceId = 1;

let profileName = $state("");
let profileIcon = $state("");
let profileDescription = $state("");
let updateProjects = $state(true);

let isCreating = $state(false);
let error = $state<string | null>(null);

let versionListElement = $state<HTMLDivElement | null>(null);
let versionListHeight = $state(360);
let scrollTop = $state(0);

let isLoading = $state(false);

const availableVersions = $derived(app.launcherService.versions ?? []);
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

    if (aIndex === -1 && bIndex === -1) return a.localeCompare(b);
    if (aIndex === -1) return 1;
    if (bIndex === -1) return -1;

    return aIndex - bIndex;
  });
});

const versionsForLoader = $derived(availableVersions.filter((version) => version.loader === selectedLoader));

const filteredVersions = $derived.by(() => {
  const query = searchQuery.trim().toLowerCase();

  if (!query) return versionsForLoader;

  return versionsForLoader.filter((version) => version.id.toLowerCase().includes(query));
});

const totalVersionHeight = $derived(filteredVersions.length * ROW_HEIGHT);

const firstVisibleIndex = $derived(Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - VIRTUALIZATION_OVERSCAN));

const lastVisibleIndex = $derived(Math.min(filteredVersions.length, Math.ceil((scrollTop + versionListHeight) / ROW_HEIGHT) + VIRTUALIZATION_OVERSCAN));

const visibleVersions = $derived(filteredVersions.slice(firstVisibleIndex, lastVisibleIndex));

const visibleOffset = $derived(firstVisibleIndex * ROW_HEIGHT);

function getSelectedProfile(profileId: string) {
  return availableProfiles.find((profile) => profile.id === profileId) ?? null;
}

function isSourceValid(source: SourceDraft): boolean {
  switch (source.type) {
    case "profile":
      return source.profileId !== "" && getSelectedProfile(source.profileId) !== null;

    case "zip":
    case "mrpack":
      return source.path !== null && source.path.trim() !== "";
  }
}

const canCreate = $derived(!isCreating && selectedVersionId !== "" && sources.every(isSourceValid));

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
  if (!versionListElement) return;

  scrollTop = versionListElement.scrollTop;
}

function handleVersionResize() {
  if (!versionListElement) return;

  versionListHeight = versionListElement.clientHeight;
}

function resetVersionScroll() {
  scrollTop = 0;

  if (versionListElement) {
    versionListElement.scrollTop = 0;
  }
}

function addSource(type: SourceKind = "profile") {
  sources.push({
    id: nextSourceId++,
    type,
    profileId: "",
    path: null,
  });

  error = null;
}

function removeSource(sourceId: number) {
  sources = sources.filter((source) => source.id !== sourceId);
  error = null;
}

function moveSource(sourceId: number, offset: -1 | 1) {
  const index = sources.findIndex((source) => source.id === sourceId);
  const targetIndex = index + offset;

  if (index < 0 || targetIndex < 0 || targetIndex >= sources.length) {
    return;
  }

  const reordered = [...sources];
  [reordered[index], reordered[targetIndex]] = [reordered[targetIndex], reordered[index]];

  sources = reordered;
  error = null;
}

function changeSourceType(sourceId: number, type: SourceKind) {
  const source = sources.find((item) => item.id === sourceId);

  if (!source || source.type === type) return;

  source.type = type;
  source.profileId = "";
  source.path = null;
  error = null;
}

function selectProfile(sourceId: number, profileId: string) {
  const source = sources.find((item) => item.id === sourceId);

  if (!source) return;

  source.profileId = profileId;
  error = null;
}

async function selectSourceFile(sourceId: number, type: "zip" | "mrpack") {
  error = null;

  try {
    const path = await open({
      multiple: false,
      directory: false,
      filters: [
        {
          name: type === "zip" ? "Exported Profile" : "Modrinth Pack",
          extensions: [type === "zip" ? "zip" : "mrpack"],
        },
      ],
    });

    if (typeof path !== "string") return;

    const source = sources.find((item) => item.id === sourceId);

    if (!source || source.type !== type) return;

    source.path = path;
  } catch (e) {
    console.error(`Failed to select ${type} file:`, e);
    error = e instanceof Error ? e.message : `Failed to select ${type === "zip" ? "ZIP file" : "Modrinth pack"}.`;
  }
}

function sourceToRequest(source: SourceDraft): ProfileSource {
  switch (source.type) {
    case "profile": {
      if (!getSelectedProfile(source.profileId)) {
        throw new Error("Select a valid profile for every existing-profile source.");
      }

      return {
        type: "existing_profile",
        profile_id: source.profileId,
      };
    }

    case "zip": {
      if (!source.path) {
        throw new Error("Select an exported profile ZIP for every ZIP source.");
      }

      return {
        type: "exported_profile",
        path: source.path,
      };
    }

    case "mrpack": {
      if (!source.path) {
        throw new Error("Select a Modrinth pack for every Modrinth source.");
      }

      return {
        type: "modrinth_modpack",
        source: {
          type: "file",
          path: source.path,
        },
      };
    }
  }
}

function buildRequest(): CreateProfileRequest {
  if (!selectedVersionId) {
    throw new Error("Select a Minecraft version.");
  }

  const orderedSources = sources.map(sourceToRequest);

  const request: CreateProfileRequest = {
    metadata: {
      name: profileName.trim() || null,
      icon: profileIcon.trim() || null,
      description: profileDescription.trim() || null,
    },
    update_projects: updateProjects,
  };

  if (orderedSources.length === 0) {
    request.base = {
      type: "version",
      version_id: selectedVersionId,
    };
  } else {
    request.base = orderedSources[0];
    request.additional_sources = orderedSources.slice(1);
    request.version_override = selectedVersionId;
  }

  return request;
}

async function createProfile(event: SubmitEvent) {
  event.preventDefault();

  if (!canCreate) return;

  isCreating = true;
  error = null;

  try {
    const request = buildRequest();

    await app.profilesService.createProfile(request);

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
    <div class="header-copy">
      <h2>Create New Installation</h2>
      <p>Choose a Minecraft version and optionally combine existing profiles or imported packs.</p>
    </div>

    <button use:clickSound type="button" class="close-button" aria-label="Close" onclick={cancel} disabled={isCreating}>
      <Icon name="x" size="sm" forceType="svg" />
    </button>
  </header>

  {#if error}
    <div class="error-message" role="alert">
      <Icon name="alert-circle" size="sm" forceType="svg" />
      <span>{error}</span>
    </div>
  {/if}

  <form onsubmit={createProfile}>
    <div class="form-column version-column">
      <section class="section">
        <div class="section-heading">
          <div class="section-label">Mod Loader</div>
          <div class="section-caption">Choose a loader to filter available versions.</div>
        </div>

        <div class="loader-select-row">
          {#each loaderOptions as loader (loader)}
            <button
              type="button"
              class:selected={selectedLoader === loader}
              class="loader-button"
              style:background={`${app.profilesService.getLoaderColor(loader)}20`}
              style:color={app.profilesService.getLoaderColor(loader)}
              aria-pressed={selectedLoader === loader}
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

      <section class="section version-section">
        <div class="section-heading">
          <div class="section-label">Minecraft Version</div>
          <div class="section-caption">
            {#if selectedVersionId}
              Selected: {selectedVersionId}
            {:else}
              Select a version to continue.
            {/if}
          </div>
        </div>

        <input
          class="version-search"
          type="text"
          bind:value={searchQuery}
          placeholder="Search versions..."
          aria-label="Search Minecraft versions"
          oninput={resetVersionScroll}
          disabled={isCreating} />

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
                    disabled={isCreating}
                    onclick={() => selectVersion(version)}>
                    <span>{version.id}</span>
                    {#if selectedVersionId === version.id}
                      <Icon name="check" size="sm" forceType="svg" />
                    {/if}
                  </button>
                {/each}
              </div>
            </div>
          </div>

          <div class="version-count">
            <span>{filteredVersions.length} version{filteredVersions.length === 1 ? "" : "s"}</span>
            {#if isLoading}
              <span>Refreshing versions…</span>
            {/if}
          </div>
        {:else}
          <div class="no-results">
            {#if isLoading}
              Loading Minecraft versions...
            {:else}
              No versions found{searchQuery ? ` matching "${searchQuery}"` : ""}.
              {#if availableVersions.length === 0}
                <button type="button" class="inline-retry" onclick={loadVersions} disabled={isLoading}> Retry </button>
              {/if}
            {/if}
          </div>
        {/if}
        <label class="update-projects-toggle">
          <input type="checkbox" bind:checked={updateProjects} disabled={isCreating} />

          <span class="update-projects-content">
            <span class="update-projects-title">Update installed projects</span>
            <span class="update-projects-description">
              The selected Minecraft version will always be set as the profile version. When enabled, Kable will also attempt to update installed mods and other projects
              to match it. Disable this to change the version without updating projects.
            </span>
          </span>
        </label>
      </section>
    </div>

    <div class="form-column sources-column">
      <section class="section">
        <div class="section-heading sources-section-heading">
          <div>
            <div class="section-label">Profile Sources</div>
            <p class="section-help">Combine any number of existing profiles, exported ZIPs, and Modrinth packs. Later sources take precedence over earlier ones.</p>
          </div>

          <span class="source-count">{sources.length} selected</span>
        </div>

        {#if sources.length === 0}
          <div class="empty-sources">
            <div class="empty-sources-title">Fresh installation</div>
            <p>
              No additional sources selected. The installation will start from the selected Minecraft version. You can add sources below to merge their contents into the
              new profile.
            </p>
          </div>
        {:else}
          <div class="source-list">
            {#each sources as source, index (source.id)}
              <article class="source-card">
                <div class="source-card-header">
                  <div class="source-order">{index + 1}</div>

                  <div class="source-heading">
                    <div class="source-title">
                      {index === 0 ? "Base source" : `Additional source ${index + 1}`}
                    </div>
                    <div class="source-subtitle">
                      {#if index === 0}
                        Initial profile data
                      {:else}
                        Merged after the preceding sources
                      {/if}
                    </div>
                  </div>

                  <div class="source-controls">
                    <button
                      use:clickSound
                      type="button"
                      class="icon-button"
                      aria-label={`Move source ${index + 1} up`}
                      title="Move up"
                      disabled={isCreating || index === 0}
                      onclick={() => moveSource(source.id, -1)}>
                      <Icon name="chevron-up" size="sm" forceType="svg" />
                    </button>

                    <button
                      use:clickSound
                      type="button"
                      class="icon-button"
                      aria-label={`Move source ${index + 1} down`}
                      title="Move down"
                      disabled={isCreating || index === sources.length - 1}
                      onclick={() => moveSource(source.id, 1)}>
                      <Icon name="chevron-down" size="sm" forceType="svg" />
                    </button>

                    <button
                      use:clickSound
                      type="button"
                      class="icon-button remove-source"
                      aria-label={`Remove source ${index + 1}`}
                      title="Remove source"
                      disabled={isCreating}
                      onclick={() => removeSource(source.id)}>
                      <Icon name="delete" size="sm" forceType="svg" />
                    </button>
                  </div>
                </div>

                <div class="source-card-content">
                  <label for={`source-type-${source.id}`}>Source type</label>

                  <select
                    id={`source-type-${source.id}`}
                    class="source-select"
                    value={source.type}
                    disabled={isCreating}
                    onchange={(event) => changeSourceType(source.id, event.currentTarget.value as SourceKind)}>
                    <option value="profile">Existing profile</option>
                    <option value="zip">Exported profile ZIP</option>
                    <option value="mrpack">Modrinth modpack</option>
                  </select>

                  {#if source.type === "profile"}
                    <label for={`source-profile-${source.id}`}>Profile</label>

                    <select
                      id={`source-profile-${source.id}`}
                      class="source-select"
                      value={source.profileId}
                      disabled={isCreating}
                      onchange={(event) => selectProfile(source.id, event.currentTarget.value)}>
                      <option value="">Select a profile...</option>

                      {#each availableProfiles as profile (profile.id)}
                        <option value={profile.id}>
                          {profile.metadata.name} ({profile.version.id})
                        </option>
                      {/each}
                    </select>

                    {#if source.profileId && !getSelectedProfile(source.profileId)}
                      <div class="field-error">This profile is no longer available. Select another profile.</div>
                    {/if}

                    <p class="source-description">Merge an existing installation into the new profile. The same profile can be selected more than once.</p>
                  {:else}
                    <!-- svelte-ignore a11y_label_has_associated_control -->
                    <label>{source.type === "zip" ? "Exported profile ZIP" : "Modrinth pack file"}</label>

                    <div class="file-picker">
                      <div class:file-selected={source.path !== null} class="file-name" title={source.path ?? ""}>
                        {source.path ?? (source.type === "zip" ? "No exported profile selected" : "No Modrinth pack selected")}
                      </div>

                      <button
                        use:clickSound
                        type="button"
                        class="btn btn-secondary file-picker-button"
                        onclick={() => selectSourceFile(source.id, source.type as "zip" | "mrpack")}
                        disabled={isCreating}>
                        <Icon name="folder" size="sm" />
                        Browse
                      </button>

                      {#if source.path}
                        <button
                          use:clickSound
                          type="button"
                          class="icon-button"
                          aria-label="Clear selected file"
                          title="Clear selected file"
                          disabled={isCreating}
                          onclick={() => (source.path = null)}>
                          <Icon name="x" size="sm" forceType="svg" />
                        </button>
                      {/if}
                    </div>

                    <p class="source-description">
                      {source.type === "zip" ? "Import a Kable exported profile ZIP." : "Import a Modrinth .mrpack file and merge its contents."}
                    </p>
                  {/if}
                </div>
              </article>
            {/each}
          </div>
        {/if}

        <div class="add-source-panel">
          <div class="add-source-label">Add another source</div>

          <div class="add-source-buttons">
            <button use:clickSound type="button" class="btn btn-secondary" onclick={() => addSource("profile")} disabled={isCreating}>
              <Icon name="plus" size="sm" forceType="svg" />
              Existing profile
            </button>

            <button use:clickSound type="button" class="btn btn-secondary" onclick={() => addSource("zip")} disabled={isCreating}>
              <Icon name="plus" size="sm" forceType="svg" />
              Kable Profile ZIP
            </button>

            <button use:clickSound type="button" class="btn btn-secondary" onclick={() => addSource("mrpack")} disabled={isCreating}>
              <Icon name="plus" size="sm" forceType="svg" />
              Modrinth modpack
            </button>
          </div>
        </div>
      </section>

      <section class="section metadata-section">
        <div class="section-label">Profile Metadata</div>

        <p class="section-help metadata-help">Optional overrides applied after all sources are merged. Leave a field empty to let the backend determine its value.</p>

        <div class="metadata-fields">
          <div class="metadata-field">
            <label for="profile-name">Profile name</label>
            <input
              id="profile-name"
              class="metadata-input"
              type="text"
              bind:value={profileName}
              placeholder="Use the default name"
              maxlength="100"
              disabled={isCreating} />
          </div>

          <div class="metadata-field">
            <label for="profile-icon">Icon</label>
            <input id="profile-icon" class="metadata-input" type="text" bind:value={profileIcon} placeholder="Icon URL or supported icon value" disabled={isCreating} />
          </div>

          <div class="metadata-field">
            <label for="profile-description">Description</label>
            <textarea
              id="profile-description"
              class="metadata-input metadata-textarea"
              bind:value={profileDescription}
              placeholder="Optional profile description"
              rows="3"
              maxlength="1000"
              disabled={isCreating}></textarea>
          </div>
        </div>
      </section>
    </div>

    <footer class="actions">
      <div class="actions-summary">
        {#if selectedVersionId}
          <span class="summary-version">{selectedVersionId}</span>
        {/if}
        <span>
          {sources.length === 0 ? "Fresh installation" : `${sources.length} source${sources.length === 1 ? "" : "s"} in merge order`}
        </span>
      </div>

      <div class="actions-buttons">
        <button use:clickSound type="button" class="btn btn-secondary" onclick={cancel} disabled={isCreating}> Cancel </button>

        <button use:successSound type="submit" class="btn btn-primary" disabled={!canCreate}>
          {#if isCreating}
            <Icon name="refresh" size="sm" className="spin" forceType="svg" />
            Creating...
          {:else}
            <Icon name="plus" size="sm" forceType="svg" />
            Create installation
          {/if}
        </button>
      </div>
    </footer>
  </form>
</div>

<style lang="scss">
.create-profile-modal {
  width: 85vw;
  max-width: 1500px;
  max-height: 90vh;
  min-width: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-sizing: border-box;
  padding: 1.25rem 1.5rem 1rem;
  background: $color-surface-1;
  color: $color-text;
  border: 1px solid $color-border;
  border-radius: $radius-lg;
  box-shadow: 0 0.75rem 3rem rgba(0, 0, 0, 0.35);

  .modal-header {
    display: flex;
    flex: 0 0 auto;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1rem;

    .header-copy {
      min-width: 0;
    }

    h2 {
      margin: 0;
      color: $color-text;
      font-size: 1.35rem;
    }

    p {
      margin: 0.35rem 0 0;
      color: $color-text-muted;
      font-size: 0.85rem;
      line-height: 1.45;
    }

    .close-button {
      display: flex;
      flex: 0 0 auto;
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

      &:hover:not(:disabled) {
        background: $color-surface-2;
        color: $color-text;
      }

      &:disabled {
        cursor: default;
        opacity: 0.5;
      }
    }
  }

  .error-message {
    display: flex;
    flex: 0 0 auto;
    align-items: flex-start;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
    padding: 0.65rem 0.85rem;
    border-radius: $radius-lg;
    background: color-mix(in srgb, $color-error 15%, transparent);
    color: $color-error;
    overflow-wrap: anywhere;

    :global(svg) {
      flex: 0 0 auto;
      margin-top: 0.1rem;
    }
  }

  form {
    display: grid;
    flex: 1 1 auto;
    grid-template-columns: minmax(0, 0.92fr) minmax(0, 1.08fr);
    grid-template-rows: minmax(0, 1fr) auto;
    gap: 1rem 1.25rem;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .form-column {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    min-width: 0;
    min-height: 0;
    overflow-x: hidden;
    overflow-y: auto;
    overscroll-behavior: contain;
    scrollbar-gutter: stable;
    padding: 0 0.35rem 0.5rem 0;

    > .section {
      flex: 0 0 auto;
    }
  }

  .section {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .section-heading {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.75rem;
    margin-bottom: 0.65rem;
  }

  .section-label {
    margin: 0;
    color: $color-text-muted;
    font-size: 0.9rem;
    font-weight: 600;
  }

  .section-caption {
    color: $color-text-muted;
    font-size: 0.75rem;
    text-align: right;
  }

  .section-help {
    margin: 0.3rem 0 0;
    color: $color-text-muted;
    font-size: 0.8rem;
    line-height: 1.45;
  }

  .loader-select-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .loader-button {
    display: flex;
    flex: 0 1 auto;
    align-items: center;
    gap: 0.45rem;
    min-height: 2.65rem;
    padding: 0.5rem 0.7rem;
    border: 2px solid transparent;
    border-radius: $radius-lg;
    cursor: pointer;
    transition:
      border-color 0.15s,
      box-shadow 0.15s;

    &.selected {
      border-color: $color-accent;
    }

    &:focus-visible {
      outline: 2px solid $color-accent;
      outline-offset: 2px;
    }

    .loader-icon {
      display: flex;
      flex: 0 0 auto;
      align-items: center;
      justify-content: center;
      width: 1.35rem;
      height: 1.35rem;
    }

    .loader-label {
      font-size: 0.82rem;
      font-weight: 500;
      white-space: nowrap;
    }
  }

  .version-section {
    min-height: 0;
  }

  .version-search,
  .metadata-input {
    width: 100%;
    box-sizing: border-box;
    padding: 0.65rem 0.75rem;
    border: 1px solid $color-border;
    border-radius: $radius-lg;
    background: $color-surface-2;
    color: $color-text;
    font: inherit;

    &:focus {
      outline: none;
      border-color: $color-accent;
    }

    &::placeholder {
      color: $color-text-muted;
    }

    &:disabled {
      cursor: default;
      opacity: 0.6;
    }
  }

  .version-search {
    margin-bottom: 0.5rem;
    font-size: 0.9rem;
  }

  .version-list {
    height: clamp(220px, 42vh, 460px);
    flex: 0 0 auto;
    overflow-y: auto;
    border: 1px solid $color-border;
    border-radius: $radius-lg;
    background: $color-surface-2;
    overscroll-behavior: contain;

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
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    width: 100%;
    height: 36px;
    padding: 0 0.75rem;
    border: none;
    border-bottom: 1px solid color-mix(in srgb, $color-border 50%, transparent);
    background: transparent;
    color: $color-text;
    text-align: left;
    font-size: 0.88rem;
    cursor: pointer;

    &:hover:not(:disabled) {
      background: color-mix(in srgb, $color-accent 10%, transparent);
    }

    &:focus-visible {
      outline: 2px solid $color-accent;
      outline-offset: -2px;
    }

    &.selected {
      background: color-mix(in srgb, $color-accent 20%, transparent);
      color: $color-accent;
      font-weight: 600;
    }

    &:disabled {
      cursor: default;
      opacity: 0.6;
    }
  }

  .version-count {
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
    margin-top: 0.4rem;
    color: $color-text-muted;
    font-size: 0.75rem;
  }

  .no-results {
    padding: 1.5rem 1rem;
    border: 1px solid $color-border;
    border-radius: $radius-lg;
    background: $color-surface-2;
    color: $color-text-muted;
    text-align: center;
    font-size: 0.85rem;
  }

  .inline-retry {
    display: inline;
    padding: 0;
    border: none;
    background: transparent;
    color: $color-accent;
    cursor: pointer;
    text-decoration: underline;
  }

  .update-projects-toggle {
    display: flex;
    align-items: flex-start;
    gap: 0.65rem;
    margin-top: 0.25rem;
    padding: 0.8rem;
    border: 1px solid $color-border;
    border-radius: $radius-lg;
    background: $color-surface-2;
    cursor: pointer;

    > input {
      flex: 0 0 auto;
      width: 1rem;
      height: 1rem;
      margin: 0.15rem 0 0;
      accent-color: $color-accent;
      cursor: pointer;

      &:disabled {
        cursor: default;
      }
    }

    .update-projects-content {
      display: flex;
      flex-direction: column;
      gap: 0.3rem;
      min-width: 0;
    }

    .update-projects-title {
      color: $color-text;
      font-size: 0.75rem;
      font-weight: 600;
    }

    .update-projects-description {
      color: $color-text-muted;
      font-size: 0.65rem;
      line-height: 1.45;
    }

    &:has(input:disabled) {
      cursor: default;
      opacity: 0.7;
    }

    &:focus-within {
      border-color: $color-accent;
    }
  }

  .sources-section-heading {
    align-items: flex-start;
  }

  .source-count {
    flex: 0 0 auto;
    padding: 0.25rem 0.5rem;
    border-radius: 999px;
    background: $color-surface-2;
    color: $color-text-muted;
    font-size: 0.72rem;
    white-space: nowrap;
  }

  .empty-sources {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 1.5rem 1rem;
    border: 1px dashed $color-border;
    border-radius: $radius-lg;
    background: color-mix(in srgb, $color-surface-2 55%, transparent);
    color: $color-text-muted;
    text-align: center;

    .empty-sources-title {
      color: $color-text;
      font-weight: 600;
    }

    p {
      max-width: 28rem;
      margin: 0;
      font-size: 0.82rem;
      line-height: 1.5;
    }
  }

  .source-list {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  .source-card {
    min-width: 0;
    overflow: hidden;
    border: 1px solid $color-border;
    border-radius: $radius-lg;
    background: $color-surface-2;
  }

  .source-card-header {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding: 0.6rem 0.7rem;
    border-bottom: 1px solid $color-border;
  }

  .source-order {
    display: flex;
    flex: 0 0 1.9rem;
    align-items: center;
    justify-content: center;
    width: 1.9rem;
    height: 1.9rem;
    border-radius: $radius-lg;
    background: color-mix(in srgb, $color-accent 15%, transparent);
    color: $color-accent;
    font-size: 0.85rem;
    font-weight: 700;
  }

  .source-heading {
    flex: 1;
    min-width: 0;
  }

  .source-title {
    color: $color-text;
    font-size: 0.86rem;
    font-weight: 600;
  }

  .source-subtitle {
    margin-top: 0.15rem;
    color: $color-text-muted;
    font-size: 0.72rem;
  }

  .source-controls {
    display: flex;
    flex-shrink: 0;
    gap: 0.1rem;
  }

  .icon-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.85rem;
    height: 1.85rem;
    padding: 0;
    border: 1px solid transparent;
    border-radius: $radius-lg;
    background: transparent;
    color: $color-text-muted;
    cursor: pointer;

    &:hover:not(:disabled) {
      border-color: $color-border;
      background: $color-surface-1;
      color: $color-text;
    }

    &:focus-visible {
      outline: 2px solid $color-accent;
      outline-offset: 1px;
    }

    &:disabled {
      cursor: default;
      opacity: 0.3;
    }

    &.remove-source:hover:not(:disabled) {
      border-color: color-mix(in srgb, $color-error 35%, transparent);
      background: color-mix(in srgb, $color-error 12%, transparent);
      color: $color-error;
    }
  }

  .source-card-content {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    min-width: 0;
    padding: 0.75rem;

    label {
      color: $color-text;
      font-size: 0.8rem;
      font-weight: 500;
    }
  }

  .source-select {
    width: 100%;
    min-width: 0;
    box-sizing: border-box;
    padding: 0.6rem 0.7rem;
    border: 1px solid $color-border;
    border-radius: $radius-lg;
    background: $color-surface-1;
    color: $color-text;
    font: inherit;
    font-size: 0.85rem;

    &:focus {
      outline: none;
      border-color: $color-accent;
    }

    &:disabled {
      cursor: default;
      opacity: 0.6;
    }
  }

  .source-description {
    margin: 0.2rem 0 0;
    color: $color-text-muted;
    font-size: 0.77rem;
    line-height: 1.45;
    overflow-wrap: anywhere;
  }

  .field-error {
    color: $color-error;
    font-size: 0.78rem;
  }

  .file-picker {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    min-width: 0;

    .file-name {
      flex: 1;
      min-width: 0;
      overflow: hidden;
      padding: 0.6rem 0.7rem;
      border: 1px solid $color-border;
      border-radius: $radius-lg;
      background: $color-surface-1;
      color: $color-text-muted;
      text-overflow: ellipsis;
      white-space: nowrap;
      font-size: 0.8rem;

      &.file-selected {
        color: $color-text;
      }
    }

    .file-picker-button {
      flex: 0 0 auto;
    }
  }

  .add-source-panel {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    margin-top: 0.75rem;
    padding: 0.8rem;
    border: 1px dashed $color-border;
    border-radius: $radius-lg;
  }

  .add-source-label {
    color: $color-text-muted;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .add-source-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
  }

  .metadata-section {
    padding-top: 1rem;
    border-top: 1px solid $color-border;
  }

  .metadata-help {
    margin-bottom: 0.8rem;
  }

  .metadata-fields {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 0.75rem;
  }

  .metadata-field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    min-width: 0;

    &:last-child {
      grid-column: 1 / -1;
    }

    label {
      color: $color-text;
      font-size: 0.8rem;
      font-weight: 500;
    }
  }

  .metadata-input {
    font-size: 0.85rem;
  }

  .metadata-textarea {
    resize: vertical;
    min-height: 4rem;
  }

  .actions {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    min-width: 0;
    padding-top: 0.85rem;
    border-top: 1px solid $color-border;
    background: $color-surface-1;
  }

  .actions-summary {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
    color: $color-text-muted;
    font-size: 0.78rem;
  }

  .summary-version {
    color: $color-text;
    font-weight: 600;
  }

  .actions-buttons {
    display: flex;
    flex: 0 0 auto;
    justify-content: flex-end;
    gap: 0.65rem;
  }

  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.4rem;
    padding: 0.6rem 0.9rem;
    border: none;
    border-radius: $radius-lg;
    cursor: pointer;
    font: inherit;
    font-size: 0.85rem;
    white-space: nowrap;

    &:disabled {
      cursor: default;
      opacity: 0.5;
    }

    &:focus-visible {
      outline: 2px solid $color-accent;
      outline-offset: 2px;
    }
  }

  .btn-primary {
    background: $color-accent;
    color: $color-text;
  }

  .btn-secondary {
    background: $color-surface-2;
    color: $color-text;

    &:hover:not(:disabled) {
      background: color-mix(in srgb, $color-surface-2 70%, $color-accent 10%);
    }
  }

  @media (max-width: 900px) {
    width: 94vw;
    max-height: 92vh;
    padding: 1rem;

    form {
      grid-template-columns: minmax(0, 1fr);
      grid-template-rows: minmax(0, 1fr) minmax(0, 1fr) auto;
      gap: 0.75rem;
    }

    .form-column {
      padding-right: 0.25rem;
    }

    .version-list {
      height: clamp(160px, 28vh, 300px);
    }

    .actions {
      gap: 0.5rem;
    }

    .actions-summary {
      font-size: 0.72rem;
    }
  }

  @media (max-width: 560px) {
    width: 96vw;
    max-height: 94vh;
    padding: 0.75rem;

    .modal-header h2 {
      font-size: 1.15rem;
    }

    .modal-header p {
      font-size: 0.78rem;
    }

    form {
      gap: 0.65rem;
    }

    .form-column {
      gap: 1rem;
    }

    .loader-select-row {
      gap: 0.4rem;
    }

    .loader-button {
      padding: 0.45rem 0.55rem;

      .loader-label {
        font-size: 0.75rem;
      }
    }

    .source-card-header {
      gap: 0.4rem;
      padding: 0.5rem;
    }

    .source-subtitle {
      font-size: 0.68rem;
    }

    .source-controls {
      gap: 0;
    }

    .icon-button {
      width: 1.65rem;
      height: 1.65rem;
    }

    .file-picker {
      flex-wrap: wrap;

      .file-name {
        flex-basis: 100%;
      }
    }

    .add-source-buttons {
      display: grid;
      grid-template-columns: minmax(0, 1fr);
    }

    .metadata-fields {
      grid-template-columns: minmax(0, 1fr);
    }

    .metadata-field:last-child {
      grid-column: auto;
    }

    .actions {
      align-items: stretch;
      flex-direction: column;
    }

    .actions-summary {
      justify-content: space-between;
    }

    .actions-buttons {
      display: grid;
      grid-template-columns: minmax(0, 1fr) minmax(0, 1.5fr);
    }

    .btn {
      padding-inline: 0.6rem;
      font-size: 0.8rem;
    }
  }
}
</style>
