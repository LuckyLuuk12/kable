<script lang="ts">
  import { onMount } from 'svelte';
  import { ModsManager, SettingsManager, Icon, type ModInstallationConfig, type InstalledMod } from '$lib';

  let selectedInstallation: string = 'global';
  let installations: ModInstallationConfig[] = [];
  let installedMods: InstalledMod[] = [];
  let isLoading = false;
  let error: string | null = null;
  let searchQuery = '';
  let useGlobalMods = true;
  let showConfigModal = false;

  // Add global option to installations list
  $: installationOptions = [
    { id: 'global', name: 'Global Mods (All Installations)', installation_type: 'global', use_global_mods: true, mods_folder_path: '' },
    ...installations
  ];

  $: filteredMods = installedMods.filter(mod => 
    mod.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    mod.version.toLowerCase().includes(searchQuery.toLowerCase())
  );

  onMount(async () => {
    // SettingsManager is already initialized in the layout
    await loadInstallations();
  });

  async function loadInstallations() {
    isLoading = true;
    error = null;
    try {
      const settings = await SettingsManager.getSettingsAsync();
      installations = await ModsManager.getModdedInstallations(settings.general.game_directory || '');
      if (installations.length > 0 && selectedInstallation === 'global') {
        // Keep global selected by default
      }
      await loadMods();
    } catch (err) {
      console.error('Failed to load installations:', err);
      error = `Failed to load installations: ${err}`;
    } finally {
      isLoading = false;
    }
  }

  async function loadMods() {
    if (!selectedInstallation) return;
    
    isLoading = true;
    error = null;
    try {
      const settings = await SettingsManager.getSettingsAsync();
      
      if (selectedInstallation === 'global') {
        // For global, we need to set up the global mods folder first
        await ModsManager.setupInstallationMods(settings.general.game_directory || '', 'kable-global', true);
        installedMods = await ModsManager.getInstalledMods(settings.general.game_directory || '', 'kable-global');
      } else {
        installedMods = await ModsManager.getInstalledMods(settings.general.game_directory || '', selectedInstallation);
      }
    } catch (err) {
      console.error('Failed to load mods:', err);
      error = `Failed to load mods: ${err}`;
      installedMods = [];
    } finally {
      isLoading = false;
    }
  }

  async function toggleMod(mod: InstalledMod) {
    try {
      await ModsManager.toggleModEnabled(mod.file_path, !mod.enabled);
      // Update the mod in the list
      mod.enabled = !mod.enabled;
      installedMods = [...installedMods];
    } catch (err) {
      console.error('Failed to toggle mod:', err);
      error = `Failed to toggle mod: ${err}`;
    }
  }

  async function openModsFolder() {
    const selectedConfig = installationOptions.find(inst => inst.id === selectedInstallation);
    if (selectedConfig) {
      // TODO: Add command to open folder in file explorer
      console.log('Opening mods folder:', selectedConfig.mods_folder_path);
    }
  }

  async function configureInstallation() {
    showConfigModal = true;
  }

  async function saveInstallationConfig() {
    if (selectedInstallation === 'global') {
      showConfigModal = false;
      return;
    }

    try {
      const settings = await SettingsManager.getSettingsAsync();
      await ModsManager.updateInstallationModConfig(
        settings.general.game_directory || '',
        selectedInstallation,
        useGlobalMods
      );
      
      // Reload installations and mods
      await loadInstallations();
      showConfigModal = false;
    } catch (err) {
      console.error('Failed to update configuration:', err);
      error = `Failed to update configuration: ${err}`;
    }
  }

  function getModLoaderIcon(loader: string): string {
    switch (loader.toLowerCase()) {
      case 'fabric': return '🧵';
      case 'forge': return '⚒️';
      case 'quilt': return '🪡';
      case 'neoforge': return '🔨';
      default: return '📦';
    }
  }

  function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }
</script>

