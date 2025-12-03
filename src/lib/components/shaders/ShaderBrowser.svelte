<!-- @component
ShaderBrowser - Browse and search for shader packs from Modrinth

Provides interface for discovering shader packs with support for:
- Multiple view modes (grid, list, compact)
- Installation modes (dedicated per-installation or global)
- Advanced filtering (categories, versions, sorting)
- Gallery preview modal

@prop {((event: { shader: ShaderDownload; installation: KableInstallation | null }) =► void) | undefined} ondownload - Callback when user clicks to download a shader pack

@example
```svelte
◄ShaderBrowser ondownload={handleDownload} /►
```
-->
<script lang="ts">
import { onMount } from "svelte";
import {
  Icon,
  ShadersService,
  selectedInstallation,
  installations,
  ShaderCard,
  ShaderGalleryModal,
  shaderDownloads,
  shadersLoading,
  shadersError,
  shadersOffset,
  shadersInstallMode,
} from "$lib";
import type {
  ShaderDownload,
  KableInstallation,
  ShaderFilterFacets,
} from "$lib";

type ViewMode = "grid" | "list" | "compact";
type InstallMode = "dedicated" | "global";

export let ondownload:
  | ((event: {
      shader: ShaderDownload;
      installation: KableInstallation | null;
    }) => void)
  | undefined = undefined;
export let selectedInstallationId: string = "global";

// Browser state
let viewMode: ViewMode = "grid";
let installMode: InstallMode = "global";
let searchQuery = "";
let currentInstallation: KableInstallation | null = null;
let showFilters = true;
let smartFilteringEnabled = true; // Auto-apply game version filter

// Gallery modal state
let showGalleryModal = false;
let selectedShaderForGallery: ShaderDownload | null = null;

// Filter state with include/exclude support
type FilterMode = "include" | "exclude";
type FilterItem = { value: string; mode: FilterMode };

let filters = {
  loader: [] as FilterItem[],
  categories: [] as FilterItem[],
};

// Collapsible filter sections
let collapsedSections = {
  loader: false,
  categories: false,
};

// Pagination state
let currentPage = 1;
let itemsPerPage = 20;
let visitedPages = new Set([1]);
let maxPageReached = 1;

// Service instance
let shadersService: ShadersService;
let isFullyMounted = false;

// View mode options
const viewModes = [
  { id: "grid", name: "Grid", icon: "grid" },
  { id: "list", name: "List", icon: "list" },
  { id: "compact", name: "Compact", icon: "layout" },
];

// Page size options
const pageSizeOptions = [10, 20, 50, 100];

// Filter configuration - Performance/Features/Categories all map to Modrinth tags/categories
const filterSections = [
  {
    id: "loader" as const,
    label: "Shader Loader",
    collapsedKey: "loader" as const,
    options: ["Canvas", "Iris", "OptiFine", "Vanilla"],
  },
  {
    id: "categories" as const,
    label: "Tags",
    collapsedKey: "categories" as const,
    options: [
      // Performance
      "High",
      "Low",
      "Medium",
      "Potato",
      "Screenshot",
      // Features
      "Atmosphere",
      "Bloom",
      "Colored Lighting",
      "Foliage",
      "Path Tracing",
      "PBR",
      "Reflections",
      "Shadows",
      // Style Categories
      "Cartoon",
      "Cursed",
      "Fantasy",
      "Realistic",
      "Semi-realistic",
      "Vanilla-like",
    ],
  },
];

// Reactive statements
$: {
  // Update install mode and installation based on selection
  if (selectedInstallationId === "global") {
    installMode = "global";
    currentInstallation = null;
  } else {
    installMode = "dedicated";
    currentInstallation =
      $installations.find((inst) => inst.id === selectedInstallationId) || null;
    if (currentInstallation) {
      selectedInstallation.set(currentInstallation);
    }
  }
  // Trigger filter update when installation changes (only after mount)
  if (isFullyMounted && shadersService) {
    console.log(
      "[ShaderBrowser] Installation changed to:",
      selectedInstallationId,
      "version:",
      currentInstallation?.version_id,
    );
    handleFiltersChange();
  }
}
$: shaders = $shaderDownloads || [];
$: loading = $shadersLoading;
$: error = $shadersError;
// Don't apply client-side filters - let backend handle it
$: paginatedShaders = shaders;
$: totalShaders = shaders.length;

// Update install mode in store
$: shadersInstallMode.set(installMode);

