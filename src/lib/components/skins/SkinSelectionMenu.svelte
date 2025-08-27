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
  let showAddModal = false;
  let addFilePath = '';
  let addName = '';
  let addCapeId = '';
  let addSlim: boolean | undefined = undefined;

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
      // Show modal to collect name/cape/model
      addFilePath = filePath;
      addName = '';
      addCapeId = '';
      addSlim = false;
      showAddModal = true;
    } catch (err) {
      error = `Failed to select skin file: ${err}`;
      console.error('Error selecting skin file:', err);
      loading = false;
    }
  }

  async function handleAddSkinSubmit() {
    loading = true;
    error = '';
    try {
      const config = {
        model: (addSlim ? 'Slim' : 'Classic') as SkinModelType,
        file_path: addFilePath,
        name: addName,
        cape_id: addCapeId
      };
      await uploadSkinToAccount(config);
      await loadSkins();
      showAddModal = false;
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
  </div>

  {#if error}
    <div class="error-message">
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
      <!-- Available Skins Section -->
      <div class="skins-section">
        <h2>Available Skins</h2>
        {#if accountSkins.length === 0}
          <div class="no-skins">
            <Icon name="image" size="lg" />
            <h3>No Skins Found</h3>
            <p>No skins found in your Microsoft account. Upload a skin to get started.</p>
            <button class="upload-button" on:click={handleUploadNewSkin} disabled={loading}>
              <Icon name="upload" size="sm" />
              Upload Your First Skin
            </button>
          </div>
        {:else}
          <div class="skins-flex">
            <!-- Fake card for uploading new skin -->
            <button class="skin-card upload-card" tabindex="0" on:click={handleUploadNewSkin} aria-label="Upload New Skin" disabled={loading}>
              <div class="skin-preview">
                <Icon name="plus" size="lg" forceType="svg" />
              </div>
              <div class="skin-info">
                <div class="skin-header">
                  <h4>{loading ? 'Uploading...' : 'Upload New Skin'}</h4>
                </div>
              </div>
            </button>
            {#if showAddModal}
              <div class="modal-backdrop">
                <div class="modal">
                  <h3>Add New Skin</h3>
                  <div class="form-group">
                    <label for="addName">Name:</label>
                    <input id="addName" type="text" bind:value={addName} placeholder="Skin name" />
                  </div>
                  <div class="form-group">
                    <label for="addCape">Cape:</label>
                    <select id="addCape" bind:value={addCapeId}>
                      <option value="">None</option>
                      {#each capes as cape}
                        <option value={cape.id}>{cape.alias || cape.id}</option>
                      {/each}
                    </select>
                  </div>
                  <div class="form-group">
                    <label for="addSlim">Model:</label>
                    <select id="addSlim" bind:value={addSlim}>
                      <option value={false}>Classic</option>
                      <option value={true}>Slim</option>
                    </select>
                  </div>
                  <div class="form-actions">
                    <button class="save-button" on:click={handleAddSkinSubmit} disabled={loading || !addFilePath || !addName}>
                      <Icon name="check" size="sm" />
                      Save
                    </button>
                    <button class="cancel-button" on:click={() => showAddModal = false} disabled={loading}>
                      <Icon name="close" size="sm" />
                      Cancel
                    </button>
                  </div>
                </div>
              </div>
            {/if}

            {#each accountSkins as skin (skin.id)}
              <div class="skin-card" 
                   role="button"
                   tabindex="0"
                   class:current={skin.is_current}
                   on:mouseenter={() => hoveredSkinId = skin.id}
                   on:mouseleave={() => hoveredSkinId = null}
                   on:click={() => !editingSkinId && handleApplySkin(skin.id)}
                   on:keypress={() => !editingSkinId && handleApplySkin(skin.id)}>
                <div class="skin-preview">
                  {#if skin.url}
                    <SkinViewer3D 
                      skinUrl={skin.url}
                      height={120}
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
                        class="remove-button"
                        on:click={() => handleRemoveSkin(skin.id)}
                        disabled={loading}
                        title="Remove"
                      >
                        <Icon name="trash" size="sm" />
                      </button>
                      <button 
                        class="edit-button"
                        on:click={() => startEditSkin(skin)}
                        disabled={loading}
                        title="Edit"
                      >
                        <Icon name="edit" size="sm" />
                      </button>
                    </div>
                  {/if}
              {#if editingSkinId === skin.id}
                <div class="skin-edit-form">
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
                    <button class="save-button" on:click={handleModifySkin} disabled={loading}>
                      <Icon name="check" size="sm" />
                      Save
                    </button>
                    <button class="cancel-button" on:click={cancelEditSkin} disabled={loading}>
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
  {/if}
</div>

<style lang="scss">
.skin-selection-menu {
    .header {
      .header-content {
        h1 {}
        p {}
      }
    }
    .error-message {
      color: var(--red);
    }
    .loading-state {
      span {
        color: var(--placeholder);
      }
    }
    .skins-section {
      width: 100%;
      display: inline-block;
      h2 {}
      .no-skins {
        .icon {}
        h3 {}
        p {}
        .btn {}
      }
      .skins-flex {
        margin: 0 auto;
        display: flex;
        flex-wrap: wrap;
        justify-content: center;
        gap: 0.75rem;
        .skin-card {
          display: flex;
          flex-direction: column;
          justify-content: space-between;
          height: 220px;
          width: 10%;
          padding: 0.15rem 0.5rem;
          background: var(--card);
          border: 1px solid var(--border);
          border-radius: var(--border-radius);
          overflow: clip;
          &.upload-card {
            justify-content: space-evenly;
          }
          .skin-preview {
            align-self: center;
            .skin-placeholder {
            }
          }
          .skin-info {
            .skin-header {
              h4 {
                max-height: 1.4rem;
                overflow: hidden;
                text-overflow: ellipsis;
              }
              .status-badge {
                &.current {
                  border-color: var(--green);
                }
              }
            }
            .skin-meta {
              .meta-item {}
            }
            .skin-actions {
              button {
                background: transparent;
                border: none;
                svg {}
              }
            }
          }
          &.current {
            border-color: var(--green);
          }
        }
      }
    }
    .modal-backdrop {
      .modal {
        h3 {}
        .form-group {
          label {}
          input {}
          select {}
        }
        .form-actions {
          button {}
        }
      }
    }
  }
</style>