<script lang="ts">
  import { onMount } from 'svelte';
  import { SymlinksAPI, type SymlinkInfo } from '$lib/api';
  import { InstallationService, Icon } from '$lib';
  import type { KableInstallation } from '$lib/types';

  let symlinks: SymlinkInfo[] = [];
  let installations: KableInstallation[] = [];
  let loading = true;
  let error: string | null = null;
  let activeTab: 'symlinks' | 'other' = 'symlinks';

  // Edit mode tracking
  let editingSymlink: SymlinkInfo | null = null;
  let editSource = '';
  let editDestinationParent = '';
  let editInstallationId: string | null = null;

  // Create/edit symlink modal state
  let showCreateModal = false;
  let newSymlinkSource = '';
  let newSymlinkDestinationParent = '';
  let newSymlinkInstallationId: string | null = null;
  let modalError: string | null = null;
  let editError: string | null = null;

  // Copy notification state
  let copiedPath: string | null = null;
  let copyTimeout: number | null = null;

  onMount(async () => {
    await Promise.all([loadSymlinks(), loadInstallations()]);
  });

  async function loadSymlinks() {
    try {
      loading = true;
      error = null;
      symlinks = await SymlinksAPI.listSymlinks();
    } catch (e) {
      error = `Failed to load symlinks: ${e}`;
      console.error(error);
    } finally {
      loading = false;
    }
  }

  async function loadInstallations() {
    try {
      installations = await InstallationService.getInstallations();
    } catch (e) {
      console.error('Failed to load installations:', e);
    }
  }

  function getInstallationName(installationId: string | null): string {
    if (!installationId) return 'Global';
    const installation = installations.find(i => i.id === installationId);
    return installation?.name || installationId;
  }

  async function handleToggleDisabled(symlink: SymlinkInfo) {
    try {
      await SymlinksAPI.toggleSymlinkDisabled(symlink.destination, symlink.id);
      await loadSymlinks();
    } catch (e) {
      error = `Failed to toggle symlink: ${e}`;
      setTimeout(() => error = null, 5000);
    }
  }

  function startEditingSymlink(symlink: SymlinkInfo) {
    editingSymlink = symlink;
    editSource = symlink.source;
    // Extract parent folder from full destination path
    const destPath = symlink.destination;
    const lastSlash = Math.max(destPath.lastIndexOf('/'), destPath.lastIndexOf('\\'));
    editDestinationParent = lastSlash > 0 ? destPath.substring(0, lastSlash) : destPath;
    editInstallationId = symlink.installation_id;
  }

  function cancelEditing() {
    editingSymlink = null;
    editSource = '';
    editDestinationParent = '';
    editInstallationId = null;
    editError = null;
  }

  async function saveEdit() {
    if (!editingSymlink) return;

    editError = null;
    try {
      await SymlinksAPI.updateSymlink(
        editingSymlink.id,
        editingSymlink.destination,
        editSource,
        editDestinationParent,
        editInstallationId
      );
      cancelEditing();
      await loadSymlinks();
    } catch (e) {
      editError = `Failed to update symlink: ${e}`;
    }
  }

  async function pickSourceFolder() {
    try {
      const path = await SymlinksAPI.selectFolderForSymlink();
      if (path) {
        if (editingSymlink) {
          editSource = path;
        } else {
          newSymlinkSource = path;
        }
      }
    } catch (e) {
      console.error('Failed to select folder:', e);
    }
  }

  async function pickSourceFile() {
    try {
      const path = await SymlinksAPI.selectFileForSymlink();
      if (path) {
        if (editingSymlink) {
          editSource = path;
        } else {
          newSymlinkSource = path;
        }
      }
    } catch (e) {
      console.error('Failed to select file:', e);
    }
  }

  async function pickDestinationFolder() {
    try {
      const path = await SymlinksAPI.selectFolderForSymlink();
      if (path) {
        if (editingSymlink) {
          editDestinationParent = path;
        } else {
          newSymlinkDestinationParent = path;
        }
      }
    } catch (e) {
      console.error('Failed to select folder:', e);
    }
  }

  async function handleCreateSymlink() {
    if (!newSymlinkSource || !newSymlinkDestinationParent) {
      modalError = 'Please fill in both source and destination parent folder paths';
      return;
    }

    modalError = null;
    try {
      await SymlinksAPI.createCustomSymlink(newSymlinkSource, newSymlinkDestinationParent, newSymlinkInstallationId);
      showCreateModal = false;
      newSymlinkSource = '';
      newSymlinkDestinationParent = '';
      newSymlinkInstallationId = null;
      modalError = null;
      await loadSymlinks();
    } catch (e) {
      modalError = `Failed to create symlink: ${e}`;
    }
  }

  function getSymlinkTypeIcon(type: string): string {
    switch (type) {
      case 'resourcepack': return 'image';
      case 'shader': return 'shaders';
      case 'world': return 'map';
      case 'mod': return 'box';
      case 'custom': return 'link';
      default: return 'file';
    }
  }

  function getSymlinkTypeLabel(type: string): string {
    switch (type) {
      case 'resourcepack': return 'Resource Pack';
      case 'shader': return 'Shader';
      case 'world': return 'World/Save';
      case 'mod': return 'Mod';
      case 'custom': return 'Custom';
      default: return type;
    }
  }

  function truncatePath(path: string, maxLength = 40): string {
    if (path.length <= maxLength) return path;
    
    const separator = path.includes('/') ? '/' : '\\';
    const parts = path.split(separator);
    
    // If path is very short, just truncate from the left
    if (parts.length <= 2) {
      return '...' + path.slice(-(maxLength - 3));
    }
    
    const fileName = parts[parts.length - 1];
    const parentFolder = parts[parts.length - 2];
    
    // Prioritize showing the last two parts (parent folder and filename)
    const endPart = parentFolder + separator + fileName;
    
    if (endPart.length >= maxLength - 3) {
      // If even the end is too long, just show from the right
      return '...' + path.slice(-(maxLength - 3));
    }
    
    // Show "...\" + parent folder + filename
    return '...' + separator + endPart;
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copiedPath = text;
      
      if (copyTimeout) clearTimeout(copyTimeout);
      
      copyTimeout = setTimeout(() => {
        copiedPath = null;
        copyTimeout = null;
      }, 2000);
    } catch (e) {
      console.error('Failed to copy to clipboard:', e);
    }
  }

