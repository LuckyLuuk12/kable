<!--
SkinViewer3D - Responsive 3D Minecraft skin viewer using skinview3d.

Automatically fills its parent when width or height is omitted.
Supports configurable zoom, interaction-aware gesture orchestration,
and globally registered animation keybinds.

Animation modes:
- all: Ambient gestures and interaction reactions.
- idle-only: Ambient gestures without interaction reactions.
- interaction-only: Interaction reactions without ambient gestures.
- none: Static rendering without animation or interaction.
-->

<script lang="ts">
import * as skinview3d from "skinview3d";
import { onDestroy, onMount } from "svelte";

import { error } from "#lib";
import { DEFAULT_GESTURES, DEFAULT_INTERACTION_GESTURES, IdleOrchestrator } from "./animations/orchestrator";

type AnimationMode = "all" | "idle-only" | "interaction-only" | "none";

interface Props {
  skinUrl: string;
  width?: number | null;
  height?: number | null;
  model?: "auto" | "slim" | "classic";
  animationMode?: AnimationMode;
  zoom?: number;
}

let { skinUrl, width = null, height = null, model = "auto", animationMode = "all", zoom = 1 }: Props = $props();

let canvas = $state<HTMLCanvasElement | undefined>(undefined);

// Keep external class instances unproxied.
let skinViewer = $state.raw<any>(null);
let idleOrchestrator: IdleOrchestrator | null = null;
let resizeObserver: ResizeObserver | null = null;

let containerWidth = $state(0);
let containerHeight = $state(0);

let pointerDownPosition: { x: number; y: number } | null = null;
let pointerIsDragging = false;
let pointerDidDrag = false;

let skinLoadVersion = 0;
let skinLoadQueue: Promise<void> = Promise.resolve();

const DRAG_THRESHOLD = 4;

function getDimensions() {
  return {
    width: width ?? containerWidth,
    height: height ?? containerHeight,
  };
}

function disposeIdleOrchestrator() {
  idleOrchestrator?.dispose();
  idleOrchestrator = null;
}

function isStaticMode(): boolean {
  return animationMode === "none";
}

/**
 * Configure rendering, animation and camera interaction.
 */
function configureAnimationMode() {
  if (!skinViewer) return;

  disposeIdleOrchestrator();

  pointerDownPosition = null;
  pointerIsDragging = false;
  pointerDidDrag = false;

  const isStatic = isStaticMode();

  if (skinViewer.controls) {
    skinViewer.controls.enableRotate = !isStatic;
    skinViewer.controls.enableZoom = !isStatic;
    skinViewer.controls.enablePan = false;
  }

  if (isStatic) {
    skinViewer.renderPaused = true;
    skinViewer.animation = null;
    skinViewer.render();
    return;
  }

  skinViewer.renderPaused = false;

  const idle = new skinview3d.IdleAnimation();
  skinViewer.animation = idle;

  idleOrchestrator = new IdleOrchestrator(
    idle,
    DEFAULT_GESTURES,
    {
      initialDelayMin: 5,
      initialDelayMax: 12,
      cooldownMin: 10,
      cooldownMax: 25,
      enabled: animationMode !== "interaction-only",
    },
    DEFAULT_INTERACTION_GESTURES,
  );
}

/**
 * Determine whether a keyboard event originated in an editable element.
 */
function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof Element)) return false;

  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    target.closest('[contenteditable]:not([contenteditable="false"]), [role="textbox"]') !== null
  );
}

/**
 * Normalize a keyboard event into a keybind string.
 */
function normalizeKeybind(event: KeyboardEvent): string | null {
  if (event.key === "Control" || event.key === "Alt" || event.key === "Shift" || event.key === "Meta" || event.key === "Dead" || event.isComposing) {
    return null;
  }

  const key = event.key.toLowerCase();
  if (!key) return null;

  const modifiers: string[] = [];

  if (event.ctrlKey) modifiers.push("ctrl");
  if (event.altKey) modifiers.push("alt");
  if (event.shiftKey) modifiers.push("shift");
  if (event.metaKey) modifiers.push("meta");

  if (modifiers.includes(key)) return null;

  return [...modifiers, key].join("+");
}

