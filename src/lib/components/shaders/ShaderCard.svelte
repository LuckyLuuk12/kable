<!-- @component
ShaderCard - Displays shader pack information in various view modes

Reusable card component for showing shader pack details including title, description,
author, downloads, and gallery preview. Supports grid, list, and compact views.

@prop {ShaderDownload} shader - The shader pack data to display
@prop {'grid' | 'list' | 'compact'} [viewMode='grid'] - Display mode
@prop {KableInstallation | null} [installation=null] - Target installation
@prop {boolean} [loading=false] - Whether pack is being downloaded
@prop {boolean} [isInstalled=false] - Whether pack is already installed
@prop {((event: { shader: ShaderDownload; installation: KableInstallation | null }) =► void) | undefined} ondownload - Callback when download button is clicked
@prop {((event: { shader: ShaderDownload }) =► void) | undefined} onviewgallery - Callback when gallery preview button is clicked

@example
```svelte
◄ShaderCard {shader} viewMode="grid" ondownload={handleDownload} /►
```
-->
<script lang="ts">
import { Icon } from "$lib";
import type { ShaderDownload, KableInstallation } from "$lib";
import { openUrl } from "$lib/api/system";

export let shader: ShaderDownload;
export let viewMode: "grid" | "list" | "compact" = "grid";
export let installation: KableInstallation | null = null;
export let loading = false;
export let isInstalled = false;
export let ondownload:
  | ((event: {
      shader: ShaderDownload;
      installation: KableInstallation | null;
    }) => void)
  | undefined = undefined;
export let onviewgallery:
  | ((event: { shader: ShaderDownload }) => void)
  | undefined = undefined;

$: hasGallery =
  (shader.gallery && shader.gallery.length > 0) || !!shader.featured_gallery;

// Debug logging
$: if (shader) {
  console.log(`[ShaderCard] ${shader.name}:`, {
    hasGallery,
    gallery: shader.gallery,
    featured_gallery: shader.featured_gallery,
    galleryLength: shader.gallery?.length ?? 0,
  });
}

// Format download count
function formatDownloads(count: number): string {
  if (count >= 1_000_000) {
    return `${(count / 1_000_000).toFixed(1)}M`;
  } else if (count >= 1_000) {
    return `${(count / 1_000).toFixed(1)}K`;
  }
  return count.toString();
}

// Get loader badge color
function getLoaderColor(loader: string): string {
  const loaderColors: Record<string, string> = {
    Canvas: "#E74C3C",
    Iris: "#9B59B6",
    OptiFine: "#3498DB",
    Vanilla: "#2ECC71",
  };
  return loaderColors[loader] || "#95A5A6";
}

// Handle download
function handleDownload(e: MouseEvent) {
  e.stopPropagation();
  ondownload?.({ shader, installation });
}

// Handle gallery view
function handleViewGallery(e: MouseEvent) {
  e.stopPropagation(); // Prevent card click
  if (hasGallery) {
    onviewgallery?.({ shader });
  }
}

// Handle visit page
async function handleVisit(e: MouseEvent | KeyboardEvent) {
  const url = `https://modrinth.com/shader/${shader.id}`;
  try {
    await openUrl(url);
  } catch (error) {
    console.error("Failed to open URL:", error);
  }
}
</script>

<div
  class="shader-card"
  class:grid={viewMode === "grid"}
  class:list={viewMode === "list"}
  class:compact={viewMode === "compact"}
  class:installed={isInstalled}
  on:click={handleVisit}
  role="button"
  tabindex="0"
  on:keydown={(e) => e.key === "Enter" && handleVisit(e)}
