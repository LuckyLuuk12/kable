<script lang="ts">
import { ProjectsPage } from "$lib";
</script>

<svelte:head>
  <title>Shaders - Kable</title>
</svelte:head>

<ProjectsPage projectType="shader" />

<!-- TODO: Make/use the components/projects/ProjectsPage.svelte with "shader" as type and reuse this accross the mods, resourcepacks pages.
<div class="page shaders-page">
  <div class="tab-navigation">
    <button class="tab-btn" class:active={currentTab === "installed"} onclick={() => (currentTab = "installed")}> 📦 Installed Shaders </button>
    <button class="tab-btn" class:active={currentTab === "browse"} onclick={() => (currentTab = "browse")}> 🔍 Browse Shaders </button>

    {#if $selectedInstallation}
      <div class="current-installation">
        Selected: <strong>{$selectedInstallation.name}</strong>
      </div>
    {/if}
  </div>

  <div class="tab-content">
    {#if currentTab === "installed"}
      <InstallationShaders bind:selectedId={sharedInstallationId} />
    {:else if currentTab === "browse"}
      <ShaderBrowser bind:selectedInstallationId={sharedInstallationId} ondownload={handleShaderDownload} />
    {/if}
  </div>
</div> -->

<style lang="scss">
.shaders-page {
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
      color: var(--text);
    }
  }
}

.tab-content {
  flex: 1;
  overflow: hidden;
}
</style>
