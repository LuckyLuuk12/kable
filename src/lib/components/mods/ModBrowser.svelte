<script lang="ts">
import { onMount, createEventDispatcher } from 'svelte';
import { get } from 'svelte/store';
import { Icon, ModsService, selectedInstallation, InstallationService, installations } from '$lib';
import Image from '$lib/components/Image.svelte';
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
let visitedPages = new Set([1]); // Track pages that had results
let maxPageReached = 1; // Highest page number user has visited

// Installed mods tracking
let installedMods: ModJarInfo[] = [];
let installedModsMap = new Map<string, ModJarInfo>();
let installedModsLoaded = false;

// Service instance
let modsService: ModsService;

// Available providers
const providers: { id: ProviderKind; name: string; description: string; available: boolean }[] = [
  { 
    id: ProviderKind.Modrinth, 
    name: 'Modrinth', 
    description: 'Flexible API, though try not to spam refresh this tab', 
    available: true 
  },
  { 
    id: ProviderKind.CurseForge, 
    name: 'CurseForge', 
    description: 'Might not work because of API limitations', 
    available: true 
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
// For backend pagination, we show all loaded mods (no client-side slicing)
$: paginatedMods = filteredMods;
$: totalMods = filteredMods.length;

// Load installed mods when installation changes
$: if (currentInstallation) {
  installedModsLoaded = false;
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
  // Don't reset visitedPages or maxPageReached - keep pagination history
  modsOffset.set(0);
  if (modsService) {
    modsService.loadMods();
  }
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
    
    installedModsLoaded = true;
  } catch (e) {
    console.error('[ModBrowser] Failed to load installed mods:', e);
    installedMods = [];
    installedModsMap = new Map();
    installedModsLoaded = true; // Still set to true to prevent infinite loading
  }
}

// Fuzzy matching utilities
function levenshteinDistance(a: string, b: string): number {
  const matrix = Array(b.length + 1).fill(null).map(() => Array(a.length + 1).fill(null));

  for (let i = 0; i <= a.length; i++) matrix[0][i] = i;
  for (let j = 0; j <= b.length; j++) matrix[j][0] = j;

  for (let j = 1; j <= b.length; j++) {
    for (let i = 1; i <= a.length; i++) {
      const indicator = a[i - 1] === b[j - 1] ? 0 : 1;
      matrix[j][i] = Math.min(
        matrix[j][i - 1] + 1,     // deletion
        matrix[j - 1][i] + 1,     // insertion
        matrix[j - 1][i - 1] + indicator  // substitution
      );
    }
  }

  return matrix[b.length][a.length];
}

function similarity(a: string, b: string): number {
  const maxLength = Math.max(a.length, b.length);
  if (maxLength === 0) return 1;
  const distance = levenshteinDistance(a, b);
  return (maxLength - distance) / maxLength;
}

function normalizeForComparison(text: string): string {
  return text
    .toLowerCase()
    .trim()
    // Remove common file extensions
    .replace(/\.(jar|zip)$/i, '')
    // Normalize separators to spaces
    .replace(/[\-_]+/g, ' ')
    // Remove extra spaces
    .replace(/\s+/g, ' ')
    .trim();
}

function findBestMatch(target: string, candidates: string[], threshold: number = 0.7): { match: string; score: number } | null {
  const normalizedTarget = normalizeForComparison(target);
  
  let bestMatch = null;
  let bestScore = 0;

  for (const candidate of candidates) {
    const normalizedCandidate = normalizeForComparison(candidate);
    
    // Skip very short strings
    if (normalizedTarget.length < 3 || normalizedCandidate.length < 3) continue;
    
    const score = similarity(normalizedTarget, normalizedCandidate);
    
    if (score > bestScore && score >= threshold) {
      bestScore = score;
      bestMatch = candidate;
    }
  }

  return bestMatch ? { match: bestMatch, score: bestScore } : null;
}

// Check if a mod is already installed
function isModInstalled(mod: ModInfoKind): boolean {
  // Don't check if installed mods haven't been loaded yet
  if (!installedModsLoaded) {
    return false;
  }

  const displayInfo = getModDisplayInfo(mod);
  
  // Skip if mod info is unavailable
  if (!displayInfo) {
    return false;
  }
  
  // First try exact matches for performance
  const modTitle = displayInfo.title.toLowerCase();
  if (installedModsMap.has(modTitle)) {
    return true;
  }
  
  // For Modrinth mods, also check by project ID or slug
  let modrinthData: any = null;
  if ('Modrinth' in mod) {
    modrinthData = mod.Modrinth;
  } else if ('kind' in mod && mod.kind === 'Modrinth') {
    modrinthData = mod.data;
  }
  
  if (modrinthData) {
    if (modrinthData.project_id && installedModsMap.has(modrinthData.project_id.toLowerCase())) {
      return true;
    }
    if (modrinthData.slug && installedModsMap.has(modrinthData.slug.toLowerCase())) {
      return true;
    }
  }

  // For CurseForge mods, also check by mod ID or slug
  let curseforgeData: any = null;
  if ('CurseForge' in mod) {
    curseforgeData = mod.CurseForge;
  } else if ('kind' in mod && mod.kind === 'CurseForge') {
    curseforgeData = mod.data;
  }
  
  if (curseforgeData) {
    if (curseforgeData.id && installedModsMap.has(curseforgeData.id.toString().toLowerCase())) {
      return true;
    }
    if (curseforgeData.slug && installedModsMap.has(curseforgeData.slug.toLowerCase())) {
      return true;
    }
  }
  
  // If no exact match, try fuzzy matching
  const candidateNames: string[] = [];
  
  installedMods.forEach(installedMod => {
    if (installedMod.mod_name) candidateNames.push(installedMod.mod_name.toLowerCase());
    if (installedMod.file_name) {
      candidateNames.push(installedMod.file_name.toLowerCase());
      // Also add filename without extension
      const nameWithoutExt = installedMod.file_name.replace(/\.(jar|zip)$/i, '').toLowerCase();
      candidateNames.push(nameWithoutExt);
    }
  });
  
  // Try fuzzy matching against mod title with lower threshold first
  const titleMatch = findBestMatch(displayInfo.title.toLowerCase(), candidateNames, 0.7);
  if (titleMatch) {
    return true;
  }
  
  // Also try fuzzy matching against slug if available
  if (modrinthData?.slug) {
    const slugMatch = findBestMatch(modrinthData.slug.toLowerCase(), candidateNames, 0.7);
    if (slugMatch) {
      return true;
    }
  }

  // Also try fuzzy matching against CurseForge slug if available
  if (curseforgeData?.slug) {
    const slugMatch = findBestMatch(curseforgeData.slug.toLowerCase(), candidateNames, 0.7);
    if (slugMatch) {
      return true;
    }
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
    console.log(`[ModBrowser] Going to page ${page}, setting offset to ${offset}`);
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

// Generate dynamic page numbers based on visited pages
function generatePageNumbers(): (number | 'ellipsis')[] {
  const totalToShow = 10; // Total page numbers to show
  const pages: (number | 'ellipsis')[] = [];
  
  // Always show at least pages 1 through current + a few ahead
  const minEndPage = Math.max(currentPage + 3, 10); // Show at least 10 pages or current + 3
  
  // Calculate the window around current page
  const halfWindow = Math.floor(totalToShow / 2);
  let startPage = Math.max(1, currentPage - halfWindow);
  let endPage = Math.min(minEndPage, startPage + totalToShow - 1);
  
  // Adjust if we're near the beginning
  if (endPage - startPage + 1 < totalToShow && endPage < minEndPage) {
    endPage = Math.min(minEndPage, startPage + totalToShow - 1);
  }
  if (endPage - startPage + 1 < totalToShow) {
    startPage = Math.max(1, endPage - totalToShow + 1);
  }
  
  // If we're showing a window that doesn't start at 1, show first few pages + ellipsis
  if (startPage > 3) {
    pages.push(1);
    pages.push(2);
    pages.push('ellipsis');
  } else if (startPage > 1) {
    // Fill in the gap if it's small
    for (let i = 1; i < startPage; i++) {
      pages.push(i);
    }
  }
  
  // Add the main window of pages
  for (let i = startPage; i <= endPage; i++) {
    // Don't duplicate pages we already added
    if (!pages.includes(i)) {
      pages.push(i);
    }
  }
  
  return pages;
}

function toggleCategory(category: string) {
  if (filters.categories.includes(category)) {
    filters.categories = filters.categories.filter(c => c !== category);
  } else {
    filters.categories = [...filters.categories, category];
  }
  currentPage = 1;
  // Don't reset pagination history for filters
  modsOffset.set(0);
  if (modsService) modsService.loadMods();
}

function handleModDownload(mod: ModInfoKind) {
  if (!currentInstallation) {
    alert('Please select an installation first');
    return;
  }
  
  let modId: string;
  let versionId: string | undefined;
  
  // Handle Modrinth - Rust enum format
  if ('Modrinth' in mod) {
    modId = mod.Modrinth.project_id;
    versionId = mod.Modrinth.latest_version || undefined;
  }
  // Handle Modrinth - TypeScript discriminated union format
  else if ('kind' in mod && mod.kind === 'Modrinth') {
    modId = mod.data.project_id;
    versionId = mod.data.latest_version || undefined;
  }
  // Handle CurseForge - Rust enum format
  else if ('CurseForge' in mod) {
    modId = mod.CurseForge.id.toString();
    versionId = mod.CurseForge.main_file_id.toString() || undefined;
  }
  // Handle CurseForge - TypeScript discriminated union format
  else if ('kind' in mod && mod.kind === 'CurseForge') {
    modId = mod.data.id.toString();
    versionId = mod.data.main_file_id.toString() || undefined;
  }
  // Unknown provider
  else {
    console.error('[ModBrowser] Unknown mod provider format:', mod);
    return;
  }
  
  dispatch('downloadMod', {
    modId,
    versionId,
    installation: currentInstallation
  });
}

function handleModInfo(mod: ModInfoKind) {
  // Handle Modrinth - Rust enum format
  if ('Modrinth' in mod) {
    const url = mod.Modrinth.source_url || mod.Modrinth.wiki_url || `https://modrinth.com/mod/${mod.Modrinth.slug}`;
    if (url) {
      window.open(url, '_blank');
    }
  }
  // Handle Modrinth - TypeScript discriminated union format
  else if ('kind' in mod && mod.kind === 'Modrinth') {
    const url = mod.data.source_url || mod.data.wiki_url || `https://modrinth.com/mod/${mod.data.slug}`;
    if (url) {
      window.open(url, '_blank');
    }
  }
  // Handle CurseForge - Rust enum format
  else if ('CurseForge' in mod) {
    const url = mod.CurseForge.links?.website_url || 
               mod.CurseForge.links?.source_url || 
               mod.CurseForge.links?.wiki_url || 
               `https://www.curseforge.com/minecraft/mc-mods/${mod.CurseForge.slug}`;
    if (url) {
      window.open(url, '_blank');
    }
  }
  // Handle CurseForge - TypeScript discriminated union format
  else if ('kind' in mod && mod.kind === 'CurseForge') {
    const url = mod.data.links?.website_url || 
               mod.data.links?.source_url || 
               mod.data.links?.wiki_url || 
               `https://www.curseforge.com/minecraft/mc-mods/${mod.data.slug}`;
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
  // Handle Modrinth - Rust enum format
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
  
  // Handle CurseForge - Rust enum format
  if ('CurseForge' in mod) {
    const curseforgeData = mod.CurseForge;
    return {
      title: curseforgeData.name || 'Unknown Mod',
      description: curseforgeData.summary || 'No description available.',
      author: curseforgeData.authors?.[0]?.name || 'Unknown Author',
      downloads: curseforgeData.download_count || 0,
      icon_url: curseforgeData.logo?.url || curseforgeData.logo?.thumbnail_url,
      categories: curseforgeData.categories?.map(cat => cat.name) || [] as string[],
      project_type: 'mod', // CurseForge doesn't distinguish project types the same way
      // Additional fields mapped from CurseForge
      follows: curseforgeData.thumbs_up_count, // Use thumbs up as follow count equivalent
      client_side: undefined, // CurseForge doesn't have this concept
      server_side: undefined, // CurseForge doesn't have this concept
      game_versions: curseforgeData.latest_files_indexes?.map(file => file.game_version) || [] as string[],
      source_url: curseforgeData.links?.source_url,
      wiki_url: curseforgeData.links?.wiki_url,
      license: undefined, // CurseForge doesn't expose license in the same way
      date_created: curseforgeData.date_created,
      date_modified: curseforgeData.date_modified,
      latest_version: curseforgeData.latest_files?.[0]?.display_name
    };
  }
  
  // Handle Modrinth - TypeScript discriminated union format
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

  // Handle CurseForge - TypeScript discriminated union format
  if ('kind' in mod && mod.kind === 'CurseForge') {
    console.log('[ModBrowser] Using CurseForge TypeScript discriminated union data structure:', mod.data);
    return {
      title: mod.data.name || 'Unknown Mod',
      description: mod.data.summary || 'No description available.',
      author: mod.data.authors?.[0]?.name || 'Unknown Author',
      downloads: mod.data.download_count || 0,
      icon_url: mod.data.logo?.url || mod.data.logo?.thumbnail_url,
      categories: mod.data.categories?.map(cat => cat.name) || [] as string[],
      project_type: 'mod', // CurseForge doesn't distinguish project types the same way
      // Additional fields mapped from CurseForge
      follows: mod.data.thumbs_up_count, // Use thumbs up as follow count equivalent
      client_side: undefined, // CurseForge doesn't have this concept
      server_side: undefined, // CurseForge doesn't have this concept
      game_versions: mod.data.latest_files_indexes?.map(file => file.game_version) || [] as string[],
      source_url: mod.data.links?.source_url,
      wiki_url: mod.data.links?.wiki_url,
      license: undefined, // CurseForge doesn't expose license in the same way
      date_created: mod.data.date_created,
      date_modified: mod.data.date_modified,
      latest_version: mod.data.latest_files?.[0]?.display_name
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
      // No installations available in store; rely on centralized bootstrap (NavBar) to load them.
      console.log('[ModBrowser] No installations available yet; waiting for centralized initialization');
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
        <!-- TODO: Change this to use the providers favicon -->
          <!-- Use Image component so users can override icons via config/images/<key>.* or fall back to /img/<key>.png -->
          {#if provider.id === ProviderKind.Modrinth}
            <Image key="modrinth" alt="Modrinth" className="provider-icon" width="1.25rem" height="1.25rem" />
          {:else if provider.id === ProviderKind.CurseForge}
            <Image key="curseforge" alt="CurseForge" className="provider-icon" width="1.25rem" height="1.25rem" />
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
  </div>

  <!-- Main Content Area -->
  <div class="browser-main">
    <!-- Filters Sidebar -->
    <div class="filters-sidebar" class:collapsed={!showFilters}>
      <div class="filters-header">
        <h3>Filters</h3>
        <div class="filters-actions">
          <button class="reset-filters" on:click={resetFilters} title="Reset all filters">
            <Icon name="refresh" size="sm" forceType="svg" />
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
                on:input={() => { 
                  currentPage = 1; 
                  // Don't reset pagination history for search
                  modsOffset.set(0);
                  if (modsService) modsService.loadMods();
                }}
                class="search-input"
              />
              {#if searchQuery}
                <button class="clear-btn" on:click={() => { 
                  searchQuery = ''; 
                  currentPage = 1; 
                  // Don't reset pagination history
                  modsOffset.set(0);
                  if (modsService) modsService.loadMods();
                }}>
                  <Icon name="x" size="sm" />
                </button>
              {/if}
            </div>
          </div>

          <!-- Client/Server Side -->
          <div class="filter-section">
            <label class="filter-label" for="client-side">Client Side</label>
            <select id="client-side" bind:value={filters.clientSide} on:change={() => { 
              currentPage = 1; 
              // Don't reset pagination history for filters
              modsOffset.set(0);
              if (modsService) modsService.loadMods();
            }} class="filter-select">
              {#each clientServerOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </div>

          <div class="filter-section">
            <label class="filter-label" for="server-side">Server Side</label>
            <select id="server-side" bind:value={filters.serverSide} on:change={() => { 
              currentPage = 1; 
              // Don't reset pagination history for filters
              modsOffset.set(0);
              if (modsService) modsService.loadMods();
            }} class="filter-select">
              {#each clientServerOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </div>

          <!-- Project Type -->
          <div class="filter-section">
            <label class="filter-label" for="project-type">Type</label>
            <select id="project-type" bind:value={filters.projectType} on:change={() => { 
              currentPage = 1; 
              // Don't reset pagination history for filters
              modsOffset.set(0);
              if (modsService) modsService.loadMods();
            }} class="filter-select">
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
          
          <!-- Compact Pagination Controls -->
          <div class="compact-pagination">
            <button 
              class="page-btn compact" 
              on:click={prevPage} 
              disabled={currentPage === 1}
              title="Previous page"
            >
              <Icon name="arrow-left" size="sm" forceType="svg" />
            </button>
            
            {#each generatePageNumbers() as pageItem}
              {#if pageItem === 'ellipsis'}
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
              {@const installed = currentInstallation && installedModsLoaded ? isModInstalled(mod) : false}
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
  border: 1px solid #{'color-mix(in srgb, var(--primary), 8%, transparent)'};
  box-shadow: 0 2px 8px #{'color-mix(in srgb, var(--dark-900), 4%, transparent)'};
  overflow: hidden;
}

// Compact Header
.browser-header {
  background: linear-gradient(135deg, 
  #{'color-mix(in srgb, var(--container), 95%, transparent)'} 0%, 
  #{'color-mix(in srgb, var(--primary), 4%, transparent)'} 30%,
  #{'color-mix(in srgb, var(--secondary), 2%, transparent)'} 70%,
  #{'color-mix(in srgb, var(--card), 80%, transparent)'} 100%
  );
  backdrop-filter: blur(12px);
  border-bottom: 1px solid #{'color-mix(in srgb, var(--primary), 15%, transparent)'};
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
  #{'color-mix(in srgb, var(--primary), 30%, transparent)'} 20%, 
  #{'color-mix(in srgb, var(--secondary), 20%, transparent)'} 80%, 
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
      background: linear-gradient(135deg, var(--primary) 0%, var(--secondary) 100%);
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
  background: #{'color-mix(in srgb, var(--primary), 8%, transparent)'};
  border: 1px solid #{'color-mix(in srgb, var(--primary), 15%, transparent)'};
      border-radius: 0.75rem;
      padding: 0.25rem 0.5rem;
      font-size: 0.75em;
      font-weight: 500;
      color: var(--primary);
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
  border: 1px solid #{'color-mix(in srgb, var(--primary), 15%, transparent)'};
      border-radius: 0.375rem;
      background: var(--card);
      color: var(--text);
      font-weight: 500;
      font-size: 0.8em;
      cursor: pointer;
      transition: all 0.15s;
      
      &:hover:not(:disabled) {
        border-color: var(--primary);
  background: #{'color-mix(in srgb, var(--primary), 5%, transparent)'};
      }
      
      &.active {
        background: linear-gradient(135deg, var(--primary) 0%, var(--secondary) 100%);
        color: white;
        border-color: transparent;
  box-shadow: 0 2px 6px #{'color-mix(in srgb, var(--primary), 25%, transparent)'};
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
  overflow: hidden;
}

// Filters Sidebar
.filters-sidebar {
  width: 240px;
  background: linear-gradient(135deg, #{'color-mix(in srgb, var(--container), 95%, transparent)'} 0%, #{'color-mix(in srgb, var(--card), 80%, transparent)'} 100%);
  backdrop-filter: blur(8px);
  border-right: 1px solid #{'color-mix(in srgb, var(--primary), 12%, transparent)'};
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
  border-bottom: 1px solid #{'color-mix(in srgb, var(--primary), 12%, transparent)'};
  background: linear-gradient(135deg, #{'color-mix(in srgb, var(--primary), 6%, transparent)'} 0%, #{'color-mix(in srgb, var(--secondary), 3%, transparent)'} 100%);
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
          background: #{'color-mix(in srgb, var(--primary), 10%, transparent)'};
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
      margin-bottom: 1rem;
      
      .filter-label {
        display: block;
        font-size: 0.75em;
        font-weight: 600;
        color: var(--text);
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
          color: var(--placeholder);
          z-index: 1;
        }
        
        .search-input {
          width: 100%;
          padding: 0.5rem 0.5rem 0.5rem 2rem;
          border: 1px solid var(--dark-600);
          border-radius: 0.375rem;
          background: var(--input);
          color: var(--text);
          font-size: 0.8em;
          
          &:focus {
            outline: none;
            border-color: var(--primary);
            box-shadow: 0 0 0 2px #{'color-mix(in srgb, var(--primary), 10%, transparent)'};
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
            background: #{'color-mix(in srgb, var(--red), 10%, transparent)'};
          }
        }
      }
      
      .filter-select {
        width: 100%;
        padding: 0.375rem 0.5rem;
        border: 1px solid var(--dark-600);
        border-radius: 0.375rem;
        background: var(--card);
        color: var(--text);
        font-size: 0.8em;
        cursor: pointer;
        
        &:focus {
          outline: none;
          border-color: var(--primary);
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
            background: #{'color-mix(in srgb, var(--primary), 5%, transparent)'};
          }
          
          input[type="checkbox"] {
            margin: 0;
            accent-color: var(--primary);
          }
          
          .checkbox-label {
            font-size: 0.75em;
            color: var(--text);
            text-transform: capitalize;
          }
        }
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
  background: linear-gradient(135deg, var(--container) 0%, #{'color-mix(in srgb, var(--card), 60%, transparent)'} 100%);
  backdrop-filter: blur(6px);
  border-bottom: 1px solid #{'color-mix(in srgb, var(--primary), 12%, transparent)'};
  
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
  border: 1px solid #{'color-mix(in srgb, var(--primary), 20%, transparent)'};
        border-radius: 0.25rem;
  background: #{'color-mix(in srgb, var(--card), 80%, transparent)'};
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
          background: #{'color-mix(in srgb, var(--primary), 10%, transparent)'};
          color: var(--primary);
        }
        
        &.active {
          background: var(--primary);
          color: white;
          border-color: transparent;
          box-shadow: 0 1px 3px #{'color-mix(in srgb, var(--primary), 30%, transparent)'};
        }
        
        &:disabled {
          opacity: 0.4;
          cursor: not-allowed;
          background: #{'color-mix(in srgb, var(--card), 40%, transparent)'};
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
          background: #{'color-mix(in srgb, var(--primary), 5%, transparent)'};
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
    background: #{'color-mix(in srgb, var(--dark-600), 10%, transparent)'};
    border-radius: 4px;
  }
  
  &::-webkit-scrollbar-thumb {
    background: linear-gradient(135deg, #{'color-mix(in srgb, var(--primary), 60%, transparent)'} 0%, #{'color-mix(in srgb, var(--secondary), 40%, transparent)'} 100%);
    border-radius: 4px;
    
    &:hover {
      background: linear-gradient(135deg, #{'color-mix(in srgb, var(--primary), 80%, transparent)'} 0%, #{'color-mix(in srgb, var(--secondary), 60%, transparent)'} 100%);
    }
  }
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
  color: var(--placeholder);
}

.error-state, .empty-state {
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

.error-state .retry-btn, .empty-state .clear-filters-btn {
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
  border-bottom: 1px solid #{'color-mix(in srgb, var(--primary), 15%, transparent)'};
  background: linear-gradient(135deg, #{'color-mix(in srgb, var(--container), 90%, transparent)'} 0%, #{'color-mix(in srgb, var(--card), 70%, transparent)'} 100%);
    
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