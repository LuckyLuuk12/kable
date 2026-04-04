<!-- @component
ModBrowser - Browse and search for mods from Modrinth and other providers

Provides interface for discovering, searching, and filtering mods with support for:
- Multiple view modes (grid, list, compact)
- Advanced filtering (categories, loaders, versions)
- Pagination and infinite scroll
- Provider switching

@prop {((event: { modId: string; versionId?: string; installation: KableInstallation }) =► void) | undefined} ondownloadmod - Callback when user clicks to download a mod

@example
```svelte
◄ModBrowser ondownloadmod={handleDownload} /►
```
-->
<script lang="ts">
import type {
  FilterFacets,
  KableInstallation,
  ModInfoKind,
  ModJarInfo,
  ModpackContext,
  MrPackDetailed,
  NormalizedModInfo,
} from "$lib";
import {
  Icon,
  InstallationService,
  ModsService,
  ProviderKind,
  installations,
  modsByProvider,
  modsError,
  modsLoading,
  modsOffset,
  modsProvider,
  selectedInstallation,
} from "$lib";
import { clickSound } from "$lib/actions";
import * as modsApi from "$lib/api/mods";
import * as systemApi from "$lib/api/system";
import ModpackDiffModal from "$lib/components/mods/ModpackDiffModal.svelte";
import { onMount } from "svelte";
import { get } from "svelte/store";
import ModCard from "./ModCard.svelte";

type ViewMode = "grid" | "list" | "compact";

export let ondownloadmod:
  | ((event: {
      modId: string;
      versionId?: string;
      installation: KableInstallation;
    }) => void)
  | undefined = undefined;

// Browser state
let currentProvider: ProviderKind = ProviderKind.Modrinth;
let viewMode: ViewMode = "grid";
let searchQuery = "";
let currentInstallation: KableInstallation | null = null;
let showFilters = true;
let smartFilteringEnabled = true; // Auto-apply loader and game version filters

// Filter state with include/exclude support
type FilterMode = "include" | "exclude";
type FilterItem = { value: string; mode: FilterMode };

let filters = {
  categories: [] as FilterItem[],
  environment: [] as FilterItem[],
  license: [] as FilterItem[],
};

// Collapsible filter sections
let collapsedSections = {
  categories: false,
  environment: false,
  license: false,
};

// Pagination state
let currentPage = 1;
let itemsPerPage = 20;
let visitedPages = new Set([1]); // Track pages that had results
let maxPageReached = 1; // Highest page number user has visited

// Installed mods tracking
let installedMods: ModJarInfo[] = [];
let installedModsLoaded = false;
let installedModsLoadToken = 0;

// Cache of installed status for displayed mods: project_id -> {isInstalled, version}
let installedStatusCache = new Map<
  string,
  { isInstalled: boolean; version: string | null }
>();
// Counter to force reactivity updates
let cacheUpdateCounter = 0;

// Service instance
let modsService: ModsService;

// Available providers
const providers: {
  id: ProviderKind;
  name: string;
  description: string;
  available: boolean;
}[] = [
  {
    id: ProviderKind.Modrinth,
    name: "Modrinth",
    description: "Flexible API, though try not to spam refresh this tab",
    available: true,
  },
  {
    id: ProviderKind.CurseForge,
    name: "CurseForge",
    description: "Might not work because of API limitations",
    available: true,
  },
];

// View mode options
const viewModes = [
  { id: "grid", name: "Grid", icon: "grid" },
  { id: "list", name: "List", icon: "list" },
  { id: "compact", name: "Compact", icon: "layout" },
];

// Page size options
const pageSizeOptions = [10, 20, 50, 100];

// Filter configuration
const filterSections = [
  {
    id: "environment" as const,
    label: "Environment",
    collapsedKey: "environment" as const,
    options: ["Client", "Server"],
  },
  {
    id: "categories" as const,
    label: "Categories",
    collapsedKey: "categories" as const,
    options: [
      "Adventure",
      "Cursed",
      "Decoration",
      "Economy",
      "Equipment",
      "Food",
      "Game Mechanics",
      "Library",
      "Magic",
      "Management",
      "Minigame",
      "Mobs",
      "Optimization",
      "Social",
      "Storage",
      "Technology",
      "Transportation",
      "Utility",
      "World Generation",
    ],
  },
  {
    id: "license" as const,
    label: "License",
    collapsedKey: "license" as const,
    options: ["Open Source"],
  },
];

