<!--
@component
  name: Image
  description: A component that attempts to find the key with backend on the file system and displays it. If not found, it falls back to a default image with relative path in static assets.
  props:
    - key: string (required) - The key to look up the image path.
    - alt: string (optional) - The alt text for the image.
    - className: string (optional) - Additional CSS classes for styling.
    - width: string (optional) - The width of the image (e.g., "100px", "50%").
    - height: string (optional) - The height of the image (e.g., "100px", "50%").
  example:
    <Image key="profile-avatar" alt="User Avatar" className="avatar" width="100px" height="100px" />
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  // Key is the logical image identifier used by the app (e.g. 'profile-avatar')
  export let key: string;
  export let alt: string = 'Image';
  export let className: string = '';
  export let width: string = 'auto';
  export let height: string = 'auto';

  // Resolved src used in the <img> tag. Defaults to app favicon while resolving.
  let resolvedSrc: string = '/favicon.png';
  let loadError = false;

  onMount(async () => {
    if (!key) return;
    try {
      // Call the backend command we added which prefers user images and falls back to /img/<key>.png
      const result = await invoke<string>('resolve_image_path', { key });

      // If the backend returned a data URL (base64) or a static /img path, use it directly
      if (result && result.startsWith('data:')) {
        resolvedSrc = result;
      } else if (result && result.startsWith('/img/')) {
        // static asset path
        resolvedSrc = result;
      } else if (result && (result.startsWith('/') || result.match(/^[a-zA-Z]:\\/))) {
        // Absolute filesystem path -> convert to file:// URL
        const normalized = result.replace(/\\/g, '/');
        resolvedSrc = `file://${normalized}`;
      } else if (result) {
        resolvedSrc = result;
      }
    } catch (err) {
      console.error('resolve_image_path failed', err);
      resolvedSrc = '/favicon.png';
    }
  });

  function handleImgError() {
    // Fallback logic: if the resolved source was a file:// path, try static images
    const staticExts = ['webp', 'png', 'jpg', 'jpeg', 'svg', 'gif'];

    // If we already are using a static /img/ path, try other extensions before falling back
    if (resolvedSrc.startsWith('/img/')) {
      const base = `/img/${key}`;
      for (const ext of staticExts) {
        const candidate = `${base}.${ext}`;
        if (candidate !== resolvedSrc) {
          // try to set and let the browser attempt load
          resolvedSrc = candidate;
          return; // return so browser tries this new candidate
        }
      }
    }

    // If the value was a file: URL or we've exhausted static attempts, fallback to favicon
    if (!resolvedSrc.startsWith('file:')) {
      // last resort: favicon
      resolvedSrc = '/favicon.png';
    } else {
      // If it was a file URL that failed, try static webp/png before final fallback
      for (const ext of staticExts) {
        const candidate = `/img/${key}.${ext}`;
        resolvedSrc = candidate;
        return;
      }
      resolvedSrc = '/favicon.png';
    }
    loadError = true;
  }
</script>
<img
  src={resolvedSrc}
  alt={alt}
  class={className}
  style="width: {width}; height: {height}; object-fit: contain;"
  on:error={handleImgError}
/>

<style>
  img {
    display: inline-block;
    vertical-align: middle;
    object-fit: contain;
    max-width: 100%;
    max-height: 100%;
  }
</style>