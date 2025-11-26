<script lang="ts">
import { InstallationMods, ModBrowser } from "$lib";
import { ProviderKind, selectedInstallation } from "$lib";
import type { KableInstallation } from "$lib";

let currentTab: "installed" | "browse" = "installed";

// Handle mod download from browser
async function handleModDownload(event: {
  modId: string;
  versionId?: string;
  installation: KableInstallation;
}) {
  const { modId, versionId, installation } = event;

  try {
    // Use the ModsService to download the mod
    const { ModsService } = await import("$lib");
    const modsService = new ModsService(ProviderKind.Modrinth); // Use appropriate provider
    await modsService.downloadMod(modId, versionId || null, installation);

    // Show success message
    console.log(`Successfully downloaded mod ${modId} to ${installation.name}`);
  } catch (error) {
    console.error("Failed to download mod:", error);
    alert(`Failed to download mod: ${error}`);
  }
}
</script>

<div class="mods-page">
  <!-- Tab Navigation -->
  <div class="tab-navigation">
    <button
      class="tab-btn"
      class:active={currentTab === "installed"}
      on:click={() => (currentTab = "installed")}
    >
      📦 Installed Mods
    </button>
    <button
      class="tab-btn"
      class:active={currentTab === "browse"}
      on:click={() => (currentTab = "browse")}
    >
      🔍 Browse Mods
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
      <InstallationMods />
    {:else if currentTab === "browse"}
      <ModBrowser ondownloadmod={handleModDownload} />
    {/if}
  </div>
</div>

<style lang="scss">
.mods-page {
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
