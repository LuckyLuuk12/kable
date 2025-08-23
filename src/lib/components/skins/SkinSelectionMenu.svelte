<script lang="ts">
  import { onMount } from 'svelte';
  import { getPlayerProfile, getCurrentSkinInfo, applyAccountSkin, selectSkinFile, uploadSkinToAccount, removeSkinById, modifySkinById, getLocalSkins } from '$lib/api/skins';
  import { Icon } from '$lib';
  import { SkinViewer3D } from '$lib/components/skins';
  import type { AccountSkin, CurrentSkin, SkinModelType, AccountCape, PlayerProfile } from '$lib/types';

  // State
  let accountSkins: AccountSkin[] = [];
  let currentSkin: CurrentSkin | null = null;
  let capes: AccountCape[] = [];
  let loading = false;
  let error = '';
  let hoveredSkinId: string | null = null;
  let editingSkinId: string | null = null;
  let editName = '';
  let editCapeId = '';
  let editSlim: boolean | undefined = undefined;

  // Load skins on component mount
  onMount(async () => {
    await loadSkins();
  });

  async function loadSkins() {
    loading = true;
    error = '';
    let remoteSkins: AccountSkin[] = [];
    let remoteCapes: AccountCape[] = [];
    try {
      // Try to fetch remote profile (skins/capes)
      const profile: PlayerProfile = await getPlayerProfile();
      remoteSkins = profile.skins || [];
      remoteCapes = profile.capes || [];
    } catch (err) {
      error = `Failed to load remote skins/capes: ${err}`;
      console.error('Error loading remote skins/capes:', err);
    }
    try {
      // Always load local skins from launcher_custom_skins.json
  const localSkins: AccountSkin[] = await getLocalSkins();
      // Merge remote and local skins, local always shown
      accountSkins = [...localSkins, ...remoteSkins];
      capes = remoteCapes;
      currentSkin = await getCurrentSkinInfo();
      // Debug log
      console.log('Loaded accountSkins:', accountSkins);
      console.log('Loaded capes:', capes);
      const missingUrls = accountSkins.filter(skin => !skin.url);
      if (missingUrls.length > 0) {
        error += `\nWarning: ${missingUrls.length} skin(s) are missing a valid file path and cannot be displayed.`;
        console.warn('Skins missing url:', missingUrls);
      }
    } catch (err) {
      error += `\nFailed to load local skins: ${err}`;
      console.error('Error loading local skins:', err);
      // Fallback: show only remote skins if local fails
      accountSkins = remoteSkins;
      capes = remoteCapes;
    } finally {
      loading = false;
    }
  }
  async function handleRemoveSkin(skinId: string) {
    loading = true;
    error = '';
    try {
      await removeSkinById(skinId);
      await loadSkins();
    } catch (err) {
      error = `Failed to remove skin: ${err}`;
      console.error('Error removing skin:', err);
    } finally {
      loading = false;
    }
  }

  function startEditSkin(skin: AccountSkin) {
  editingSkinId = skin.id;
  editName = skin.name || '';
  editCapeId = '';
  // Type-safe check for Slim model
  editSlim = String(skin.model).toLowerCase() === 'slim';
  }

  function cancelEditSkin() {
  editingSkinId = null;
  editName = '';
  editCapeId = '';
  editSlim = undefined;
  }

  async function handleModifySkin() {
    if (!editingSkinId) return;
    loading = true;
    error = '';
    try {
      await modifySkinById(editingSkinId, editName, editCapeId, editSlim);
      await loadSkins();
      cancelEditSkin();
    } catch (err) {
      error = `Failed to modify skin: ${err}`;
      console.error('Error modifying skin:', err);
    } finally {
      loading = false;
    }
  }

  async function handleApplySkin(skinId: string) {
    loading = true;
    error = '';
    
    try {
      await applyAccountSkin(skinId);
      await loadSkins(); // Refresh the skin list
    } catch (err) {
      error = `Failed to apply skin: ${err}`;
      console.error('Error applying skin:', err);
    } finally {
      loading = false;
    }
  }

  async function handleUploadNewSkin() {
    loading = true;
    error = '';
    
    try {
      // Open file dialog
      const filePath = await selectSkinFile();
      if (!filePath) {
        loading = false;
        return; // User cancelled
      }

      // For now default to Classic model
      // TODO: Add UI to let user choose between Classic and Slim
      const config = {
        model: 'Classic' as SkinModelType,
        file_path: filePath
      };

      await uploadSkinToAccount(config);
      await loadSkins(); // Refresh the skin list
    } catch (err) {
      error = `Failed to upload skin: ${err}`;
      console.error('Error uploading skin:', err);
    } finally {
      loading = false;
    }
  }

  // Convert skin model to skinview3d format
  function getSkinModel(skinModel: string): 'classic' | 'slim' | 'auto' {
    if (skinModel?.toLowerCase() === 'slim') return 'slim';
    if (skinModel?.toLowerCase() === 'classic') return 'classic';
    return 'auto';
  }

  // Get skin display name
  function getSkinDisplayName(skin: AccountSkin): string {
    if (skin.name && skin.name !== 'Current Skin') {
      return skin.name;
    }
    return `${skin.model} Skin`;
  }