// Handle filter changes - debounced to avoid too many API calls
let filterChangeTimeout: number | null = null;
function handleFiltersChange() {
  if (filterChangeTimeout) clearTimeout(filterChangeTimeout);
  filterChangeTimeout = window.setTimeout(async () => {
    await applyFiltersToBackend();
  }, 300);
}

// Apply filters to backend
async function applyFiltersToBackend() {
  if (!shadersService) return;

  // Build filter object for backend
  const includeLoaders = filters.loader
    .filter((f) => f.mode === "include")
    .map((f) => f.value);
  const excludeLoaders = filters.loader
    .filter((f) => f.mode === "exclude")
    .map((f) => f.value);
  const includeCategories = filters.categories
    .filter((f) => f.mode === "include")
    .map((f) => f.value);
  const excludeCategories = filters.categories
    .filter((f) => f.mode === "exclude")
    .map((f) => f.value);

  // If no filters and no search and no installation version, clear filters
  if (
    !searchQuery &&
    includeLoaders.length === 0 &&
    excludeLoaders.length === 0 &&
    includeCategories.length === 0 &&
    excludeCategories.length === 0 &&
    !currentInstallation
  ) {
    currentPage = 1;
    shadersOffset.set(0);
    await shadersService.setFilter(null);
    return;
  }

  // Build filter facets for Modrinth
  // Each filter is AND'd together - put each in separate array
  // Operations: ':' for include, '!=' for exclude

  const includeLoaderFilters: [string, string][] = includeLoaders.map(
    (l) => [":", l.toLowerCase()] as [string, string],
  );
  const excludeLoaderFilters: [string, string][] = excludeLoaders.map(
    (l) => ["!=", l.toLowerCase()] as [string, string],
  );

  const loaderFilters = [...includeLoaderFilters, ...excludeLoaderFilters];

  const includeCategoryFilters: [string, string][] = includeCategories.map(
    (c) => [":", c.toLowerCase()] as [string, string],
  );
  const excludeCategoryFilters: [string, string][] = excludeCategories.map(
    (c) => ["!=", c.toLowerCase()] as [string, string],
  );

  const categoryFilters = [
    ...includeCategoryFilters,
    ...excludeCategoryFilters,
  ];

  const filterFacets: ShaderFilterFacets = {
    query: searchQuery || undefined,
    loaders: loaderFilters.length > 0 ? loaderFilters : undefined,
    categories: categoryFilters.length > 0 ? categoryFilters : undefined,
    game_versions:
      smartFilteringEnabled &&
      currentInstallation &&
      currentInstallation.version_id
        ? [currentInstallation.version_id]
        : undefined,
  };

  console.log(
    "[ShaderBrowser] Applying filters to backend:",
    JSON.stringify(filterFacets, null, 2),
  );

  // Reset to first page when filters change
  currentPage = 1;
  shadersOffset.set(0);

  try {
    await shadersService.setFilter(filterFacets);
  } catch (e) {
    console.error("[ShaderBrowser] Failed to apply filters:", e);
  }
}

// Handle search query changes - debounced via handleFiltersChange
async function handleSearch() {
  if (!shadersService) return;
  handleFiltersChange();
}

// Handle smart filtering toggle
async function onSmartFilteringChange() {
  console.log(
    `[ShaderBrowser] Smart filtering ${smartFilteringEnabled ? "enabled" : "disabled"}`,
  );

  // Re-apply filters with new setting
  if (shadersService) {
    handleFiltersChange();
  }
}

// Functions
function toggleFilter(category: "loader" | "categories", value: string) {
  const existing = filters[category].find((f) => f.value === value);
  if (existing) {
    if (existing.mode === "include") {
      // Include -> None (remove the filter)
      filters[category] = filters[category].filter((f) => f.value !== value);
    } else {
      // Exclude -> Include (switch mode by creating new array)
      filters[category] = filters[category].map((f) =>
        f.value === value ? { ...f, mode: "include" as const } : f,
      );
    }
  } else {
    // None -> Include (add as include filter)
    filters[category] = [
      ...filters[category],
      { value, mode: "include" as const },
    ];
  }
  // Trigger reactivity and backend update
  filters = { ...filters };
  handleFiltersChange();
}