// Reactive statements
$: currentInstallation = $selectedInstallation;
$: mods = $modsByProvider[currentProvider] || [];
$: loading = $modsLoading;
$: error = $modsError;
// Don't apply client-side filters - let backend handle it
$: paginatedMods = mods;
$: totalMods = mods.length;

// Load installed mods when installation changes
$: if (currentInstallation) {
  installedModsLoaded = false;
  loadInstalledMods(currentInstallation);
}

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
  if (!modsService) return;

  // Build filter object for backend
  const includeCategories = filters.categories
    .filter((f) => f.mode === "include")
    .map((f) => f.value);
  const excludeCategories = filters.categories
    .filter((f) => f.mode === "exclude")
    .map((f) => f.value);
  const includeEnvironments = filters.environment
    .filter((f) => f.mode === "include")
    .map((f) => f.value);
  const excludeEnvironments = filters.environment
    .filter((f) => f.mode === "exclude")
    .map((f) => f.value);
  const openSource = filters.license.some(
    (f) => f.mode === "include" && f.value === "Open Source",
  );

  // If no filters and no search, clear filters
  if (
    !searchQuery &&
    includeCategories.length === 0 &&
    excludeCategories.length === 0 &&
    includeEnvironments.length === 0 &&
    excludeEnvironments.length === 0 &&
    !openSource
  ) {
    currentPage = 1;
    modsOffset.set(0);
    // Only pass installation if smart filtering is enabled
    await modsService.setFilter(
      null,
      smartFilteringEnabled ? currentInstallation : null,
    );
    return;
  }

  // Build filter facets for Modrinth (backend expects specific format)
  // Note: FilterFacets uses tuples [operation, value] for filters
  // Operations: ':' or '=' for include, '!=' for exclude
  // See: https://docs.modrinth.com/api/operations/searchprojects/

  // Each filter is AND'd together - put each in separate array
  // To OR filters together, put them in the same array
  // Example: [["categories:adventure"], ["categories!=equipment"]] = adventure AND (not equipment)
  // Example: [["categories:adventure", "categories:magic"]] = adventure OR magic

  const includeCategoryFilters: [string, string][] = includeCategories.map(
    (c) => [":", c.toLowerCase()] as [string, string],
  );
  const excludeCategoryFilters: [string, string][] = excludeCategories.map(
    (c) => ["!=", c.toLowerCase()] as [string, string],
  );

  // Each filter is its own entry (AND'd together)
  const categoryFilters = [
    ...includeCategoryFilters,
    ...excludeCategoryFilters,
  ];

  const filterFacets: FilterFacets = {
    query: searchQuery || undefined,
    categories: categoryFilters.length > 0 ? categoryFilters : undefined,
    // For environment, handle both include and exclude
    // If both Client include and exclude, prioritize exclude (!=)
    client_side: excludeEnvironments.includes("Client")
      ? ["!=", "required"]
      : includeEnvironments.includes("Client")
        ? [":", "required"]
        : undefined,
    server_side: excludeEnvironments.includes("Server")
      ? ["!=", "required"]
      : includeEnvironments.includes("Server")
        ? [":", "required"]
        : undefined,
    index: undefined,
    open_source: openSource || undefined,
    license: undefined,
    downloads: undefined,
  };

  // Wrap in ModFilter discriminated union based on current provider
  // Rust enum format uses externally tagged representation: { "Modrinth": {...} }
  const modFilter =
    currentProvider === ProviderKind.Modrinth
      ? { Modrinth: filterFacets }
      : { CurseForge: filterFacets }; // TODO: Implement proper CurseForge filter format

  console.log(
    "[ModBrowser] Applying filters to backend:",
    JSON.stringify(modFilter, null, 2),
  );

  // Reset to first page when filters change
  currentPage = 1;
  modsOffset.set(0);

  try {
    // Only pass installation if smart filtering is enabled
    await modsService.setFilter(
      modFilter,
      smartFilteringEnabled ? currentInstallation : null,
    );
  } catch (e) {
    console.error("[ModBrowser] Failed to apply filters:", e);
  }
}

// Handle search query changes - debounced via handleFiltersChange
async function handleSearch() {
  if (!modsService) return;
  handleFiltersChange();
}

// Filter helper functions
function toggleFilter(
  category: "categories" | "environment" | "license",
  value: string,
) {
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
  handleSearch();
}

