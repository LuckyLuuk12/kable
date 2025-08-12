<script lang="ts">
import { onMount, createEventDispatcher } from 'svelte';
import { get } from 'svelte/store';
import { Icon, ModsService, selectedInstallation, InstallationService, installations } from '$lib';
import { modsByProvider, modsLoading, modsError, modsLimit, modsOffset, modsProvider } from '$lib/stores/mods';
import { ProviderKind } from '$lib/runtimeTypes';
import type { ModInfoKind, KableInstallation, ModJarInfo } from '$lib';
import ModCard from './ModCard.svelte';

type ViewMode = 'grid' | 'list' | 'compact';

const dispatch = createEventDispatcher<{
  downloadMod: { modId: string; versionId?: string; installation: KableInstallation };
}>();

// Browser state
let currentProvider: ProviderKind = ProviderKind.Modrinth;
let viewMode: ViewMode = 'grid';
let searchQuery = '';
let currentInstallation: KableInstallation | null = null;
let showFilters = true;

// Filter state
let filters = {
  clientSide: 'all', // all, required, optional, unsupported
  serverSide: 'all', // all, required, optional, unsupported
  projectType: 'all', // all, mod, plugin, datapack, shader
  categories: [] as string[],
  gameVersions: [] as string[]
};

// Pagination state
let currentPage = 1;
let itemsPerPage = 20;

// Installed mods tracking
let installedMods: ModJarInfo[] = [];
let installedModsMap = new Map<string, ModJarInfo>();

// Service instance
let modsService: ModsService;

// Available providers (for now just Modrinth, but ready for expansion)
const providers: { id: ProviderKind; name: string; description: string; available: boolean }[] = [
  { 
    id: ProviderKind.Modrinth, 
    name: 'Modrinth', 
    description: 'Open-source mod platform', 
    available: true 
  },
  { 
    id: 'CurseForge' as ProviderKind, 
    name: 'CurseForge', 
    description: 'Popular mod repository', 
    available: false // For visual testing
  }
];

// View mode options
const viewModes = [
  { id: 'grid', name: 'Grid', icon: 'grid' },
  { id: 'list', name: 'List', icon: 'list' },
  { id: 'compact', name: 'Compact', icon: 'layout' }
];

// Page size options
const pageSizeOptions = [10, 20, 50, 100];

// Filter options
const clientServerOptions = [
  { value: 'all', label: 'All' },
  { value: 'required', label: 'Required' },
  { value: 'optional', label: 'Optional' },
  { value: 'unsupported', label: 'Unsupported' }
];

const projectTypeOptions = [
  { value: 'all', label: 'All Types' },
  { value: 'mod', label: 'Mods' },
  { value: 'plugin', label: 'Plugins' },
  { value: 'datapack', label: 'Data Packs' },
  { value: 'shader', label: 'Shaders' }
];

const commonCategories = [
  'technology', 'adventure', 'magic', 'utility', 'decoration', 
  'food', 'mobs', 'equipment', 'transportation', 'worldgen'
];

// Reactive statements
$: currentInstallation = $selectedInstallation;
$: mods = $modsByProvider[currentProvider] || [];
$: loading = $modsLoading;
$: error = $modsError;
$: filteredMods = applyFilters(mods);
$: totalMods = filteredMods.length;
$: totalPages = Math.ceil(totalMods / itemsPerPage);
$: startIndex = (currentPage - 1) * itemsPerPage;
$: endIndex = Math.min(startIndex + itemsPerPage, totalMods);
$: paginatedMods = filteredMods.slice(startIndex, endIndex);

// Load installed mods when installation changes
$: if (currentInstallation) {
  loadInstalledMods(currentInstallation);
}

