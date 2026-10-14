<!-- @component
PlayerHead - Renders a player's Minecraft head from their current skin

Uses skinview3d to render just the head portion of a player's skin.
Falls back to user icon if no skin is available.

@prop {KableAccount | null} account - The account to display the head for
@prop {number} [size=40] - Size of the head in pixels

@example
```svelte
◄PlayerHead account={$currentAccount} size={64} /►
```
-->
<script lang="ts">
import { type KableAccount, Icon } from "$lib";
import * as skinview3d from "skinview3d";
import { onDestroy } from "svelte";

let { account = null, size = 40 }: { account?: KableAccount | null; size?: number } = $props();

let canvas: HTMLCanvasElement | undefined = $state();
let skinViewer: skinview3d.SkinViewer | null = $state(null);
let skinUrl = $state("");
let isLoading = $state(true);
let hasError = $state(false);
let currentUuid = $state("");

$effect(() => {
  const uuid = account?.minecraft_profile?.id;

  if (!uuid) {
    currentUuid = "";
    skinUrl = "";
    hasError = true;
    isLoading = false;
    return;
  }

  if (uuid === currentUuid) {
    return;
  }

  disposeSkinViewer();

  currentUuid = uuid;
  loadSkin(uuid);
});

$effect(() => {
  if (!canvas || !skinUrl || skinViewer) {
    return;
  }

  initSkinViewer();
});

async function loadSkin(uuid: string) {
  isLoading = true;
  hasError = false;

  try {
    const url = `https://crafatar.com/skins/${uuid}`;

    skinUrl = url;
    isLoading = false;
  } catch (error) {
    console.error("Failed to load skin for UUID:", uuid, error);

    hasError = true;
    isLoading = false;
  }
}

function initSkinViewer() {
  if (!canvas || !skinUrl) {
    return;
  }

  disposeSkinViewer();

  try {
    skinViewer = new skinview3d.SkinViewer({
      canvas,
      width: size,
      height: size,
      skin: skinUrl,
    });

    skinViewer.camera.setViewOffset(215, 500, 5, -5, 210, 200);

    if (skinViewer.controls) {
      skinViewer.controls.enableRotate = false;
      skinViewer.controls.enableZoom = false;
      skinViewer.controls.enablePan = false;
    }

    skinViewer.animation = new skinview3d.IdleAnimation();

    hasError = false;
    isLoading = false;
  } catch (error) {
    console.error("Failed to initialize skin viewer:", error);

    hasError = true;
    isLoading = false;
  }
}

function disposeSkinViewer() {
  if (!skinViewer) {
    return;
  }

  skinViewer.dispose();
  skinViewer = null;
}

onDestroy(() => {
  disposeSkinViewer();
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
  background: linear-gradient(135deg, var(--color-accent), var(--primary-600));
  color: white;
  border-radius: 50%;
}
</style>