function toggleFilterExclude(
  category: "categories" | "environment" | "license",
  value: string,
) {
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
  handleSearch();
}

function getFilterState(
  category: "categories" | "environment" | "license",
  value: string,
): FilterMode | null {
  const filter = filters[category].find((f) => f.value === value);
  return filter ? filter.mode : null;
}

function toggleSection(section: keyof typeof collapsedSections) {
  collapsedSections[section] = !collapsedSections[section];
}

function resetFilters() {
  filters = {
    categories: [],
    environment: [],
    license: [],
  };
  searchQuery = "";
  currentPage = 1;
  // Don't reset visitedPages or maxPageReached - keep pagination history
  modsOffset.set(0);
  if (modsService) {
    modsService.setFilter(null, currentInstallation);
  }
}

// Load installed mods for the current installation
async function loadInstalledMods(installation: KableInstallation) {
  const loadToken = ++installedModsLoadToken;

  try {
    installedMods = await InstallationService.getModInfo(installation);

    // Load metadata for all installed mods to get project IDs
    const metadataPromises = installedMods.map(async (mod) => {
      try {
        const metadata = await modsApi.getModMetadata(
          installation,
          mod.file_name,
        );
        return {
          fileName: mod.file_name,
          projectId: metadata.project_id,
          version: metadata.version_number,
        };
      } catch (e) {
        return null;
      }
    });

    const metadataResults = await Promise.all(metadataPromises);

    // Ignore stale async results if installation switched while loading
    if (loadToken !== installedModsLoadToken) {
      return;
    }

    const nextInstalledStatusCache = new Map<
      string,
      { isInstalled: boolean; version: string | null }
    >();

    // Build cache from metadata (use project_id as key)
    for (const result of metadataResults) {
      if (result) {
        nextInstalledStatusCache.set(result.projectId, {
          isInstalled: true,
          version: result.version,
        });
      }
    }

    installedStatusCache = nextInstalledStatusCache;
    installedModsLoaded = true;
    // Force reactivity update by incrementing counter
    cacheUpdateCounter++;
    console.log(
      `[ModBrowser] Loaded ${installedMods.length} installed mods, ${installedStatusCache.size} with metadata project IDs`,
    );
  } catch (e) {
    if (loadToken !== installedModsLoadToken) {
      return;
    }
    console.error("[ModBrowser] Failed to load installed mods:", e);
    installedMods = [];
    installedStatusCache = new Map();
    installedModsLoaded = true;
    cacheUpdateCounter++;
  }
}

// Get cached installed info (synchronous, for template use)
// Include cacheUpdateCounter to make this reactive to cache updates
function getCachedInstalledInfo(
  mod: ModInfoKind,
  _counter = cacheUpdateCounter,
): { isInstalled: boolean; version: string | null } {
  const projectId = ModsService.getProjectId(mod);
  if (!projectId) {
    return { isInstalled: false, version: null };
  }

  return (
    installedStatusCache.get(projectId) || { isInstalled: false, version: null }
  );
}

// Get unique key for each mod (for keyed each blocks)
function getModKey(mod: ModInfoKind): string {
  return ModsService.getModKey(mod);
}

// Track if we're currently initializing to prevent duplicate calls
let isInitializing = false;

// Initialize service when provider changes
$: if (
  currentProvider &&
  currentProvider !== $modsProvider &&
  !isInitializing
) {
  initializeProvider();
}

async function initializeProvider() {
  if (isInitializing) {
    console.log("[ModBrowser] Already initializing, skipping duplicate call");
    return;
  }

  isInitializing = true;
  try {
    console.log(`[ModBrowser] Initializing provider: ${currentProvider}`);
    modsService = new ModsService(currentProvider);
    await modsService.initialize();

    // Apply installation-based filters if available
    if (currentInstallation) {
      await applyInstallationFilters();
    }
  } catch (e) {
    console.error("Failed to initialize provider:", e);
  } finally {
    isInitializing = false;
  }
}

async function applyInstallationFilters() {
  if (!modsService) return;

  // Apply filters based on installation (loader, MC version, etc.)
  // Only pass installation if smart filtering is enabled
  try {
    await modsService.setFilter(
      null,
      smartFilteringEnabled ? currentInstallation : null,
    );
  } catch (e) {
    console.error("Failed to apply installation filters:", e);
  }
}