// Apply client-side filters
function applyFilters(modsList: ModInfoKind[]) {
  if (!modsList || modsList.length === 0) return [];
  
  return modsList.filter(mod => {
    const info = getModDisplayInfo(mod);
    
    // Skip if mod info is unavailable
    if (!info) return false;
    
    // Search filter
    if (searchQuery) {
      const searchLower = searchQuery.toLowerCase();
      if (!info.title.toLowerCase().includes(searchLower) && 
          !info.description.toLowerCase().includes(searchLower) &&
          !info.author.toLowerCase().includes(searchLower)) {
        return false;
      }
    }
    
    // Client side filter
    if (filters.clientSide !== 'all' && info.client_side !== filters.clientSide) {
      return false;
    }
    
    // Server side filter
    if (filters.serverSide !== 'all' && info.server_side !== filters.serverSide) {
      return false;
    }
    
    // Project type filter
    if (filters.projectType !== 'all' && info.project_type !== filters.projectType) {
      return false;
    }
    
    // Categories filter
    if (filters.categories.length > 0) {
      const hasCategory = filters.categories.some(cat => 
        info.categories.includes(cat)
      );
      if (!hasCategory) return false;
    }
    
    return true;
  });
}

function resetFilters() {
  filters = {
    clientSide: 'all',
    serverSide: 'all', 
    projectType: 'all',
    categories: [],
    gameVersions: []
  };
  currentPage = 1;
}

// Load installed mods for the current installation
async function loadInstalledMods(installation: KableInstallation) {
  try {
    installedMods = await InstallationService.getModInfo(installation);
    // Create a map for quick lookups using mod name and file name for matching
    installedModsMap = new Map();
    installedMods.forEach(mod => {
      // Add multiple keys for different ways to match
      if (mod.mod_name) {
        installedModsMap.set(mod.mod_name.toLowerCase(), mod);
      }
      if (mod.file_name) {
        installedModsMap.set(mod.file_name.toLowerCase(), mod);
        // Also add without file extension
        const nameWithoutExt = mod.file_name.replace(/\.(jar|zip)$/i, '');
        installedModsMap.set(nameWithoutExt.toLowerCase(), mod);
      }
    });
  } catch (e) {
    console.error('Failed to load installed mods:', e);
    installedMods = [];
    installedModsMap = new Map();
  }
}

// Check if a mod is already installed
function isModInstalled(mod: ModInfoKind): boolean {
  const displayInfo = getModDisplayInfo(mod);
  
  // Skip if mod info is unavailable
  if (!displayInfo) return false;
  
  const modTitle = displayInfo.title.toLowerCase();
  
  // Check various ways the mod might be identified
  if (installedModsMap.has(modTitle)) return true;
  
  // For Modrinth mods, also check by project ID or slug
  if ('Modrinth' in mod) {
    const modrinthData = mod.Modrinth;
    if (modrinthData.project_id && installedModsMap.has(modrinthData.project_id.toLowerCase())) return true;
    if (modrinthData.slug && installedModsMap.has(modrinthData.slug.toLowerCase())) return true;
  } else if ('kind' in mod && mod.kind === 'Modrinth') {
    const modrinthData = mod.data;
    if (modrinthData.project_id && installedModsMap.has(modrinthData.project_id.toLowerCase())) return true;
    if (modrinthData.slug && installedModsMap.has(modrinthData.slug.toLowerCase())) return true;
  }
  
  return false;
}

// Initialize service when provider changes
$: if (currentProvider && currentProvider !== $modsProvider) {
  initializeProvider();
}

async function initializeProvider() {
  try {
    modsService = new ModsService(currentProvider);
    await modsService.initialize();
    
    // Apply installation-based filters if available
    if (currentInstallation) {
      await applyInstallationFilters();
    }
  } catch (e) {
    console.error('Failed to initialize provider:', e);
  }
}

async function applyInstallationFilters() {
  if (!currentInstallation || !modsService) return;
  
  // Apply filters based on installation (loader, MC version, etc.)
  try {
    await modsService.setFilter(null, currentInstallation);
  } catch (e) {
    console.error('Failed to apply installation filters:', e);
  }
}

async function changeProvider(provider: ProviderKind) {
  if (!providers.find(p => p.id === provider)?.available) return;
  currentProvider = provider;
  currentPage = 1;
}

async function changePageSize(newSize: number) {
  itemsPerPage = newSize;
  currentPage = 1;
  
  if (modsService) {
    await modsService.setLimit(newSize);
  }
}

async function goToPage(page: number) {
  if (page < 1 || page > totalPages) return;
  currentPage = page;
  
  if (modsService) {
    const offset = (page - 1) * itemsPerPage;
    modsOffset.set(offset);
    await modsService.loadMods();
  }
}

async function nextPage() {
  if (currentPage < totalPages) {
    await goToPage(currentPage + 1);
  }
}

