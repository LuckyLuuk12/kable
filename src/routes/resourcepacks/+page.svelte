<script lang="ts">
import { ProjectsPage } from "$lib";
</script>

<svelte:head>
  <title>Resource Packs - Kable</title>
</svelte:head>

<ProjectsPage projectType="resourcepack" />

<!-- <div class="page resourcepacks-page">

  <div class="tab-navigation">
    <button class="tab-btn" class:active={currentTab === "installed"} on:click={() => (currentTab = "installed")}> 📦 Installed Packs </button>
    <button class="tab-btn" class:active={currentTab === "browse"} on:click={() => (currentTab = "browse")}> 🔍 Browse Packs </button>

    {#if $selectedInstallation}
      <div class="current-installation">
        Selected: <strong>{$selectedInstallation.name}</strong>
      </div>
    {/if}
  </div>

  <div class="tab-content">
    {#if currentTab === "installed"}
      <InstallationResourcePacks />
    {:else if currentTab === "browse"}
      <ResourcePackBrowser />
    {/if}
  </div>
</div> -->

<style lang="scss">
.resourcepacks-page {
  max-width: 100vw;
  height: 100vh;
  max-height: 100%;
  display: flex;
  flex-direction: column;
}

.tab-navigation {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding-bottom: 0.5rem;
  background: $color-background;
  border-bottom: 1px solid color-mix(in srgb, $color-accent, 8%, transparent);

  .tab-btn {
    padding: 0.6rem 1.2rem;
    border: 1px solid $color-border;
    border-radius: 0.5rem;
    background: var(--card);
    color: var(--text);
    font-weight: 500;
    font-size: 0.9em;
    cursor: pointer;
    transition: all 0.15s;

    &:hover {
      border-color: $color-accent;
      background: color-mix(in srgb, $color-accent, 5%, transparent);
    }

    &.active {
      background: linear-gradient(135deg, $color-accent 0%, $color-accent-secondary 100%);
      color: var(--text-white);
      border-color: var(--text-transparent);
      box-shadow: 0 2px 8px color-mix(in srgb, $color-accent, 25%, transparent);
    }
  }

  .current-installation {
    margin-left: auto;
    padding: 0.6rem 1rem;
    background: color-mix(in srgb, $color-accent, 8%, transparent);
    border: 1px solid color-mix(in srgb, $color-accent, 15%, transparent);
    border-radius: 0.5rem;
    font-size: 0.85em;
    color: $color-accent;

    strong {
      font-weight: 600;
    }
  }
}

.tab-content {
  flex: 1;
  overflow: hidden;
}

@media (max-width: 768px) {
  .tab-navigation {
    flex-direction: column;
    align-items: stretch;
    gap: 0.75rem;

    .current-installation {
      margin-left: 0;
      text-align: center;
    }
  }
}
</style>