// Handle smart filtering toggle
async function onSmartFilteringChange() {
  console.log(
    `[ModBrowser] Smart filtering ${smartFilteringEnabled ? "enabled" : "disabled"}`,
  );

  // Re-apply filters with new setting
  if (modsService) {
    await applyFiltersToBackend();
  }
}

async function changeProvider(provider: ProviderKind) {
  if (!providers.find((p) => p.id === provider)?.available) return;
  currentProvider = provider;
  currentPage = 1;
  visitedPages = new Set([1]);
  maxPageReached = 1;
  modsOffset.set(0);
}

async function changePageSize(newSize: number) {
  itemsPerPage = newSize;
  currentPage = 1;
  // Don't reset pagination history when changing page size
  modsOffset.set(0);

  if (modsService) {
    await modsService.setLimit(newSize);
  }
}

async function goToPage(page: number) {
  if (page < 1) return;
  currentPage = page;

  // Track this page as visited and update max reached
  visitedPages.add(page);
  if (page > maxPageReached) {
    maxPageReached = page;
  }

  if (modsService) {
    const offset = (page - 1) * itemsPerPage;
    console.log(
      `[ModBrowser] Going to page ${page}, setting offset to ${offset}`,
    );
    modsOffset.set(offset);
    await modsService.loadMods();
  }
}

async function nextPage() {
  await goToPage(currentPage + 1);
}

