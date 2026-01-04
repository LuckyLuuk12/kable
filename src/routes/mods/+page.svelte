<script lang="ts">
import { InstallationMods, ModBrowser, Launcher, Icon } from "$lib";
import { ProviderKind, selectedInstallation } from "$lib";
import type { KableInstallation } from "$lib";
import { launchSound } from "$lib/actions";

let currentTab: "installed" | "browse" = "installed";
let isLaunching = false;

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

// Handle launching the selected installation
async function handleLaunch() {
  if (!$selectedInstallation || isLaunching) return;

  isLaunching = true;
  try {
    const result = await Launcher.launchInstallation($selectedInstallation);
    if (!result.success) {
      alert(`Launch failed: ${result.error || "Unknown error"}`);
    }
  } catch (error) {
    console.error("Failed to launch installation:", error);
    alert(`Launch failed: ${error}`);
  } finally {
    setTimeout(() => {
      isLaunching = false;
    }, 2000);
  }
}
</script>

<div class="mods-page">
  <!-- Tab Navigation -->
  <div class="tab-navigation">
    <button
      class="tab-btn"
      class:active={currentTab === "installed"}
      on:click={() => (currentTab = "installed")}>
      📦 Installed Mods
    </button>
    <button
      class="tab-btn"
      class:active={currentTab === "browse"}
      on:click={() => (currentTab = "browse")}>
      🔍 Browse Mods
    </button>

    {#if $selectedInstallation}
      <div class="current-installation">
        Selected: <strong>{$selectedInstallation.name}</strong>
      </div>

      <button
        class="launch-btn"
        on:click={handleLaunch}
        use:launchSound
        disabled={isLaunching}>
        {#if isLaunching}
          <Icon name="refresh" size="sm" forceType="svg" className="spin" />
          <span>Launching...</span>
        {:else}
          <Icon name="play" size="sm" forceType="svg" />
          <span>Launch</span>
        {/if}
      </button>
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

  .launch-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 1.2rem;
    border: 1px solid var(--dark-600);
    border-radius: 0.5rem;
    background: linear-gradient(
      135deg,
      color-mix(in srgb, var(--green), 90%, transparent) 0%,
      color-mix(in srgb, var(--green), 75%, transparent) 100%
    );
    color: var(--text-white);
    font-weight: 600;
    font-size: 0.9em;
    cursor: pointer;
    transition: all 0.15s;
    border-color: color-mix(in srgb, var(--green), 40%, transparent);
    box-shadow: 0 2px 6px color-mix(in srgb, var(--green), 20%, transparent);

    &:hover:not(:disabled) {
      background: linear-gradient(
        135deg,
        var(--green) 0%,
        color-mix(in srgb, var(--green), 90%, transparent) 100%
      );
      transform: translateY(-1px);
      box-shadow: 0 3px 10px color-mix(in srgb, var(--green), 30%, transparent);
    }

    &:active:not(:disabled) {
      transform: translateY(0);
    }

    &:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }

    :global(.spin) {
      animation: spin 1s linear infinite;
    }
  }
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
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

    .launch-btn {
      width: 100%;
      justify-content: center;
    }
  }
}
</style>