function toggleFilterExclude(category: "loader" | "categories", value: string) {
  const existing = filters[category].find((f) => f.value === value);
  if (existing) {
    if (existing.mode === "exclude") {
      // Exclude -> None (remove the filter)
      filters[category] = filters[category].filter((f) => f.value !== value);
    } else {
      // Include -> Exclude (switch mode by creating new array)
      filters[category] = filters[category].map((f) =>
        f.value === value ? { ...f, mode: "exclude" as const } : f,
      );
    }
  } else {
    // None -> Exclude (add as exclude filter)
    filters[category] = [
      ...filters[category],
      { value, mode: "exclude" as const },
    ];
  }
  // Trigger reactivity and backend update
  filters = { ...filters };
  handleFiltersChange();
}

function getFilterState(
  category: "loader" | "categories",
  value: string,
): FilterMode | null {
  const filter = filters[category].find((f) => f.value === value);
  return filter ? filter.mode : null;
}

function resetFilters() {
  // Create new object to trigger reactivity
  filters = {
    loader: [],
    categories: [],
  };
  searchQuery = "";
  currentPage = 1;
  shadersOffset.set(0);
  // Force reactivity update and re-apply filters (preserving installation version)
  filters = { ...filters };
  handleFiltersChange();
}

function toggleSection(section: keyof typeof collapsedSections) {
  collapsedSections[section] = !collapsedSections[section];
}

// Generate dynamic page numbers based on current page (reactive)
$: pageNumbers = (() => {
  const pages: (number | "ellipsis")[] = [];

  // Always show pages 1, 2, 3
  pages.push(1, 2, 3);

  // If current page is beyond 10, show ellipsis and last 7 pages
  if (currentPage > 10) {
    pages.push("ellipsis");

    // Show last 7 pages ending at current page (current-6 through current)
    const startPage = currentPage - 6;
    for (let i = startPage; i <= currentPage; i++) {
      pages.push(i);
    }
  } else {
    // For pages 1-10, fill in the gap from 4 to 10
    for (let i = 4; i <= 10; i++) {
      pages.push(i);
    }
  }

  return pages;
})();

async function changePageSize(newSize: number) {
  itemsPerPage = newSize;
  currentPage = 1;
  shadersOffset.set(0);

  if (shadersService) {
    await shadersService.setLimit(newSize);
  }
}

async function goToPage(page: number) {
  if (page < 1) return;
  currentPage = page;

  visitedPages.add(page);
  if (page > maxPageReached) {
    maxPageReached = page;
  }

  if (shadersService) {
    shadersOffset.set((page - 1) * itemsPerPage);
    await shadersService.loadShaders();
  }
}

async function nextPage() {
  await goToPage(currentPage + 1);
}

async function previousPage() {
  await goToPage(currentPage - 1);
}

async function loadShaders() {
  if (shadersService) {
    await shadersService.loadShaders();
  }
}

function handleDownload(event: {
  shader: ShaderDownload;
  installation: KableInstallation | null;
}) {
  ondownload?.(event);
}

// Handle viewing gallery
function handleViewGallery(event: { shader: ShaderDownload }) {
  selectedShaderForGallery = event.shader;
  showGalleryModal = true;
}

// Close gallery modal
function closeGallery() {
  showGalleryModal = false;
  selectedShaderForGallery = null;
}

// Initialize on mount
onMount(async () => {
  shadersService = new ShadersService();
  await shadersService.initialize();

  // Default to global mode
  selectedInstallationId = "global";

  // Mark as fully mounted after initialization
  isFullyMounted = true;

  console.log("[ShaderBrowser] Fully mounted and initialized");
});
</script>

