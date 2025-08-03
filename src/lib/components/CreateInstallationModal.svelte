<script lang="ts">
  import { type VersionData, Loader, InstallationService, type LoaderKind } from '$lib';
  import { onMount } from 'svelte';
  import * as installationsApi from '$lib/api/installations';

  let dialogRef: HTMLDialogElement;
  let availableVersions: VersionData[] = [];
  let loaderOptions: LoaderKind[] = [];
  let selectedLoader: LoaderKind = "Vanilla";
  let selectedVersionId: string = '';
  let isLoading = false;
  let error: string | null = null;

  $: filteredVersions = availableVersions.filter(v => v.loader === selectedLoader);
  $: if (filteredVersions.length > 0 && !filteredVersions.find(v => v.version_id === selectedVersionId)) {
    selectedVersionId = filteredVersions[0]?.version_id ?? '';
  }

  function open() {
    dialogRef?.showModal();
  }
  function close() {
    dialogRef?.close();
  }

  onMount(async () => {
    isLoading = true;
    try {
      availableVersions = await installationsApi.get_all_versions() ?? InstallationService.getFallbackVersions();
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
            <svg width="24" height="24" style="vertical-align: middle;">
              <use href={`#icon-${InstallationService.getLoaderIcon(loader)}`} />
            </svg>
          </span>
          <span class="loader-label">{loader.replace(/_/g, ' ').replace(/(^|\s)([a-z])/g, (_, p1, p2) => p1 + p2.toUpperCase())}</span>
        </button>
      {/each}
    </div>
    <label>
      Version:
      <select bind:value={selectedVersionId}>
        {#each filteredVersions as version}
          <option value={version.version_id}>{version.version_id}</option>
        {/each}
      </select>
    </label>
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
  background: $container;
  border-radius: $border-radius;
  max-width: 28rem;
  margin: 0 auto;
  h2 {
    margin-bottom: 1rem;
    color: $text;
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
        border-radius: $border-radius;
        border: none;
        font-size: 1rem;
        cursor: pointer;
        background: $container;
        color: $text;
        transition: box-shadow 0.2s;
        &.selected {
          box-shadow: 0 0 0 2px $primary;
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
      color: $text;
      font-size: 1rem;
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }
    .actions {
      display: flex;
      gap: 1rem;
      button {
        padding: 0.5rem 1.5rem;
        border-radius: $border-radius;
        border: none;
        font-size: 1rem;
        cursor: pointer;
        &.btn-primary {
          background: $primary;
          color: $text;
        }
        &.btn-secondary {
          background: $container;
          color: $text;
        }
      }
    }
  }
  .error-message {
    color: $red;
    margin-bottom: 1rem;
  }
}
</style>
