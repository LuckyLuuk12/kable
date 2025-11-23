<script lang="ts">
import { ResourcePackBrowser, InstallationResourcePacks } from "$lib";
import { selectedInstallation } from "$lib";
import type { KableInstallation } from "$lib";

let currentTab: "installed" | "browse" = "installed";
</script>

<svelte:head>
  <title>Resource Packs - Kable</title>
</svelte:head>

<div class="page resourcepacks-page">
  <!-- Tab Navigation -->
  <div class="tab-navigation">
    <button
      class="tab-btn"
      class:active={currentTab === "installed"}
      on:click={() => (currentTab = "installed")}
    >
      📦 Installed Packs
    </button>
    <button
      class="tab-btn"
      class:active={currentTab === "browse"}
      on:click={() => (currentTab = "browse")}
    >
      🔍 Browse Packs
    </button>

    {#if $selectedInstallation}
      <div class="current-installation">
        Selected: <strong>{$selectedInstallation.name}</strong>
      </div>
    {/if}
  </div>

  <!-- Tab Content -->
  <div class="tab-content">
    {#if currentTab === "installed"}
      <InstallationResourcePacks />
    {:else if currentTab === "browse"}
      <ResourcePackBrowser />
    {/if}
  </div>
</div>

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
  background: var(--background);
  border-bottom: 1px solid color-mix(in srgb, var(--primary), 8%, transparent);

  .tab-btn {
    padding: 0.6rem 1.2rem;
    border: 1px solid var(--dark-600);
    border-radius: 0.5rem;
    background: var(--card);
    color: var(--text);
    font-weight: 500;
    font-size: 0.9em;
    cursor: pointer;
    transition: all 0.15s;

    &:hover {
      border-color: var(--primary);
      background: color-mix(in srgb, var(--primary), 5%, transparent);
    }

    &.active {
      background: linear-gradient(
        135deg,
        var(--primary) 0%,
        var(--secondary) 100%
      );
      color: var(--text-white);
      border-color: var(--text-transparent);
      box-shadow: 0 2px 8px color-mix(in srgb, var(--primary), 25%, transparent);
    }
  }

  .current-installation {
    margin-left: auto;
    padding: 0.6rem 1rem;
    background: color-mix(in srgb, var(--primary), 8%, transparent);
    border: 1px solid color-mix(in srgb, var(--primary), 15%, transparent);
    border-radius: 0.5rem;
    font-size: 0.85em;
    color: var(--primary);

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
