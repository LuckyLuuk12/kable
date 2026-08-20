<!--
@component 

-->
<script lang="ts">
import { type Project } from "$lib";

let { project }: { project: Project } = $props();

let selectedIndex = $state(0);

let gallery = $derived(project.gallery ?? []);
let selectedImage = $derived(gallery[selectedIndex] ?? null);

function previous() {
  if (gallery.length === 0) return;

  selectedIndex = (selectedIndex - 1 + gallery.length) % gallery.length;
}

function next() {
  if (gallery.length === 0) return;

  selectedIndex = (selectedIndex + 1) % gallery.length;
}

function select(index: number) {
  selectedIndex = index;
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "ArrowLeft") {
    event.preventDefault();
    previous();
  } else if (event.key === "ArrowRight") {
    event.preventDefault();
    next();
  }
}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="project-gallery-modal">
  <header class="header">
    <div class="title">
      <h2>{project.title}</h2>
      <span>
        {gallery.length}
        {gallery.length === 1 ? "image" : "images"}
      </span>
    </div>
  </header>

  {#if selectedImage}
    <div class="viewer">
      <button class="navigation previous" type="button" aria-label="Previous image" disabled={gallery.length <= 1} onclick={previous}> ‹ </button>

      <div class="image-container">
        <img src={selectedImage} alt={`${project.title} gallery image ${selectedIndex + 1}`} />

        <div class="counter">
          {selectedIndex + 1} / {gallery.length}
        </div>
      </div>

      <button class="navigation next" type="button" aria-label="Next image" disabled={gallery.length <= 1} onclick={next}> › </button>
    </div>

    {#if gallery.length > 1}
      <div class="thumbnails">
        {#each gallery as image, index (image)}
          <button
            class:active={index === selectedIndex}
            type="button"
            aria-label={`View image ${index + 1}`}
            aria-current={index === selectedIndex}
            onclick={() => select(index)}>
            <img src={image} alt="" />
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
  width: min($layout-container-6, 90vw);
  max-height: 85vh;
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
  padding: $space-lg $space-xl;
  border-bottom: 1px solid $color-border-muted;
}

.title {
  display: flex;
  align-items: baseline;
  gap: $space-md;

  h2 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
  }

  span {
    color: $color-text-muted;
    font-size: 0.75rem;
  }
}

.viewer {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: $space-md;
  min-height: 0;
  flex: 1;
  padding: $space-lg;
}

.image-container {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 0;
  min-height: 0;
  max-width: 100%;
  max-height: 65vh;
  overflow: hidden;
  border-radius: $radius-lg;
  background: $color-surface-0;

  img {
    display: block;
    width: auto;
    height: auto;
    max-width: 100%;
    max-height: 65vh;
    object-fit: contain;
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

.counter {
  position: absolute;
  right: $space-md;
  bottom: $space-md;
  padding: $space-1 $space-sm;
  border-radius: $radius-round;
  background: $color-overlay;
  color: $color-text;
  font-size: 0.7rem;
  backdrop-filter: blur(4px);
}

.thumbnails {
  display: flex;
  gap: $space-sm;
  overflow-x: auto;
  padding: $space-md $space-lg $space-lg;
  border-top: 1px solid $color-border-muted;

  button {
    flex: 0 0 auto;
    width: 80px;
    height: 52px;
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
  }
}

.empty {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 300px;
  padding: $space-2xl;
  color: $color-text-muted;
}
</style>