</script>

<div class="advanced-page">
  <header class="page-header">
    <h1>
      <Icon name="settings" />
      Advanced Settings
    </h1>
    <p class="subtitle">Manage advanced features and configurations</p>
  </header>

  <!-- Tab Navigation -->
  <nav class="tab-nav">
    <button
      class="tab-button"
      class:active={activeTab === 'symlinks'}
      on:click={() => activeTab = 'symlinks'}
    >
      <Icon name="link" />
      Symlink Manager
    </button>
    <button
      class="tab-button"
      class:active={activeTab === 'other'}
      on:click={() => activeTab = 'other'}
      disabled
    >
      <Icon name="settings" />
      Other Features (Coming Soon)
    </button>
  </nav>

  <!-- Symlinks Tab Content -->
  {#if activeTab === 'symlinks'}
    <div class="tab-content">
      <div class="section-header">
        <div>
          <h2>Symlink Manager</h2>
          <p>Manage symbolic links for resource packs, shaders, worlds, mods, and custom files</p>
          <p class="hint-text">All symlinks in your .minecraft folder are automatically detected</p>
        </div>
        <button class="btn-primary" on:click={() => showCreateModal = true}>
          <Icon name="plus" forceType="svg" />
          Create Symlink
        </button>
      </div>

      {#if loading}
        <div class="loading-state">
          <Icon name="load" />
          <p>Loading symlinks...</p>
        </div>
      {:else if error}
        <div class="error-state">
          <Icon name="alert" />
          <p>{error}</p>
          <button class="btn-secondary" on:click={loadSymlinks}>Retry</button>
        </div>
      {:else if symlinks.length === 0}
        <div class="empty-state">
          <Icon name="link" size="xl" />
          <p>No symlinks found</p>
          <p class="empty-hint">Symlinks are automatically created when you launch installations with dedicated folders</p>
        </div>
      {:else}
        <div class="symlinks-table">
          <table>
            <thead>
              <tr>
                <th>Status</th>
                <th>Source</th>
                <th>Destination</th>
                <th>Scope</th>
                <th>Installation</th>
                <th>Actions</th>
                <th>Type</th>
              </tr>
            </thead>
            <tbody>
              {#each symlinks as symlink}
                <tr class:disabled={symlink.is_disabled} class:editing={editingSymlink === symlink}>
                  <td>
                    {#if symlink.is_disabled}
                      <span class="status-badge status-disabled">Disabled</span>
                    {:else if symlink.exists}
                      <span class="status-badge status-active">Active</span>
                    {:else}
                      <span class="status-badge status-missing">Missing</span>
                    {/if}
                  </td>
                  <td class="path-cell">
                    {#if editingSymlink === symlink}
                      <div class="path-edit-wrapper">
                        <input
                          type="text"
                          class="path-input"
                          bind:value={editSource}
                          placeholder="Source path"
                        />
                        <button class="btn-icon-small" on:click={pickSourceFolder} title="Pick folder">
                          <Icon name="folder" size="sm" />
                        </button>
                        <button class="btn-icon-small" on:click={pickSourceFile} title="Pick file">
                          <Icon name="file" size="sm" />
                        </button>
                      </div>
                    {:else}
                      <button 
                        class="path-button" 
                        title="Click to copy: {symlink.source}"
                        on:click={() => copyToClipboard(symlink.source)}
                      >
                        {truncatePath(symlink.source)}
                      </button>
                    {/if}
                  </td>
                  <td class="path-cell">
                    {#if editingSymlink === symlink}
                      <div class="path-edit-wrapper">
                        <input
                          type="text"
                          class="path-input"
                          bind:value={editDestinationParent}
                          placeholder="Destination parent folder"
                        />
                        <button class="btn-icon-small" on:click={pickDestinationFolder} title="Pick parent folder">
                          <Icon name="folder" size="sm" />
                        </button>
                      </div>
                    {:else}
                      <button 
                        class="path-button" 
                        title="Click to copy: {symlink.destination}"
                        on:click={() => copyToClipboard(symlink.destination)}
                      >
                        {truncatePath(symlink.destination)}
                      </button>
                    {/if}
                  </td>
                  <td>
                    <span class="badge" class:badge-global={symlink.is_global} class:badge-local={!symlink.is_global}>
                      {symlink.is_global ? 'Global' : 'Installation'}
                    </span>
                  </td>
                  <td>
                    {#if editingSymlink === symlink}
                      <select class="path-input" bind:value={editInstallationId}>
                        <option value={null}>Global (Always Active)</option>
                        {#each installations as installation}
                          <option value={installation.id}>{installation.name}</option>
                        {/each}
                      </select>
                    {:else if symlink.is_global}
                      <span class="muted">—</span>
                    {:else}
                      <span class="installation-name">
                        {getInstallationName(symlink.installation_id)}
                      </span>
                    {/if}
                  </td>
                  <td>
                    {#if editingSymlink === symlink && editError}
                      <div class="inline-error" title={editError}>
                        <Icon name="alert" size="sm" />
                      </div>
                    {/if}
                    <div class="actions">
                      {#if editingSymlink === symlink}
                        <button
                          class="btn-icon btn-success"
                          title="Save changes"
                          on:click={saveEdit}
                        >
                          <Icon name="check" forceType="svg" />
                        </button>
                        <button
                          class="btn-icon"
                          title="Cancel"
                          on:click={cancelEditing}
                        >
                          <Icon name="x" forceType="svg" />
                        </button>
                      {:else}
                        <button
                          class="btn-icon"
                          title="Edit paths"
                          on:click={() => startEditingSymlink(symlink)}
                        >
                          <Icon name="edit" forceType="svg" />
                        </button>
                        <button
                          class="btn-icon"
                          title={symlink.is_disabled ? 'Enable symlink' : 'Disable symlink'}
                          on:click={() => handleToggleDisabled(symlink)}
                        >
                          <Icon name={symlink.is_disabled ? 'eye' : 'eye-off'} forceType="svg" />
                        </button>
                      {/if}
                    </div>
                  </td>
                  <td>
                    <div class="type-cell" title={getSymlinkTypeLabel(symlink.symlink_type)}>
                      <Icon name={getSymlinkTypeIcon(symlink.symlink_type)} />
                    </div>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  {/if}
</div>

<!-- Copy Notification -->
{#if copiedPath}
  <div class="copy-notification">
    <Icon name="check" size="sm" />
    <span>Path copied to clipboard</span>
  </div>
{/if}

<!-- Create Symlink Modal -->
{#if showCreateModal}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-overlay" on:click={() => showCreateModal = false}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal" on:click|stopPropagation>
      <div class="modal-header">
        <h3>Create Custom Symlink</h3>
        <button class="btn-icon" on:click={() => { showCreateModal = false; modalError = null; }}>
          <Icon name="x" />
        </button>
      </div>
      {#if modalError}
        <div class="error-banner">
          <Icon name="alert" size="sm" />
          <span>{modalError}</span>
        </div>
      {/if}
      <div class="modal-body">
        <div class="form-group">
          <label for="source">Source Path</label>
          <div class="input-with-buttons">
            <input
              id="source"
              type="text"
              bind:value={newSymlinkSource}
              placeholder="C:\Path\to\source"
            />
            <button type="button" class="btn-icon" on:click={pickSourceFolder} title="Select folder">
              <Icon name="folder" size="sm" />
            </button>
            <button type="button" class="btn-icon" on:click={pickSourceFile} title="Select file">
              <Icon name="file" size="sm" />
            </button>
          </div>
          <p class="hint">The file or directory to link from</p>
        </div>
        <div class="form-group">
          <label for="destination">Destination Parent Folder</label>
          <div class="input-with-buttons">
            <input
              id="destination"
              type="text"
              bind:value={newSymlinkDestinationParent}
              placeholder="C:\Users\YourName\AppData\Roaming\.minecraft\resourcepacks"
            />
            <button type="button" class="btn-icon" on:click={pickDestinationFolder} title="Select parent folder">
              <Icon name="folder" size="sm" />
            </button>
          </div>
          <p class="hint">The parent folder where the symlink will be created (symlink will have the same name as source)</p>
        </div>
        <div class="form-group">
          <label for="installation">Installation (Optional)</label>
          <select
            id="installation"
            bind:value={newSymlinkInstallationId}
          >
            <option value={null}>Global (Always Active)</option>
            {#each installations as installation}
              <option value={installation.id}>{installation.name}</option>
            {/each}
          </select>
          <p class="hint">Choose which installation this symlink is active for, or leave as Global to always apply</p>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn-secondary" on:click={() => { showCreateModal = false; modalError = null; }}>Cancel</button>
        <button class="btn-primary" on:click={handleCreateSymlink}>Create</button>
      </div>
    </div>
  </div>
{/if}

<style lang="scss">
  .advanced-page {
    width: 100%;
    margin: 0;
    padding: 0;
    height: 100%;
    overflow-x: clip;
  }

  .page-header {
    margin-bottom: 0.5rem;
    padding: 0 2vw;

    h1 {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      font-size: 2rem;
      margin: 0 0 0.5rem 0;
    }

    .subtitle {
      color: var(--text-secondary);
      margin: 0;
    }
  }

  .tab-nav {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
    padding: 0;
    border-bottom: 1px solid color-mix(in srgb, var(--primary), 8%, transparent);
  }

  .tab-button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 1.2rem;
    border: 1px solid var(--dark-600);
    border-radius: 0.5rem;
    background: var(--card);
    color: var(--text);
    font-weight: 500;
    font-size: 0.9em;
    cursor: pointer;
    transition: all 0.15s;

    &:hover:not(:disabled) {
      border-color: var(--primary);
      background: color-mix(in srgb, var(--primary), 5%, transparent);
    }

    &.active {
      background: linear-gradient(135deg, var(--primary) 0%, var(--secondary) 100%);
      color: var(--text-white);
      border-color: var(--text-transparent);
      box-shadow: 0 2px 8px color-mix(in srgb, var(--primary), 25%, transparent);
    }

    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
  }

  .tab-content {
    padding: 0;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1.5rem;
    padding: 1rem;
    background: var(--container);
    border-radius: var(--border-radius);

    h2 {
      margin: 0 0 0.25rem 0;
      font-size: 1.5rem;
    }

    p {
      color: var(--text-secondary);
      margin: 0;
      font-size: 0.9rem;
    }

    .hint-text {
      margin-top: 0.25rem;
      font-size: 0.8rem;
      font-style: italic;
      opacity: 0.8;
    }
  }

  .loading-state,
  .error-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem 1rem;
    text-align: center;
    color: var(--text-secondary);
    background: var(--container);
    border-radius: var(--border-radius);

    p {
      margin: 1rem 0 0 0;
    }
  }

  .empty-hint {
    font-size: 0.9rem;
    max-width: 500px;
  }

  .symlinks-table {
    max-width: 100%;
    overflow-x: scroll;
    background: var(--container);
    border-radius: var(--border-radius);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);

    table {
      width: 100%;
      border-collapse: collapse;
    }

    thead {
      background: var(--dark-700);

      th {
        padding: 0.5rem 0.25rem;
        text-align: left;
        font-weight: 600;
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--placeholder);
        white-space: nowrap;

        &:first-child {
          border-top-left-radius: var(--border-radius);
        }

        &:last-child {
          border-top-right-radius: var(--border-radius);
        }
      }
    }

    tbody {
      tr {
        border-bottom: 1px solid var(--dark-600);
        transition: background 0.2s;

        &:hover {
          background: var(--dark-700);
        }

        &.disabled {
          opacity: 0.6;
        }

        &:last-child td {
          &:first-child {
            border-bottom-left-radius: var(--border-radius);
          }

          &:last-child {
            border-bottom-right-radius: var(--border-radius);
          }
        }
      }

      td {
        padding: 0.5rem 0.25rem;
        vertical-align: middle;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;

        // Fit-content columns for short data
        &:nth-child(1),  // Status
        &:nth-child(4),  // Scope
        &:nth-child(5),  // Installation
        &:nth-child(6),  // Actions
        &:nth-child(7) { // Type
          width: fit-content;
          white-space: nowrap;
        }
      }
    }

  }

  .type-cell {
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    cursor: help;
  }

  .path-cell {
    font-family: 'Courier New', monospace;
    font-size: 0.8rem;
    max-width: 100%;
    overflow: hidden;
  }

  .path-button {
    background: transparent;
    border: none;
    color: var(--text);
    font-family: 'Courier New', monospace;
    font-size: 0.8rem;
    padding: 0.25rem 0.5rem;
    cursor: pointer;
    text-align: left;
    width: 100%;
    border-radius: 4px;
    transition: all 0.2s;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;

    &:hover {
      background: var(--dark-600);
      color: var(--primary);
    }

    &:active {
      transform: scale(0.98);
    }
  }

  .path-edit-wrapper {
    display: flex;
    gap: 0.25rem;
    align-items: center;
  }

  .path-input {
    flex: 1;
    min-width: 0;
    padding: 0.4rem 0.6rem;
    background: var(--dark-600);
    border: 1px solid var(--dark-500);
    border-radius: 4px;
    color: var(--text);
    font-family: 'Courier New', monospace;
    font-size: 0.8rem;

    &:focus {
      outline: none;
      border-color: var(--primary);
    }
  }

  .btn-icon-small {
    padding: 0.3rem;
    background: var(--dark-600);
    border: 1px solid var(--dark-500);
    border-radius: 4px;
    color: var(--text);
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;

    &:hover {
      background: var(--dark-500);
      border-color: var(--primary);
    }
  }

  tr.editing {
    background: color-mix(in srgb, var(--primary), 5%, transparent);
  }

  .badge {
    display: inline-block;
    padding: 0.2rem 0.6rem;
    border-radius: 1rem;
    font-size: 0.7rem;
    font-weight: 600;

    &.badge-global {
      background: var(--primary);
      color: white;
    }

    &.badge-local {
      background: var(--dark-600);
      color: var(--text);
    }
  }

  .installation-name {
    font-weight: 500;
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.85rem;
  }

  .status-badge {
    display: inline-block;
    padding: 0.2rem 0.6rem;
    border-radius: 0.25rem;
    font-size: 0.7rem;
    font-weight: 600;

    &.status-active {
      background: rgba(34, 197, 94, 0.2);
      color: #22c55e;
    }

    &.status-disabled {
      background: rgba(251, 191, 36, 0.2);
      color: #fbbf24;
    }

    &.status-missing {
      background: rgba(239, 68, 68, 0.2);
      color: #ef4444;
    }
  }

  .actions {
    display: flex;
    gap: 0.35rem;
  }

  .btn-icon {
    padding: 0.4rem;
    background: var(--dark-600);
    border: none;
    border-radius: var(--border-radius);
    color: var(--text);
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;

    &:hover {
      background: var(--dark-500);
    }

    &.btn-success {
      background: rgba(34, 197, 94, 0.2);
      color: #22c55e;

      &:hover {
        background: rgba(34, 197, 94, 0.3);
      }
    }
  }

  .btn-primary,
  .btn-secondary {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 1.2rem;
    border: none;
    border-radius: var(--border-radius);
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-primary {
    background: linear-gradient(135deg, var(--primary) 0%, var(--secondary) 100%);
    color: white;

    &:hover {
      transform: translateY(-1px);
      box-shadow: 0 4px 12px color-mix(in srgb, var(--primary), 30%, transparent);
    }
  }

  .btn-secondary {
    background: var(--dark-600);
    color: var(--text);
    border: 1px solid var(--dark-500);

    &:hover {
      background: var(--dark-500);
      border-color: var(--primary);
    }
  }

  .muted {
    color: var(--text-secondary);
  }

  /* Modal Styles */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: var(--container);
    border-radius: var(--border-radius);
    width: 90%;
    max-width: 550px;
    max-height: 85vh;
    overflow: auto;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.25rem 1.5rem;
    border-bottom: 1px solid var(--dark-600);

    h3 {
      margin: 0;
      font-size: 1.25rem;
    }
  }

  .modal-body {
    padding: 1.5rem;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    padding: 1.25rem 1.5rem;
    border-top: 1px solid var(--dark-600);
  }

  .form-group {
    margin-bottom: 1.25rem;

    &:last-child {
      margin-bottom: 0;
    }

    label {
      display: block;
      margin-bottom: 0.5rem;
      font-weight: 600;
      font-size: 0.9rem;
    }

    input,
    select {
      width: 100%;
      padding: 0.7rem 0.9rem;
      background: var(--dark-700);
      border: 1px solid var(--dark-600);
      border-radius: var(--border-radius);
      color: var(--text);
      font-size: 0.9rem;
      transition: border-color 0.2s;

      &:focus {
        outline: none;
        border-color: var(--primary);
        box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary), 10%, transparent);
      }
    }

    input {
      font-family: 'Courier New', monospace;
    }

    select {
      cursor: pointer;
    }

    .hint {
      margin-top: 0.4rem;
      font-size: 0.8rem;
      color: var(--text-secondary);
    }
  }

  .input-with-buttons {
    display: flex;
    gap: 0.5rem;
    align-items: center;

    input {
      flex: 1;
      min-width: 0;
    }

    .btn-icon {
      flex-shrink: 0;
      padding: 0.7rem;
    }
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1.5rem;
    background: rgba(239, 68, 68, 0.1);
    border-bottom: 1px solid rgba(239, 68, 68, 0.3);
    color: #ef4444;
    font-size: 0.9rem;

    span {
      flex: 1;
    }
  }

  .inline-error {
    display: inline-flex;
    align-items: center;
    color: #ef4444;
    margin-right: 0.5rem;
    animation: shake 0.5s;
  }

  @keyframes shake {
    0%, 100% { transform: translateX(0); }
    25% { transform: translateX(-5px); }
    75% { transform: translateX(5px); }
  }

  .copy-notification {
    position: fixed;
    bottom: 2rem;
    right: 2rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.25rem;
    background: rgba(34, 197, 94, 0.95);
    color: white;
    border-radius: var(--border-radius);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    font-weight: 600;
    font-size: 0.9rem;
    z-index: 1001;
    animation: slideIn 0.2s ease-out, slideOut 0.2s ease-in 1.8s;

    @keyframes slideIn {
      from {
        transform: translateY(100px);
        opacity: 0;
      }
      to {
        transform: translateY(0);
        opacity: 1;
      }
    }

    @keyframes slideOut {
      from {
        transform: translateY(0);
        opacity: 1;
      }
      to {
        transform: translateY(100px);
        opacity: 0;
      }
    }
  }
</style>
