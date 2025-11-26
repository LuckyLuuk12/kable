<!-- @component
A component that attempts to find the key with backend on the file system and displays it. If not found, it falls back to a default image with relative path in static assets.

@prop {string} key - The logical image identifier used by the app (e.g. 'profile-avatar')
@prop {string} [alt='Image'] - The alt text for the image
@prop {string} [className=''] - Additional CSS classes for styling
@prop {string} [width='auto'] - The width of the image (e.g., "100px", "50%").
@prop {string} [height='auto'] - The height of the image (e.g., "100px", "50%").

@example
```svelte
  ◄Image key="profile-avatar" alt="User Avatar" className="avatar" width="100px" height="100px" /►
```
-->
<script lang="ts">
import { onMount } from "svelte";
import { invoke } from "@tauri-apps/api/core";

// Key is the logical image identifier used by the app (e.g. 'profile-avatar')
export let key: string;
export let alt: string = "Image";
export let className: string = "";
export let width: string = "auto";
export let height: string = "auto";

// Resolved src used in the <img> tag. Defaults to app favicon while resolving.
let resolvedSrc: string = "/favicon.png";
let loadError = false;
let retryCount = 0;
const MAX_RETRIES = 12; // Limit retries to prevent infinite loop
let lastErrorTime = 0;
let isVisible = false; // Hide image until successfully loaded
let imgElement: HTMLImageElement;

onMount(async () => {
  if (!key) return;
  
  // If key is already a full URL (http/https/data), use it directly
  if (key.startsWith("http://") || key.startsWith("https://") || key.startsWith("data:")) {
    resolvedSrc = key;
    // Log abbreviated version for data URLs to avoid console spam
    const logKey = key.startsWith("data:") ? `data:${key.substring(5, 30)}...` : key;
    console.log(`Image component using direct URL: ${logKey}`);
    setTimeout(() => {
      if (imgElement && imgElement.complete && imgElement.naturalHeight !== 0) {
        isVisible = true;
        console.log(`Image loaded successfully from direct URL: ${logKey}`);
      }
    }, 0);
    return;
  }
  
  try {
    // Call the backend command we added which prefers user images and falls back to /img/<key>.png
    const result = await invoke<string>("resolve_image_path", { key });

    // If the backend returned a data URL (base64) or a static /img path, use it directly
    if (result && result.startsWith("data:")) {
      resolvedSrc = result;
    } else if (result && (result.startsWith("/img/") || result.startsWith("/"))) {
      // static asset path (either /img/ or root static folder)
      resolvedSrc = result;
    } else if (
      result &&
      result.match(/^[a-zA-Z]:\\/)
    ) {
      // Absolute filesystem path -> convert to file:// URL
      const normalized = result.replace(/\\/g, "/");
      resolvedSrc = `file://${normalized}`;
    } else if (result) {
      resolvedSrc = result;
    }

    // Check if image is already loaded after setting src
    setTimeout(() => {
      if (imgElement && imgElement.complete && imgElement.naturalHeight !== 0) {
        isVisible = true;
        console.log(`Image already loaded from cache: ${key}`);
      }
    }, 0);
  } catch (err) {
    console.error("resolve_image_path failed", err);
    resolvedSrc = "/favicon.png";
  }
});

function handleImgError() {
  // Hide image on error
  isVisible = false;
  // Prevent infinite loops - stop after MAX_RETRIES attempts
  if (retryCount >= MAX_RETRIES) {
    console.warn(
      `Image load failed after ${MAX_RETRIES} retries for key: ${key}`,
    );
    loadError = true;
    resolvedSrc = "/favicon.png";
    return;
  }

  // Prevent rapid-fire errors (debounce)
  const now = Date.now();
  if (now - lastErrorTime < 100) {
    return;
  }
  lastErrorTime = now;

  retryCount++;

  // Fallback logic: try different paths and extensions
  const staticExts = ["webp", "png", "jpg", "jpeg", "svg", "gif"];

  // If we already are using a static /img/ path, try other extensions
  if (resolvedSrc.startsWith("/img/")) {
    const base = `/img/${key}`;
    const currentExt = resolvedSrc.split(".").pop()?.toLowerCase();
    const currentIndex = staticExts.indexOf(currentExt || "");

    // Try next extension in the list
    if (currentIndex >= 0 && currentIndex < staticExts.length - 1) {
      resolvedSrc = `${base}.${staticExts[currentIndex + 1]}`;
      return;
    }
  }

  // If we're using root static path (not /img/), try extensions then fall back to /img/
  if (resolvedSrc.startsWith("/") && !resolvedSrc.startsWith("/img/")) {
    const currentExt = resolvedSrc.split(".").pop()?.toLowerCase();
    const currentIndex = staticExts.indexOf(currentExt || "");

    // Try next extension in root static folder
    if (currentIndex >= 0 && currentIndex < staticExts.length - 1) {
      resolvedSrc = `/${key}.${staticExts[currentIndex + 1]}`;
      return;
    }

    // All root extensions failed, try /img/ folder
    resolvedSrc = `/img/${key}.${staticExts[0]}`;
    return;
  }

  // If it was a file URL that failed, try root static folder first
  if (resolvedSrc.startsWith("file://")) {
    resolvedSrc = `/${key}.${staticExts[0]}`;
    return;
  }

  // Final fallback
  resolvedSrc = "/favicon.png";
  loadError = true;
}

function handleMouseEnter() {
  // On hover, allow retry if it previously failed and enough time has passed
  if (loadError && retryCount >= MAX_RETRIES) {
    const timeSinceLastError = Date.now() - lastErrorTime;
    const RETRY_COOLDOWN = 5 * 60 * 1000; // 5 minutes

    if (timeSinceLastError > RETRY_COOLDOWN) {
      console.log(`Retrying image load for key: ${key} after cooldown`);
      retryCount = 0;
      loadError = false;
      isVisible = false; // Hide while retrying
      resolvedSrc = `/img/${key}.webp`; // Start fresh with first extension
    }
  }
}

function handleImgLoad() {
  // Show image when it successfully loads
  isVisible = true;
  console.log(`Image loaded successfully: ${key}, src: ${resolvedSrc}`);
}

// Bind to img element and check after each src change
$: if (imgElement && resolvedSrc) {
  // Small delay to allow browser to process the src change
  setTimeout(() => {
    if (imgElement.complete && imgElement.naturalHeight !== 0) {
      if (!isVisible) {
        isVisible = true;
        console.log(`Image became visible for: ${key}, src: ${resolvedSrc}`);
      }
    }
  }, 10);
}
</script>

<img
  bind:this={imgElement}
  src={resolvedSrc}
  {alt}
  class={className}
  class:visible={isVisible}
  style="width: {width}; height: {height};"
  on:error={handleImgError}
  on:load={handleImgLoad}
  on:mouseenter={handleMouseEnter}
/>

<style>
img {
  background: transparent;
  display: inline-block;
  vertical-align: middle;
  object-fit: cover;
  max-width: 100%;
  max-height: 100%;
  visibility: hidden;
}

img.visible {
  visibility: visible;
}
</style>
