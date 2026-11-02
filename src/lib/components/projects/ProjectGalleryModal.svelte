<script lang="ts">
import { type Project } from "$lib";

let { project }: { project: Project } = $props();

let selectedIndex = $state(0);
let zoom = $state(1);
let panX = $state(0);
let panY = $state(0);
let isDragging = $state(false);

let gallery = $derived(project.gallery ?? []);
let selectedImage = $derived(gallery[selectedIndex] ?? null);

// svelte-ignore non_reactive_update
let imageContainer: HTMLDivElement;
// svelte-ignore non_reactive_update
let thumbnailContainer: HTMLDivElement;

let dragStartX = 0;
let dragStartY = 0;
let dragOriginX = 0;
let dragOriginY = 0;

const MIN_ZOOM = 1;
const MAX_ZOOM = 5;
const ZOOM_STEP = 0.25;

function resetZoom() {
  zoom = 1;
  panX = 0;
  panY = 0;
  isDragging = false;
}

function previous() {
  if (gallery.length <= 1) return;

  selectedIndex = (selectedIndex - 1 + gallery.length) % gallery.length;
  resetZoom();
}

function next() {
  if (gallery.length <= 1) return;

  selectedIndex = (selectedIndex + 1) % gallery.length;
  resetZoom();
}

function select(index: number) {
  if (index < 0 || index >= gallery.length || index === selectedIndex) return;

  selectedIndex = index;
  resetZoom();
}

function handleKeydown(event: KeyboardEvent) {
  if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
    return;
  }

  if (event.key === "ArrowLeft") {
    event.preventDefault();
    previous();
  } else if (event.key === "ArrowRight") {
    event.preventDefault();
    next();
  } else if (event.key === "Escape") {
    resetZoom();
  }
}

function handleWheel(event: WheelEvent) {
  event.preventDefault();

  const direction = event.deltaY < 0 ? 1 : -1;
  const nextZoom = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom + direction * ZOOM_STEP));

  if (nextZoom === zoom) return;

  if (!imageContainer) return;

  const rect = imageContainer.getBoundingClientRect();

  const cursorX = event.clientX - rect.left - rect.width / 2;
  const cursorY = event.clientY - rect.top - rect.height / 2;

  const scale = nextZoom / zoom;

  panX = cursorX - (cursorX - panX) * scale;
  panY = cursorY - (cursorY - panY) * scale;
  zoom = nextZoom;

  if (zoom === 1) {
    panX = 0;
    panY = 0;
  }
}

function toggleZoom(event: MouseEvent) {
  if (zoom > 1) {
    resetZoom();
    return;
  }

  const rect = imageContainer.getBoundingClientRect();

  const cursorX = event.clientX - rect.left - rect.width / 2;
  const cursorY = event.clientY - rect.top - rect.height / 2;

  zoom = 2;
  panX = -cursorX;
  panY = -cursorY;
}

function startDrag(event: PointerEvent) {
  if (zoom <= 1) return;
  if (event.button !== 0) return;

  isDragging = true;

  dragStartX = event.clientX;
  dragStartY = event.clientY;
  dragOriginX = panX;
  dragOriginY = panY;

  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
}

function drag(event: PointerEvent) {
  if (!isDragging) return;

  panX = dragOriginX + event.clientX - dragStartX;
  panY = dragOriginY + event.clientY - dragStartY;
}

function stopDrag() {
  isDragging = false;
}

function handleThumbnailWheel(event: WheelEvent) {
  event.preventDefault();

  const delta = Math.abs(event.deltaX) > Math.abs(event.deltaY) ? event.deltaX : event.deltaY;

  if (delta > 0) {
    next();
  } else if (delta < 0) {
    previous();
  }
}

function scrollThumbnailIntoView(index: number) {
  if (!thumbnailContainer) return;

  const thumbnail = thumbnailContainer.children[index] as HTMLElement | undefined;

  thumbnail?.scrollIntoView({
    behavior: "smooth",
    block: "nearest",
    inline: "nearest",
  });
}