async function prevPage() {
  if (currentPage > 1) {
    await goToPage(currentPage - 1);
  }
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

function handleModDownload(mod: ModInfoKind) {
  if (!currentInstallation) {
    alert("Please select an installation first");
    return;
  }

  let normalized: NormalizedModInfo;
  try {
    normalized = ModsService.normalizeMod(mod);
  } catch {
    console.error("[ModBrowser] Unknown mod provider format:", mod);
    return;
  }

  ondownloadmod?.({
    modId: normalized.projectId,
    versionId: normalized.versionId || undefined,
    installation: currentInstallation,
  });
}

function handleDownloadVersion(event: {
  mod: ModInfoKind;
  versionId: string;
  versionNumber: string;
}) {
  const { mod, versionId } = event;

  if (!currentInstallation) {
    alert("Please select an installation first");
    return;
  }

  let normalized: NormalizedModInfo;
  try {
    normalized = ModsService.normalizeMod(mod);
  } catch {
    console.error("[ModBrowser] Unknown mod provider format:", mod);
    return;
  }

  // Dispatch with specific version ID
  ondownloadmod?.({
    modId: normalized.projectId,
    versionId,
    installation: currentInstallation,
  });
}

function handleModInfo(mod: ModInfoKind) {
  const url = ModsService.getModInfoUrl(mod);
  if (url) {
    systemApi.openUrl(url);
  }
}

// Event handlers for ModCard component

// Modal state for modpack diff/install

let showModpackModal = false;
let modpackDiff: MrPackDetailed | null = null;
let modpackContext: ModpackContext | null = null;
let downloadError: string | null = null;

async function handleDownloadMod(event: { mod: ModInfoKind }) {
  if (!currentInstallation) {
    downloadError = "Please select an installation first.";
    return;
  }

  try {
    downloadError = null;
    const result = await modsService.downloadOrPrepareFromMod(
      event.mod,
      currentInstallation,
    );

    if (result.kind === "modpack") {
      modpackDiff = result.modpack;
      modpackContext = result.context;
      showModpackModal = true;
    }
  } catch (e: any) {
    downloadError =
      "Failed to download or process mod: " + (e && e.message ? e.message : e);
  }
}

function closeModpackModal() {
  showModpackModal = false;
  modpackDiff = null;
  modpackContext = null;
  // Optionally, refresh installed mods after modal closes
  if (currentInstallation) {
    loadInstalledMods(currentInstallation);
  }
}
function handleInfoMod(event: { mod: ModInfoKind }) {
  handleModInfo(event.mod);
}

function getModDisplayInfo(mod: ModInfoKind): {
  title: string;
  description: string;
  author: string;
  downloads: number;
  icon_url: string | null | undefined;
  categories: string[];
  project_type: string;
  follows?: number;
  client_side?: string;
  server_side?: string;
  game_versions: string[];
  loaders?: string[];
  source_url?: string;
  wiki_url?: string;
  license?: string;
  date_created?: string;
  date_modified?: string;
  latest_version?: string;
} {
  return ModsService.getDisplayInfo(mod);
}

// Initialize on mount
onMount(async () => {
  // Ensure we have a selected installation for mod browsing
  const currentSelected = get(selectedInstallation);
  if (!currentSelected) {
    const availableInstallations = get(installations);
    if (availableInstallations.length > 0) {
      // Select the first available installation if none is selected
      selectedInstallation.set(availableInstallations[0]);
      console.log(
        "[ModBrowser] Auto-selected installation:",
        availableInstallations[0].name,
      );
    } else {
      // No installations available in store; rely on centralized bootstrap (NavBar) to load them.
      console.log(
        "[ModBrowser] No installations available yet; waiting for centralized initialization",
      );
    }
  }

  initializeProvider();
});
</script>

{#if downloadError}
  <div class="error-banner">{downloadError}</div>
{/if}
{#if showModpackModal && modpackDiff && modpackContext}
  <ModpackDiffModal
    open={showModpackModal}
    modpack={modpackDiff}
    context={modpackContext}
    installation={currentInstallation}
    onCancel={closeModpackModal}
  />
{/if}

<div class="mod-browser">
  <!-- Compact Header -->
  <div class="browser-header">
    <div class="header-main">
      <h2>Mod Browser</h2>
      {#if currentInstallation}
        <div class="installation-badge">
          <Icon name="package" size="sm" />
          <span>{currentInstallation.name}</span>
        </div>
      {/if}
    </div>
    <!-- !NOTE: Provider tabs has been disabled for now since curseforge isn't supported yet (as fully as modrinth) -- >
    <div class="provider-tabs">
      {#each providers as provider}
        <button
          class="provider-tab"
          class:active={currentProvider === provider.id}
          class:disabled={!provider.available}
          on:click={() => changeProvider(provider.id)}
          disabled={!provider.available}
          title={provider.description}
        >
          {#if provider.id === ProviderKind.Modrinth}
            <Image
              key="modrinth"
              alt="Modrinth"
              className="provider-icon"
              width="1.25rem"
              height="1.25rem"
            />
          {:else if provider.id === ProviderKind.CurseForge}
            <Image
              key="curseforge"
              alt="CurseForge"
              className="provider-icon"
              width="1.25rem"
              height="1.25rem"
            />
          {:else}
            <Icon name="package" size="sm" />
          {/if}
          {provider.name}
          {#if !provider.available}
            <span class="coming-soon">(Soon)</span>
          {/if}
        </button>
      {/each}
    </div>
-->
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
            use:clickSound
            title="Reset all filters"
          >
            <Icon name="refresh" size="sm" forceType="svg" />
          </button>
          <button
            class="toggle-filters"
            on:click={() => (showFilters = !showFilters)}
            use:clickSound
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
          <!-- Search -->
          <div class="filter-section">
            <label class="filter-label" for="search">Search</label>
            <div class="search-input-wrapper">
              <Icon name="search" size="sm" />
              <input
                type="text"
                placeholder="Search mods..."
                bind:value={searchQuery}
                on:input={handleSearch}
                on:keydown={(e) => {
                  if (e.key === "Enter") handleSearch();
                }}
                class="search-input"
              />
              {#if searchQuery}
                <button
                  class="clear-btn"
                  on:click={() => {
                    searchQuery = "";
                    handleSearch();
                  }}
                  use:clickSound
                >
                  <Icon name="x" size="sm" />
                </button>
              {/if}
            </div>
          </div>

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
                title="When enabled, only shows mods compatible with your installation's loader and Minecraft version. Disable to browse all mods."
              >
                Smart Filtering
              </span>
            </label>
            <p class="smart-filter-hint">
              {smartFilteringEnabled
                ? "Showing mods compatible with your installation"
                : "Showing all mods (compatibility not filtered)"}
            </p>
          </div>

          <!-- Dynamic Filter Sections -->
          {#each filterSections as section}
            <div class="filter-section">
              <button
                class="filter-header"
                on:click={() => toggleSection(section.collapsedKey)}
                use:clickSound
              >
                <span class="filter-label">{section.label}</span>
                <Icon
                  name={collapsedSections[section.collapsedKey]
                    ? "chevron-down"
                    : "chevron-up"}
                  size="md"
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
                        use:clickSound
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
                        use:clickSound
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
          <button
            class="mobile-filters-toggle"
            on:click={() => (showFilters = !showFilters)}
            use:clickSound
          >
            <Icon name="filter" size="sm" />
            Filters
          </button>
          <span class="results-count">{totalMods} mods</span>

          <!-- Compact Pagination Controls -->
          <div class="compact-pagination">
            <button
              class="page-btn compact"
              on:click={prevPage}
              use:clickSound
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
                  use:clickSound
                >
                  {pageItem}
                </button>
              {/if}
            {/each}

            <button
              class="page-btn compact"
              on:click={nextPage}
              use:clickSound
              title="Next page"
            >
              <Icon name="arrow-right" size="sm" forceType="svg" />
            </button>
          </div>
        </div>

        <div class="toolbar-right">
          <!-- View Mode -->
          <div class="view-controls">
            {#each viewModes as mode}
              <button
                class="view-mode-btn"
                class:active={viewMode === mode.id}
                on:click={() => (viewMode = mode.id as ViewMode)}
                use:clickSound
                title={mode.name}
              >
                <Icon name={mode.icon} size="sm" />
              </button>
            {/each}
          </div>

          <!-- Page Size -->
          <select
            bind:value={itemsPerPage}
            on:change={() => changePageSize(itemsPerPage)}
            class="page-size-select"
          >
            {#each pageSizeOptions as size}
              <option value={size}>{size}/page</option>
            {/each}
          </select>
        </div>
      </div>

      <!-- Mods Content -->
      <div class="mods-content">
        {#if loading}
          <div class="loading-state">
            <Icon name="refresh" size="lg" className="spin" />
            <span>Loading mods from {currentProvider}...</span>
          </div>
        {:else if error}
          <div class="error-state">
            <Icon name="warning" size="lg" />
            <h3>Failed to load mods</h3>
            <p>{error}</p>
            <button class="retry-btn" on:click={() => initializeProvider()}>
              <Icon name="refresh" size="sm" />
              Retry
            </button>
          </div>
        {:else if paginatedMods.length === 0}
          <div class="empty-state">
            <Icon name="search" size="xl" />
            <h3>No mods found</h3>
            <p>
              {#if currentInstallation}
                Try adjusting your filters or search for {currentInstallation.name}.
              {:else}
                Select an installation first, then try adjusting your filters.
              {/if}
            </p>

            {#if filters.categories.length > 0 || filters.environment.length > 0 || filters.license.length > 0 || searchQuery}
              <button class="clear-filters-btn" on:click={resetFilters}>
                <Icon name="refresh" size="sm" />
                Clear Filters & Search
              </button>
            {/if}
          </div>
        {:else}
          <!-- Mods Grid/List -->
          <div
            class="mods-container"
            class:grid={viewMode === "grid"}
            class:list={viewMode === "list"}
            class:compact={viewMode === "compact"}
          >
            {#each paginatedMods as mod (getModKey(mod))}
              {@const installedInfo =
                currentInstallation && installedModsLoaded
                  ? getCachedInstalledInfo(mod, cacheUpdateCounter)
                  : { isInstalled: false, version: null }}
              <ModCard
                {mod}
                {viewMode}
                {currentInstallation}
                loading={false}
                isInstalled={installedInfo.isInstalled}
                installedVersion={installedInfo.version}
                ondownloadmod={handleDownloadMod}
                ondownloadversion={handleDownloadVersion}
                oninfomod={handleInfoMod}
              />
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;
@use "sass:color";

.mod-browser {
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
    margin-bottom: 0.5rem;

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
    }

    .installation-badge {
      display: flex;
      align-items: center;
      gap: 0.375rem;
      background: #{"color-mix(in srgb, var(--primary), 8%, transparent)"};
      border: 1px solid
        #{"color-mix(in srgb, var(--primary), 15%, transparent)"};
      border-radius: 0.75rem;
      padding: 0.25rem 0.5rem;
      font-size: 0.75em;
      font-weight: 500;
      color: var(--primary);
    }
  }

  /* !NOTE: Provider tabs has been disabled for now since curseforge isn't supported yet (as fully as modrinth)
  .provider-tabs {
    display: flex;
    gap: 0.375rem;

    .provider-tab {
      display: flex;
      align-items: center;
      gap: 0.375rem;
      padding: 0.375rem 0.75rem;
      border: 1px solid
        #{"color-mix(in srgb, var(--primary), 15%, transparent)"};
      border-radius: 0.375rem;
      background: var(--card);
      color: var(--text);
      font-weight: 500;
      font-size: 0.8em;
      cursor: pointer;
      transition: all 0.15s;

      &:hover:not(:disabled) {
        border-color: var(--primary);
        background: #{"color-mix(in srgb, var(--primary), 5%, transparent)"};
      }

      &.active {
        background: linear-gradient(
          135deg,
          var(--primary) 0%,
          var(--secondary) 100%
        );
        color: white;
        border-color: transparent;
        box-shadow: 0 2px 6px
          #{"color-mix(in srgb, var(--primary), 25%, transparent)"};
      }

      &.disabled {
        opacity: 0.5;
        cursor: not-allowed;

        .coming-soon {
          font-size: 0.75em;
          opacity: 0.7;
        }
      }
    }
  }
  */
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
          border-color: #{"color-mix(in srgb, var(--primary), 20%, transparent)"};
        }

        .filter-label {
          margin: 0;
          font-size: 0.8em;
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
          width: 90%;
          padding: 0.5rem 0.5rem 0.5rem 2rem;
          border: 1px solid
            #{"color-mix(in srgb, var(--primary), 15%, transparent)"};
          border-radius: 0.375rem;
          background: var(--input);
          color: var(--text);
          font-size: 0.8em;
          transition: all 0.15s;

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
          transition: all 0.15s;

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
            background: #{"color-mix(in srgb, var(--green), 5%, transparent)"};
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

              &:hover {
                background: #{"color-mix(in srgb, var(--green), 8%, transparent)"};

                :global(.icon) {
                  color: var(--green);
                }
              }

              &.active {
                background: #{"color-mix(in srgb, var(--green), 12%, transparent)"};
                color: var(--green);

                :global(.icon) {
                  color: var(--green);
                }
              }
            }

            &.exclude-btn {
              padding: 0.5rem;

              &:hover {
                background: #{"color-mix(in srgb, var(--red), 8%, transparent)"};

                :global(.icon) {
                  color: var(--red);
                }
              }

              &.active {
                background: #{"color-mix(in srgb, var(--red), 12%, transparent)"};

                :global(.icon) {
                  color: var(--red);
                }
              }
            }
          }
        }
      }
    }

    // Smart Filter Toggle Section
    .smart-filter-section {
      padding: 0.75rem;
      background: #{"color-mix(in srgb, var(--primary), 3%, transparent)"};
      border: 1px solid
        #{"color-mix(in srgb, var(--primary), 12%, transparent)"};
      border-radius: 0.375rem;
      margin-bottom: 1rem;

      .smart-filter-toggle {
        display: flex;
        align-items: center;
        gap: 0.625rem;
        cursor: pointer;
        user-select: none;

        input[type="checkbox"] {
          width: 1rem;
          height: 1rem;
          cursor: pointer;
          accent-color: var(--primary);
        }

        .toggle-label {
          font-size: 0.85em;
          font-weight: 600;
          color: var(--text);
          cursor: help;
        }
      }

      .smart-filter-hint {
        margin: 0.5rem 0 0 0;
        padding: 0.375rem 0.5rem;
        background: #{"color-mix(in srgb, var(--dark-800), 50%, transparent)"};
        border-radius: 0.25rem;
        font-size: 0.75em;
        color: var(--text-secondary);
        line-height: 1.4;
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

    .mobile-filters-toggle {
      display: none;
      align-items: center;
      gap: 0.375rem;
      padding: 0.375rem 0.5rem;
      border: 1px solid var(--dark-600);
      border-radius: 0.25rem;
      background: var(--card);
      color: var(--text);
      font-size: 0.8em;
      cursor: pointer;

      &:hover {
        border-color: var(--primary);
      }
    }

    .results-count {
      font-size: 0.75em;
      color: var(--placeholder);
      font-weight: 500;
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

// Mods Content
.mods-content {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0; // Important: allows flex child to shrink
  overflow-y: scroll; // Enable scrolling on the content area

  /* Custom scrollbar styling */
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

// Mods Container
.mods-container {
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

  .content-toolbar {
    .mobile-filters-toggle {
      display: flex;
    }

    .toolbar-left,
    .toolbar-right {
      gap: 0.5rem;
    }

    .toolbar-left .compact-pagination {
      gap: 0.125rem;

      .page-btn.compact {
        min-width: 20px;
        height: 20px;
        padding: 0.125rem 0.25rem;
        font-size: 0.65em;
      }
    }
  }

  .mods-container {
    padding: 0.5rem;

    &.grid {
      grid-template-columns: 1fr;
    }
  }
}
</style>
