<script lang="ts">
  import { InstallationsList, CreateInstallationModal, EditInstallationModal, Icon, type KableInstallation, InstallationService } from '$lib';

  let showEditModal = false;
  let editingInstallation: KableInstallation | null = null;
  let createModalRef: CreateInstallationModal;
  let editModalRef;

  let isSmall = false;
  let isGrid = true;
  let isRefreshing = false;

  function editInstallation(installation: KableInstallation) {
    editingInstallation = installation;
    showEditModal = true;
  }

  function openCreateModal() {
    createModalRef?.open();
  }

  async function refreshInstallations() {
    isRefreshing = true;
    try {
      await InstallationService.loadInstallations();
    } finally {
      isRefreshing = false;
    }
  }
</script>

<div class="installations-page">
  <div class="page-header">
    <div class="header-content">
      <h1>Installations</h1>
      <p>Manage your Minecraft installations, versions, and mod loaders</p>
    </div>
    
  </div>


  <div class="controls-container">
    <button 
      class="btn btn-primary new-installation-btn"
      on:click={openCreateModal}
    >
      <Icon name="plus" size="md" forceType="svg" />
      New Installation
    </button>
    <div class="view-controls">
      <button 
        class="btn btn-secondary {isRefreshing ? 'spinning' : ''}" 
        on:click={refreshInstallations}
        disabled={isRefreshing}
        title="Refresh installations list"
      >
        <Icon name="refresh" size="md" forceType="svg" />
      </button>
      <button 
        class="btn btn-secondary" 
        on:click={() => isGrid = !isGrid} 
        class:is-active={isGrid}
        title={isGrid ? 'Switch to list view' : 'Switch to grid view'}
      >
        <Icon name={isGrid ? 'list' : 'grid'} size="md" />
      </button>
      <button class="btn btn-secondary" on:click={() => isSmall = !isSmall} class:is-active={isSmall} title={"Turn compact mode " + (isSmall ? 'off' : 'on')}>
        <Icon name="minimize" size="md" />
      </button>
    </div>
  </div>

  <InstallationsList {isGrid} isSmall={isSmall} on:edit={(e) => editInstallation(e.detail)} />

  <CreateInstallationModal bind:this={createModalRef} />
  {#if showEditModal && editingInstallation}
    <EditInstallationModal bind:this={editModalRef} installation={editingInstallation} on:close={() => { showEditModal = false; editingInstallation = null; }} />
  {/if}
</div>

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;

  .installations-page {
    width: 100%;
    max-width: none;
    margin: 0;
    padding: 0 2vw;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 2rem;

    .header-content {
      text-align: center;
      width: 100%;
      h1 {
        margin: 0 0 0.5rem;
        font-size: 2rem;
        font-weight: 700;
        color: var(--text);
        text-align: center;
      }
      p {
        margin: 0;
        color: var(--placeholder);
        font-size: 1rem;
        text-align: center;
      }
    }
  }
  .controls-container {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1.5rem;
    flex-wrap: wrap;
    .new-installation-btn {
      display: flex;
      align-items: center;
      font-size: 1.1rem;
      padding: 0.75rem 1.5rem;
      background: none;
      color: var(--primary);
      border-radius: var(--border-radius);
      box-shadow: none;
      border: 1.5px solid var(--primary);
      font-weight: 600;
      transition: color 0.13s, background 0.13s, border 0.13s;
      &:hover, &:focus {
        background: color-mix(in srgb, var(--primary), 10%, transparent);
        color: var(--primary-900);
        border-color: var(--primary-700);
      }
    }
    .view-controls {
      display: flex;
      gap: 0.5rem;
      button {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0.6rem 0.9rem;
        font-size: 1rem;
        border-radius: var(--border-radius);
        background: var(--card);
        color: var(--text);
        border: 1px solid var(--dark-500);
        transition: background 0.13s, color 0.13s, border-color 0.13s;
        
        &:hover {
          background: color-mix(in srgb, var(--primary), 10%, transparent);
          color: var(--primary-900);
          border-color: var(--primary-800);
        }
        
        &.is-active {
          background: color-mix(in srgb, var(--primary), 10%, transparent);
          color: var(--primary-900);
          border-color: var(--primary-800);
        }
        
        &:focus {
          outline: none;
        }
      }
    }
  }

  :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