</script>

<div class="skin-selection-menu">
  <div class="header">
    <div class="header-content">
      <h1>Skin Management</h1>
      <p>Manage your Minecraft character appearance</p>
    </div>
    <button 
      class="btn btn-primary glass-btn" 
      on:click={handleUploadNewSkin}
      disabled={loading}
    >
      <Icon name="upload" size="sm" />
      {loading ? 'Uploading...' : 'Upload New Skin'}
    </button>
  </div>

  {#if error}
    <div class="error-message glass-card">
      <Icon name="alert" size="sm" />
      {error}
    </div>
  {/if}

  {#if loading}
    <div class="loading-state">
      <Icon name="refresh" size="md" />
      <span>Loading skins...</span>
    </div>
  {:else}
    <div class="skins-content skins-row">
      <!-- Current Skin Section -->
      <div class="current-skin-section">
        <h2>Current Skin</h2>
        {#if currentSkin && currentSkin.has_skin}
          <div class="current-skin-card glass-card"
               role="button"
               tabindex="0"
               on:mouseenter={() => hoveredSkinId = 'current'}
               on:mouseleave={() => hoveredSkinId = null}>
            <div class="skin-preview large">
              {#if currentSkin.url}
                <SkinViewer3D 
                  skinUrl={currentSkin.url} 
                  width={140} 
                  height={140}
                  model={getSkinModel(currentSkin.model)}
                  animation={hoveredSkinId === 'current' ? 'walk' : 'idle'}
                />
              {:else}
                <div class="skin-placeholder">
                  <Icon name="user" size="lg" />
                </div>
              {/if}
            </div>
            <div class="skin-details">
              <h3>Active Skin</h3>
              <div class="skin-info">
                <span class="info-item">
                  <Icon name="user" size="sm" />
                  Model: {currentSkin.model}
                </span>
                <span class="status-badge active">Active</span>
              </div>
            </div>
          </div>
        {:else}
          <div class="no-current-skin glass-card">
            <Icon name="user" size="lg" />
            <h3>No Current Skin</h3>
            <p>Upload a skin to get started</p>
          </div>
        {/if}
      </div>

      <!-- Available Skins Section -->
      <div class="skins-section">
        <h2>Available Skins</h2>
        {#if accountSkins.length === 0}
          <div class="no-skins glass-card">
            <Icon name="image" size="lg" />
            <h3>No Skins Found</h3>
            <p>No skins found in your Microsoft account. Upload a skin to get started.</p>
            <button class="btn btn-primary glass-btn" on:click={handleUploadNewSkin} disabled={loading}>
              <Icon name="upload" size="sm" />
              Upload Your First Skin
            </button>
          </div>
        {:else}
          <div class="skins-grid">
            {#each accountSkins as skin (skin.id)}
              <div class="skin-card glass-card" 
                   role="button"
                   tabindex="0"
                   class:current={skin.is_current}
                   on:mouseenter={() => hoveredSkinId = skin.id}
                   on:mouseleave={() => hoveredSkinId = null}>
                <div class="skin-preview">
                  {#if skin.url}
                    <SkinViewer3D 
                      skinUrl={skin.url} 
                      width={220} 
                      height={180}
                      model={getSkinModel(skin.model)}
                      animation={hoveredSkinId === skin.id ? 'walk' : 'idle'}
                    />
                  {:else}
                    <div class="skin-placeholder">
                      <Icon name="user" size="md" />
                    </div>
                  {/if}
                </div>
                <div class="skin-info">
                  <div class="skin-header">
                    <h4>{getSkinDisplayName(skin)}</h4>
                    {#if skin.is_current}
                      <span class="status-badge current">Current</span>
                    {/if}
                  </div>
                  <div class="skin-meta">
                    <span class="meta-item">
                      <Icon name="user" size="sm" />
                      {skin.model}
                    </span>
                    {#if skin.uploaded_date}
                      <span class="meta-item">
                        <Icon name="calendar" size="sm" />
                        {new Date(skin.uploaded_date * 1000).toLocaleDateString()}
                      </span>
                    {/if}
                  </div>
                  {#if !skin.is_current}
                    <div class="skin-actions">
                      <button 
                        class="btn btn-secondary btn-sm glass-btn"
                        on:click={() => handleApplySkin(skin.id)}
                        disabled={loading}
                      >
                        <Icon name="check" size="sm" />
                        Use
                      </button>
                      <button 
                        class="btn btn-danger btn-sm glass-btn"
                        on:click={() => handleRemoveSkin(skin.id)}
                        disabled={loading}
                      >
                        <Icon name="trash" size="sm" />
                        Remove
                      </button>
                      <button 
                        class="btn btn-info btn-sm glass-btn"
                        on:click={() => startEditSkin(skin)}
                        disabled={loading}
                      >
                        <Icon name="edit" size="sm" />
                        Edit
                      </button>
                    </div>
                  {/if}
              {#if editingSkinId === skin.id}
                <div class="skin-edit-form glass-card">
                  <h5>Edit Skin</h5>
                  <div class="form-group">
                    <label for="editName">Name:</label>
                    <input id="editName" type="text" bind:value={editName} />
                  </div>
                  <div class="form-group">
                    <label for="editCape">Cape:</label>
                    <select id="editCape" bind:value={editCapeId}>
                      <option value="">None</option>
                      {#each capes as cape}
                        <option value={cape.id}>{cape.alias || cape.id}</option>
                      {/each}
                    </select>
                  </div>
                  <div class="form-group">
                    <label for="editSlim">Model:</label>
                    <select id="editSlim" bind:value={editSlim}>
                      <option value={false}>Classic</option>
                      <option value={true}>Slim</option>
                    </select>
                  </div>
                  <div class="form-actions">
                    <button class="btn btn-success btn-sm glass-btn" on:click={handleModifySkin} disabled={loading}>
                      <Icon name="check" size="sm" />
                      Save
                    </button>
                    <button class="btn btn-secondary btn-sm glass-btn" on:click={cancelEditSkin} disabled={loading}>
                      <Icon name="close" size="sm" />
                      Cancel
                    </button>
                  </div>
                </div>
              {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;

.skin-selection-menu {
  width: 100%;
  max-height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

// Glass morphism effects
.glass-card {
  background: rgba(var(--card), 0.7);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(var(--primary), 0.1);
  box-shadow:
    0 8px 32px rgba(var(--dark-900), 0.12),
    inset 0 1px 0 rgba(255, 255, 255, 0.2);
}

.glass-btn {
  background: rgba(var(--primary), 0.1);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(var(--primary), 0.2);
  transition: all 0.3s ease;

  &:hover:not(:disabled) {
    background: rgba(var(--primary), 0.2);
    border-color: rgba(var(--primary), 0.3);
    transform: translateY(-1px);
  }

  &:disabled {
    background: rgba(var(--placeholder), 0.1);
    border-color: rgba(var(--placeholder), 0.2);
  }
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 1.5rem 2rem 0;
  margin-bottom: 1.5rem;
  flex-shrink: 0;

  .header-content {
    h1 {
      margin: 0 0 0.5rem;
      font-size: 1.75rem;
      font-weight: 700;
      color: var(--text);
    }

    p {
      margin: 0;
      color: var(--placeholder);
      font-size: 0.9rem;
    }
  }

  .btn {
    flex-shrink: 0;
  }
}

.error-message {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem 1.5rem;
  margin: 0 2rem 1.5rem;
  background: rgba(var(--red), 0.1);
  border: 1px solid rgba(var(--red), 0.3);
  border-radius: var(--border-radius);
  color: var(--red);
  font-size: 0.9rem;
  flex-shrink: 0;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  padding: 4rem 2rem;
  color: var(--placeholder);
  flex: 1;

  :global(.icon) {
    animation: spin 1s linear infinite;
  }

  span {
    font-size: 1.1rem;
    font-weight: 500;
  }
}

// Row layout for current skin and available skins side-by-side
.skins-content.skins-row {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 0 2rem 2rem;
  display: flex;
  flex-direction: row;
  gap: 2rem;
}

// Responsive: stack on mobile
@media (max-width: 900px) {
  .skins-content.skins-row {
    flex-direction: column;
    gap: 2rem;
  }
}

// Custom scrollbar for .skin-selection-menu
.skin-selection-menu::-webkit-scrollbar {
  width: 8px;
}

.skin-selection-menu::-webkit-scrollbar-track {
  background: rgba(var(--placeholder), 0.1);
  border-radius: 4px;
}

.skin-selection-menu::-webkit-scrollbar-thumb {
  background: rgba(var(--primary), 0.3);
  border-radius: 4px;
}

.skin-selection-menu::-webkit-scrollbar-thumb:hover {
  background: rgba(var(--primary), 0.5);
}

// Current Skin Section
  .current-skin-section {
    flex-shrink: 0;

    h2 {
      margin: 0 0 1rem;
      font-size: 1.25rem;
      font-weight: 600;
      color: var(--text);
    }
  }

  .current-skin-card {
    display: flex;
    gap: 1.5rem;
    padding: 2rem;
    border-radius: var(--border-radius-large);
    align-items: center;
    transition: all 0.3s ease;
    cursor: pointer;

    &:hover {
      transform: translateY(-2px);
      box-shadow: 
        0 12px 35px rgba(var(--dark-900), 0.2),
        0 0 20px rgba(var(--primary), 0.1),
        inset 0 1px 0 rgba(255, 255, 255, 0.3);
    }

    .skin-preview.large {
      width: 140px;
      height: 140px;
      border-radius: var(--border-radius);
      overflow: hidden;
      background: rgba(var(--placeholder), 0.05);
      border: 2px solid rgba(var(--primary), 0.1);
      flex-shrink: 0;
      display: flex;
      align-items: center;
      justify-content: center;
      transition: all 0.3s ease;

    .skin-placeholder {
      width: 100%;
      height: 100%;
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--placeholder);
    }
  }

  .skin-details {
    flex: 1;

    h3 {
      margin: 0 0 1rem;
      font-size: 1.25rem;
      font-weight: 600;
      color: var(--text);
    }

    .skin-info {
      display: flex;
      flex-direction: column;
      gap: 0.75rem;

      .info-item {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        color: var(--placeholder);
        font-size: 0.9rem;
      }

      .status-badge {
        align-self: flex-start;
        padding: 0.25rem 0.75rem;
        border-radius: 12px;
        font-size: 0.65rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;

        &.active {
          background: rgba(var(--green), 0.8);
          backdrop-filter: blur(10px);
          color: var(--text-white);
          border: 1px solid rgba(var(--green), 0.3);
        }
      }
    }
  }
} // <-- Add this missing closing brace here
// ...existing code...

  .no-current-skin {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem 2rem;
    background: rgba(var(--placeholder), 0.03);
    border: 2px dashed rgba(var(--placeholder), 0.2);
    border-radius: var(--border-radius-large);
    text-align: center;

    :global(.icon) {
      margin-bottom: 1rem;
      color: var(--placeholder);
    }

  h3 {
    margin: 0 0 0.5rem;
    color: var(--text);
    font-weight: 600;
  }

  p {
    margin: 0;
    color: var(--placeholder);
    font-size: 0.9rem;
  }
}

// Available Skins Section
// Limit available skins section to scroll independently
.skins-section {
  flex: 1;
  min-width: 0;
  max-height: 600px;
  overflow-y: auto;

  h2 {
    margin: 0 0 1.5rem;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text);
  }

  // Custom scrollbar for available skins
  &::-webkit-scrollbar {
    width: 8px;
  }
  &::-webkit-scrollbar-track {
    background: rgba(var(--placeholder), 0.1);
    border-radius: 4px;
  }
  &::-webkit-scrollbar-thumb {
    background: rgba(var(--primary), 0.3);
    border-radius: 4px;
    &:hover {
      background: rgba(var(--primary), 0.5);
    }
  }
}

  .no-skins {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 2rem;
    background: rgba(var(--placeholder), 0.03);
    border: 2px dashed rgba(var(--placeholder), 0.2);
    border-radius: var(--border-radius-large);
    text-align: center;

    :global(.icon) {
      margin-bottom: 1.5rem;
      color: var(--placeholder);
    }

  h3 {
    margin: 0 0 0.5rem;
    color: var(--text);
    font-weight: 600;
    font-size: 1.1rem;
  }

  p {
    margin: 0 0 2rem;
    color: var(--placeholder);
    font-size: 0.9rem;
  }

  .btn {
    margin: 0 auto;
  }
}

// More compact grid for available skins
.skins-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 0.75rem;
  max-height: 600px;
  overflow-y: auto;
  padding-right: 0.5rem;

  // Custom scrollbar for grid
  &::-webkit-scrollbar {
    width: 6px;
  }

  &::-webkit-scrollbar-track {
    background: rgba(var(--placeholder), 0.1);
    border-radius: 3px;
  }

  &::-webkit-scrollbar-thumb {
    background: rgba(var(--primary), 0.3);
    border-radius: 3px;
    
    &:hover {
      background: rgba(var(--primary), 0.5);
    }
  }
}

.skin-card {
  border-radius: var(--border-radius-large);
  padding: 0.75rem;
  transition: all 0.3s ease;
  cursor: pointer;

  &:hover {
    transform: translateY(-2px);
    box-shadow: 
      0 8px 20px rgba(var(--dark-900), 0.15),
      0 0 10px rgba(var(--primary), 0.08),
      inset 0 1px 0 rgba(255, 255, 255, 0.2);
    border-color: rgba(var(--primary), 0.15);
  }

  &.current {
    background: rgba(var(--green), 0.04);
    backdrop-filter: blur(16px);
    border: 1px solid rgba(var(--green), 0.15);
    box-shadow: 
      0 4px 16px rgba(var(--green), 0.10),
      inset 0 1px 0 rgba(var(--green), 0.15);
  }

  .skin-preview {
    width: 100%;
    height: 120px;
    border-radius: var(--border-radius);
    overflow: hidden;
    background: rgba(var(--placeholder), 0.03);
    margin-bottom: 0.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.3s ease;

    .skin-placeholder {
      width: 100%;
      height: 100%;
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--placeholder);
      background: rgba(var(--placeholder), 0.03);
    }
  }

  .skin-info {
    .skin-header {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      margin-bottom: 0.5rem;

      h4 {
        margin: 0;
        font-size: 0.95rem;
        font-weight: 600;
        color: var(--text);
        flex: 1;
        line-height: 1.2;
      }

      .status-badge {
        padding: 0.12rem 0.4rem;
        border-radius: 8px;
        font-size: 0.55rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        margin-left: 0.4rem;
        flex-shrink: 0;

        &.current {
          background: rgba(var(--green), 0.8);
          backdrop-filter: blur(8px);
          color: var(--text-white);
          border: 1px solid rgba(var(--green), 0.25);
        }
      }
    }

    .skin-meta {
      display: flex;
      flex-direction: column;
      gap: 0.35rem;
      margin-bottom: 0.5rem;

      .meta-item {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        color: var(--placeholder);
        font-size: 0.75rem;
      }
    }

    .skin-actions {
      display: flex;
      gap: 0.5rem;
    }
  }
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@media (max-width: 768px) {
  .skin-selection-menu {
    height: 100vh;
  }

  .header {
    flex-direction: column;
    gap: 1rem;
    align-items: stretch;

    .btn {
      align-self: flex-start;
    }
  }

  .skins-content {
    padding: 0 1rem 2rem;
  }

  .current-skin-card {
    flex-direction: column;
    text-align: center;
    padding: 1.5rem;

    .skin-preview.large {
      align-self: center;
    }
  }

  .skins-grid {
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 1rem;
    max-height: 500px;
  }

  .skin-card {
    padding: 1rem;

    .skin-preview {
      height: 150px;
    }
  }
}
</style>
