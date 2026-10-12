<!--
SkinViewer3D - Responsive 3D Minecraft skin viewer using skinview3d.

Automatically fills its parent when width or height is omitted.
Supports configurable zoom, interaction-aware gesture orchestration,
and globally registered animation keybinds.

Animation modes:
- all: Ambient gestures and interaction reactions.
- idle-only: Ambient gestures without interaction reactions.
- interaction-only: Interaction reactions without ambient gestures.
- none: No animation or gesture orchestration.
-->

<script lang="ts">
import * as skinview3d from "skinview3d";
import { onDestroy, onMount } from "svelte";

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
let loadedSkinUrl = $state<string | null>(null);
let initializedModel = $state<string | null>(null);

let pointerDownPosition: { x: number; y: number } | null = null;
let pointerIsDragging = false;
let pointerDidDrag = false;

const DRAG_THRESHOLD = 4;

/**
 * Resolve explicit dimensions first, falling back to the parent's size.
 */
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

/**
 * Set up the base idle animation and its gesture orchestrator.
 *
 * skinview3d retains ownership of camera rotation and zoom controls.
 * The orchestrator adds gesture animations to the same IdleAnimation.
 */
function configureAnimationMode() {
  if (!skinViewer) return;

  disposeIdleOrchestrator();

  pointerDownPosition = null;
  pointerIsDragging = false;
  pointerDidDrag = false;

  if (animationMode === "none") {
    skinViewer.animation = null;
    return;
  }

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
 *
 * Examples:
 *   D                 -> d
 *   Ctrl+Alt+D        -> ctrl+alt+d
 *   Shift+F           -> shift+f
 *   Escape            -> escape
 *
 * Modifier order is deterministic.
 */
function normalizeKeybind(event: KeyboardEvent): string | null {
  // Modifier keys alone are not actionable shortcuts.
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

  // Avoid duplicate modifier names if the key itself is a modifier.
  if (modifiers.includes(key)) return null;

  return [...modifiers, key].join("+");
}

/**
 * Forward global keyboard events to the registered gestures.
 *
 * This handler does not contain gesture-specific knowledge. The
 * orchestrator decides whether a normalized keybind is registered.
 */
function handleGlobalKeyDown(event: KeyboardEvent) {
  if (event.repeat || isEditableTarget(event.target)) return;

  // Do not take over browser, OS, or application shortcuts.
  // Unmodified keys and explicit Ctrl+Alt combinations are permitted.
  const hasCtrlAlt = event.ctrlKey && event.altKey;
  const hasOtherModifier = event.shiftKey || event.metaKey;
  const hasOnlyCtrl = event.ctrlKey && !event.altKey;

  if (hasOtherModifier || hasOnlyCtrl) return;

  // Require focus within this viewer so multiple viewers do not all
  // react to the same global key event.
  if (!canvas?.matches(":focus")) return;

  const keybind = normalizeKeybind(event);
  if (!keybind) return;

  if (idleOrchestrator?.onKeybind(keybind)) {
    event.preventDefault();
  }
}

/**
 * Apply the requested zoom without changing the viewer dimensions.
 */
function applyZoom() {
  if (!skinViewer || !Number.isFinite(zoom) || zoom <= 0) return;

  try {
    skinViewer.zoom = zoom;
  } catch (error) {
    console.error("Failed to update skin viewer zoom:", error);
  }
}

/**
 * Create the viewer once its canvas has usable dimensions.
 */
function initSkinViewer() {
  if (!canvas || !skinUrl || skinViewer) return;

  const dimensions = getDimensions();

  if (dimensions.width <= 0 || dimensions.height <= 0) return;

  try {
    const options: Record<string, unknown> = {
      canvas,
      width: dimensions.width,
      height: dimensions.height,
      skin: skinUrl,
    };

    if (model === "classic") {
      options.model = "default";
    } else if (model === "slim") {
      options.model = "slim";
    }

    skinViewer = new skinview3d.SkinViewer(options as any);
    loadedSkinUrl = skinUrl;
    initializedModel = model;

    skinViewer.camera.position.set(30, 20, 50);
    skinViewer.camera.lookAt(0, 10, 0);

    if (skinViewer.controls) {
      skinViewer.controls.enableRotate = true;
      skinViewer.controls.enableZoom = true;
      skinViewer.controls.enablePan = false;
    }

    applyZoom();
    configureAnimationMode();
  } catch (error) {
    console.error("Failed to initialize SkinViewer3D:", error);

    disposeIdleOrchestrator();

    skinViewer?.dispose();
    skinViewer = null;
    loadedSkinUrl = null;
    initializedModel = null;
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
      if (typeof skinViewer.setSize === "function") {
        skinViewer.setSize(dimensions.width, dimensions.height);
      } else {
        skinViewer.width = dimensions.width;
        skinViewer.height = dimensions.height;
      }
    }
  } catch (error) {
    console.error("Failed to resize SkinViewer3D:", error);
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
  if (event.button !== 0) return;

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

    // Trigger the gesture once. skinview3d still handles camera rotation.
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

  disposeIdleOrchestrator();

  if (skinViewer) {
    skinViewer.dispose();
    skinViewer = null;
  }

  loadedSkinUrl = null;
  initializedModel = null;
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
 * Load a new skin texture without recreating the viewer.
 */
$effect(() => {
  const currentViewer = skinViewer;
  const nextSkinUrl = skinUrl;

  if (!currentViewer || !nextSkinUrl || loadedSkinUrl === nextSkinUrl) {
    return;
  }

  try {
    currentViewer.loadSkin(nextSkinUrl);
    loadedSkinUrl = nextSkinUrl;
  } catch (error) {
    console.error("Failed to load skin:", error);
  }
});

/**
 * Reapply the model if the requested model changes.
 */
$effect(() => {
  const currentViewer = skinViewer;
  const nextModel = model;

  if (!currentViewer || initializedModel === nextModel) return;

  try {
    if (nextModel === "classic") {
      currentViewer.loadSkin(skinUrl, { model: "default" });
    } else if (nextModel === "slim") {
      currentViewer.loadSkin(skinUrl, { model: "slim" });
    } else {
      currentViewer.loadSkin(skinUrl);
    }

    initializedModel = nextModel;
    loadedSkinUrl = skinUrl;
  } catch (error) {
    console.error("Failed to update skin model:", error);
  }
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
 * Reconfigure the animation controller when the mode changes.
 */
$effect(() => {
  const nextMode = animationMode;

  if (skinViewer) {
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
  onclick={handleClick}></canvas>

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
