<script lang="ts">
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { 
    Icon, ModDetectionService, InstallationManager, type InstallationForm, InstallationsList,
    type MinecraftInstallation, type MinecraftVersion, type ModDetectionResult 
  } from '$lib';

  // State variables
  let installations: MinecraftInstallation[] = [];
  let availableVersions: MinecraftVersion[] = [];
  let isLoading = false;
  let error: string | null = null;
  let showCreateModal = false;
  let showEditModal = false;
  let editingInstallation: MinecraftInstallation | null = null;
  
  // Mod detection results
  let modDetectionResults: Map<string, ModDetectionResult> = new Map();
  
  // New installation form
  let newInstallation: InstallationForm = InstallationManager.getEmptyInstallationForm();

  onMount(async () => {
    await loadInstallations();
    await loadAvailableVersions();
  });

  async function loadInstallations() {
    try {
      isLoading = true;
      error = null;
      installations = await InstallationManager.getInstallations();
      await analyzeAllInstallations();
    } catch (err) {
      console.error('Failed to load installations:', err);
      error = `Failed to load installations: ${err}`;
    } finally {
      isLoading = false;
    }
  }

  async function analyzeAllInstallations() {
    const detectionPromises = installations.map(async (installation) => {
      try {
        const detection = await InstallationManager.analyzeInstallation(installation);
        modDetectionResults.set(installation.id, detection);
      } catch (err) {
        console.error(`Failed to analyze installation ${installation.name}:`, err);
      }
    });
    await Promise.all(detectionPromises);
    modDetectionResults = new Map(modDetectionResults);
  }

  async function loadAvailableVersions() {
    try {
      availableVersions = await InstallationManager.getMinecraftVersions();
    } catch (err) {
      console.error('Failed to load Minecraft versions:', err);
      availableVersions = InstallationManager.getFallbackVersions();
    }
  }

  async function createInstallation() {
    try {
      isLoading = true;
      error = null;
      const installation = await InstallationManager.createInstallation(newInstallation);
      installations = [...installations, installation];
      try {
        const detection = await InstallationManager.analyzeInstallation(installation);
        modDetectionResults.set(installation.id, detection);
        modDetectionResults = new Map(modDetectionResults);
      } catch (err) {
        console.error('Failed to analyze new installation:', err);
      }
      showCreateModal = false;
      newInstallation = InstallationManager.getEmptyInstallationForm();
    } catch (err) {
      console.error('Failed to create installation:', err);
      error = `Failed to create installation: ${err}`;
    } finally {
      isLoading = false;
    }
  }

  async function deleteInstallation(installationId: string) {
    if (!confirm('Are you sure you want to delete this installation? This action cannot be undone.')) {
      return;
    }
    try {
      await InstallationManager.deleteInstallation(installationId);
      installations = installations.filter(inst => inst.id !== installationId);
    } catch (err) {
      console.error('Failed to delete installation:', err);
      error = `Failed to delete installation: ${err}`;
    }
  }

  async function launchInstallation(installation: MinecraftInstallation) {
    try {
      isLoading = true;
      error = null;
      const canLaunch = await InstallationManager.canLaunch(installation);
      if (!canLaunch.canLaunch) {
        error = canLaunch.reason || 'Cannot launch';
        return;
      }
      await InstallationManager.launchInstallation(installation);
      setTimeout(() => {
        loadInstallations();
      }, 1000);
    } catch (err) {
      console.error('Failed to launch Minecraft:', err);
      error = `Failed to launch Minecraft: ${err}`;
    } finally {
      isLoading = false;
    }
  }

  async function openInstallationFolder(installation: MinecraftInstallation) {
    try {
      await InstallationManager.openInstallationFolder(installation.id);
    } catch (err) {
      console.error('Failed to open installation folder:', err);
      error = `Failed to open installation folder: ${err}`;
    }
  }

  function getModLoaderIcon(installation: MinecraftInstallation) {
    const detection = modDetectionResults.get(installation.id);
    if (detection) {
      return ModDetectionService.getModLoaderIcon(detection.modLoaderType);
    }
    return ModDetectionService.getModLoaderIcon(installation.mod_loader);
  }

  function getModLoaderDisplay(installation: MinecraftInstallation): string {
    const detection = modDetectionResults.get(installation.id);
    if (detection) {
      return ModDetectionService.getModdingStatusDescription(detection);
    }
    
    // Fallback to basic display
    if (installation.mod_loader === 'vanilla') {
      return 'Vanilla Minecraft';
    }
    
    let display = installation.mod_loader.charAt(0).toUpperCase() + installation.mod_loader.slice(1);
    if (installation.loader_version) {
      display += ` ${installation.loader_version}`;
    }
    return display;
  }

  function getModLoaderColor(installation: MinecraftInstallation): string {
    const detection = modDetectionResults.get(installation.id);
    if (detection) {
      return ModDetectionService.getModLoaderColor(detection.modLoaderType);
    }
    return ModDetectionService.getModLoaderColor(installation.mod_loader);
  }

  function getVersionTypeColor(type: string) {
    switch (type) {
      case 'release': return '#28a745';
      case 'snapshot': return '#ffc107';
      case 'old_beta': return '#6c757d';
      case 'old_alpha': return '#6c757d';
      default: return '#6c757d';
    }
  }

  function editInstallation(installation: MinecraftInstallation) {
    editingInstallation = installation;
    // Pre-fill the form with existing installation data
    newInstallation = {
      name: installation.name,
      version: installation.version,
      mod_loader: installation.mod_loader,
      game_directory: installation.game_directory || '',
      java_path: installation.java_path || '',
      jvm_args: installation.jvm_args || '-Xmx2G',
      memory: installation.memory || 2048,
      description: installation.description || ''
    };
    showEditModal = true;
  }

  async function updateInstallation() {
    if (!editingInstallation) return;
    try {
      isLoading = true;
      error = null;
      const updatedInstallation = await InstallationManager.updateInstallation(editingInstallation.id, newInstallation);
      installations = installations.map(inst => 
        inst.id === editingInstallation?.id ? updatedInstallation : inst
      );
      try {
        const detection = await ModDetectionService.analyzeInstallation(updatedInstallation);
        modDetectionResults.set(updatedInstallation.id, detection);
        modDetectionResults = new Map(modDetectionResults);
      } catch (err) {
        console.error('Failed to analyze updated installation:', err);
      }
      showEditModal = false;
      editingInstallation = null;
      newInstallation = InstallationManager.getEmptyInstallationForm();
    } catch (err) {
      console.error('Failed to update installation:', err);
      error = `Failed to update installation: ${err}`;
    } finally {
      isLoading = false;
    }
  }

  function closeEditModal() {
    showEditModal = false;
    editingInstallation = null;
    const modal = document.querySelector('.modern-modal') as HTMLDialogElement;
    if (modal && modal.close) {
      modal.close();
    }
  }

  function openEditModal() {
    const modal = document.querySelector('.modern-modal') as HTMLDialogElement;
    if (modal && modal.showModal) {
      modal.showModal();
    }
  }

  // Watch for showEditModal changes to control the dialog
  $: if (showEditModal) {
    setTimeout(() => openEditModal(), 10);
  }