/**
 * Forward global keyboard events to the registered gestures.
 */
function handleGlobalKeyDown(event: KeyboardEvent) {
  if (isStaticMode() || event.repeat || isEditableTarget(event.target)) return;

  const hasOtherModifier = event.shiftKey || event.metaKey;
  const hasOnlyCtrl = event.ctrlKey && !event.altKey;

  if (hasOtherModifier || hasOnlyCtrl) return;

  if (!canvas?.matches(":focus")) return;

  const keybind = normalizeKeybind(event);
  if (!keybind) return;

  if (idleOrchestrator?.onKeybind(keybind)) {
    event.preventDefault();
  }
}

/**
 * Apply the requested zoom.
 */
function applyZoom() {
  if (!skinViewer || !Number.isFinite(zoom) || zoom <= 0) return;

  try {
    skinViewer.zoom = zoom;

    if (skinViewer.renderPaused) {
      skinViewer.render();
    }
  } catch (err) {
    error("Failed to update skin viewer zoom:", err);
  }
}

/**
 * Create the viewer once its canvas has usable dimensions.
 *
 * Skin loading is handled separately so that model and URL changes
 * use the same asynchronous loading path.
 */
function initSkinViewer() {
  if (!canvas || skinViewer) return;

  const dimensions = getDimensions();

  if (dimensions.width <= 0 || dimensions.height <= 0) return;

  try {
    const options: Record<string, unknown> = {
      canvas,
      width: dimensions.width,
      height: dimensions.height,
      pixelRatio: 1,
      renderPaused: animationMode === "none",
    };

    skinViewer = new skinview3d.SkinViewer(options as any);

    skinViewer.camera.position.set(30, 20, 50);
    skinViewer.camera.lookAt(0, 10, 0);

    applyZoom();
    configureAnimationMode();
  } catch (err) {
    error("Failed to initialize SkinViewer3D:", err);

    disposeIdleOrchestrator();

    skinViewer?.dispose();
    skinViewer = null;
  }
}

/**
 * Keep rendering dimensions synchronized with the parent.
 */
function updateViewerSize() {
  if (!canvas) return;

  const parent = canvas.parentElement;
  if (!parent) return;

  const bounds = parent.getBoundingClientRect();

  containerWidth = Math.max(0, Math.floor(bounds.width));
  containerHeight = Math.max(0, Math.floor(bounds.height));

  const dimensions = getDimensions();

  if (dimensions.width <= 0 || dimensions.height <= 0) return;

  if (!skinViewer) {
    initSkinViewer();
    return;
  }

  try {
    if (skinViewer.width !== dimensions.width || skinViewer.height !== dimensions.height) {
      skinViewer.setSize(dimensions.width, dimensions.height);

      if (skinViewer.renderPaused) {
        skinViewer.render();
      }
    }
  } catch (err) {
    error("Failed to resize SkinViewer3D:", err);
  }
}

/**
 * Whether this animation mode accepts pointer interaction reactions.
 */
function acceptsInteractions(): boolean {
  return animationMode === "all" || animationMode === "interaction-only";
}

function handlePointerEnter() {
  if (acceptsInteractions()) {
    idleOrchestrator?.onHover();
  }
}

function handlePointerDown(event: PointerEvent) {
  if (isStaticMode() || event.button !== 0) return;

  pointerDownPosition = {
    x: event.clientX,
    y: event.clientY,
  };

  pointerIsDragging = false;
  pointerDidDrag = false;
}

function handlePointerMove(event: PointerEvent) {
  if (!pointerDownPosition || !(event.buttons & 1)) return;

  const dx = event.clientX - pointerDownPosition.x;
  const dy = event.clientY - pointerDownPosition.y;

  if (!pointerIsDragging && Math.hypot(dx, dy) >= DRAG_THRESHOLD) {
    pointerIsDragging = true;
    pointerDidDrag = true;

    if (acceptsInteractions()) {
      idleOrchestrator?.onDrag();
    }
  }
}