async function prevPage() {
  if (currentPage > 1) {
    await goToPage(currentPage - 1);
  }
}

function toggleCategory(category: string) {
  if (filters.categories.includes(category)) {
    filters.categories = filters.categories.filter(c => c !== category);
  } else {
    filters.categories = [...filters.categories, category];
  }
  currentPage = 1;
}

function handleModDownload(mod: ModInfoKind) {
  if (!currentInstallation) {
    alert('Please select an installation first');
    return;
  }
  
  let modId: string;
  let versionId: string | undefined;
  
  // Handle both Rust enum and discriminated union formats
  if ('Modrinth' in mod) {
    modId = mod.Modrinth.project_id;
    versionId = mod.Modrinth.latest_version || undefined;
  } else if ('kind' in mod && mod.kind === 'Modrinth') {
    modId = mod.data.project_id;
    versionId = mod.data.latest_version || undefined;
  } else {
    // Handle other providers when implemented
    return;
  }
  
  dispatch('downloadMod', {
    modId,
    versionId,
    installation: currentInstallation
  });
}

function handleModInfo(mod: ModInfoKind) {
  // Handle both Rust enum and discriminated union formats
  if ('Modrinth' in mod) {
    const url = mod.Modrinth.source_url || mod.Modrinth.wiki_url || `https://modrinth.com/mod/${mod.Modrinth.slug}`;
    if (url) {
      window.open(url, '_blank');
    }
  } else if ('kind' in mod && mod.kind === 'Modrinth') {
    const url = mod.data.source_url || mod.data.wiki_url || `https://modrinth.com/mod/${mod.data.slug}`;
    if (url) {
      window.open(url, '_blank');
    }
  }
}

// Event handlers for ModCard component
function handleDownloadMod(event: CustomEvent<{ mod: ModInfoKind }>) {
  handleModDownload(event.detail.mod);
}

