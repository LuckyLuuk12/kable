<!-- @component
ShaderGalleryModal - Modal for viewing shader pack gallery images

Displays a lightbox-style gallery with navigation controls for browsing
shader pack screenshots and preview images.

@prop {Object | null} [shader=null] - Shader pack with gallery images
@prop {boolean} [visible=false] - Whether modal is visible
@prop {(() =► void) | undefined} onclose - Callback when modal is closed

@example
```svelte
◄ShaderGalleryModal {shader} {visible} onclose={handleClose} /►
```
-->
<script lang="ts">
import { Icon } from "$lib";

export let shader: {
  name: string;
  gallery: string[] | null;
  featured_gallery: string | null;
} | null = null;
export let visible = false;
export let onclose: (() => void) | undefined = undefined;

let currentIndex = 0;
let images: string[] = [];

$: if (shader) {
  images = [];
  if (shader.featured_gallery) {
    images.push(shader.featured_gallery);
  }
  if (shader.gallery && shader.gallery.length > 0) {
    images.push(...shader.gallery);
  }
  // Remove duplicates
  images = [...new Set(images)];
  currentIndex = 0;

  console.log(`[ShaderGalleryModal] Processing shader:`, {
    name: shader.name,
    featured_gallery: shader.featured_gallery,
    gallery: shader.gallery,
    totalImages: images.length,
    images,
  });
}

function close() {
  visible = false;
  onclose?.();
}

function nextImage() {
  currentIndex = (currentIndex + 1) % images.length;
}

function prevImage() {
  currentIndex = (currentIndex - 1 + images.length) % images.length;
}

function goToImage(index: number) {
  currentIndex = index;
}

function handleKeydown(e: KeyboardEvent) {
  if (!visible) return;

  switch (e.key) {
    case "Escape":
      close();
      break;
    case "ArrowLeft":
      prevImage();
      break;
    case "ArrowRight":
      nextImage();
      break;
  }
}

function handleBackdropClick(e: MouseEvent) {
  if (e.target === e.currentTarget) {
    close();
  }
}
</script>

<svelte:window on:keydown={handleKeydown} />

{#if visible && shader && images.length > 0}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    class="gallery-modal"
    on:click={handleBackdropClick}
    role="dialog"
    aria-modal="true"
    aria-labelledby="gallery-title"
    tabindex="-1"
  >
    <div class="gallery-container">
      <!-- Header -->
      <div class="gallery-header">
        <h2 id="gallery-title">{shader.name} - Gallery</h2>
        <button class="close-btn" on:click={close} aria-label="Close gallery">
          <Icon name="x" size="lg" forceType="svg" />
        </button>
      </div>

      <!-- Main Image -->
      <div class="gallery-main">
        {#if images.length > 1}
          <button
            class="nav-btn prev"
            on:click={prevImage}
            aria-label="Previous image"
          >
            <Icon name="chevron-left" size="xl" forceType="svg" />
          </button>
        {/if}

        <div class="image-container">
          <img
            src={images[currentIndex]}
            alt="{shader.name} screenshot {currentIndex + 1}"
            class="main-image"
          />
          <div class="image-counter">
            {currentIndex + 1} / {images.length}
          </div>
        </div>

        {#if images.length > 1}
          <button
            class="nav-btn next"
            on:click={nextImage}
            aria-label="Next image"
          >
            <Icon name="chevron-right" size="xl" forceType="svg" />
          </button>
        {/if}
      </div>

      <!-- Thumbnails -->
      {#if images.length > 1}
        <div class="gallery-thumbnails">
          {#each images as image, index}
            <button
              class="thumbnail"
              class:active={index === currentIndex}
              on:click={() => goToImage(index)}
              aria-label="View image {index + 1}"
            >
              <img src={image} alt="{shader.name} thumbnail {index + 1}" />
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;

.gallery-modal {
  width: 100%;
  height: 100%;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.95);
  backdrop-filter: blur(8px);
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.gallery-container {
  width: 100%;
  max-width: 1400px;
  max-height: 100vh;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.gallery-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 1rem;

  h2 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text);
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text);
    cursor: pointer;
    padding: 0.5rem;
    border-radius: 0.375rem;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;

    &:hover {
      background: color-mix(in srgb, var(--text), 10%, transparent);
    }
  }
}

.gallery-main {
  position: relative;
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  min-height: 0;
}

.image-container {
  position: relative;
  max-width: 100%;
  max-height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;

  .main-image {
    max-width: 100%;
    max-height: calc(100vh - 100px);
    width: 100%;
    height: 100%;
    object-fit: contain;
    border-radius: 0.5rem;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  }

  .image-counter {
    position: absolute;
    bottom: 1rem;
    right: 1rem;
    background: rgba(0, 0, 0, 0.8);
    color: white;
    padding: 0.375rem 0.75rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
  }
}

.nav-btn {
  background: color-mix(in srgb, var(--container), 80%, transparent);
  backdrop-filter: blur(8px);
  border: 1px solid color-mix(in srgb, var(--text), 10%, transparent);
  color: var(--text);
  cursor: pointer;
  padding: 1rem;
  border-radius: 0.5rem;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;

  &:hover:not(:disabled) {
    background: var(--container);
    border-color: var(--primary);
    transform: scale(1.05);
  }

  &:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  &.prev {
    margin-right: auto;
  }

  &.next {
    margin-left: auto;
  }
}

.gallery-thumbnails {
  display: flex;
  gap: 0.5rem;
  padding: 0 1rem;
  overflow-x: auto;
  overflow-y: hidden;
  scrollbar-width: thin;
  scrollbar-color: var(--primary) transparent;

  &::-webkit-scrollbar {
    height: 6px;
  }

  &::-webkit-scrollbar-track {
    background: transparent;
  }

  &::-webkit-scrollbar-thumb {
    background: var(--primary);
    border-radius: 3px;
  }

  .thumbnail {
    flex-shrink: 0;
    width: 120px;
    height: 68px;
    border: 2px solid transparent;
    border-radius: 0.375rem;
    overflow: hidden;
    cursor: pointer;
    transition: all 0.2s ease;
    padding: 0;
    background: var(--container);

    img {
      width: 100%;
      height: 100%;
      object-fit: cover;
    }

    &:hover {
      border-color: var(--secondary);
      transform: scale(1.05);
    }

    &.active {
      border-color: var(--primary);
      box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary), 30%, transparent);
    }
  }
}
</style>
