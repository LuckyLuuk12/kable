<script lang="ts">
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { 
    Icon, InstallationService, ModDetectionService, GameManager, 
    type MinecraftInstallation, type MinecraftVersion, type ModDetectionResult 
  } from '$lib';
  import { installations as gameInstallations } from '$lib/stores/game';

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
  let newInstallation = {
    name: '',
    version: '',
    mod_loader: 'vanilla' as 'vanilla' | 'fabric' | 'forge' | 'quilt' | 'neoforge',
    game_directory: '',
    java_path: '',
    jvm_args: '-Xmx2G',
    memory: 2048, // Memory in MB
    description: ''
  };

  onMount(async () => {
    await loadInstallations();
    await loadAvailableVersions();
  });

  async function loadInstallations() {
    try {
      isLoading = true;
      error = null;
      
      // Get installations from GameManager's store (already loaded in layout)
      installations = get(gameInstallations);
      
      // Run mod detection on each installation
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
        const detection = await ModDetectionService.analyzeInstallation(installation);
        modDetectionResults.set(installation.id, detection);
      } catch (err) {
        console.error(`Failed to analyze installation ${installation.name}:`, err);
      }
    });
    
    await Promise.all(detectionPromises);
    // Force reactivity update
    modDetectionResults = new Map(modDetectionResults);
  }

  async function loadAvailableVersions() {
    try {
      // Load available Minecraft versions from Mojang API
      const result = await InstallationService.getMinecraftVersions();
      availableVersions = result || [];
      
    } catch (err) {
      console.error('Failed to load Minecraft versions:', err);
      // Fallback to some common versions
      availableVersions = [
        { id: '1.21.3', type: 'release', releaseTime: '2024-10-23T12:00:00Z', url: '', time: '2024-10-23T12:00:00Z' },
        { id: '1.21.2', type: 'release', releaseTime: '2024-10-22T12:00:00Z', url: '', time: '2024-10-22T12:00:00Z' },
        { id: '1.21.1', type: 'release', releaseTime: '2024-08-08T12:00:00Z', url: '', time: '2024-08-08T12:00:00Z' },
        { id: '1.21', type: 'release', releaseTime: '2024-06-13T12:00:00Z', url: '', time: '2024-06-13T12:00:00Z' },
        { id: '1.20.6', type: 'release', releaseTime: '2024-04-29T12:00:00Z', url: '', time: '2024-04-29T12:00:00Z' },
        { id: '1.20.4', type: 'release', releaseTime: '2023-12-07T12:00:00Z', url: '', time: '2023-12-07T12:00:00Z' },
        { id: '1.19.4', type: 'release', releaseTime: '2023-03-14T12:00:00Z', url: '', time: '2023-03-14T12:00:00Z' },
        { id: '1.18.2', type: 'release', releaseTime: '2022-02-28T12:00:00Z', url: '', time: '2022-02-28T12:00:00Z' },
      ];
    }
  }

  async function createInstallation() {
    try {
      isLoading = true;
      error = null;
      
      const installation = await InstallationService.createInstallation(
        newInstallation.name,
        newInstallation.version,
        newInstallation.mod_loader,
        newInstallation.game_directory,
        newInstallation.java_path,
        newInstallation.jvm_args,
        newInstallation.memory,
        newInstallation.description
      );
      
      installations = [...installations, installation];
      
      // Analyze the new installation for mod detection
      try {
        const detection = await ModDetectionService.analyzeInstallation(installation);
        modDetectionResults.set(installation.id, detection);
        modDetectionResults = new Map(modDetectionResults);
      } catch (err) {
        console.error('Failed to analyze new installation:', err);
      }
      
      showCreateModal = false;
      
      // Reset form
      newInstallation = {
        name: '',
        version: '',
        mod_loader: 'vanilla',
        game_directory: '',
        java_path: '',
        jvm_args: '-Xmx2G',
        memory: 2048,
        description: ''
      };
      
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
      await InstallationService.deleteInstallation(installationId);
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
      
      // Select the installation and check if we can launch
      GameManager.selectInstallation(installation);
      const { canLaunch, reason } = GameManager.canLaunch();
      if (!canLaunch) {
        error = reason || 'Cannot launch';
        return;
      }
      
      // Launch using GameManager for better integration
      await GameManager.launchGame();
      
      // Refresh installations to update last played
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
      await InstallationService.openInstallationFolder(installation.id);
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
      
      const updatedInstallation = await InstallationService.updateInstallation(
        editingInstallation.id,
        newInstallation.name,
        newInstallation.version,
        newInstallation.mod_loader,
        newInstallation.game_directory,
        newInstallation.java_path,
        newInstallation.jvm_args,
        newInstallation.memory,
        newInstallation.description
      );
      
      // Update the installation in the list
      installations = installations.map(inst => 
        inst.id === editingInstallation?.id ? updatedInstallation : inst
      );
      
      // Re-analyze the updated installation
      try {
        const detection = await ModDetectionService.analyzeInstallation(updatedInstallation);
        modDetectionResults.set(updatedInstallation.id, detection);
        modDetectionResults = new Map(modDetectionResults);
      } catch (err) {
        console.error('Failed to analyze updated installation:', err);
      }
      
      showEditModal = false;
      editingInstallation = null;
      
      // Reset form
      newInstallation = {
        name: '',
        version: '',
        mod_loader: 'vanilla',
        game_directory: '',
        java_path: '',
        jvm_args: '-Xmx2G',
        memory: 2048,
        description: ''
      };
      
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

  {#if isLoading && installations.length === 0}
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
  {/if}
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
              <span class="memory-gb">({Math.round(newInstallation.memory / 1024 * 10) / 10}GB)</span>
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
            <span class="memory-gb">({Math.round(newInstallation.memory / 1024 * 10) / 10}GB)</span>
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

  .installations-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
    gap: 1.5rem;
  }

  .installation-card {
    background: $container;
    border: 1px solid $dark-600;
    border-radius: $border-radius;
    padding: 1.5rem;
    transition: all 0.2s ease;

    &:hover {
      border-color: $primary;
      transform: translateY(-2px);
    }

    .installation-header {
      display: flex;
      gap: 1rem;
      margin-bottom: 1rem;

      .installation-icon {
        width: 60px;
        height: 60px;
        border-radius: $border-radius;
        background: rgba($primary, 0.1);
        display: flex;
        align-items: center;
        justify-content: center;
        color: $primary;
        flex-shrink: 0;
      }

      .installation-info {
        flex: 1;

        h3 {
          margin: 0 0 0.5rem;
          font-size: 1.25rem;
          font-weight: 600;
          color: $text;
        }

        .installation-details {
          display: flex;
          gap: 1rem;
          margin-bottom: 0.5rem;

          .version {
            font-weight: 500;
            color: $green;
            font-family: monospace;
          }

          .mod-loader {
            font-size: 0.875rem;
            color: $placeholder;
            text-transform: capitalize;
          }
        }

        .description {
          margin: 0;
          font-size: 0.875rem;
          color: $placeholder;
          line-height: 1.4;
        }
      }
    }

    .installation-stats {
      margin-bottom: 1rem;
      padding: 1rem 0;
      border-top: 1px solid $dark-600;
      border-bottom: 1px solid $dark-600;

      .stat {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        margin-bottom: 0.5rem;
        font-size: 0.875rem;
        color: $placeholder;

        &:last-child {
          margin-bottom: 0;
        }
      }
    }

    .installation-actions {
      display: flex;
      gap: 0.75rem;
      align-items: center;        .dropdown {
          position: relative;
          margin-left: auto;

          .dropdown-toggle {
            padding: 0.5rem;
          }

          .dropdown-menu {
            position: absolute;
            top: 100%;
            right: 0;
            margin-top: 0.25rem;
            background: $container;
            border: 1px solid $dark-600;
            border-radius: $border-radius;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
            min-width: 150px;
            z-index: 10;
            opacity: 0;
            visibility: hidden;
            transform: translateY(-10px);
            transition: opacity 0.2s ease, visibility 0.2s ease, transform 0.2s ease;
            transition-delay: 0s;

            button {
              display: flex;
              align-items: center;
              gap: 0.5rem;
              width: 100%;
              padding: 0.75rem 1rem;
              border: none;
              background: none;
              color: $text;
              font-size: 0.875rem;
              cursor: pointer;
              transition: background-color 0.2s ease;

              &:hover {
                background: rgba($primary, 0.1);
              }

              &.danger {
                color: $red;
              }
            }

            .dropdown-separator {
              height: 1px;
              background: $dark-600;
              margin: 0.5rem 0;
            }
          }

          &:hover .dropdown-menu {
            opacity: 1;
            visibility: visible;
            transform: translateY(0);
            transition-delay: 0.1s;
          }

          // Add a small invisible bridge to help with mouse navigation
          &::before {
            content: '';
            position: absolute;
            top: 100%;
            right: 0;
            width: 100%;
            height: 0.25rem;
            background: transparent;
            z-index: 9;
          }
        }
    }
  }

  .empty-state {
    text-align: center;
    padding: 4rem 2rem;

    .empty-icon {
      margin-bottom: 1.5rem;
      color: $placeholder;
    }

    h3 {
      margin: 0 0 1rem;
      font-size: 1.5rem;
      font-weight: 600;
      color: $text;
    }

    p {
      margin: 0 0 2rem;
      color: $placeholder;
      font-size: 1rem;
    }
  }

  .loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 4rem 2rem;
    color: $placeholder;

    :global(.icon) {
      animation: spin 1s linear infinite;
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
    .installations-grid {
      grid-template-columns: 1fr;
    }

    .page-header {
      flex-direction: column;
      gap: 1rem;
      align-items: stretch;
    }

    .installation-card {
      .installation-actions {
        flex-wrap: wrap;
      }
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