<div class="shader-browser">
  <!-- Compact Header -->
  <div class="browser-header">
    <div class="header-main">
      <h2>Shader Browser</h2>

      <!-- Installation Selector -->
      <div class="installation-selector-inline">
        <label for="installation-select-inline">
          <Icon
            name={selectedInstallationId === "global" ? "globe" : "package"}
            size="sm"
          />
          <span>Install to:</span>
        </label>
        <select
          id="installation-select-inline"
          class="installation-select"
          bind:value={selectedInstallationId}
        >
          <option value="global">🌍 Global (All Installations)</option>
          {#if $installations.length > 0}
            <optgroup label="Installations">
              {#each $installations as installation}
                <option value={installation.id}
                  >📦 {installation.name ?? installation.version_id}</option
                >
              {/each}
            </optgroup>
          {/if}
        </select>
      </div>
    </div>
  </div>

  <!-- Main Content Area -->
  <div class="browser-main">
    <!-- Filters Sidebar -->
    <div class="filters-sidebar" class:collapsed={!showFilters}>
      <div class="filters-header">
        <h3>Filters</h3>
        <div class="filters-actions">
          <button
            class="reset-filters"
            on:click={resetFilters}
            title="Reset all filters"
          >
            <Icon name="refresh" size="sm" forceType="svg" />
          </button>
          <button
            class="toggle-filters"
            on:click={() => (showFilters = !showFilters)}
            title="Toggle filters"
          >
            <Icon
              name={showFilters ? "arrow-left" : "arrow-right"}
              size="sm"
              forceType="svg"
            />
          </button>
        </div>
      </div>

      {#if showFilters}
        <div class="filters-content">
          <!-- Smart Filtering Toggle -->
          <div class="filter-section smart-filter-section">
            <label class="smart-filter-toggle">
              <input
                type="checkbox"
                bind:checked={smartFilteringEnabled}
                on:change={onSmartFilteringChange}
              />
              <span
                class="toggle-label"
                title="When enabled, only shows shader packs compatible with your installation's Minecraft version. Disable to browse all shader packs."
              >
                Smart Filtering
              </span>
            </label>
            <p class="smart-filter-hint">
              {smartFilteringEnabled
                ? "Showing shaders compatible with your installation"
                : "Showing all shaders (compatibility not filtered)"}
            </p>
          </div>

          <!-- Search -->
          <div class="filter-section">
            <label class="filter-label" for="search">Search</label>
            <div class="search-input-wrapper">
              <Icon name="search" size="sm" />
              <input
                type="text"
                placeholder="Search shaders..."
                bind:value={searchQuery}
                on:input={handleSearch}
                class="search-input"
              />
              {#if searchQuery}
                <button
                  class="clear-btn"
                  on:click={() => {
                    searchQuery = "";
                    handleSearch();
                  }}
                >
                  <Icon name="x" size="sm" />
                </button>
              {/if}
            </div>
          </div>

          <!-- Dynamic Filter Sections -->
          {#each filterSections as section}
            <div class="filter-section">
              <button
                class="filter-header"
                on:click={() => toggleSection(section.collapsedKey)}
              >
                <span class="filter-label">{section.label}</span>
                <Icon
                  name={collapsedSections[section.collapsedKey]
                    ? "chevron-down"
                    : "chevron-up"}
                  size="lg"
                  forceType="svg"
                />
              </button>
              {#if !collapsedSections[section.collapsedKey]}
                <div class="filter-options">
                  {#each section.options as option}
                    <div
                      class="filter-option"
                      class:included={getFilterState(section.id, option) ===
                        "include"}
                      class:excluded={getFilterState(section.id, option) ===
                        "exclude"}
                    >
                      <button
                        class="filter-option-btn include-btn"
                        class:active={getFilterState(section.id, option) ===
                          "include"}
                        on:click={() => toggleFilter(section.id, option)}
                        title={getFilterState(section.id, option) === "include"
                          ? "Remove filter"
                          : "Include filter"}
                      >
                        <span class="option-label">{option}</span>
                        {#if getFilterState(section.id, option) === "include"}
                          <Icon name="x" size="sm" forceType="svg" />
                        {:else}
                          <Icon name="check" size="sm" forceType="svg" />
                        {/if}
                      </button>
                      <button
                        class="filter-option-btn exclude-btn"
                        class:active={getFilterState(section.id, option) ===
                          "exclude"}
                        on:click={() => toggleFilterExclude(section.id, option)}
                        title={getFilterState(section.id, option) === "exclude"
                          ? "Remove exclusion"
                          : "Exclude filter"}
                      >
                        <Icon name="trash" size="sm" forceType="svg" />
                      </button>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Content Area -->
    <div class="content-area">
      <!-- Toolbar -->
      <div class="content-toolbar">
        <div class="toolbar-left">
          <p class="results-count">
            {#if loading}
              Loading...
            {:else if error}
              Error: {error}
            {:else if paginatedShaders.length === 0}
              No shaders found
            {:else}
              Showing {paginatedShaders.length} shader{paginatedShaders.length !==
              1
                ? "s"
                : ""}
            {/if}
          </p>

          <!-- Compact Pagination Controls -->
          <div class="compact-pagination">
            <button
              class="page-btn compact"
              on:click={previousPage}
              disabled={currentPage === 1}
              title="Previous page"
            >
              <Icon name="arrow-left" size="sm" forceType="svg" />
            </button>

            {#each pageNumbers as pageItem}
              {#if pageItem === "ellipsis"}
                <span class="pagination-ellipsis">...</span>
              {:else}
                <button
                  class="page-btn compact"
                  class:active={currentPage === pageItem}
                  on:click={() => goToPage(pageItem)}
                >
                  {pageItem}
                </button>
              {/if}
            {/each}

            <button
              class="page-btn compact"
              on:click={nextPage}
              title="Next page"
            >
              <Icon name="arrow-right" size="sm" forceType="svg" />
            </button>
          </div>
        </div>

        <div class="toolbar-right">
          <div class="view-controls">
            {#each viewModes as mode}
              <button
                class="view-mode-btn"
                class:active={viewMode === mode.id}
                on:click={() => (viewMode = mode.id as ViewMode)}
                title={mode.name}
              >
                <Icon name={mode.icon} size="sm" />
              </button>
            {/each}
          </div>

          <select
            class="page-size-select"
            bind:value={itemsPerPage}
            on:change={() => changePageSize(itemsPerPage)}
          >
            {#each pageSizeOptions as size}
              <option value={size}>{size} per page</option>
            {/each}
          </select>
        </div>
      </div>

      <!-- Shaders Content -->
      <div class="shaders-content">
        {#if loading}
          <div class="loading-state">
            <Icon name="loader" size="xl" />
            <p>Loading shader packs...</p>
          </div>
        {:else if error}
          <div class="error-state">
            <Icon name="alert-circle" size="xl" />
            <h3>Error Loading Shaders</h3>
            <p>{error}</p>
            <button class="retry-btn" on:click={loadShaders}>
              <Icon name="refresh" size="sm" />
              Retry
            </button>
          </div>
        {:else if paginatedShaders.length === 0}
          <div class="empty-state">
            <Icon name="inbox" size="xl" />
            <h3>No Shaders Found</h3>
            <p>Try adjusting your search or filters</p>
            {#if searchQuery || filters.loader.length > 0 || filters.categories.length > 0}
              <button class="clear-filters-btn" on:click={resetFilters}>
                <Icon name="refresh" size="sm" />
                Clear Filters
              </button>
            {/if}
          </div>
        {:else}
          <!-- Shaders Grid/List -->
          <div
            class="shaders-container"
            class:grid={viewMode === "grid"}
            class:list={viewMode === "list"}
            class:compact={viewMode === "compact"}
          >
            {#each paginatedShaders as shader}
              <ShaderCard
                {shader}
                {viewMode}
                installation={installMode === "dedicated"
                  ? currentInstallation
                  : null}
                loading={false}
                isInstalled={false}
                ondownload={handleDownload}
                onviewgallery={handleViewGallery}
              />
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<!-- Gallery Modal -->
<ShaderGalleryModal
  shader={selectedShaderForGallery}
  bind:visible={showGalleryModal}
  on:close={closeGallery}
/>

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;
@use "sass:color";

.shader-browser {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--container);
  border-radius: 0.5rem;
  border: 1px solid #{"color-mix(in srgb, var(--primary), 8%, transparent)"};
  box-shadow: 0 2px 8px
    #{"color-mix(in srgb, var(--dark-900), 4%, transparent)"};
  overflow: hidden;
}

// Compact Header
.browser-header {
  background: linear-gradient(
    135deg,
    #{"color-mix(in srgb, var(--container), 95%, transparent)"} 0%,
    #{"color-mix(in srgb, var(--primary), 4%, transparent)"} 30%,
    #{"color-mix(in srgb, var(--secondary), 2%, transparent)"} 70%,
    #{"color-mix(in srgb, var(--card), 80%, transparent)"} 100%
  );
  backdrop-filter: blur(12px);
  border-bottom: 1px solid
    #{"color-mix(in srgb, var(--primary), 15%, transparent)"};
  padding: 0.75rem 1rem;
  position: relative;

  &::before {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(
      90deg,
      transparent 0%,
      #{"color-mix(in srgb, var(--primary), 30%, transparent)"} 20%,
      #{"color-mix(in srgb, var(--secondary), 20%, transparent)"} 80%,
      transparent 100%
    );
  }

  .header-main {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;

    h2 {
      margin: 0;
      background: linear-gradient(
        135deg,
        var(--primary) 0%,
        var(--secondary) 100%
      );
      background-clip: text;
      -webkit-background-clip: text;
      color: transparent;
      font-weight: 700;
      font-size: 1.2em;
      white-space: nowrap;
    }
  }

  .installation-selector-inline {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex: 1;
    max-width: 400px;

    label {
      display: flex;
      align-items: center;
      gap: 0.375rem;
      font-size: 0.8em;
      font-weight: 500;
      color: var(--text);
      white-space: nowrap;
    }

    .installation-select {
      flex: 1;
      padding: 0.5rem 0.75rem;
      border: 1px solid
        #{"color-mix(in srgb, var(--primary), 15%, transparent)"};
      border-radius: 0.375rem;
      background: var(--card);
      color: var(--text);
      font-size: 0.85em;
      font-weight: 500;
      cursor: pointer;
      transition: all 0.15s;

      &:focus {
        outline: none;
        border-color: var(--primary);
        box-shadow: 0 0 0 2px
          #{"color-mix(in srgb, var(--primary), 10%, transparent)"};
      }

      &:hover {
        border-color: var(--primary);
        background: #{"color-mix(in srgb, var(--primary), 3%, transparent)"};
      }

      option {
        background: var(--card);
        color: var(--text);
        padding: 0.5rem;
      }

      optgroup {
        font-weight: 600;
        color: var(--placeholder);
      }
    }
  }
}

// Main Layout
.browser-main {
  display: flex;
  overflow: hidden;
}

// Filters Sidebar
.filters-sidebar {
  width: 240px;
  background: linear-gradient(
    135deg,
    #{"color-mix(in srgb, var(--container), 95%, transparent)"} 0%,
    #{"color-mix(in srgb, var(--card), 80%, transparent)"} 100%
  );
  backdrop-filter: blur(8px);
  border-right: 1px solid
    #{"color-mix(in srgb, var(--primary), 12%, transparent)"};
  display: flex;
  flex-direction: column;
  transition: all 0.3s ease;

  &.collapsed {
    width: 48px;

    .filters-content {
      display: none;
    }

    .filters-header {
      h3 {
        display: none;
      }

      .reset-filters {
        display: none;
      }

      justify-content: center;
    }
  }

  .filters-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    height: 2.6875rem;
    border-bottom: 1px solid
      #{"color-mix(in srgb, var(--primary), 12%, transparent)"};
    background: linear-gradient(
      135deg,
      #{"color-mix(in srgb, var(--primary), 6%, transparent)"} 0%,
      #{"color-mix(in srgb, var(--secondary), 3%, transparent)"} 100%
    );
    backdrop-filter: blur(4px);

    h3 {
      margin: 0;
      font-size: 0.9em;
      font-weight: 600;
      color: var(--text);
    }

    .filters-actions {
      display: flex;
      gap: 0.25rem;

      .reset-filters,
      .toggle-filters {
        padding: 0.25rem;
        border: none;
        border-radius: 0.25rem;
        background: transparent;
        color: var(--placeholder);
        cursor: pointer;
        transition: all 0.15s;

        &:hover {
          background: #{"color-mix(in srgb, var(--primary), 10%, transparent)"};
          color: var(--primary);
        }
      }
    }
  }

  .filters-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 0.5rem;

    .filter-section {
      margin-bottom: 0.75rem;

      .filter-label {
        display: block;
        font-size: 0.95em;
        font-weight: 600;
        color: var(--text);
        margin-bottom: 0.375rem;
        text-transform: uppercase;
        letter-spacing: 0.5px;
      }

      .filter-header {
        width: 100%;
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0.375rem 0.5rem;
        background: #{"color-mix(in srgb, var(--primary), 5%, transparent)"};
        border: 1px solid
          #{"color-mix(in srgb, var(--primary), 12%, transparent)"};
        border-radius: 0.25rem;
        cursor: pointer;
        transition: all 0.15s;
        margin-bottom: 0.375rem;

        &:hover {
          background: #{"color-mix(in srgb, var(--primary), 8%, transparent)"};
          border-color: var(--primary);
        }

        .filter-label {
          margin: 0;
          text-align: left;
        }
      }

      .search-input-wrapper {
        position: relative;
        display: flex;
        align-items: center;

        :global(.icon) {
          position: absolute;
          left: 0.5rem;
          color: var(--placeholder);
          z-index: 1;
        }

        .search-input {
          max-width: 90%;
          min-width: fit-content;
          padding: 0.5rem 0.5rem 0.5rem 2rem;
          border: 1px solid var(--dark-600);
          border-radius: 0.375rem;
          background: var(--input);
          color: var(--text);
          font-size: 0.8em;

          &:focus {
            outline: none;
            border-color: var(--primary);
            box-shadow: 0 0 0 2px
              #{"color-mix(in srgb, var(--primary), 10%, transparent)"};
          }

          &::placeholder {
            color: var(--placeholder);
          }
        }

        .clear-btn {
          position: absolute;
          right: 0.375rem;
          background: none;
          border: none;
          color: var(--placeholder);
          cursor: pointer;
          padding: 0.125rem;
          border-radius: 0.125rem;

          &:hover {
            color: var(--red);
            background: #{"color-mix(in srgb, var(--red), 10%, transparent)"};
          }
        }
      }

      .filter-options {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;

        .filter-option {
          display: flex;
          border-radius: 0.25rem;
          overflow: hidden;
          border: 1px solid var(--dark-600);
          transition: all 0.15s;

          &:hover {
            border-color: var(--dark-400);
          }

          &.included {
            border-color: var(--green);
            background: #{"color-mix(in srgb, var(--primary), 5%, transparent)"};
          }

          &.excluded {
            border-color: var(--red);
            background: #{"color-mix(in srgb, var(--red), 5%, transparent)"};
          }

          .filter-option-btn {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0.5rem 0.625rem;
            background: transparent;
            border: none;
            color: var(--text);
            cursor: pointer;
            transition: all 0.15s;

            &:hover {
              background: #{"color-mix(in srgb, var(--primary), 3%, transparent)"};
            }

            .option-label {
              font-size: 0.75em;
              font-weight: 500;
              text-align: left;
              text-transform: capitalize;
              flex: 1;
            }

            :global(.icon) {
              color: var(--placeholder);
              transition: color 0.15s;
            }

            &.include-btn {
              flex: 1;
              gap: 0.5rem;

              &.active {
                :global(.icon) {
                  color: var(--red);
                }
              }

              &:not(.active):hover {
                :global(.icon) {
                  color: var(--green);
                }
              }
            }

            &.exclude-btn {
              padding: 0.5rem;
              border-left: 1px solid var(--dark-600);
              min-width: 2rem;
              justify-content: center;

              &.active {
                background: #{"color-mix(in srgb, var(--red), 10%, transparent)"};

                :global(.icon) {
                  color: var(--red);
                }
              }

              &:not(.active):hover {
                :global(.icon) {
                  color: var(--red);
                }
              }
            }
          }
        }
      }
    }

    .smart-filter-section {
      padding: 0.75rem;
      background: #{"color-mix(in srgb, var(--primary), 5%, transparent)"};
      border: 1px solid
        #{"color-mix(in srgb, var(--primary), 15%, transparent)"};
      border-radius: 0.375rem;
      margin-bottom: 0.75rem;

      .smart-filter-toggle {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        cursor: pointer;
        user-select: none;

        input[type="checkbox"] {
          width: 16px;
          height: 16px;
          cursor: pointer;
          accent-color: var(--primary);
        }

        .toggle-label {
          font-size: 0.85em;
          font-weight: 600;
          color: var(--text);
        }
      }

      .smart-filter-hint {
        margin: 0.375rem 0 0 1.5rem;
        font-size: 0.7em;
        color: var(--placeholder);
        line-height: 1.3;
      }
    }
  }
}

// Content Area
.content-area {
  overflow: hidden;
  width: 100%;
  display: flex;
  flex-direction: column;
}

// Toolbar
.content-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  height: 2.6875rem;
  background: linear-gradient(
    135deg,
    var(--container) 0%,
    #{"color-mix(in srgb, var(--card), 60%, transparent)"} 100%
  );
  backdrop-filter: blur(6px);
  border-bottom: 1px solid
    #{"color-mix(in srgb, var(--primary), 12%, transparent)"};

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;

    .results-count {
      font-size: 0.75em;
      color: var(--placeholder);
      font-weight: 500;
      margin: 0;
    }

    .compact-pagination {
      display: flex;
      align-items: center;
      gap: 0.25rem;

      .page-btn.compact {
        padding: 0.25rem 0.375rem;
        border: 1px solid
          #{"color-mix(in srgb, var(--primary), 20%, transparent)"};
        border-radius: 0.25rem;
        background: #{"color-mix(in srgb, var(--card), 80%, transparent)"};
        color: var(--text);
        font-size: 0.7em;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.15s;
        min-width: 24px;
        height: 24px;
        display: flex;
        align-items: center;
        justify-content: center;

        &:hover:not(:disabled) {
          border-color: var(--primary);
          background: #{"color-mix(in srgb, var(--primary), 10%, transparent)"};
          color: var(--primary);
        }

        &.active {
          background: var(--card);
          color: var(--primary);
          border-color: var(--primary);
          font-weight: 600;
        }

        &:disabled {
          opacity: 0.4;
          cursor: not-allowed;
          background: #{"color-mix(in srgb, var(--card), 40%, transparent)"};
        }
      }

      .pagination-ellipsis {
        padding: 0.125rem 0.25rem;
        color: var(--placeholder);
        font-size: 0.7em;
        font-weight: 500;
      }
    }
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 0.75rem;

    .view-controls {
      display: flex;
      border: 1px solid var(--dark-600);
      border-radius: 0.25rem;
      overflow: hidden;

      .view-mode-btn {
        padding: 0.25rem 0.375rem;
        border: none;
        background: var(--card);
        color: var(--placeholder);
        cursor: pointer;
        transition: all 0.15s;

        &:hover {
          background: #{"color-mix(in srgb, var(--primary), 5%, transparent)"};
          color: var(--text);
        }

        &.active {
          background: var(--primary);
          color: white;
        }

        &:not(:last-child) {
          border-right: 1px solid var(--dark-600);
        }
      }
    }

    .page-size-select {
      padding: 0.25rem 0.375rem;
      border: 1px solid var(--dark-600);
      border-radius: 0.25rem;
      background: var(--card);
      color: var(--text);
      font-size: 0.75em;
      cursor: pointer;

      &:focus {
        outline: none;
        border-color: var(--primary);
      }
    }
  }
}

// Shaders Content
.shaders-content {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow-y: scroll;

  &::-webkit-scrollbar {
    width: 8px;
  }

  &::-webkit-scrollbar-track {
    background: #{"color-mix(in srgb, var(--dark-600), 10%, transparent)"};
    border-radius: 4px;
  }

  &::-webkit-scrollbar-thumb {
    background: linear-gradient(
      135deg,
      #{"color-mix(in srgb, var(--primary), 60%, transparent)"} 0%,
      #{"color-mix(in srgb, var(--secondary), 40%, transparent)"} 100%
    );
    border-radius: 4px;

    &:hover {
      background: linear-gradient(
        135deg,
        #{"color-mix(in srgb, var(--primary), 80%, transparent)"} 0%,
        #{"color-mix(in srgb, var(--secondary), 60%, transparent)"} 100%
      );
    }
  }
}

// Loading/Error/Empty States
.loading-state,
.error-state,
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  gap: 0.75rem;
  color: var(--placeholder);
}

.error-state,
.empty-state {
  h3 {
    margin: 0;
    color: var(--text);
    font-weight: 600;
    font-size: 1.1em;
  }

  p {
    margin: 0;
    text-align: center;
    max-width: 400px;
    line-height: 1.4;
    font-size: 0.9em;
  }
}

.error-state .retry-btn,
.empty-state .clear-filters-btn {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 1rem;
  border: 1px solid var(--primary);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--primary), 10%, transparent);
  color: var(--primary);
  font-weight: 500;
  font-size: 0.8em;
  cursor: pointer;
  transition: all 0.15s;

  &:hover {
    background: var(--primary);
    color: white;
  }
}

// Shaders Container
.shaders-container {
  padding: 0.75rem;

  &.grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 0.75rem;
  }

  &.list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  &.compact {
    display: flex;
    flex-wrap: wrap;
    gap: 0.375rem;
    align-items: stretch;
    justify-content: space-between;
  }
}

// Responsive Design
@media (max-width: 768px) {
  .browser-header {
    .header-main {
      flex-direction: column;
      align-items: stretch;
      gap: 0.5rem;

      h2 {
        text-align: center;
      }
    }

    .installation-selector-inline {
      max-width: 100%;

      label {
        font-size: 0.75em;
      }

      .installation-select {
        font-size: 0.8em;
      }
    }
  }

  .browser-main {
    flex-direction: column;
  }

  .filters-sidebar {
    width: 100%;
    max-height: 200px;
    border-right: none;
    border-bottom: 1px solid
      #{"color-mix(in srgb, var(--primary), 15%, transparent)"};
    background: linear-gradient(
      135deg,
      #{"color-mix(in srgb, var(--container), 90%, transparent)"} 0%,
      #{"color-mix(in srgb, var(--card), 70%, transparent)"} 100%
    );

    &.collapsed {
      max-height: 48px;
    }
  }

  .shaders-container {
    padding: 0.5rem;

    &.grid {
      grid-template-columns: 1fr;
    }
  }
}
</style>
