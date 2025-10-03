<script lang="ts">
  import { settings, SettingsUI } from '$lib';

  let saveStatus = '';

</script>

<div class="settings-page">
  <div class="page-header">
    <h1>Settings</h1>
    <p>Configure your launcher preferences</p>
    {#if saveStatus}
      <div class="warning-card" class:success={saveStatus.includes('success')} class:error={saveStatus.includes('Failed')}>
        {saveStatus}
      </div>
    {/if}
  </div>

  {#if $settings}
    <SettingsUI />
  {:else}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading settings...</p>
    </div>
  {/if}
</div>

<style lang="scss">

  .settings-page {
    max-width: 100%;
    max-height: 100vh;
    overflow: auto;
    margin: 0;
  }

  .warning-card {
    position: fixed;
    bottom: 1rem;
    width: fit-content;
    @extend .card !optional;
    padding: 1rem;
    margin-bottom: 1.5rem;
    color: var(--text);
    background: var(--container);
    border-radius: 0.75rem;
    font-size: 0.875rem;

    &.success {
      border-color: var(--green);
      background-color: var(--green-900);
    }
    
    &.error {
      border-color: var(--red-700);
      background-color: var(--red-900);
    }
  }

  .loading-state {
    @extend .empty-state !optional;
    padding: 4rem;
    
    .spinner {
      width: 40px;
      height: 40px;
      border: 4px solid var(--input);
      border-top: 4px solid var(--primary);
      border-radius: 50%;
      animation: spin 1s linear infinite;
      margin-bottom: 1rem;
    }
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
</style>
