<script lang="ts">
import type { KableAccount } from "#lib";
import { Icon, api, app, error } from "#lib";
import * as skinview3d from "skinview3d";

let {
  size = 40,
  account,
}: {
  size?: number;
  account?: KableAccount;
} = $props();

const skinsService = app.skinsService;

let canvas: HTMLCanvasElement | undefined = $state();
let isLoading = $state(false);
let hasError = $state(false);

let activeSkinUrl = $derived(skinsService.currentSkin?.url ?? skinsService.allSkins.find((skin) => skin.is_current)?.url ?? null);

$effect(() => {
  const element = canvas;
  const targetAccount = account;
  const fallbackUrl = targetAccount ? null : activeSkinUrl;
  const viewerSize = size;

  if (!element) return;

  let cancelled = false;
  let viewer: skinview3d.SkinViewer | null = null;

  hasError = false;
  isLoading = true;

  async function initialize() {
    let url = fallbackUrl;

    if (targetAccount) {
      const currentSkin = await api.getCurrentSkinInfo(targetAccount);
      url = currentSkin.url;
    }

    if (cancelled) return;

    if (!url) {
      isLoading = false;
      return;
    }

    viewer = new skinview3d.SkinViewer({
      canvas: element,
      width: viewerSize,
      height: viewerSize,
      pixelRatio: 1,
      renderPaused: true,
    });
    try {
      viewer.camera.setViewOffset(215, 500, 5, -5, 210, 200);
    } catch (err) {
      error("Failed to set camera view offset:", err);
    }
    if (viewer.controls) {
      viewer.controls.enableRotate = false;
      viewer.controls.enableZoom = false;
      viewer.controls.enablePan = false;
    }

    await viewer.loadSkin(url);

    if (cancelled) return;

    viewer.render();

    isLoading = false;
    hasError = false;
  }

  void initialize().catch((err: unknown) => {
    if (cancelled) return;

    error("Failed to load player skin:", err);
    isLoading = false;
    hasError = true;
  });

  return () => {
    cancelled = true;
    viewer?.dispose();
    viewer = null;
  };
});
</script>

<div class="player-head" style:width="{size}px" style:height="{size}px">
  <canvas bind:this={canvas} width={size} height={size} class:canvas-hidden={isLoading || hasError}></canvas>

  {#if isLoading}
    <div class="loading">
      <Icon name="loader" size="sm" />
    </div>
  {:else if hasError}
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
