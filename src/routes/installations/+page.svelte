<script lang="ts">
  import InstallationsList from '$lib/components/InstallationsList.svelte';
  import CreateInstallationModal from '$lib/components/CreateInstallationModal.svelte';
  import EditInstallationModal from '$lib/components/EditInstallationModal.svelte';
  import type { KableInstallation } from '$lib/types';

  let showCreateModal = false;
  let showEditModal = false;
  let editingInstallation: KableInstallation | null = null;
  let createModalRef;
  let editModalRef;

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
    <button 
      class="btn btn-primary" 
      on:click={() => showCreateModal = true}
    >
      New Installation
    </button>
  </div>

  <InstallationsList on:edit={(e) => editInstallation(e.detail)} />

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
    // max-width: 1200px;
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
</style>
