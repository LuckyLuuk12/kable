<!--
@component
CreateInstallationModal - Modal dialog for creating new Minecraft installations

Allows users to select a Minecraft version, mod loader (Vanilla, Fabric, Forge, etc.),
and configure installation settings. Supports searching and filtering versions.

@example
```svelte
<CreateInstallationModal bind:this={createModal} />
<button on:click={() => createModal.open()}>Create Installation</button>
```
-->
<script lang="ts">
  import { type VersionData, Loader, InstallationService, type LoaderKind } from '$lib';
  import { onMount } from 'svelte';
  import * as installationsApi from '$lib/api/installations';
  import Icon from '../Icon.svelte';

  let dialogRef: HTMLDialogElement;
  let availableVersions: VersionData[] = [];
  let loaderOptions: LoaderKind[] = [];
  let selectedLoader: LoaderKind = "Vanilla";
  let selectedVersionId: string = '';
  let searchQuery: string = '';
  let isLoading = false;
  let error: string | null = null;
  let versionListRef: HTMLSelectElement;
  
  const INITIAL_DISPLAY_COUNT = 50;
  const LOAD_MORE_COUNT = 50;
  let displayCount = INITIAL_DISPLAY_COUNT;

  // Create a map of loader -> versions for O(1) lookup instead of filtering every time
  $: versionsByLoader = availableVersions.reduce((map, version) => {
    if (!map.has(version.loader)) {
      map.set(version.loader, []);
    }
    map.get(version.loader)!.push(version);
    return map;
  }, new Map<LoaderKind, VersionData[]>());

  $: allVersionsForLoader = versionsByLoader.get(selectedLoader) ?? [];
  
  // Filter by search query
  $: filteredVersions = searchQuery.trim() 
    ? allVersionsForLoader.filter(v => 
        v.version_id.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : allVersionsForLoader;
  
  // Only display a subset for performance
  $: displayedVersions = filteredVersions.slice(0, displayCount);
  $: hasMoreVersions = displayedVersions.length < filteredVersions.length;
  
  // Reset display count when loader or search changes
  $: {
    if (selectedLoader || searchQuery) {
      displayCount = INITIAL_DISPLAY_COUNT;
    }
  }
  
  $: if (filteredVersions.length > 0 && !filteredVersions.find(v => v.version_id === selectedVersionId)) {
    selectedVersionId = filteredVersions[0]?.version_id ?? '';
  }

  function loadMoreVersions() {
    displayCount += LOAD_MORE_COUNT;
  }

  function handleScroll(event: Event) {
    const target = event.target as HTMLSelectElement;
    const scrollThreshold = 100; // pixels from bottom
    const isNearBottom = target.scrollHeight - target.scrollTop - target.clientHeight < scrollThreshold;
    
    if (isNearBottom && hasMoreVersions) {
      loadMoreVersions();
    }
  }

  export function open() {
    dialogRef?.showModal();
  }
  
  export function close() {
    dialogRef?.close();
  }

  onMount(async () => {
    isLoading = true;
    try {
      availableVersions = await installationsApi.getAllVersions() ?? InstallationService.getFallbackVersions();
      loaderOptions = Array.from(new Set(availableVersions.map(v => v.loader)));
      selectedLoader = loaderOptions[0] ?? Loader.Vanilla;
      if (filteredVersions.length > 0) {
        selectedVersionId = filteredVersions[0].version_id;
      }
    } catch (e) {
      error = 'Failed to load versions.';
    } finally {
      isLoading = false;
    }
  });

  async function confirmCreate() {
    if (!selectedVersionId) return;
    isLoading = true;
    try {
      await InstallationService.createInstallation(selectedVersionId);
      close();
    } catch (e) {
      error = 'Failed to create installation.';
    } finally {
      isLoading = false;
    }
  }

  function cancelCreate() {
    close();
  }
</script>

<dialog bind:this={dialogRef} class="create-installation-modal">
  <h2>Create New Installation</h2>
  {#if error}
    <div class="error-message">{error}</div>
  {/if}
  <form on:submit|preventDefault={confirmCreate}>
    <div class="loader-select-row">
      {#each loaderOptions as loader}
        <button
          type="button"
          class="loader-btn {selectedLoader === loader ? 'selected' : ''}"
          style="background: {InstallationService.getLoaderColor(loader)}20; color: {InstallationService.getLoaderColor(loader)};"
          on:click={() => selectedLoader = loader}
        >
          <span class="loader-icon">
            <!-- TODO: Change this to Image and add images for all loaders to the assets -->
            <Icon
              name={InstallationService.getLoaderIcon(loader)}
              size="md"
              forceType="svg"
            />
          </span>
          <span class="loader-label">{loader.replace(/_/g, ' ').replace(/(^|\s)([a-z])/g, (_, p1, p2) => p1 + p2.toUpperCase())}</span>
        </button>
      {/each}
    </div>
    <div class="version-select-section">
      <label for="version-search">
        Search Version:
        <input 
          id="version-search"
          type="text" 
          bind:value={searchQuery} 
          placeholder="Search for a version..."
          class="version-search"
        />
      </label>
      <label for="version-select">
        Version:
        <select 
          id="version-select" 
          bind:value={selectedVersionId} 
          bind:this={versionListRef}
          on:scroll={handleScroll}
          size="10" 
          class="version-list"
        >
          {#each displayedVersions as version (version.version_id)}
            <option value={version.version_id}>{version.version_id}</option>
          {/each}
        </select>
      </label>
      {#if hasMoreVersions}
        <button type="button" class="load-more-btn" on:click={loadMoreVersions}>
          Load more... ({displayedVersions.length} of {filteredVersions.length})
        </button>
      {:else if filteredVersions.length > 0}
        <div class="version-count">
          Showing all {filteredVersions.length} version{filteredVersions.length !== 1 ? 's' : ''}
        </div>
      {/if}
      {#if searchQuery && filteredVersions.length === 0}
        <div class="no-results">No versions found matching "{searchQuery}"</div>
      {/if}
    </div>
    <div class="actions">
      <button type="submit" class="btn btn-primary" disabled={isLoading}>Create</button>
      <button type="button" class="btn btn-secondary" on:click={cancelCreate} disabled={isLoading}>Cancel</button>
    </div>
  </form>
</dialog>

<style lang="scss">
@use '@kablan/clean-ui/scss/_variables.scss' as *;
.create-installation-modal {
  padding: 2rem;
  background: var(--container);
  border-radius: var(--border-radius);
  width: 85%;
  height: 85%;
  margin: auto auto;
  h2 {
    margin-bottom: 1rem;
    color: var(--text);
  }
  form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    .loader-select-row {
      display: flex;
      gap: 1rem;
      margin-bottom: 1rem;
      .loader-btn {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.75rem 1.25rem;
        border-radius: var(--border-radius);
        border: none;
        font-size: 1rem;
        cursor: pointer;
        background: var(--container);
        color: var(--text);
        transition: box-shadow 0.2s;
        &.selected {
          box-shadow: 0 0 0 2px var(--primary);
        }
        .loader-icon {
          display: flex;
          align-items: center;
        }
        .loader-label {
          font-weight: 500;
        }
      }
    }
    label {
      color: var(--text);
      font-size: 1rem;
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }
    .version-select-section {
      display: flex;
      flex-direction: column;
      gap: 0.75rem;
      
      .version-search {
        width: 100%;
        padding: 0.6rem;
        border-radius: var(--border-radius);
        border: 1px solid var(--dark-300);
        background: var(--card);
        color: var(--text);
        font-size: 1rem;
        transition: border-color 0.2s;
        
        &:focus {
          outline: none;
          border-color: var(--primary);
        }
        
        &::placeholder {
          color: var(--placeholder);
        }
      }
      
      .version-list {
        width: 100%;
        padding: 0.5rem;
        border-radius: var(--border-radius);
        border: 1px solid var(--dark-300);
        background: var(--card);
        color: var(--text);
        font-size: 0.95rem;
        min-height: 300px;
        
        &:focus {
          outline: none;
          border-color: var(--primary);
        }
        
        option {
          padding: 0.4rem;
          cursor: pointer;
          
          &:hover {
            background: color-mix(in srgb, var(--primary), 10%, transparent);
          }
        }
      }
      
      .load-more-btn {
        padding: 0.5rem 1rem;
        border-radius: var(--border-radius);
        border: 1px solid var(--dark-300);
        background: var(--card);
        color: var(--text);
        font-size: 0.9rem;
        cursor: pointer;
        transition: background 0.2s, border-color 0.2s;
        
        &:hover {
          background: color-mix(in srgb, var(--primary), 10%, transparent);
          border-color: var(--primary);
        }
      }
      
      .version-count {
        font-size: 0.85rem;
        color: var(--placeholder);
        text-align: center;
        padding: 0.25rem;
      }
      
      .no-results {
        font-size: 0.9rem;
        color: var(--placeholder);
        text-align: center;
        padding: 1rem;
        font-style: italic;
      }
    }
    .actions {
      display: flex;
      gap: 1rem;
      button {
        padding: 0.5rem 1.5rem;
        border-radius: var(--border-radius);
        border: none;
        font-size: 1rem;
        cursor: pointer;
        &.btn-primary {
          background: var(--primary);
          color: var(--text);
        }
        &.btn-secondary {
          background: var(--container);
          color: var(--text);
        }
      }
    }
  }
  .error-message {
    color: var(--red);
    margin-bottom: 1rem;
  }
}
</style>
