<script lang="ts">
  import { onMount } from 'svelte';
import { Icon, SkinsService, type MinecraftSkin } from '$lib';

  // State variables
  let skins: MinecraftSkin[] = [];
  let isLoading = false;
  let error: string | null = null;
  let selectedSkinIndex = 0;

  onMount(async () => {
    await loadSkins();
  });

  async function loadSkins() {
    try {
      isLoading = true;
      error = null;

      // Load skins from backend
      const result = await SkinsService.getSkins();
      skins = result || getDefaultSkins();

    } catch (err) {
      console.error('Failed to load skins:', err);
      error = `Failed to load skins: ${err}`;
      skins = getDefaultSkins();
    } finally {
      isLoading = false;
    }
  }

  function getDefaultSkins(): MinecraftSkin[] {
    return [
      {
        id: 'steve',
        name: 'Steve (Default)',
        url: '',
        type: 'steve',
        premium: false,
        file_name: 'steve_default',
        is_slim: false,
        source: 'Default',
        created_date: 0
      },
      {
        id: 'alex', 
        name: 'Alex (Default)',
        url: '',
        type: 'alex',
        premium: false,
        file_name: 'alex_default',
        is_slim: true,
        source: 'Default',
        created_date: 0
      }
    ];
  }

  async function changeSkin(index: number) {
    try {
      selectedSkinIndex = index;
      const skin = skins[index];
      
      // Apply the skin
      await SkinsService.applySkin(skin.id);
      
    } catch (err) {
      console.error('Failed to change skin:', err);
      error = `Failed to change skin: ${err}`;
    }
  }

  async function uploadSkin() {
    try {
      // Trigger file picker and upload skin
      const newSkin = await SkinsService.uploadSkin();
      if (newSkin) {
        skins = [...skins, newSkin];
      }
    } catch (err) {
      console.error('Failed to upload skin:', err);
      error = `Failed to upload skin: ${err}`;
    }
  }

  async function deleteSkin(skinId: string) {
    if (!confirm('Are you sure you want to delete this skin?')) {
      return;
    }

    try {
      await SkinsService.deleteSkin(skinId);
      skins = skins.filter(skin => skin.id !== skinId);
      
      // Reset selection if deleted skin was selected
      if (selectedSkinIndex >= skins.length) {
        selectedSkinIndex = 0;
      }
    } catch (err) {
      console.error('Failed to delete skin:', err);
      error = `Failed to delete skin: ${err}`;
    }
  }
</script>

<div class="skins-page">
  <div class="page-header">
    <div class="header-content">
      <h1>Skins & Appearance</h1>
      <p>Customize your Minecraft character appearance</p>
    </div>
    <button 
      class="btn btn-primary" 
      on:click={uploadSkin}
      disabled={isLoading}
    >
      <Icon name="upload" size="sm" />
      Upload Skin
    </button>
  </div>

  {#if error}
    <div class="error-message">
      <Icon name="alert" size="sm" />
      {error}
    </div>
  {/if}

  {#if isLoading}
    <div class="loading-state">
      <Icon name="refresh" size="md" />
      <span>Loading skins...</span>
    </div>
  {:else}
    <div class="skins-grid">
      {#each skins as skin, index}
        <div 
          class="skin-card" 
          class:selected={index === selectedSkinIndex}
          on:click={() => changeSkin(index)}
          on:keydown={(e) => e.key === 'Enter' && changeSkin(index)}
          role="button"
          tabindex="0"
        >
          <div class="skin-preview">
            <div class="skin-placeholder">
              <Icon name="user" size="lg" />
            </div>
            <div class="skin-model-type">
              {skin.type === 'alex' ? 'Slim' : 'Classic'}
            </div>
          </div>
          <div class="skin-info">
            <h4>{skin.name}</h4>
            {#if index === selectedSkinIndex}
              <span class="current-skin">Current</span>
            {/if}
            {#if skin.premium}
              <span class="premium-badge">Premium</span>
            {/if}
          </div>
          
          {#if skin.id !== 'steve' && skin.id !== 'alex'}
            <div class="skin-actions">
              <button 
                class="btn-icon danger" 
                on:click|stopPropagation={() => deleteSkin(skin.id)}
                title="Delete skin"
              >
                <Icon name="trash" size="sm" />
              </button>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;

  .skins-page {
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

  .skins-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 1.5rem;
  }

  .skin-card {
    background: $container;
    border: 1px solid $dark-600;
    border-radius: $border-radius;
    padding: 1.5rem;
    cursor: pointer;
    transition: all 0.2s ease;
    position: relative;

    &:hover {
      border-color: $primary;
      transform: translateY(-2px);
    }

    &.selected {
      border-color: $primary;
      background: rgba($primary, 0.1);
    }

    .skin-preview {
      text-align: center;
      margin-bottom: 1rem;

      .skin-placeholder {
        width: 80px;
        height: 80px;
        margin: 0 auto 0.5rem;
        background: rgba($primary, 0.1);
        border-radius: $border-radius;
        display: flex;
        align-items: center;
        justify-content: center;
        color: $primary;
      }

      .skin-model-type {
        font-size: 0.75rem;
        color: $placeholder;
        text-transform: uppercase;
        letter-spacing: 0.5px;
      }
    }

    .skin-info {
      text-align: center;

      h4 {
        margin: 0 0 0.5rem;
        font-size: 1rem;
        font-weight: 600;
        color: $text;
      }

      .current-skin {
        display: inline-block;
        padding: 0.25rem 0.5rem;
        background: $primary;
        color: white;
        font-size: 0.75rem;
        border-radius: 12px;
        font-weight: 500;
      }

      .premium-badge {
        display: inline-block;
        padding: 0.25rem 0.5rem;
        background: #ffd700;
        color: white;
        font-size: 0.75rem;
        border-radius: 12px;
        font-weight: 500;
        margin-left: 0.5rem;
      }
    }

    .skin-actions {
      position: absolute;
      top: 0.5rem;
      right: 0.5rem;

      .btn-icon {
        background: rgba($background, 0.9);
        border: 1px solid $dark-600;
        border-radius: 50%;
        width: 32px;
        height: 32px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: all 0.2s ease;

        &:hover {
          background: $background;
        }

        &.danger {
          color: $red;
          
          &:hover {
            background: rgba($red, 0.1);
            border-color: $red;
          }
        }
      }
    }
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  @media (max-width: 768px) {
    .skins-grid {
      grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    }

    .page-header {
      flex-direction: column;
      gap: 1rem;
      align-items: stretch;
    }
  }
</style>