>
  <!-- Thumbnail -->
  {#if viewMode !== "compact"}
    <div class="shader-thumbnail">
      {#if shader.thumbnail}
        <img src={shader.thumbnail} alt={shader.name} />
      {:else}
        <div class="placeholder-thumbnail">
          <Icon name="image" size="xl" />
        </div>
      {/if}

      {#if hasGallery}
        <button
          class="gallery-overlay"
          on:click={handleViewGallery}
          title="View gallery"
        >
          <Icon name="images" size="lg" forceType="svg" />
          <span>View Gallery</span>
        </button>
      {/if}

      {#if isInstalled}
        <div class="installed-badge">
          <Icon name="check-circle" size="sm" />
          <span>Installed</span>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Content -->
  <div class="shader-content">
    <!-- Header -->
    <div class="shader-header">
      <h3 class="shader-title" title={shader.name}>{shader.name}</h3>

      {#if shader.author}
        <p class="shader-author">by {shader.author}</p>
      {/if}
    </div>

    <!-- Description -->
    {#if viewMode !== "compact" && shader.description}
      <p class="shader-description">{shader.description}</p>
    {/if}

    <!-- Footer -->
    <div class="shader-footer">
      <div class="shader-meta">
        <!-- Loader -->
        <div class="shader-loaders">
          <span
            class="loader-badge"
            style="background-color: {getLoaderColor(shader.shader_loader)}"
          >
            {shader.shader_loader}
          </span>
        </div>

        <!-- Downloads -->
        <div class="shader-stats">
          <Icon name="download" size="sm" />
          <span>{formatDownloads(shader.downloads)}</span>
        </div>
      </div>

      <!-- Actions -->
      <button
        class="download-btn"
        class:loading
        disabled={loading || isInstalled}
        on:click={handleDownload}
        title={isInstalled
          ? "Already installed"
          : installation
            ? `Install to ${installation.name}`
            : "Install globally"}
      >
        {#if loading}
          <Icon name="loader" size="sm" forceType="svg" />
        {:else if isInstalled}
          <Icon name="check" size="sm" forceType="svg" />
        {:else}
          <Icon name="download" size="sm" forceType="svg" />
        {/if}

        {#if viewMode !== "compact"}
          <span>{isInstalled ? "Installed" : "Install"}</span>
        {/if}
      </button>
    </div>
  </div>
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;
@use "sass:color";

.shader-card {
  display: flex;
  background: linear-gradient(
    135deg,
    var(--card) 0%,
    #{"color-mix(in srgb, var(--container), 80%, transparent)"} 100%
  );
  backdrop-filter: blur(8px);
  border: 1px solid transparent;
  border-radius: 0.5rem;
  overflow: hidden;
  transition: all 0.2s ease;
  position: relative;
  cursor: pointer;

  &::before {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: linear-gradient(
      90deg,
      transparent 0%,
      #{"color-mix(in srgb, var(--primary), 40%, transparent)"} 50%,
      transparent 100%
    );
    opacity: 0;
    transition: opacity 0.2s ease;
  }

  &:hover {
    border: 1px solid var(--secondary);
    box-shadow: 0 4px 12px
      #{"color-mix(in srgb, var(--primary), 15%, transparent)"};
    transform: translateY(-2px);

    &::before {
      opacity: 1;
    }
  }

  &.installed {
    border-color: var(--green);
    background: linear-gradient(
      135deg,
      #{"color-mix(in srgb, var(--green), 5%, transparent)"} 0%,
      var(--card) 100%
    );
  }

  // Grid Layout
  &.grid {
    flex-direction: column;

    .shader-thumbnail {
      width: 100%;
      aspect-ratio: 16 / 9;
    }

    .shader-content {
      padding: 0.75rem;
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      flex: 1;
    }

    .shader-footer {
      margin-top: auto;
    }
  }

  // List Layout
  &.list {
    flex-direction: row;

    .shader-thumbnail {
      width: 160px;
      min-width: 160px;
      aspect-ratio: 16 / 9;
    }

    .shader-content {
      padding: 0.75rem;
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      flex: 1;
    }

    .shader-footer {
      margin-top: auto;
      display: flex;
      align-items: flex-end;
      justify-content: space-between;
    }

    .shader-meta {
      flex-direction: row;
      gap: 0.75rem;
    }
  }

  // Compact Layout
  &.compact {
    flex-direction: row;
    align-items: center;
    padding: 0.5rem 0.75rem;
    gap: 0.5rem;
    min-width: 240px;
    max-width: 300px;

    .shader-content {
      flex: 1;
      min-width: 0;
    }

    .shader-header {
      margin-bottom: 0;
    }

    .shader-title {
      font-size: 0.85em;
      margin-bottom: 0;
    }

    .shader-author {
      display: none;
    }

    .shader-footer {
      flex-direction: row;
      gap: 0.5rem;
      align-items: center;
    }

    .shader-meta {
      flex-direction: row;
      gap: 0.375rem;
    }

    .download-btn {
      padding: 0.25rem 0.5rem;
      font-size: 0.75em;
    }
  }
}

// Thumbnail
.shader-thumbnail {
  position: relative;
  background: var(--dark-700);
  overflow: hidden;

  :global(img) {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .placeholder-thumbnail {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(
      135deg,
      var(--dark-700) 0%,
      var(--dark-600) 100%
    );
    color: var(--dark-400);
  }

  .gallery-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.375rem;
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(4px);
    color: white;
    border: none;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.2s ease;
    font-size: 0.875rem;
    font-weight: 500;

    &:hover {
      opacity: 1;
      background: rgba(0, 0, 0, 0.85);
    }
  }

  &:hover .gallery-overlay {
    opacity: 1;
  }

  .installed-badge {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem 0.5rem;
    background: var(--green);
    color: white;
    border-radius: 0.25rem;
    font-size: 0.7em;
    font-weight: 600;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
  }
}

// Content
.shader-content {
  flex: 1;
  min-width: 0;
}

.shader-header {
  margin-bottom: 0.5rem;

  .shader-title {
    margin: 0 0 0.25rem 0;
    font-size: 1em;
    font-weight: 600;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .shader-author {
    margin: 0;
    font-size: 0.75em;
    color: var(--placeholder);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.shader-description {
  margin: 0;
  font-size: 0.8em;
  color: var(--text);
  line-height: 1.4;
  overflow: hidden;
  display: -webkit-box;
  line-clamp: 2;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  text-overflow: ellipsis;
}

// Footer
.shader-footer {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.shader-meta {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;

  .shader-loaders {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;

    .loader-badge {
      padding: 0.125rem 0.375rem;
      border-radius: 0.25rem;
      color: white;
      font-size: 0.65em;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.3px;
      box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
    }
  }

  .shader-stats {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    color: var(--placeholder);
    font-size: 0.75em;

    span {
      font-weight: 500;
    }
  }
}

// Download Button
.download-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
  padding: 0.5rem 0.75rem;
  border: 0px solid transparent;
  border-radius: 0.375rem;
  background: linear-gradient(135deg, var(--primary) 0%, var(--secondary) 100%);
  color: white;
  font-weight: 600;
  font-size: 0.8em;
  cursor: pointer;
  transition: all 0.15s;

  &:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 4px 8px
      #{"color-mix(in srgb, var(--primary), 25%, transparent)"};
  }

  &:active:not(:disabled) {
    transform: translateY(0);
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  &.loading {
    :global(.icon) {
      animation: spin 1s linear infinite;
    }
  }
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
