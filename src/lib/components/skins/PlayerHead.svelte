<!-- @component
PlayerHead - Renders a player's head from their active Minecraft skin.

Uses the active skin provided by the skins service.
Falls back to a user icon if no skin is available.

@prop {number} [size=40] - Size of the head in pixels
-->
<script lang="ts">
import { Icon, app, error } from "$lib";
import * as skinview3d from "skinview3d";
import { onDestroy } from "svelte";

let {
  size = 40,
}: {
  size?: number;
} = $props();

const skinsService = app.skinsService;

let canvas: HTMLCanvasElement | undefined = $state();
let skinViewer: skinview3d.SkinViewer | null = null;
let isLoading = $state(false);
let hasError = $state(false);

let skinUrl = $derived(skinsService.currentSkin?.url ?? skinsService.allSkins.find((skin) => skin.is_current)?.url ?? null);

function disposeSkinViewer(): void {
  if (!skinViewer) return;

  skinViewer.dispose();
  skinViewer = null;
}

$effect(() => {
  const element = canvas;
  const url = skinUrl;
  const viewerSize = size;

  if (!element) return;

  let cancelled = false;

  disposeSkinViewer();
  hasError = false;

  if (!url) {
    isLoading = false;
    return;
  }

  isLoading = true;

  try {
    const viewer = new skinview3d.SkinViewer({
      canvas: element,
      width: viewerSize,
      height: viewerSize,
      skin: url,
    });

    skinViewer = viewer;

    // Preserve the original head framing.
    viewer.camera.setViewOffset(215, 500, 5, -5, 210, 200);

    if (viewer.controls) {
      viewer.controls.enableRotate = false;
      viewer.controls.enableZoom = false;
      viewer.controls.enablePan = false;
    }

    viewer.animation = new skinview3d.IdleAnimation();

    viewer
      .loadSkin(url)
      .then(() => {
        if (cancelled) return;
        isLoading = false;
        hasError = false;
      })
      .catch((err: unknown) => {
        if (cancelled) return;

        error("Failed to load player skin:", err);
        isLoading = false;
        hasError = true;
      });
  } catch (err) {
    error("Failed to initialize player skin viewer:", err);
    isLoading = false;
    hasError = true;
  }

  return () => {
    cancelled = true;
    disposeSkinViewer();
  };
});

onDestroy(() => {
  disposeSkinViewer();
});
</script>

<div class="player-head" style:width="{size}px" style:height="{size}px">
  {#if skinUrl}
    <canvas bind:this={canvas} width={size} height={size} class:canvas-hidden={isLoading || hasError}></canvas>
  {/if}

  {#if isLoading}
    <div class="loading">
      <Icon name="loader" size="sm" />
    </div>
  {:else if hasError || !skinUrl}
    <div class="fallback">
      <Icon name="user" size="sm" />
    </div>
  {/if}
</div>

<style lang="scss">
.player-head {
  position: relative;
  display: inline-block;
  flex-shrink: 0;
  border-radius: 50%;
  overflow: hidden;
}

canvas {
  display: block;
  width: 100%;
  height: 100%;
  border-radius: 50%;
}

canvas.canvas-hidden {
  visibility: hidden;
}

.loading,
.fallback {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--color-accent), var(--primary-600));
  color: white;
  border-radius: 50%;
}
</style>