</script>

<div class="installations-page">
  <div class="page-header">
    <div class="header-content">
      <h1>Installations</h1>
      <p>Manage your Minecraft installations, versions, and mod loaders</p>
    </div>
    <button 
      class="btn btn-primary" 
      on:click={() => showCreateModal = true}
      disabled={isLoading}
    >
      <Icon name="plus" size="sm" />
      New Installation
    </button>
  </div>

  {#if error}
    <div class="error-message">
      <Icon name="alert" size="sm" />
      {error}
    </div>
  {/if}

  <!-- {#if isLoading && installations.length === 0}
    <div class="loading-state">
      <Icon name="refresh" size="md" />
      <span>Loading installations...</span>
    </div>
  {:else if installations.length === 0}
    <div class="empty-state">
      <div class="empty-icon">
        <Icon name="cube" size="xl" />
      </div>
      <h3>No installations found</h3>
      <p>Create your first Minecraft installation to get started</p>
      <button 
        class="btn btn-primary" 
        on:click={() => showCreateModal = true}
      >
        <Icon name="plus" size="sm" />
        Create Installation
      </button>
    </div>
  {:else}
    <div class="installations-grid">
      {#each installations as installation}
        <div class="installation-card">
          <div class="installation-header">
            <div class="installation-icon" style="background-color: {getModLoaderColor(installation)}20; color: {getModLoaderColor(installation)};">
              <Icon name={getModLoaderIcon(installation)} size="lg" />
            </div>
            <div class="installation-info">
              <h3>{installation.name}</h3>
              <div class="installation-details">
                <span class="version">{installation.version}</span>
                <span class="mod-loader" style="color: {getModLoaderColor(installation)};">{getModLoaderDisplay(installation)}</span>
              </div>
              {#if installation.description}
                <p class="description">{installation.description}</p>
              {/if}
            </div>
          </div>

          <div class="installation-stats">
            <div class="stat">
              <Icon name="clock" size="sm" />
              <span>Last played: {installation.last_played ? new Date(installation.last_played).toLocaleDateString() : 'Never'}</span>
            </div>
            <div class="stat">
              <Icon name="folder" size="sm" />
              <span>Game directory: {installation.game_directory || 'Default'}</span>
            </div>
          </div>

          <div class="installation-actions">
            <button 
              class="btn btn-primary" 
              on:click={() => launchInstallation(installation)}
              disabled={isLoading}
            >
              <Icon name="play" size="sm" />
              Play
            </button>
            
            <button 
              class="btn btn-secondary" 
              on:click={() => openInstallationFolder(installation)}
            >
              <Icon name="folder-open" size="sm" />
              Open Folder
            </button>
            
            <div class="dropdown">
              <button class="btn btn-secondary dropdown-toggle">
                <Icon name="more-horizontal" size="sm" />
              </button>
              <div class="dropdown-menu">
                <button on:click={() => editInstallation(installation)}>
                  <Icon name="edit" size="sm" />
                  Edit
                </button>
                <button on:click={() => console.log('Duplicate installation')}>
                  <Icon name="duplicate" size="sm" />
                  Duplicate
                </button>
                <div class="dropdown-separator"></div>
                <button 
                  class="danger" 
                  on:click={() => deleteInstallation(installation.id)}
                >
                  <Icon name="trash" size="sm" />
                  Delete
                </button>
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if} -->
  <InstallationsList />
</div>

<!-- Create Installation Modal -->
{#if showCreateModal}
  <div 
    class="modal-overlay" 
    on:click={() => showCreateModal = false}
    on:keydown={(e) => e.key === 'Escape' && (showCreateModal = false)}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
    <div 
      class="modal" 
      on:click|stopPropagation
      on:keydown={() => {}}
      role="document"
    >
      <div class="modal-header">
        <h2>Create New Installation</h2>
        <button class="modal-close" on:click={() => showCreateModal = false}>
          <Icon name="x" size="sm" />
        </button>
      </div>

      <form on:submit|preventDefault={createInstallation} class="modal-content">
        <div class="form-group">
          <label for="installation-name">Installation Name</label>
          <input
            id="installation-name"
            type="text"
            bind:value={newInstallation.name}
            placeholder="My Minecraft Installation"
            required
          />
        </div>

        <div class="form-group">
          <label for="minecraft-version">Minecraft Version</label>
          <select id="minecraft-version" bind:value={newInstallation.version} required>
            <option value="">Select a version</option>
            {#each availableVersions as version}
              <option value={version.id} style="color: {getVersionTypeColor(version.type)}">
                {version.id} ({version.type})
              </option>
            {/each}
          </select>
        </div>

        <div class="form-group">
          <label for="mod-loader">Mod Loader</label>
          <select id="mod-loader" bind:value={newInstallation.mod_loader}>
            <option value="vanilla">Vanilla (No mods)</option>
            <option value="fabric">Fabric</option>
            <option value="forge">Forge</option>
          </select>
        </div>

        <div class="form-group">
          <label for="game-directory">Game Directory</label>
          <input
            id="game-directory"
            type="text"
            bind:value={newInstallation.game_directory}
            placeholder="Leave empty for default (.minecraft)"
          />
          <small>Custom game directory for this installation</small>
        </div>

        <div class="form-group">
          <label for="java-path">Java Path</label>
          <input
            id="java-path"
            type="text"
            bind:value={newInstallation.java_path}
            placeholder="Leave empty for auto-detection"
          />
          <small>Path to Java executable (java.exe)</small>
        </div>

        <div class="form-group">
          <label for="jvm-args">JVM Arguments</label>
          <input
            id="jvm-args"
            type="text"
            bind:value={newInstallation.jvm_args}
            placeholder="-XX:+UseG1GC"
          />
          <small>Additional Java Virtual Machine arguments (memory is set separately below)</small>
        </div>

        <div class="form-group">
          <label for="memory">Memory Allocation</label>
          <div class="memory-control">
            <input 
              id="memory"
              type="range" 
              min="512" 
              max="16384" 
              step="256"
              bind:value={newInstallation.memory}
              class="memory-slider"
            />
            <div class="memory-input-row">
              <input 
                type="number"
                bind:value={newInstallation.memory}
                min="512"
                max="16384"
                step="256"
                class="memory-text-input"
              />
              <span class="memory-unit">MB</span>
              <span class="memory-gb">({Math.round((newInstallation?.memory || 0) / 1024 * 10) / 10}GB)</span>
            </div>
          </div>
          <small>RAM allocated to this installation (overrides global memory setting)</small>
        </div>

        <div class="form-group">
          <label for="description">Description</label>
          <textarea
            id="description"
            bind:value={newInstallation.description}
            placeholder="Optional description for this installation"
            rows="3"
          ></textarea>
        </div>

        <div class="modal-actions">
          <button type="button" class="btn btn-secondary" on:click={() => showCreateModal = false}>
            Cancel
          </button>
          <button type="submit" class="btn btn-primary" disabled={isLoading || !newInstallation.name || !newInstallation.version}>
            {isLoading ? 'Creating...' : 'Create Installation'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- Edit Installation Modal -->
{#if showEditModal}
  <dialog class="modern-modal" open>
    <div class="modal-header">
      <h2>Edit Installation</h2>
      <button class="modal-close" on:click={() => { showEditModal = false; editingInstallation = null; }}>
        <Icon name="x" size="sm" />
      </button>
    </div>

    <form on:submit|preventDefault={updateInstallation} class="modal-content">
      <div class="form-group">
        <label for="edit-installation-name">Installation Name</label>
        <input
          id="edit-installation-name"
          type="text"
          bind:value={newInstallation.name}
          placeholder="My Minecraft Installation"
          required
        />
      </div>

      <div class="form-group">
        <label for="edit-minecraft-version">Minecraft Version</label>
        <select id="edit-minecraft-version" bind:value={newInstallation.version} required>
          <option value="">Select a version</option>
          {#each availableVersions as version}
            <option value={version.id} style="color: {getVersionTypeColor(version.type)}">
              {version.id} ({version.type})
            </option>
          {/each}
        </select>
      </div>

      <div class="form-group">
        <label for="edit-mod-loader">Mod Loader</label>
        <select id="edit-mod-loader" bind:value={newInstallation.mod_loader}>
          <option value="vanilla">Vanilla (No mods)</option>
          <option value="fabric">Fabric</option>
          <option value="forge">Forge</option>
        </select>
      </div>

      <div class="form-group">
        <label for="edit-game-directory">Game Directory</label>
        <input
          id="edit-game-directory"
          type="text"
          bind:value={newInstallation.game_directory}
          placeholder="Leave empty for default (.minecraft)"
        />
        <small>Custom game directory for this installation</small>
      </div>

      <div class="form-group">
        <label for="edit-java-path">Java Path</label>
        <input
          id="edit-java-path"
          type="text"
          bind:value={newInstallation.java_path}
          placeholder="Leave empty for auto-detection"
        />
        <small>Path to Java executable (java.exe)</small>
      </div>

      <div class="form-group">
        <label for="edit-jvm-args">JVM Arguments</label>
        <input
          id="edit-jvm-args"
          type="text"
          bind:value={newInstallation.jvm_args}
          placeholder="-XX:+UseG1GC"
        />
        <small>Additional Java Virtual Machine arguments (memory is set separately below)</small>
      </div>

      <div class="form-group">
        <label for="edit-memory">Memory Allocation</label>
        <div class="memory-control">
          <input 
            id="edit-memory"
            type="range" 
            min="512" 
            max="16384" 
            step="256"
            bind:value={newInstallation.memory}
            class="memory-slider"
          />
          <div class="memory-input-row">
            <input 
              type="number"
              bind:value={newInstallation.memory}
              min="512"
              max="16384"
              step="256"
              class="memory-text-input"
            />
            <span class="memory-unit">MB</span>
            <span class="memory-gb">({Math.round((newInstallation.memory || 0) / 1024 * 10) / 10}GB)</span>
          </div>
        </div>
        <small>RAM allocated to this installation (overrides global memory setting)</small>
      </div>

      <div class="form-group">
        <label for="edit-description">Description</label>
        <textarea
          id="edit-description"
          bind:value={newInstallation.description}
          placeholder="Optional description for this installation"
          rows="3"
        ></textarea>
      </div>

      <div class="modal-actions">
        <button type="button" class="btn btn-secondary" on:click={() => { showEditModal = false; editingInstallation = null; }}>
          Cancel
        </button>
        <button type="submit" class="btn btn-primary" disabled={isLoading || !newInstallation.name || !newInstallation.version}>
          {isLoading ? 'Updating...' : 'Update Installation'}
        </button>
      </div>
    </form>
  </dialog>
{/if}

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;

  .installations-page {
    max-width: 1200px;
    margin: 0 auto;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 2rem;

    .header-content {
      h1 {
        margin: 0 0 0.5rem;
        font-size: 2rem;
        font-weight: 700;
        color: $text;
      }

      p {
        margin: 0;
        color: $placeholder;
        font-size: 1rem;
      }
    }
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem;
    background: rgba($red, 0.1);
    border: 1px solid $red;
    border-radius: $border-radius;
    color: $red;
    margin-bottom: 1rem;
  }

  // Modal styles
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
    background: $container;
    border: 1px solid $dark-600;
    border-radius: $border-radius;
    width: 100%;
    max-width: 600px;
    max-height: 90vh;
    overflow-y: auto;
    margin: 1rem;

    .modal-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 1.5rem;
      border-bottom: 1px solid $dark-600;

      h2 {
        margin: 0;
        font-size: 1.5rem;
        font-weight: 600;
        color: $text;
      }

      .modal-close {
        background: none;
        border: none;
        padding: 0.5rem;
        color: $placeholder;
        cursor: pointer;
        border-radius: 4px;
        transition: all 0.2s ease;

        &:hover {
          background: rgba($placeholder, 0.1);
          color: $text;
        }
      }
    }

    .modal-content {
      padding: 1.5rem;

      .form-group {
        margin-bottom: 1.5rem;

        label {
          display: block;
          margin-bottom: 0.5rem;
          font-weight: 500;
          color: $text;
        }

        input, select, textarea {
          width: 100%;
          padding: 0.75rem;
          border: 1px solid $dark-600;
          border-radius: $border-radius;
          background: $background;
          color: $text;
          font-size: 0.875rem;

          &:focus {
            outline: none;
            border-color: $primary;
          }

          &::placeholder {
            color: $placeholder;
          }
        }

        select {
          cursor: pointer;
        }

        small {
          display: block;
          margin-top: 0.25rem;
          font-size: 0.75rem;
          color: $placeholder;
        }
      }

      .memory-control {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        
        .memory-slider {
          width: 100%;
        }
        
        .memory-input-row {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          
          .memory-text-input {
            background: transparent;
            border: 1px solid $dark-600;
            color: $primary;
            font-weight: 600;
            font-size: 0.9rem;
            text-align: right;
            min-width: 60px;
            max-width: 80px;
            padding: 0.5rem;
            border-radius: 4px;
            transition: all 0.2s ease;
            
            &:hover {
              border-color: $primary;
              color: rgba($primary, 0.8);
            }
            
            &:focus {
              border-color: $primary;
              color: rgba($primary, 0.75);
            }
          }
          
          .memory-gb {
            color: $placeholder;
            font-size: 0.875rem;
            min-width: 60px;
          }
          
          .memory-unit {
            color: $primary;
            font-weight: 600;
            font-size: 0.9rem;
          }
        }
      }

      .modal-actions {
        display: flex;
        gap: 1rem;
        justify-content: flex-end;
        margin-top: 2rem;
      }
    }
  }

  // Modern dialog element styles
  .modern-modal {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background: $container;
    border: 1px solid $dark-600;
    border-radius: $border-radius;
    width: 100%;
    max-width: 600px;
    max-height: 90vh;
    overflow-y: auto;
    margin: 0;
    padding: 0;
    z-index: 9999;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);

    &::backdrop {
      background: rgba(0, 0, 0, 0.7);
    }

    .modal-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 1.5rem;
      border-bottom: 1px solid $dark-600;

      h2 {
        margin: 0;
        font-size: 1.5rem;
        font-weight: 600;
        color: $text;
      }

      .modal-close {
        background: none;
        border: none;
        padding: 0.5rem;
        color: $placeholder;
        cursor: pointer;
        border-radius: 4px;
        transition: all 0.2s ease;

        &:hover {
          background: rgba($placeholder, 0.1);
          color: $text;
        }
      }
    }

    .modal-content {
      padding: 1.5rem;

      .form-group {
        margin-bottom: 1.5rem;

        label {
          display: block;
          margin-bottom: 0.5rem;
          font-weight: 500;
          color: $text;
        }

        input, select, textarea {
          width: 100%;
          padding: 0.75rem;
          border: 1px solid $dark-600;
          border-radius: $border-radius;
          background: $background;
          color: $text;
          font-size: 0.875rem;

          &:focus {
            outline: none;
            border-color: $primary;
          }

          &::placeholder {
            color: $placeholder;
          }
        }

        select {
          cursor: pointer;
        }

        small {
          display: block;
          margin-top: 0.25rem;
          font-size: 0.75rem;
          color: $placeholder;
        }
      }

      .memory-control {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        
        .memory-slider {
          width: 100%;
        }
        
        .memory-input-row {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          
          .memory-text-input {
            background: transparent;
            border: 1px solid $dark-600;
            color: $primary;
            font-weight: 600;
            font-size: 0.9rem;
            text-align: right;
            min-width: 60px;
            max-width: 80px;
            padding: 0.5rem;
            border-radius: 4px;
            transition: all 0.2s ease;
            
            &:hover {
              border-color: $primary;
              color: rgba($primary, 0.8);
            }
            
            &:focus {
              border-color: $primary;
              color: rgba($primary, 0.75);
            }
          }
          
          .memory-gb {
            color: $placeholder;
            font-size: 0.875rem;
            min-width: 60px;
          }
          
          .memory-unit {
            color: $primary;
            font-weight: 600;
            font-size: 0.9rem;
          }
        }
      }

      .modal-actions {
        display: flex;
        gap: 1rem;
        justify-content: flex-end;
        margin-top: 2rem;
      }
    }
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  @media (max-width: 768px) {
    .page-header {
      flex-direction: column;
      gap: 1rem;
      align-items: stretch;
    }

    .modal {
      margin: 0.5rem;
      max-height: 95vh;

      .modal-content {
        padding: 1rem;

        .modal-actions {
          flex-direction: column-reverse;
        }
      }
    }
  }
</style>