function handlePointerUp() {
  if (pointerIsDragging && acceptsInteractions()) {
    idleOrchestrator?.onDragEnd();
  }

  pointerDownPosition = null;
  pointerIsDragging = false;
}

function handlePointerCancel() {
  handlePointerUp();
}

function handleClick() {
  if (pointerDidDrag) {
    pointerDidDrag = false;
    return;
  }

  if (acceptsInteractions()) {
    idleOrchestrator?.onClick();
  }
}

function handlePointerLeave() {
  if (pointerIsDragging && acceptsInteractions()) {
    idleOrchestrator?.onDragEnd();
  }

  if (acceptsInteractions()) {
    idleOrchestrator?.onLeave();
  }

  pointerDownPosition = null;
  pointerIsDragging = false;
  pointerDidDrag = false;
}

onMount(() => {
  window.addEventListener("keydown", handleGlobalKeyDown);

  if (canvas?.parentElement) {
    resizeObserver = new ResizeObserver(updateViewerSize);
    resizeObserver.observe(canvas.parentElement);
  }

  updateViewerSize();
});

onDestroy(() => {
  window.removeEventListener("keydown", handleGlobalKeyDown);

  resizeObserver?.disconnect();
  resizeObserver = null;

  // Invalidate any pending skin-load completion.
  skinLoadVersion++;

  disposeIdleOrchestrator();

  if (skinViewer) {
    skinViewer.dispose();
    skinViewer = null;
  }
});

/**
 * Initialize or resize the viewer when its container changes.
 */
$effect(() => {
  const currentCanvas = canvas;
  const explicitWidth = width;
  const explicitHeight = height;

  if (currentCanvas) {
    void explicitWidth;
    void explicitHeight;
    updateViewerSize();
  }
});

/**
 * Load the current skin and model.
 *
 * Loads are serialized so an older asynchronous request cannot finish
 * after a newer request and overwrite the currently requested skin.
 */
$effect(() => {
  const currentViewer = skinViewer;
  const nextSkinUrl = skinUrl;
  const nextModel = model;

  if (!currentViewer || !nextSkinUrl) return;

  const requestVersion = ++skinLoadVersion;

  const loadOptions: skinview3d.SkinViewer["playerObject"]["skin"] extends never ? never : skinview3d.SkinLoadOptions = {
    model: nextModel === "classic" ? "default" : nextModel === "slim" ? "slim" : "auto-detect",
    makeVisible: false,
  };

  skinLoadQueue = skinLoadQueue
    .catch(() => undefined)
    .then(async () => {
      if (requestVersion !== skinLoadVersion || currentViewer !== skinViewer) {
        return;
      }

      try {
        await currentViewer.loadSkin(nextSkinUrl, loadOptions);

        if (requestVersion !== skinLoadVersion || currentViewer !== skinViewer) {
          return;
        }

        currentViewer.playerObject.skin.visible = true;

        if (currentViewer.renderPaused) {
          currentViewer.render();
        }
      } catch (err) {
        if (requestVersion === skinLoadVersion) {
          error("Failed to load skin:", err);
        }
      }
    });
});

/**
 * Update zoom when its prop changes.
 */
$effect(() => {
  const nextZoom = zoom;

  if (skinViewer && Number.isFinite(nextZoom) && nextZoom > 0) {
    applyZoom();
  }
});

/**
 * Reconfigure rendering and animation when the mode changes.
 */
$effect(() => {
  const nextMode = animationMode;

  if (skinViewer) {
    void nextMode;
    configureAnimationMode();
  }
});
</script>

<canvas
  bind:this={canvas}
  class="skin-viewer-canvas"
  tabindex="0"
  style:width={width === null ? "100%" : `${width}px`}
  style:height={height === null ? "100%" : `${height}px`}
  onpointerenter={handlePointerEnter}
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
  onpointercancel={handlePointerCancel}
  onpointerleave={handlePointerLeave}
  onclick={handleClick}
></canvas>

<style>
.skin-viewer-canvas {
  display: block;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  border-radius: 8px;
  outline-offset: 2px;
}
</style>