$effect(() => {
  selectedIndex;

  scrollThumbnailIntoView(selectedIndex);
});
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="project-gallery-modal">
  <header class="header">
    <div class="title">
      <h2>{project.title}</h2>
      <span>{gallery.length > 0 ? `${selectedIndex + 1} / ${gallery.length}` : "0 images"}</span>
    </div>

    {#if selectedImage}
      <div class="zoom-controls">
        <button
          type="button"
          aria-label="Zoom out"
          title="Zoom out"
          disabled={zoom <= MIN_ZOOM}
          onclick={() => {
            zoom = Math.max(MIN_ZOOM, zoom - ZOOM_STEP);
            if (zoom === 1) resetZoom();
          }}>
          −
        </button>

        <span>{Math.round(zoom * 100)}%</span>

        <button type="button" aria-label="Zoom in" title="Zoom in" disabled={zoom >= MAX_ZOOM} onclick={() => (zoom = Math.min(MAX_ZOOM, zoom + ZOOM_STEP))}> + </button>

        <button type="button" aria-label="Reset zoom" title="Reset zoom" disabled={zoom === 1} onclick={resetZoom}>
          <span>↺</span>
        </button>
      </div>
    {/if}
  </header>

  {#if selectedImage}
    <div class="viewer">
      <button class="navigation previous" type="button" aria-label="Previous image" disabled={gallery.length <= 1} onclick={previous}> ‹ </button>

      <div
        class="image-container"
        bind:this={imageContainer}
        onwheel={handleWheel}
        ondblclick={toggleZoom}
        onpointerdown={startDrag}
        onpointermove={drag}
        onpointerup={stopDrag}
        onpointercancel={stopDrag}
        onlostpointercapture={stopDrag}
        class:zoomed={zoom > 1}
        class:dragging={isDragging}
        role="img">
        <img
          src={selectedImage}
          alt={`${project.title} gallery image ${selectedIndex + 1}`}
          draggable="false"
          style:transform={`translate(${panX}px, ${panY}px) scale(${zoom})`} />
      </div>

      <button class="navigation next" type="button" aria-label="Next image" disabled={gallery.length <= 1} onclick={next}> › </button>
    </div>

    {#if gallery.length > 1}
      <div class="thumbnails" bind:this={thumbnailContainer} onwheel={handleThumbnailWheel}>
        {#each gallery as image, index (image)}
          <button
            class:active={index === selectedIndex}
            type="button"
            aria-label={`View image ${index + 1}`}
            aria-current={index === selectedIndex ? "true" : undefined}
            onclick={() => select(index)}>
            <img src={image} alt="" draggable="false" />
          </button>
        {/each}
      </div>
    {/if}
  {:else}
    <div class="empty">
      <span>No gallery images available.</span>
    </div>
  {/if}
</div>

<style lang="scss">
.project-gallery-modal {
  display: flex;
  flex-direction: column;
  width: min($layout-container-8, 94vw);
  height: 90vh;
  max-height: 1000px;
  min-height: 300px;
  overflow: hidden;
  background: $color-surface-1;
  color: $color-text;
  border: 1px solid $color-border;
  border-radius: $radius-xl;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex: 0 0 auto;
  gap: $space-lg;
  padding: $space-lg $space-xl;
  border-bottom: 1px solid $color-border-muted;
}

.title {
  display: flex;
  align-items: baseline;
  gap: $space-md;
  min-width: 0;

  h2 {
    margin: 0;
    overflow: hidden;
    font-size: 1.1rem;
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  span {
    flex: 0 0 auto;
    color: $color-text-muted;
    font-size: 0.75rem;
    font-variant-numeric: tabular-nums;
  }
}

.zoom-controls {
  display: flex;
  align-items: center;
  gap: $space-sm;

  span {
    min-width: 42px;
    color: $color-text-muted;
    font-size: 0.75rem;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }

  button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    padding: 0;
    border: 1px solid $color-border;
    border-radius: $radius-md;
    background: $color-surface-2;
    color: $color-text;
    font-size: 1.1rem;
    cursor: pointer;

    &:hover:not(:disabled) {
      background: $color-surface-3;
      border-color: $color-accent;
    }

    &:disabled {
      cursor: default;
      opacity: 0.35;
    }
  }
}

.viewer {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: $space-md;
  min-height: 0;
  flex: 1;
  padding: $space-md $space-lg;
}

.image-container {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  border-radius: $radius-lg;
  background: $color-surface-0;
  touch-action: none;
  cursor: zoom-in;

  &.zoomed {
    cursor: grab;
  }

  &.dragging {
    cursor: grabbing;
  }

  img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: contain;
    user-select: none;
    pointer-events: none;
    transform-origin: center;
    will-change: transform;
  }
}

.navigation {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  width: 40px;
  height: 40px;
  padding: 0;
  border: 1px solid $color-border;
  border-radius: $radius-round;
  background: $color-surface-2;
  color: $color-text;
  font-size: 1.8rem;
  line-height: 1;
  cursor: pointer;
  transition:
    background-color 120ms ease,
    border-color 120ms ease;

  &:hover:not(:disabled) {
    background: $color-surface-3;
    border-color: $color-accent;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }

  &:disabled {
    cursor: default;
    opacity: 0.35;
  }
}

.thumbnails {
  display: flex;
  flex: 0 0 auto;
  gap: $space-sm;
  overflow-x: auto;
  overscroll-behavior-x: contain;
  padding: $space-md $space-lg $space-lg;
  border-top: 1px solid $color-border-muted;
  scrollbar-width: thin;

  button {
    flex: 0 0 auto;
    width: 88px;
    height: 58px;
    padding: 2px;
    overflow: hidden;
    border: 1px solid $color-border;
    border-radius: $radius-md;
    background: $color-surface-2;
    cursor: pointer;
    opacity: 0.6;
    transition:
      opacity 120ms ease,
      border-color 120ms ease;

    &:hover {
      opacity: 0.9;
    }

    &.active {
      border-color: $color-accent;
      opacity: 1;
    }

    &:focus-visible {
      outline: 2px solid $color-focus;
      outline-offset: 2px;
    }
  }

  img {
    display: block;
    width: 100%;
    height: 100%;
    border-radius: $radius-sm;
    object-fit: cover;
    pointer-events: none;
  }
}

.empty {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  min-height: 0;
  padding: $space-2xl;
  color: $color-text-muted;
}

@media (max-width: 600px) {
  .project-gallery-modal {
    width: 96vw;
    height: 90vh;
  }

  .header {
    padding: $space-md;
  }

  .viewer {
    gap: $space-xs;
    padding: $space-sm;
  }

  .navigation {
    width: 32px;
    height: 32px;
  }

  .thumbnails {
    padding: $space-sm;
  }

  .thumbnails button {
    width: 68px;
    height: 46px;
  }
}
</style>
