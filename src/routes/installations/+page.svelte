<script lang="ts">
  import { InstallationsList, CreateInstallationModal, EditInstallationModal, Icon, type KableInstallation } from '$lib';

  let showCreateModal = false;
  let showEditModal = false;
  let editingInstallation: KableInstallation | null = null;
  let createModalRef;
  let editModalRef;

  let isSmall = false;
  let isGrid = true;

  function editInstallation(installation: KableInstallation) {
    editingInstallation = installation;
    showEditModal = true;
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
      on:click={() => showCreateModal = true}
    >
      <Icon name="plus" size="md" />
      New Installation
    </button>
    <div class="view-controls">
      <button 
        class="btn btn-secondary" 
        on:click={() => isGrid = !isGrid} 
        class:is-active={isGrid}
        title={isGrid ? 'Switch to list view' : 'Switch to grid view'}
      >
        <Icon name={isGrid ? 'list' : 'grid'} size="md" />
      </button>
      <button class="btn btn-secondary" on:click={() => isSmall = !isSmall} class:is-active={isSmall} title="Toggle compact mode">
        <Icon name="minimize" size="md" />
      </button>
    </div>
  </div>

  <InstallationsList {isGrid} isSmall={isSmall} on:edit={(e) => editInstallation(e.detail)} />

  {#if showCreateModal}
    <CreateInstallationModal bind:this={createModalRef} on:close={() => showCreateModal = false} />
  {/if}
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
        color: $text;
        text-align: center;
      }
      p {
        margin: 0;
        color: $placeholder;
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
      color: $primary;
      border-radius: $border-radius;
      box-shadow: none;
      border: 1.5px solid $primary;
      font-weight: 600;
      transition: color 0.13s, background 0.13s, border 0.13s;
      &:hover, &:focus {
        background: rgba($primary, 0.10);
        color: $primary-900;
        border-color: $primary-700;
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
        border-radius: $border-radius;
        background: $card;
        color: $text;
        border: 1px solid $dark-500;
        transition: background 0.13s, color 0.13s;
        &.is-active, &:hover, &:focus {
          background: rgba($primary, 0.10);
          color: $primary-900;
          border-color: $primary-800;
        }
      }
    }
  }
</style>
