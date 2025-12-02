<script lang="ts">
import { ShaderBrowser, InstallationShaders } from "$lib";
import { selectedInstallation, installations } from "$lib";
import type { KableInstallation, ShaderDownload } from "$lib";

let currentTab: "installed" | "browse" = "installed";
let sharedInstallationId: string = "global";

// Sync shared installation ID with the store
$: if ($selectedInstallation) {
  sharedInstallationId = $selectedInstallation.id;
}

// Update selected installation when shared ID changes
$: {
  if (sharedInstallationId === "global") {
    selectedInstallation.set(null);
  } else {
    const inst = $installations.find((i) => i.id === sharedInstallationId);
    if (inst) {
      selectedInstallation.set(inst);
    }
  }
}

// Handle shader download from browser
async function handleShaderDownload(event: {
  shader: ShaderDownload;
  installation: KableInstallation | null;
}) {
  const { shader, installation } = event;

  try {
    const { ShadersService } = await import("$lib");

    if (installation) {
      // Download to specific installation (dedicated mode)
      await ShadersService.downloadShaderToDedicated(shader, installation);
      console.log(
        `Successfully downloaded shader ${shader.name} to ${installation.name}`,
      );
    } else {
      // Download globally
      await ShadersService.downloadShaderGlobal(shader);
      console.log(`Successfully downloaded shader ${shader.name} globally`);
    }
  } catch (error) {
    console.error("Failed to download shader:", error);
    alert(`Failed to download shader: ${error}`);
  }
}
</script>

<svelte:head>
  <title>Shaders - Kable</title>
</svelte:head>

<div class="page shaders-page">
  <!-- Tab Navigation -->
  <div class="tab-navigation">
    <button
      class="tab-btn"
      class:active={currentTab === "installed"}
      on:click={() => (currentTab = "installed")}
    >
      📦 Installed Shaders
    </button>
    <button
      class="tab-btn"
      class:active={currentTab === "browse"}
      on:click={() => (currentTab = "browse")}
    >
      🔍 Browse Shaders
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
      <InstallationShaders bind:selectedId={sharedInstallationId} />
    {:else if currentTab === "browse"}
      <ShaderBrowser bind:selectedInstallationId={sharedInstallationId} ondownload={handleShaderDownload} />
    {/if}
  </div>
</div>

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
      color: var(--text);
    }
  }
}

.tab-content {
  flex: 1;
  overflow: hidden;
}
</style>