<div class="mods-page">
  <div class="page-header">
    <h1>Mod Management</h1>
    <p>Manage mods for your Minecraft installations</p>
  </div>

  {#if error}
    <div class="error-message">
      <Icon name="alert" size="sm" />
      {error}
    </div>
  {/if}

  <!-- Installation Selection -->
  <section class="installation-section">
    <div class="section-header">
      <h2>Select Installation</h2>
      <div class="header-actions">
        <button on:click={configureInstallation} class="btn btn-secondary btn-sm">
          <Icon name="settings" size="sm" />
          Configure
        </button>
        <button on:click={openModsFolder} class="btn btn-secondary btn-sm">
          <Icon name="folder" size="sm" />
          Open Folder
        </button>
        <button on:click={loadInstallations} class="btn btn-secondary btn-sm" disabled={isLoading}>
          <Icon name="refresh" size="sm" />
          Refresh
        </button>
      </div>
    </div>

    <div class="installation-selector">
      <select bind:value={selectedInstallation} on:change={loadMods} class="installation-select">
        {#each installationOptions as installation}
          <option value={installation.id}>
            {installation.name}
            {#if installation.installation_type !== 'global'}
              ({installation.installation_type})
            {/if}
          </option>
        {/each}
      </select>

      {#if installationOptions.find(inst => inst.id === selectedInstallation)}
        {@const selectedConfig = installationOptions.find(inst => inst.id === selectedInstallation)}
        {#if selectedConfig}
          <div class="installation-info">
            <div class="info-item">
              <span class="label">Type:</span>
              <span class="value">
                {getModLoaderIcon(selectedConfig.installation_type)}
                {selectedConfig.installation_type === 'global' ? 'Global' : selectedConfig.installation_type}
              </span>
            </div>
            <div class="info-item">
              <span class="label">Mods Folder:</span>
              <span class="value">{selectedConfig.use_global_mods ? 'Global' : 'Installation-specific'}</span>
            </div>
          </div>
        {/if}
      {/if}
    </div>
  </section>

  <!-- Search and Actions -->
  <section class="search-section">
    <div class="search-bar">
      <div class="search-input-wrapper">
        <Icon name="search" size="sm" className="search-icon" />
        <input 
          type="text" 
          placeholder="Search installed mods..." 
          bind:value={searchQuery}
          class="search-input"
        />
      </div>
      
      <div class="action-buttons">
        <!-- TODO: Add mod installation from Modrinth/CurseForge -->
        <button class="btn btn-primary" disabled>
          <Icon name="download" size="sm" />
          Browse Mods
        </button>
      </div>
    </div>
  </section>

  <!-- Installed Mods -->
  <section class="mods-section">
    <div class="section-header">
      <h2>Installed Mods ({filteredMods.length})</h2>
    </div>

    {#if isLoading}
      <div class="loading-state">
        <Icon name="loader" size="lg" />
        <p>Loading mods...</p>
      </div>
    {:else if filteredMods.length > 0}
      <div class="mods-grid">
        {#each filteredMods as mod}
          <div class="mod-card" class:disabled={!mod.enabled}>
            <div class="mod-header">
              <div class="mod-icon">
                {getModLoaderIcon(mod.mod_loader)}
              </div>
              <div class="mod-info">
                <h3 class="mod-name">{mod.name}</h3>
                <p class="mod-version">Version {mod.version}</p>
              </div>
              <div class="mod-status">
                <label class="toggle-switch">
                  <input 
                    type="checkbox" 
                    checked={mod.enabled}
                    on:change={() => toggleMod(mod)}
                  />
                  <span class="slider"></span>
                </label>
              </div>
            </div>
            
            <div class="mod-details">
              <div class="detail-row">
                <span class="label">Loader:</span>
                <span class="value">{mod.mod_loader}</span>
              </div>
              
              <div class="detail-row">
                <span class="label">MC Version:</span>
                <span class="value">{mod.minecraft_version}</span>
              </div>
              
              <div class="detail-row">
                <span class="label">Source:</span>
                <span class="value">{mod.source}</span>
              </div>
              
              {#if mod.dependencies.length > 0}
                <div class="detail-row">
                  <span class="label">Dependencies:</span>
                  <span class="value">{mod.dependencies.length} mod(s)</span>
                </div>
              {/if}
            </div>

            <div class="mod-actions">
              <button class="action-btn secondary-btn" disabled>
                <Icon name="info" size="sm" />
                Details
              </button>
              
              <button class="action-btn danger-btn" disabled>
                <Icon name="trash" size="sm" />
                Remove
              </button>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="empty-state">
        <Icon name="package" size="xl" className="empty-icon" />
        <h3>No mods installed</h3>
        <p>
          {#if searchQuery.trim()}
            No mods match your search criteria.
          {:else if selectedInstallation === 'global'}
            No mods are installed in the global mods folder.
          {:else}
            No mods are installed for this installation.
          {/if}
        </p>
        <button class="btn btn-primary" disabled>
          <Icon name="download" size="sm" />
          Browse Available Mods
        </button>
      </div>
    {/if}
  </section>
</div>

<!-- Configuration Modal -->
{#if showConfigModal}
  <div 
    class="modal-overlay" 
    role="button" 
    tabindex="0"
    on:click={() => showConfigModal = false}
    on:keydown={(e) => e.key === 'Escape' && (showConfigModal = false)}
  >
    <div 
      class="modal" 
      role="dialog"
      aria-labelledby="modal-title"
      tabindex="-1"
      on:click|stopPropagation
      on:keydown|stopPropagation
    >
      <div class="modal-header">
        <h3 id="modal-title">Configure Mod Settings</h3>
        <button class="close-btn" on:click={() => showConfigModal = false}>
          <Icon name="x" size="sm" />
        </button>
      </div>
      
      <div class="modal-content">
        {#if selectedInstallation === 'global'}
          <p>Global mod settings affect all installations that use global mods.</p>
        {:else}
          <div class="config-option">
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={useGlobalMods} />
              <span class="checkmark"></span>
              Use global mods folder
            </label>
            <p class="option-description">
              {#if useGlobalMods}
                This installation will use the global mods folder, sharing mods with other installations.
              {:else}
                This installation will use its own mods folder, separate from other installations.
              {/if}
            </p>
          </div>
        {/if}
      </div>
      
      <div class="modal-actions">
        <button class="btn btn-secondary" on:click={() => showConfigModal = false}>
          Cancel
        </button>
        <button class="btn btn-primary" on:click={saveInstallationConfig}>
          Save Changes
        </button>
      </div>
    </div>
  </div>
{/if}

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;

  .mods-page {
    max-width: 1200px;
    margin: 0 auto;
    padding: 1rem;
  }

  .page-header {
    text-align: center;
    margin-bottom: 2rem;
    
    h1 {
      margin: 0 0 0.5rem 0;
      color: $text;
      font-size: 2.5rem;
      font-weight: 700;
    }
    
    p {
      margin: 0;
      color: $placeholder;
      font-size: 1.1rem;
    }
  }

  .error-message {
    background: rgba($red, 0.1);
    color: $red;
    padding: 1rem;
    border-radius: 0.5rem;
    margin-bottom: 1rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 500;
  }

  .installation-section, .search-section, .mods-section {
    background: $card;
    border: 1px solid $dark-600;
    border-radius: 1rem;
    padding: 1.5rem;
    margin-bottom: 2rem;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
    
    h2 {
      margin: 0;
      color: $text;
      font-size: 1.5rem;
      font-weight: 600;
    }
    
    .header-actions {
      display: flex;
      gap: 0.75rem;
    }
  }

  .installation-selector {
    display: flex;
    gap: 1.5rem;
    align-items: flex-start;
    flex-wrap: wrap;
  }

  .installation-select {
    flex: 1;
    min-width: 300px;
    padding: 0.75rem 1rem;
    border: 1px solid $dark-600;
    border-radius: 0.75rem;
    background: $input;
    color: $text;
    font-size: 1rem;
    
    &:focus {
      outline: none;
      border-color: $primary;
    }
  }

  .installation-info {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-width: 250px;
    
    .info-item {
      display: flex;
      justify-content: space-between;
      align-items: center;
      
      .label {
        color: $placeholder;
        font-size: 0.875rem;
        font-weight: 500;
      }
      
      .value {
        color: $text;
        font-size: 0.875rem;
        font-weight: 500;
      }
    }
  }

  .search-bar {
    display: flex;
    gap: 1rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .search-input-wrapper {
    flex: 1;
    position: relative;
    min-width: 300px;
    
    :global(.search-icon) {
      position: absolute;
      left: 1rem;
      top: 50%;
      transform: translateY(-50%);
      color: $placeholder;
    }
    
    .search-input {
      width: 100%;
      padding: 0.75rem 1rem 0.75rem 2.5rem;
      border: 1px solid $dark-600;
      border-radius: 0.75rem;
      background: $input;
      color: $text;
      font-size: 1rem;
      
      &:focus {
        outline: none;
        border-color: $primary;
      }
    }
  }

  .action-buttons {
    display: flex;
    gap: 0.75rem;
  }

  .loading-state {
    text-align: center;
    padding: 3rem 1rem;
    
    :global(.loader) {
      color: $primary;
      margin-bottom: 1rem;
      animation: spin 1s linear infinite;
    }
    
    p {
      color: $placeholder;
      font-size: 1.1rem;
    }
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .mods-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: 1.5rem;
  }

  .mod-card {
    background: $container;
    border: 1px solid $dark-600;
    border-radius: 0.75rem;
    padding: 1.5rem;
    transition: all 0.2s ease;
    
    &:hover {
      border-color: $primary;
      transform: translateY(-2px);
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    }
    
    &.disabled {
      opacity: 0.6;
      
      .mod-name {
        text-decoration: line-through;
      }
    }
    
    .mod-header {
      display: flex;
      align-items: center;
      gap: 1rem;
      margin-bottom: 1rem;
      
      .mod-icon {
        width: 3rem;
        height: 3rem;
        background: $dark-600;
        border-radius: 0.5rem;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 1.5rem;
      }
      
      .mod-info {
        flex: 1;
        
        .mod-name {
          margin: 0 0 0.25rem 0;
          color: $text;
          font-size: 1.1rem;
          font-weight: 600;
        }
        
        .mod-version {
          margin: 0;
          color: $placeholder;
          font-size: 0.875rem;
        }
      }
      
      .mod-status {
        display: flex;
        align-items: center;
      }
    }
    
    .mod-details {
      margin-bottom: 1.5rem;
      
      .detail-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.5rem;
        
        .label {
          color: $placeholder;
          font-size: 0.875rem;
          font-weight: 500;
        }
        
        .value {
          color: $text;
          font-size: 0.875rem;
          font-weight: 500;
        }
      }
    }
    
    .mod-actions {
      display: flex;
      gap: 0.75rem;
      
      .action-btn {
        flex: 1;
        padding: 0.75rem;
        border-radius: 0.5rem;
        font-weight: 600;
        font-size: 0.875rem;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        border: none;
        cursor: pointer;
        transition: all 0.2s ease;
        
        &.secondary-btn {
          background: $dark-600;
          color: $text;
          
          &:hover:not(:disabled) {
            background: $dark-500;
          }
        }
        
        &.danger-btn {
          background: $red;
          color: white;
          
          &:hover:not(:disabled) {
            background: $red-600;
          }
        }
        
        &:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
      }
    }
  }

  .toggle-switch {
    position: relative;
    display: inline-block;
    width: 44px;
    height: 24px;
    
    input {
      opacity: 0;
      width: 0;
      height: 0;
    }
    
    .slider {
      position: absolute;
      cursor: pointer;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background-color: $dark-600;
      transition: .4s;
      border-radius: 24px;
      
      &:before {
        position: absolute;
        content: "";
        height: 18px;
        width: 18px;
        left: 3px;
        bottom: 3px;
        background-color: white;
        transition: .4s;
        border-radius: 50%;
      }
    }
    
    input:checked + .slider {
      background-color: $primary;
    }
    
    input:checked + .slider:before {
      transform: translateX(20px);
    }
  }

  .empty-state {
    text-align: center;
    padding: 3rem 1rem;
    
    :global(.empty-icon) {
      color: $placeholder;
      margin-bottom: 1rem;
    }
    
    h3 {
      margin: 0 0 0.5rem 0;
      color: $text;
      font-size: 1.25rem;
      font-weight: 600;
    }
    
    p {
      margin: 0 0 1.5rem 0;
      color: $placeholder;
      font-size: 1rem;
      max-width: 500px;
      margin-left: auto;
      margin-right: auto;
    }
  }

  // Modal styles
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: $card;
    border: 1px solid $dark-600;
    border-radius: 1rem;
    max-width: 500px;
    width: 90%;
    max-height: 80vh;
    overflow-y: auto;
    
    .modal-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 1.5rem;
      border-bottom: 1px solid $dark-600;
      
      h3 {
        margin: 0;
        color: $text;
        font-size: 1.25rem;
        font-weight: 600;
      }
      
      .close-btn {
        background: none;
        border: none;
        color: $placeholder;
        cursor: pointer;
        padding: 0.5rem;
        border-radius: 0.25rem;
        
        &:hover {
          background: $dark-600;
          color: $text;
        }
      }
    }
    
    .modal-content {
      padding: 1.5rem;
      
      .config-option {
        margin-bottom: 1.5rem;
        
        .checkbox-label {
          display: flex;
          align-items: center;
          gap: 0.75rem;
          cursor: pointer;
          
          input[type="checkbox"] {
            appearance: none;
            width: 1.25rem;
            height: 1.25rem;
            border: 2px solid $dark-600;
            border-radius: 0.25rem;
            background: $input;
            cursor: pointer;
            position: relative;
            
            &:checked {
              background: $primary;
              border-color: $primary;
              
              &::after {
                content: '✓';
                position: absolute;
                top: 50%;
                left: 50%;
                transform: translate(-50%, -50%);
                color: white;
                font-size: 0.875rem;
                font-weight: bold;
              }
            }
          }
          
          span:not(.checkmark) {
            color: $text;
            font-weight: 500;
          }
        }
        
        .option-description {
          margin: 0.5rem 0 0 2rem;
          color: $placeholder;
          font-size: 0.875rem;
        }
      }
    }
    
    .modal-actions {
      display: flex;
      gap: 1rem;
      padding: 1.5rem;
      border-top: 1px solid $dark-600;
      justify-content: flex-end;
    }
  }

  // Responsive design
  @media (max-width: 768px) {
    .mods-page {
      padding: 0.5rem;
    }
    
    .mods-grid {
      grid-template-columns: 1fr;
    }
    
    .installation-selector {
      flex-direction: column;
    }
    
    .search-bar {
      flex-direction: column;
      align-items: stretch;
    }
    
    .search-input-wrapper {
      min-width: auto;
    }
    
    .mod-actions {
      flex-direction: column;
    }
    
    .section-header {
      flex-direction: column;
      gap: 1rem;
      align-items: stretch;
      
      .header-actions {
        justify-content: center;
      }
    }
  }
</style>
