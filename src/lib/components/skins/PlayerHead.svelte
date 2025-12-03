<!-- @component
PlayerHead - Renders a player's Minecraft head from their current skin

Uses skinview3d to render just the head portion of a player's skin.
Falls back to user icon if no skin is available.

@prop {LauncherAccount | null} account - The account to display the head for
@prop {number} [size=40] - Size of the head in pixels

@example
```svelte
◄PlayerHead account={$currentAccount} size={64} /►
```
-->
<script lang="ts">
import { onMount, onDestroy } from "svelte";
import * as skinview3d from "skinview3d";
import { Icon } from "$lib";
import * as skinsApi from "$lib/api/skins";
import type { LauncherAccount } from "$lib";

export let account: LauncherAccount | null = null;
export let size: number = 40;

let canvas: HTMLCanvasElement;
let skinViewer: any = null;
let skinUrl = "";
let isLoading = true;
let hasError = false;
let currentUuid = "";

// Load skin when account changes
$: {
  const uuid = account?.minecraft_profile?.id;
  if (uuid && uuid !== currentUuid) {
    // Clean up existing viewer when switching accounts
    if (skinViewer) {
      skinViewer.dispose();
      skinViewer = null;
    }
    skinUrl = "";
    currentUuid = uuid;
    loadSkin(uuid);
  } else if (!uuid) {
    hasError = true;
    isLoading = false;
  }
}

// Initialize viewer when canvas and skinUrl are both ready
$: if (canvas && skinUrl && !skinViewer) {
  initSkinViewer();
}

async function loadSkin(uuid: string) {
  if (!uuid) {
    console.log("No UUID provided to loadSkin");
    return;
  }

  console.log("Loading skin for UUID:", uuid);
  isLoading = true;
  hasError = false;

  try {
    // Fetch skin URL from backend (avoids CORS issues)
    const url = await skinsApi.getSkinUrlByUuid(uuid);
    console.log("Skin URL from backend:", url);

    skinUrl = url;
    isLoading = false;
    // Canvas will be ready after isLoading becomes false
    // initSkinViewer will be called by the reactive statement
  } catch (error) {
    console.error("Failed to load skin for UUID:", uuid, error);
    hasError = true;
    isLoading = false;
  }
}

function initSkinViewer() {
  if (!canvas || !skinUrl) {
    console.log("Cannot init skin viewer - missing canvas or skinUrl", {
      canvas: !!canvas,
      skinUrl,
    });
    return;
  }

  // Clean up existing viewer
  if (skinViewer) {
    skinViewer.dispose();
    skinViewer = null;
  }

  try {
    console.log("Initializing skin viewer with URL:", skinUrl);

    // Create a small viewer focused on the head
    skinViewer = new skinview3d.SkinViewer({
      canvas,
      width: size,
      height: size,
      skin: skinUrl,
    });

    // Position camera to show just the head (head is at y=24 in Minecraft model)
    // skinViewer.camera.position.x = -2;
    // skinViewer.camera.position.y = 12;
    // skinViewer.camera.position.z = 10;
    // skinViewer.camera.lookAt(0, 205, 0);
    // skinViewer.camera.setViewOffset(fullWidth: number, fullHeight: number, x: number, y: number, width: number, height: number)
    skinViewer.camera.setViewOffset(215, 500, 5, -5, 210, 200);

    // Disable controls for static head display
    if (skinViewer.controls) {
      skinViewer.controls.enableRotate = false;
      skinViewer.controls.enableZoom = false;
      skinViewer.controls.enablePan = false;
    }

    // Set idle animation
    if (skinview3d.IdleAnimation) {
      skinViewer.animation = new skinview3d.IdleAnimation();
    }

    hasError = false;
    isLoading = false;
    console.log("Skin viewer initialized successfully");
  } catch (error) {
    console.error("Failed to initialize skin viewer:", error);
    hasError = true;
    isLoading = false;
  }
}

onMount(() => {
  if (account?.minecraft_profile?.id) {
    loadSkin(account.minecraft_profile.id);
  }
});

onDestroy(() => {
  if (skinViewer) {
    skinViewer.dispose();
    skinViewer = null;
  }
});
</script>

<div class="player-head" style="width: {size}px; height: {size}px;">
  {#if isLoading}
    <div class="loading">
      <Icon name="loader" size="sm" />
    </div>
  {:else if hasError || !skinUrl}
    <div class="fallback">
      <Icon name="user" size="sm" />
    </div>
  {:else}
    <canvas bind:this={canvas} width={size} height={size}></canvas>
  {/if}
</div>

<style>
.player-head {
  position: relative;
  display: inline-block;
  border-radius: 50%;
  overflow: hidden;
}

canvas {
  display: block;
  width: 100%;
  height: 100%;
  border-radius: 50%;
}

.loading,
.fallback {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--primary), var(--primary-600));
  color: white;
  border-radius: 50%;
}
</style>