function handleInfoMod(event: CustomEvent<{ mod: ModInfoKind }>) {
  handleModInfo(event.detail.mod);
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
  source_url?: string;
  wiki_url?: string;
  license?: string;
  date_created?: string;
  date_modified?: string;
  latest_version?: string;
} {
  // Type guard for Rust enum format
  if ('Modrinth' in mod) {
    const modrinthData = mod.Modrinth;
    return {
      title: modrinthData.title || 'Unknown Mod',
      description: modrinthData.description || 'No description available.',
      author: modrinthData.author || 'Unknown Author',
      downloads: modrinthData.downloads || 0,
      icon_url: modrinthData.icon_url,
      categories: modrinthData.categories || [] as string[],
      project_type: modrinthData.project_type || 'mod',
      // Additional Modrinth-specific fields
      follows: modrinthData.follows,
      client_side: modrinthData.client_side,
      server_side: modrinthData.server_side,
      game_versions: modrinthData.game_versions || [] as string[],
      source_url: modrinthData.source_url,
      wiki_url: modrinthData.wiki_url,
      license: modrinthData.license,
      date_created: modrinthData.date_created,
      date_modified: modrinthData.date_modified,
      latest_version: modrinthData.latest_version
    };
  }
  
  // Handle the TypeScript discriminated union format
  if ('kind' in mod && mod.kind === 'Modrinth') {
    console.log('[ModBrowser] Using TypeScript discriminated union data structure:', mod.data);
    return {
      title: mod.data.title || 'Unknown Mod',
      description: mod.data.description || 'No description available.',
      author: mod.data.author || 'Unknown Author',
      downloads: mod.data.downloads || 0,
      icon_url: mod.data.icon_url,
      categories: mod.data.categories || [] as string[],
      project_type: mod.data.project_type || 'mod',
      // Additional Modrinth-specific fields
      follows: mod.data.follows,
      client_side: mod.data.client_side,
      server_side: mod.data.server_side,
      game_versions: mod.data.game_versions || [] as string[],
      source_url: mod.data.source_url,
      wiki_url: mod.data.wiki_url,
      license: mod.data.license,
      date_created: mod.data.date_created,
      date_modified: mod.data.date_modified,
      latest_version: mod.data.latest_version
    };
  }
  
  console.log('[ModBrowser] Using fallback data structure for unknown mod:', mod);
  
  // Fallback for other providers or unknown data
  return {
    title: 'Unknown Mod',
    description: 'No description available.',
    author: 'Unknown Author',
    downloads: 0,
    icon_url: null,
    categories: [] as string[],
    project_type: 'mod',
    follows: undefined,
    client_side: undefined,
    server_side: undefined,
    game_versions: [] as string[],
    source_url: undefined,
    wiki_url: undefined,
    license: undefined,
    date_created: undefined,
    date_modified: undefined,
    latest_version: undefined
  };
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
      console.log('[ModBrowser] Auto-selected installation:', availableInstallations[0].name);
    } else {
      // Try to load installations if they haven't been loaded yet
      try {
        const loadedInstallations = await InstallationService.loadInstallations();
        if (loadedInstallations.length > 0) {
          selectedInstallation.set(loadedInstallations[0]);
          console.log('[ModBrowser] Loaded and auto-selected installation:', loadedInstallations[0].name);
        }
      } catch (error) {
        console.error('[ModBrowser] Failed to load installations:', error);
      }
    }
  }
  
  initializeProvider();
});
</script>


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
    
    <!-- Provider tabs -->
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
          <Icon name={provider.id === ProviderKind.Modrinth ? 'download' : 'package'} size="sm" />
          {provider.name}
          {#if !provider.available}
            <span class="coming-soon">(Soon)</span>
          {/if}
        </button>
      {/each}
    </div>
  </div>

  <!-- Main Content Area -->
  <div class="browser-main">
    <!-- Filters Sidebar -->
    <div class="filters-sidebar" class:collapsed={!showFilters}>
      <div class="filters-header">
        <h3>Filters</h3>
        <div class="filters-actions">
          <button class="reset-filters" on:click={resetFilters} title="Reset all filters">
            <Icon name="refresh" size="sm" />
          </button>
          <button class="toggle-filters" on:click={() => showFilters = !showFilters} title="Toggle filters">
            <Icon name={showFilters ? 'arrow-left' : 'arrow-right'} size="sm" forceType="svg" />
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
                on:input={() => currentPage = 1}
                class="search-input"
              />
              {#if searchQuery}
                <button class="clear-btn" on:click={() => { searchQuery = ''; currentPage = 1; }}>
                  <Icon name="x" size="sm" />
                </button>
              {/if}
            </div>
          </div>

          <!-- Client/Server Side -->
          <div class="filter-section">
            <label class="filter-label" for="client-side">Client Side</label>
            <select id="client-side" bind:value={filters.clientSide} on:change={() => currentPage = 1} class="filter-select">
              {#each clientServerOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </div>

          <div class="filter-section">
            <label class="filter-label" for="server-side">Server Side</label>
            <select id="server-side" bind:value={filters.serverSide} on:change={() => currentPage = 1} class="filter-select">
              {#each clientServerOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </div>

          <!-- Project Type -->
          <div class="filter-section">
            <label class="filter-label" for="project-type">Type</label>
            <select id="project-type" bind:value={filters.projectType} on:change={() => currentPage = 1} class="filter-select">
              {#each projectTypeOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </div>

          <!-- Categories -->
          <div class="filter-section">
            <label class="filter-label" for="categories">Categories</label>
            <div class="checkbox-group">
              {#each commonCategories as category}
                <label class="checkbox-item">
                  <input 
                    type="checkbox" 
                    checked={filters.categories.includes(category)}
                    on:change={() => toggleCategory(category)}
                  />
                  <span class="checkbox-label">{category}</span>
                </label>
              {/each}
            </div>
          </div>
        </div>
      {/if}
    </div>

    <!-- Content Area -->
    <div class="content-area">
      <!-- Toolbar -->
      <div class="content-toolbar">
        <div class="toolbar-left">
          <button class="mobile-filters-toggle" on:click={() => showFilters = !showFilters}>
            <Icon name="filter" size="sm" />
            Filters
          </button>
          <span class="results-count">{totalMods} mods</span>
        </div>

        <div class="toolbar-right">
          <!-- View Mode -->
          <div class="view-controls">
            {#each viewModes as mode}
              <button
                class="view-mode-btn"
                class:active={viewMode === mode.id}
                on:click={() => viewMode = mode.id as ViewMode}
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
            
            {#if filteredMods.length !== mods.length}
              <button class="clear-filters-btn" on:click={resetFilters}>
                <Icon name="refresh" size="sm" />
                Clear Filters ({mods.length} total mods)
              </button>
            {/if}
          </div>
        {:else}
          <!-- Mods Grid/List -->
          <div class="mods-container" class:grid={viewMode === 'grid'} class:list={viewMode === 'list'} class:compact={viewMode === 'compact'}>
            {#each paginatedMods as mod}
              {@const installed = currentInstallation ? isModInstalled(mod) : false}
              <ModCard 
                {mod} 
                {viewMode} 
                {currentInstallation}
                loading={false}
                isInstalled={installed}
                on:downloadMod={handleDownloadMod}
                on:infoMod={handleInfoMod}
              />
            {/each}
          </div>

          <!-- Pagination -->
          {#if totalPages > 1}
            <div class="pagination">
              <div class="pagination-info">
                Showing {startIndex + 1}-{endIndex} of {totalMods} mods
              </div>
              
              <div class="pagination-controls">
                <button 
                  class="page-btn" 
                  on:click={prevPage} 
                  disabled={currentPage === 1}
                  title="Previous page"
                >
                  <Icon name="arrow-left" size="sm" />
                </button>
                
                {#if totalPages <= 7}
                  {#each Array(totalPages) as _, i}
                    <button
                      class="page-btn"
                      class:active={currentPage === i + 1}
                      on:click={() => goToPage(i + 1)}
                    >
                      {i + 1}
                    </button>
                  {/each}
                {:else}
                  <button
                    class="page-btn"
                    class:active={currentPage === 1}
                    on:click={() => goToPage(1)}
                  >
                    1
                  </button>
                  
                  {#if currentPage > 3}
                    <span class="pagination-ellipsis">...</span>
                  {/if}
                  
                  {#each Array(Math.min(3, totalPages - 2)) as _, i}
                    {@const pageNum = Math.max(2, Math.min(totalPages - 1, currentPage - 1 + i))}
                    {#if pageNum > 1 && pageNum < totalPages}
                      <button
                        class="page-btn"
                        class:active={currentPage === pageNum}
                        on:click={() => goToPage(pageNum)}
                      >
                        {pageNum}
                      </button>
                    {/if}
                  {/each}
                  
                  {#if currentPage < totalPages - 2}
                    <span class="pagination-ellipsis">...</span>
                  {/if}
                  
                  <button
                    class="page-btn"
                    class:active={currentPage === totalPages}
                    on:click={() => goToPage(totalPages)}
                  >
                    {totalPages}
                  </button>
                {/if}
                
                <button 
                  class="page-btn" 
                  on:click={nextPage} 
                  disabled={currentPage === totalPages}
                  title="Next page"
                >
                  <Icon name="arrow-right" size="sm" />
                </button>
              </div>
            </div>
          {/if}
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
  background: $container;
  border-radius: 0.5rem;
  border: 1px solid rgba($primary, 0.08);
  box-shadow: 0 2px 8px rgba($dark-900, 0.04);
  overflow: hidden;
}

// Compact Header
.browser-header {
  background: linear-gradient(135deg, 
    rgba($card, 0.95) 0%, 
    rgba($primary, 0.04) 30%,
    rgba($secondary, 0.02) 70%,
    rgba($card, 0.8) 100%
  );
  backdrop-filter: blur(12px);
  border-bottom: 1px solid rgba($primary, 0.15);
  padding: 0.75rem 1rem;
  position: relative;
  
  &::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg, 
      transparent 0%, 
      rgba($primary, 0.3) 20%, 
      rgba($secondary, 0.2) 80%, 
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
      background: linear-gradient(135deg, $primary 0%, $secondary 100%);
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
      background: rgba($primary, 0.08);
      border: 1px solid rgba($primary, 0.15);
      border-radius: 0.75rem;
      padding: 0.25rem 0.5rem;
      font-size: 0.75em;
      font-weight: 500;
      color: $primary;
    }
  }
  
  .provider-tabs {
    display: flex;
    gap: 0.375rem;
    
    .provider-tab {
      display: flex;
      align-items: center;
      gap: 0.375rem;
      padding: 0.375rem 0.75rem;
      border: 1px solid rgba($primary, 0.15);
      border-radius: 0.375rem;
      background: $card;
      color: $text;
      font-weight: 500;
      font-size: 0.8em;
      cursor: pointer;
      transition: all 0.15s;
      
      &:hover:not(:disabled) {
        border-color: $primary;
        background: rgba($primary, 0.05);
      }
      
      &.active {
        background: linear-gradient(135deg, $primary 0%, $secondary 100%);
        color: white;
        border-color: transparent;
        box-shadow: 0 2px 6px rgba($primary, 0.25);
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
}

// Main Layout
.browser-main {
  display: flex;
  flex: 1;
  overflow: hidden;
}

// Filters Sidebar
.filters-sidebar {
  width: 240px;
  background: linear-gradient(135deg, rgba($container, 0.95) 0%, rgba($card, 0.8) 100%);
  backdrop-filter: blur(8px);
  border-right: 1px solid rgba($primary, 0.12);
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
    border-bottom: 1px solid rgba($primary, 0.12);
    background: linear-gradient(135deg, rgba($primary, 0.06) 0%, rgba($secondary, 0.03) 100%);
    backdrop-filter: blur(4px);
    
    h3 {
      margin: 0;
      font-size: 0.9em;
      font-weight: 600;
      color: $text;
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
        color: $placeholder;
        cursor: pointer;
        transition: all 0.15s;
        
        &:hover {
          background: rgba($primary, 0.1);
          color: $primary;
        }
      }
    }
  }
  
  .filters-content {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
    
    .filter-section {
      margin-bottom: 1rem;
      
      .filter-label {
        display: block;
        font-size: 0.75em;
        font-weight: 600;
        color: $text;
        margin-bottom: 0.375rem;
        text-transform: uppercase;
        letter-spacing: 0.5px;
      }
      
      .search-input-wrapper {
        position: relative;
        display: flex;
        align-items: center;
        
        :global(.icon) {
          position: absolute;
          left: 0.5rem;
          color: $placeholder;
          z-index: 1;
        }
        
        .search-input {
          width: 100%;
          padding: 0.5rem 0.5rem 0.5rem 2rem;
          border: 1px solid $dark-600;
          border-radius: 0.375rem;
          background: $input;
          color: $text;
          font-size: 0.8em;
          
          &:focus {
            outline: none;
            border-color: $primary;
            box-shadow: 0 0 0 2px rgba($primary, 0.1);
          }
          
          &::placeholder {
            color: $placeholder;
          }
        }
        
        .clear-btn {
          position: absolute;
          right: 0.375rem;
          background: none;
          border: none;
          color: $placeholder;
          cursor: pointer;
          padding: 0.125rem;
          border-radius: 0.125rem;
          
          &:hover {
            color: $red;
            background: rgba($red, 0.1);
          }
        }
      }
      
      .filter-select {
        width: 100%;
        padding: 0.375rem 0.5rem;
        border: 1px solid $dark-600;
        border-radius: 0.375rem;
        background: $card;
        color: $text;
        font-size: 0.8em;
        cursor: pointer;
        
        &:focus {
          outline: none;
          border-color: $primary;
        }
      }
      
      .checkbox-group {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
        max-height: 120px;
        overflow-y: auto;
        
        .checkbox-item {
          display: flex;
          align-items: center;
          gap: 0.375rem;
          padding: 0.25rem;
          border-radius: 0.25rem;
          cursor: pointer;
          transition: background 0.15s;
          
          &:hover {
            background: rgba($primary, 0.05);
          }
          
          input[type="checkbox"] {
            margin: 0;
            accent-color: $primary;
          }
          
          .checkbox-label {
            font-size: 0.75em;
            color: $text;
            text-transform: capitalize;
          }
        }
      }
    }
  }
}

// Content Area
.content-area {
  flex: 1;
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
  background: linear-gradient(135deg, $container 0%, rgba($card, 0.6) 100%);
  backdrop-filter: blur(6px);
  border-bottom: 1px solid rgba($primary, 0.12);
  
  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    
    .mobile-filters-toggle {
      display: none;
      align-items: center;
      gap: 0.375rem;
      padding: 0.375rem 0.5rem;
      border: 1px solid $dark-600;
      border-radius: 0.25rem;
      background: $card;
      color: $text;
      font-size: 0.8em;
      cursor: pointer;
      
      &:hover {
        border-color: $primary;
      }
    }
    
    .results-count {
      font-size: 0.75em;
      color: $placeholder;
      font-weight: 500;
    }
  }
  
  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    
    .view-controls {
      display: flex;
      border: 1px solid $dark-600;
      border-radius: 0.25rem;
      overflow: hidden;
      
      .view-mode-btn {
        padding: 0.25rem 0.375rem;
        border: none;
        background: $card;
        color: $placeholder;
        cursor: pointer;
        transition: all 0.15s;
        
        &:hover {
          background: rgba($primary, 0.05);
          color: $text;
        }
        
        &.active {
          background: $primary;
          color: white;
        }
        
        &:not(:last-child) {
          border-right: 1px solid $dark-600;
        }
      }
    }
    
    .page-size-select {
      padding: 0.25rem 0.375rem;
      border: 1px solid $dark-600;
      border-radius: 0.25rem;
      background: $card;
      color: $text;
      font-size: 0.75em;
      cursor: pointer;
      
      &:focus {
        outline: none;
        border-color: $primary;
      }
    }
  }
}

// Mods Content
.mods-content {
  display: flex;
  flex-direction: column;
  height: fit-content;
}

// Loading/Error/Empty States
.loading-state, .error-state, .empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  gap: 0.75rem;
  color: $placeholder;
}

.error-state, .empty-state {
  h3 {
    margin: 0;
    color: $text;
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

.error-state .retry-btn, .empty-state .clear-filters-btn {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 1rem;
  border: 1px solid $primary;
  border-radius: 0.375rem;
  background: rgba($primary, 0.1);
  color: $primary;
  font-weight: 500;
  font-size: 0.8em;
  cursor: pointer;
  transition: all 0.15s;
  
  &:hover {
    background: $primary;
    color: white;
  }
}

// Mods Container
.mods-container {
  flex: 1;
  padding: 0.75rem;
  overflow-y: auto;
  min-height: 0;
  
  /* Custom scrollbar styling */
  &::-webkit-scrollbar {
    width: 8px;
  }
  
  &::-webkit-scrollbar-track {
    background: rgba($dark-600, 0.1);
    border-radius: 4px;
  }
  
  &::-webkit-scrollbar-thumb {
    background: linear-gradient(135deg, rgba($primary, 0.6) 0%, rgba($secondary, 0.4) 100%);
    border-radius: 4px;
    
    &:hover {
      background: linear-gradient(135deg, rgba($primary, 0.8) 0%, rgba($secondary, 0.6) 100%);
    }
  }
  
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

// Pagination
.pagination {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem;
  background: linear-gradient(135deg, rgba($container, 0.8) 0%, rgba($card, 0.4) 100%);
  backdrop-filter: blur(4px);
  border-top: 1px solid rgba($primary, 0.12);
  
  .pagination-info {
    font-size: 0.75em;
    color: $placeholder;
    font-weight: 500;
  }
  
  .pagination-controls {
    display: flex;
    align-items: center;
    gap: 0.1875rem;
    
    .page-btn {
      padding: 0.25rem 0.375rem;
      border: 1px solid $dark-600;
      border-radius: 0.25rem;
      background: $card;
      color: $text;
      font-size: 0.75em;
      cursor: pointer;
      transition: all 0.15s;
      min-width: 28px;
      
      &:hover:not(:disabled) {
        border-color: $primary;
        background: rgba($primary, 0.05);
      }
      
      &.active {
        background: $primary;
        color: white;
        border-color: transparent;
      }
      
      &:disabled {
        opacity: 0.5;
        cursor: not-allowed;
      }
    }
    
    .pagination-ellipsis {
      padding: 0.25rem;
      color: $placeholder;
      font-size: 0.75em;
    }
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
    border-bottom: 1px solid rgba($primary, 0.15);
    background: linear-gradient(135deg, rgba($container, 0.9) 0%, rgba($card, 0.7) 100%);
    
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
  }
  
  .mods-container {
    padding: 0.5rem;
    
    &.grid {
      grid-template-columns: 1fr;
    }
  }
  
  .pagination {
    flex-direction: column;
    gap: 0.5rem;
    
    .pagination-controls {
      flex-wrap: wrap;
      justify-content: center;
    }
  }
}
</style>