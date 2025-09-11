<script lang="ts">
import { installations, selectedInstallation, InstallationService, Icon, ModsService } from '$lib';
import * as installationsApi from '$lib/api/installations';
import { extendedModInfo } from '$lib/stores/mods';
import { openUrl } from '$lib/api/system';
import { onMount } from 'svelte';
import { get } from 'svelte/store';
import type { KableInstallation, ModJarInfo } from '$lib';

let currentInstallation: KableInstallation | null = null;
let selectedId: string = '';
let mods: ModJarInfo[] = [];
$: $extendedModInfo;
let loading = false;
let error: string | null = null;
let installationListContainer: HTMLElement;

// Track which mods we've attempted to fetch extended info for to prevent infinite loops
let attemptedExtendedInfo = new Set<string>();

// --- Semantic search/filter state ---
let searchQuery = '';

// --- Installation carousel logic ---
function selectInstallation(installation: KableInstallation) {
  selectedId = installation.id;
  currentInstallation = installation;
  selectedInstallation.set(installation);
  // Let the reactive statement handle loading mods to prevent duplicate calls
}

// Handle custom scroll behavior - cycle through installations
let scrollTimeout: number;
let scrollOffset = 0; // Track our virtual scroll position

function handleWheel(event: WheelEvent) {
  event.preventDefault(); // Prevent default scrolling
  
  if (scrollTimeout) clearTimeout(scrollTimeout);
  
  // Determine scroll direction and magnitude
  const scrollDelta = event.deltaY;
  const scrollThreshold = 50; // Minimum scroll to trigger selection change
  
  scrollOffset += scrollDelta;
  
  // Only change selection if we've scrolled enough
  if (Math.abs(scrollOffset) >= scrollThreshold) {
    const selectedIndex = sortedInstallations.findIndex(inst => inst.id === selectedId);
    let newIndex = selectedIndex;
    
    if (scrollOffset > 0) {
      // Scroll down - select next installation (with wrapping)
      newIndex = (selectedIndex + 1) % sortedInstallations.length;
    } else if (scrollOffset < 0) {
      // Scroll up - select previous installation (with wrapping)
      newIndex = (selectedIndex - 1 + sortedInstallations.length) % sortedInstallations.length;
    }
    
    if (newIndex !== selectedIndex) {
      const installation = sortedInstallations[newIndex];
      selectedId = installation.id;
      currentInstallation = installation;
      selectedInstallation.set(installation);
      
      // The reactive statement will handle loading mods when selectedId changes
    }
    
    // Reset scroll offset after triggering change
    scrollOffset = 0;
  }
  
  // Reset scroll offset after a delay if no change was triggered
  scrollTimeout = setTimeout(() => {
    scrollOffset = 0;
  }, 200);
}

// Handle keyboard navigation
function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'ArrowUp' || event.key === 'ArrowDown') {
    event.preventDefault();
    
    const selectedIndex = sortedInstallations.findIndex(inst => inst.id === selectedId);
    let newIndex = selectedIndex;
    
    if (event.key === 'ArrowDown') {
      // Select next installation (with wrapping)
      newIndex = (selectedIndex + 1) % sortedInstallations.length;
    } else if (event.key === 'ArrowUp') {
      // Select previous installation (with wrapping)
      newIndex = (selectedIndex - 1 + sortedInstallations.length) % sortedInstallations.length;
    }
    
    if (newIndex !== selectedIndex) {
      const installation = sortedInstallations[newIndex];
      selectInstallation(installation);
    }
  }
}

// Calculate carousel scaling and positioning for centered layout with wrapping
function getCarouselScale(currentIndex: number, selectedIndex: number, totalItems: number): { 
  scale: number; 
  opacity: number; 
  fontSize: number; 
  translateY: number;
  zIndex: number;
  visible: boolean;
} {
  // Calculate wrapped distance (shortest path around the carousel)
  const directDistance = Math.abs(currentIndex - selectedIndex);
  const wrapDistance = totalItems - directDistance;
  const distance = Math.min(directDistance, wrapDistance);
  
  // Determine relative position considering wrapping
  let relativePosition = currentIndex - selectedIndex;
  if (Math.abs(relativePosition) > totalItems / 2) {
    // Use wrapping path
    relativePosition = relativePosition > 0 ? relativePosition - totalItems : relativePosition + totalItems;
  }
  
  // Only show items within a certain distance from the selected item
  const maxVisibleDistance = Math.min(4, Math.ceil(totalItems / 2));
  const visible = distance <= maxVisibleDistance;
  
  if (!visible) {
    return { scale: 0, opacity: 0, fontSize: 0, translateY: 0, zIndex: 0, visible: false };
  }
  
  // More dramatic scaling for centered layout, but adapt based on how well items fit
  // Compute fit ratio using the container height and total required height
  const containerHeight = installationListContainer ? installationListContainer.clientHeight : (totalItems * 120);
  const baseItemHeight = 120;
  const fitRatio = Math.min(1, containerHeight / Math.max(1, totalItems * baseItemHeight)); // 0..1

  // When items fit well (fitRatio ~ 1) we want smaller spacing and less aggressive scale shrink
  const spacing = 20 * (1 - fitRatio) + 8; // ranges ~8..28

  const baseScaleFactors = [1.0, 0.85, 0.7, 0.55, 0.4];
  // scaleReduction closer to 0 means less shrink (when fitRatio=1), when fitRatio=0 keep original
  const scaleReduction = 1 - fitRatio * 0.3; // between 0.7 and 1
  const scaleFactors = baseScaleFactors.map(s => 1 - (1 - s) * scaleReduction);

  const opacityFactors = [1.0, 0.85, 0.7, 0.55, 0.4].map(o => o * (0.9 + 0.1 * fitRatio));
  const fontFactors = [1.0, 0.95, 0.9, 0.85, 0.8];

  const scale = scaleFactors[Math.min(distance, scaleFactors.length - 1)];
  const opacity = opacityFactors[Math.min(distance, opacityFactors.length - 1)];
  const fontSize = fontFactors[Math.min(distance, fontFactors.length - 1)];

  // Calculate vertical offset to center the selected item
  const itemHeight = baseItemHeight; // base height used for spacing calc
  // Compress spacing for near neighbors so selected item appears closer
  const distanceNorm = Math.min(distance, 4) / 4; // 0..1
  // Use a stronger compression floor so nearest items sit noticeably closer.
  // compressionFloor controls how much spacing nearest neighbors keep (0.0..1.0)
  const compressionFloor = 0.5; // previously ~0.6, lower -> tighter grouping
  const compression = compressionFloor + (1 - compressionFloor) * distanceNorm; // ranges compressionFloor..1.0
  const translateY = relativePosition * (itemHeight * scale + spacing * compression);
  
  // Z-index for layering (selected item on top)
  const zIndex = 100 - distance;
  
  return { scale, opacity, fontSize, translateY, zIndex, visible: true };
}

// --- Loader styling helpers (inspired by InstallationsList) ---
$: loaderIcons = Object.fromEntries(
  $installations.map(installation => [
    installation.id,
    InstallationService.getLoaderIcon(InstallationService.getVersionData(installation).loader)
  ])
);
$: loaderColors = Object.fromEntries(
  $installations.map(installation => [
    installation.id,
    InstallationService.getLoaderColor(InstallationService.getVersionData(installation).loader)
  ])
);

// --- Sort installations by favorite and date (same as InstallationsList) ---
$: sortedInstallations = $installations
  .slice()
  .sort((a, b) => {
    // Favorites first
    if ((a.favorite ? 1 : 0) !== (b.favorite ? 1 : 0)) {
      return (b.favorite ? 1 : 0) - (a.favorite ? 1 : 0);
    }
    // Then by last_used (most recent first)
    const aTime = a.last_used ? new Date(a.last_used).getTime() : 0;
    const bTime = b.last_used ? new Date(b.last_used).getTime() : 0;
    return bTime - aTime;
  })
  .filter(i => InstallationService.getVersionData(i).loader !== 'Vanilla');

// Track which installation we've loaded mods for to prevent infinite reactive loops
let loadedInstallationId: string | null = null;

// Reactively update currentInstallation and mods when selectedId changes
$: {
  const inst = get(installations).find(i => i.id === selectedId) || null;
  currentInstallation = inst;
  selectedInstallation.set(inst);
  
  // Only load mods if we haven't already loaded for this installation and we're not currently loading
  if (currentInstallation && 
      currentInstallation.id !== loadedInstallationId && 
      !loading) {
    loadedInstallationId = currentInstallation.id;
    loadMods(currentInstallation);
  } else if (!currentInstallation) {
    mods = [];
    loadedInstallationId = null;
  }
}

// Auto-select first installation if none is selected and installations are available
$: {
  if (!selectedId && sortedInstallations.length > 0) {
    const firstInstallation = sortedInstallations[0];
    selectedId = firstInstallation.id;
    currentInstallation = firstInstallation;
    selectedInstallation.set(firstInstallation);
    // The reactive statement above will handle loading mods
  }
}

// When mods change, trigger async loading of extended mod info for each mod (but NOT in the template)
$: if (mods && mods.length > 0) {
  // Only fetch for mods that are missing in the store (undefined means not attempted, null means failed)
  const missing = mods.filter(mod => 
    $extendedModInfo[mod.file_name] === undefined && 
    !attemptedExtendedInfo.has(mod.file_name)
  );
  if (missing.length > 0) {
    // Mark these mods as attempted to prevent infinite loops
    missing.forEach(mod => attemptedExtendedInfo.add(mod.file_name));
    Promise.all(missing.map(mod => ModsService.getExtendedModInfo(mod)));
  }
}

// --- Fuzzy search helper function ---
function fuzzyMatch(text: string, query: string): boolean {
  if (!query) return true;
  const textLower = text.toLowerCase();
  const queryLower = query.toLowerCase();
  
  // Exact match gets priority
  if (textLower.includes(queryLower)) return true;
  
  // Fuzzy matching: check if all query characters appear in order
  let textIndex = 0;
  let queryIndex = 0;
  
  while (textIndex < textLower.length && queryIndex < queryLower.length) {
    if (textLower[textIndex] === queryLower[queryIndex]) {
      queryIndex++;
    }
    textIndex++;
  }
  
  return queryIndex === queryLower.length;
}

// --- Semantic search/filter logic with fuzzy matching ---
$: filteredMods = mods.filter(mod => {
  const info = $extendedModInfo[mod.file_name];
  if (searchQuery) {
    const name = info?.mod_jar_info?.mod_name || mod.mod_name || '';
    const desc = info?.description || '';
    const file = mod.file_name;
    
    return fuzzyMatch(name, searchQuery) || 
           fuzzyMatch(desc, searchQuery) || 
           fuzzyMatch(file, searchQuery);
  }
  return true;
});

async function handleModClick(mod: ModJarInfo) {
  const extendedInfo = $extendedModInfo[mod.file_name];
  if (extendedInfo?.page_uri) {
    try {
      await openUrl(extendedInfo.page_uri);
    } catch (error) {
      console.error('Failed to open mod page:', error);
    }
  }
}

// Toggle disabled state via the backend API. If Ctrl/Cmd is held when activating,
// we open the mod page instead (preserves previous behavior).
async function toggleModDisabledAction(mod: ModJarInfo) {
  if (!currentInstallation) return;
  try {
    const newDisabled = await installationsApi.toggleModDisabled(currentInstallation, mod.file_name);
    // Update local list optimistically so UI reacts immediately
    mods = mods.map(m => (m.file_name === mod.file_name ? { ...m, disabled: newDisabled } : m));
  } catch (err) {
    console.error('Failed to toggle disabled state for', mod.file_name, err);
    // Try reloading mods to resync state
    try { await loadMods(currentInstallation); } catch (_) {}
  }
}

async function onModActivate(event: MouseEvent, mod: ModJarInfo) {
  // Ctrl/Cmd + click opens mod page
  if (event.ctrlKey || event.metaKey) {
    await handleModClick(mod);
    return;
  }
  await toggleModDisabledAction(mod);
}

async function onModKeyDown(event: KeyboardEvent, mod: ModJarInfo) {
  if (event.key !== 'Enter') return;
  if (event.ctrlKey || event.metaKey) {
    await handleModClick(mod);
    return;
  }
  await toggleModDisabledAction(mod);
}

function updateTooltipPosition(event: MouseEvent) {
  const modIcon = event.currentTarget as HTMLElement;
  const tooltip = modIcon.querySelector('.mod-tooltip') as HTMLElement;
  if (!tooltip) return;

  const iconRect = modIcon.getBoundingClientRect();
  const containerRect = modIcon.closest('.mods-content')?.getBoundingClientRect();
  if (!containerRect) return;

  // Calculate available space on each side
  const spaceRight = containerRect.right - iconRect.right;
  const spaceLeft = iconRect.left - containerRect.left;
  const spaceTop = iconRect.top - containerRect.top;
  const spaceBottom = containerRect.bottom - iconRect.bottom;

  // Tooltip dimensions (approximate)
  const tooltipWidth = 280; // max-width from CSS
  const tooltipHeight = 120; // approximate height

  // Reset all positioning classes and inline styles
  tooltip.classList.remove('tooltip-right', 'tooltip-left', 'tooltip-top', 'tooltip-bottom');
  tooltip.style.left = '';
  tooltip.style.right = '';
  tooltip.style.top = '';
  tooltip.style.bottom = '';

  // Determine best position based on available space
  if (spaceRight >= tooltipWidth + 20) {
    // Default right position has enough space
    tooltip.classList.add('tooltip-right');
  } else if (spaceLeft >= tooltipWidth + 20) {
    // Switch to left position
    tooltip.classList.add('tooltip-left');
  } else if (spaceTop >= tooltipHeight + 20) {
    // Switch to top position
    tooltip.classList.add('tooltip-top');
  } else if (spaceBottom >= tooltipHeight + 20) {
    // Switch to bottom position
    tooltip.classList.add('tooltip-bottom');
  } else {
    // Default to right but adjust horizontal position if needed
    tooltip.classList.add('tooltip-right');
    
    // If still not enough space, position closer to the icon
    if (spaceRight < tooltipWidth) {
      const adjustment = tooltipWidth - spaceRight + 10;
      tooltip.style.left = `calc(105% - ${adjustment}px)`;
    }
  }
}

function resetTooltipPosition(event: MouseEvent) {
  const modIcon = event.currentTarget as HTMLElement;
  const tooltip = modIcon.querySelector('.mod-tooltip') as HTMLElement;
  if (!tooltip) return;

  // Reset all positioning classes and inline styles
  tooltip.classList.remove('tooltip-right', 'tooltip-left', 'tooltip-top', 'tooltip-bottom');
  tooltip.style.left = '';
  tooltip.style.right = '';
  tooltip.style.top = '';
  tooltip.style.bottom = '';
}

async function loadMods(installation: KableInstallation) {
  loading = true;
  error = null;
  try {
    mods = await InstallationService.getModInfo(installation);
    // Clear the attempted set when loading new mods
    attemptedExtendedInfo.clear();
    // Successfully loaded, keep the loadedInstallationId
  } catch (e: any) {
    error = e?.message || e || 'Failed to load mods info';
    mods = [];
    attemptedExtendedInfo.clear();
    // Reset the loaded installation ID so we can retry if user switches away and back
    loadedInstallationId = null;
  } finally {
    loading = false;
  }
}

onMount(() => {
  const inst = get(selectedInstallation);
  if (inst) {
    selectedId = inst.id;
  } else if (sortedInstallations.length > 0) {
    // If no installation is pre-selected, select the first one (favorite/most recent)
    const firstInstallation = sortedInstallations[0];
    selectedId = firstInstallation.id;
    selectedInstallation.set(firstInstallation);
  }
});
</script>

<div class="installation-mods">
  <div class="main-layout">
    <!-- Left sidebar: Installation carousel -->
    <div class="installation-sidebar">
      <h2>Installations</h2>
      <div class="installation-carousel" 
           bind:this={installationListContainer} 
           on:wheel={handleWheel} 
           on:keydown={handleKeydown}
           tabindex="-1" role="listbox">
        <div class="carousel-container">
          {#each sortedInstallations as installation, index}
            {@const selectedIndex = sortedInstallations.findIndex(inst => inst.id === selectedId)}
            {@const carouselEffects = getCarouselScale(index, selectedIndex >= 0 ? selectedIndex : 0, sortedInstallations.length)}
            {#if carouselEffects.visible}
          <div 
            class="installation-item" 
            class:selected={installation.id === selectedId}
            data-installation-id={installation.id}
            style="
              background: linear-gradient(135deg, {loaderColors[installation.id]}22 0%, {loaderColors[installation.id]}08 40%); 
              --loader-color: {loaderColors[installation.id]}; 
              --loader-icon: '{loaderIcons[installation.id]}';
              --carousel-scale: {carouselEffects.scale};
              --carousel-opacity: {carouselEffects.opacity};
              --carousel-font-size: {carouselEffects.fontSize};
              transform: scale({carouselEffects.scale}) translateY({carouselEffects.translateY}px) translateX(-50%);
              opacity: {carouselEffects.opacity};
              font-size: calc({carouselEffects.fontSize} * 0.9em);
              z-index: {carouselEffects.zIndex};
            "
            on:click={() => selectInstallation(installation)}
            on:keydown={(e) => e.key === 'Enter' && selectInstallation(installation)}
            tabindex="0"
            role="button"
          >
            <div class="installation-icon">
              <Icon name={loaderIcons[installation.id]} size="md" />
            </div>
            <div class="installation-meta">
              <div class="installation-name">{installation.name}</div>
              <div class="installation-details">
                <span class="installation-version">{InstallationService.getVersionData(installation).version_id}</span>
              </div>
            </div>
          </div>
            {/if}
        {/each}
        </div>
      </div>
    </div>

    <!-- Right content: Search and mods -->
    <div class="mods-section">
      <div class="mods-header">
        <div class="search-controls">
          <div class="search-input-wrapper">
            <span class="search-icon">🔍</span>
            <input
              type="text"
              placeholder="Search mods (fuzzy search enabled)..."
              bind:value={searchQuery}
              class="search-input"
            />
            {#if searchQuery}
              <button class="clear-btn" on:click={() => (searchQuery = '')} title="Clear search">✕</button>
            {/if}
          </div>
        </div>
        
        {#if currentInstallation}
          <div class="mods-title-section">
            <h3>Mods for {currentInstallation.name}</h3>
            {#if mods.length > 0}
              <div class="mods-count-badge">
                {#if searchQuery}
                  <span class="filtered-count">{filteredMods.length}</span>
                  <span class="count-separator">of</span>
                  <span class="total-count">{mods.length}</span>
                  <span class="count-label">mods</span>
                {:else}
                  <span class="total-count">{mods.length}</span>
                  <span class="count-label">{mods.length === 1 ? 'mod' : 'mods'}</span>
                {/if}
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <div class="mods-content">
        {#if currentInstallation}
          {#if loading}
            <div class="loading-state">
              <Icon name="refresh" size="md" className="spin" />
              <span>Loading mods...</span>
            </div>
          {:else if error}
            <div class="error-message">
              <Icon name="alert" size="sm" />
              {error}
            </div>
          {:else if mods.length > 0}
          <span class="mods-instructions">Click mods to disable/enable them</span>
            <div class="mods-icon-grid">
              {#each filteredMods as mod}
                {#if $extendedModInfo[mod.file_name]}
                  <div
                    class="mod-icon-link"
                    class:clickable={!!$extendedModInfo[mod.file_name]?.page_uri}
                    class:disabled={!!mod.disabled}
                    on:click={(e) => onModActivate(e as MouseEvent, mod)}
                    on:keydown={(e) => onModKeyDown(e as KeyboardEvent, mod)}
                    on:mouseenter={updateTooltipPosition}
                    on:mouseleave={resetTooltipPosition}
                    role="button"
                    tabindex="0"
                    title=""
                    aria-label={$extendedModInfo[mod.file_name]?.mod_jar_info.mod_name || $extendedModInfo[mod.file_name]?.mod_jar_info.file_name}
                    aria-pressed={mod.disabled ? 'true' : 'false'}
                  >
                    {#if $extendedModInfo[mod.file_name]?.icon_uri}
                      <img class="mod-icon" src={$extendedModInfo[mod.file_name]?.icon_uri} alt="" title=""/>
                    {:else}
                      <Icon name="package" size="lg" />
                    {/if}
                    
                    <div class="mod-tooltip">
                      <div class="mod-tooltip-title">{$extendedModInfo[mod.file_name]?.mod_jar_info.mod_name || $extendedModInfo[mod.file_name]?.mod_jar_info.file_name}</div>
                      {#if $extendedModInfo[mod.file_name]?.mod_jar_info.mod_version}
                        <div class="mod-tooltip-version">{$extendedModInfo[mod.file_name]?.mod_jar_info.mod_version}</div>
                      {/if}
                      {#if $extendedModInfo[mod.file_name]?.description}
                        <div class="mod-tooltip-desc">{$extendedModInfo[mod.file_name]?.description || ''}</div>
                      {/if}
                      {#if $extendedModInfo[mod.file_name]?.page_uri}
                        <div class="mod-tooltip-link">Click to view on Modrinth</div>
                      {/if}
                    </div>
                  </div>
                {:else}
                  <div class="mod-icon-link loading"><Icon name="package" size="lg" /></div>
                {/if}
              {/each}
            </div>
          {:else}
            <div class="empty-state">
              <Icon name="cube" size="xl" />
              <span>No mods installed for this installation.</span>
            </div>
          {/if}
        {:else}
          <div class="empty-state">
            <Icon name="package" size="xl" />
            <span>Select an installation to view mods.</span>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;
.installation-mods {
  margin: 0;
  height: 100%;
  overflow: clip;
}

.main-layout {
  display: flex;
  height: 100%;
  gap: 1.5rem;
  background: var(--container);
  border-radius: 0.75rem;
  border: 1px solid #{'color-mix(in srgb, var(--primary), 8%, transparent)'};
  box-shadow: 0 2px 12px #{'color-mix(in srgb, var(--dark-900), 6%, transparent)'};
  overflow: hidden;
}

// --- Left sidebar: Installation carousel ---
.installation-sidebar {
  width: 320px;
  min-width: 320px;
  border-right: 1px solid color-mix(in srgb, var(--primary), 8%, transparent);
  display: flex;
  flex-direction: column;
  
  h2 {
    margin: 0;
    padding: 1.5rem 1.5rem 1rem 1.5rem;
    background: linear-gradient(135deg, var(--primary) 0%, var(--secondary) 100%);
    background-clip: text;
    -webkit-background-clip: text;
    color: transparent;
    font-weight: 700;
    font-size: 1.4em;
    border-bottom: 1px solid #{'color-mix(in srgb, var(--primary), 8%, transparent)'};
  }
}

.mod-icon-link.disabled {
  border: 2px solid var(--red-600, #d9534f);
  box-shadow: 0 1px 6px 0 color-mix(in srgb, var(--red), 12%, transparent) !important;
  filter: grayscale(70%) opacity(0.9);
}

.mod-icon-link.disabled .mod-tooltip {
  background: color-mix(in srgb, var(--red), 6%, var(--card));
  border-color: color-mix(in srgb, var(--red), 20%, var(--card));
}
.mods-instructions {
  font-size: 0.875em;
  color: var(--placeholder);
  margin-top: 0.5rem;
}

.installation-carousel {
  flex: 1;
  overflow: hidden; // Change from auto to hidden for custom scroll
  padding: 0.5rem; // Reduced padding to use more height
  position: relative;
  display: flex;
  align-items: center; // Center the carousel vertically
  
  // Remove scrollbar styles since we're not using native scroll
  &:focus {
    outline: none;
  }
}

.carousel-container {
  width: 100%;
  height: 100%;
  position: relative;
  display: flex;
  flex-direction: column;
  // align-items: center;
  // justify-content: center;
  pointer-events: none; // Allow clicks to pass through to individual items
}

.installation-item {
  padding: 1rem 1.2rem; // Increased padding for better use of space
  margin: 0;
  border-radius: 0.6rem;
  cursor: pointer;
  border: 1px solid transparent;
  position: absolute; // Absolute positioning for custom carousel layout
  top: 35%; // Center vertically
  left: 50%; // Center horizontally
  transform-origin: center center;
  display: flex;
  align-items: center;
  gap: 0.5rem; // Increased gap
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  width: 100%; // Slightly increased width
  // margin-left: -145px; // Half of width to center
  pointer-events: auto; // Re-enable clicks for individual items
  
  // Enhanced transitions for smooth movement
  transition: all 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94);
  
  &:hover {
  border-color: var(--loader-color, #{'color-mix(in srgb, var(--primary), 15%, transparent)'});
  // Properly maintain center position on hover
  box-shadow: 0 2px 8px #{'color-mix(in srgb, var(--loader-color, var(--primary)), 10%, transparent)'};
  }

  &.selected {
    border-color: var(--loader-color, var(--primary));
    box-shadow: 
      0 4px 16px #{'color-mix(in srgb, var(--loader-color, var(--primary)), 15%, transparent)'},
       inset 0 1px 0 rgba(255, 255, 255, 0.1);
    z-index: 10; // Bring selected item to front
    
    // Add green selection indicator
    border: 2px solid var(--green-800);
    
    &:hover {
      // Keep the same transform as base state but with slight scale increase
      box-shadow: 
        0 6px 20px #{'color-mix(in srgb, var(--loader-color, var(--primary)), 20%, transparent)'},
        0 0 0 3px #{'color-mix(in srgb, var(--green-800), 30%, transparent)'},
        inset 0 1px 0 #{'color-mix(in srgb, #fff, 15%, transparent)'};
    }
    
    &::before {
      content: '';
      position: absolute;
      left: -0.4rem;
      top: 50%;
      transform: translateY(-50%);
      width: 4px;
      height: 60%;
      background: linear-gradient(to bottom, var(--green-700), var(--green-900));
      box-shadow: 0 0 8px #{'color-mix(in srgb, var(--green-800), 40%, transparent)'};
    }
  }
  
  &:focus {
    outline: none;
    box-shadow: 0 0 0 2px #{'color-mix(in srgb, var(--loader-color, var(--primary)), 30%, transparent)'};
  }
}

.installation-icon {
  width: calc(48px * var(--carousel-scale, 1)); // Increased icon size
  height: calc(48px * var(--carousel-scale, 1));
  border-radius: 0.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--container);
  color: var(--loader-color, var(--primary));
  box-shadow: 0 2px 6px #{'color-mix(in srgb, var(--dark-900), 8%, transparent)'};
  flex-shrink: 0;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  
  .installation-item.selected & {
  background: linear-gradient(135deg, var(--loader-color, var(--primary)) 0%, #{'color-mix(in srgb, var(--loader-color, var(--secondary)), 80%, transparent)'} 100%);
   color: white;
  box-shadow: 0 3px 12px #{'color-mix(in srgb, var(--loader-color, var(--primary)), 30%, transparent)'};
   transform: scale(1.05);
   }
}

.installation-meta {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.installation-name {
  font-weight: calc(500 + (var(--carousel-scale, 1) * 100));
  margin-bottom: 0.2em;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  transition: all 0.25s;
  font-size: calc(var(--carousel-font-size, 1) * 1em);
  
  // Default state
  color: var(--text);
  
  // Selected state - use solid color instead of gradient to avoid transparency issues
  .installation-item.selected & {
    color: var(--loader-color, var(--primary));
    font-weight: 700;
    text-shadow: 0 0 8px #{'color-mix(in srgb, var(--loader-color, var(--primary)), 30%, transparent)'};
   }
}

.installation-details {
  display: flex;
  gap: 0.4em;
  flex-wrap: wrap;
}

.installation-version {
  font-size: calc(var(--carousel-font-size, 1) * 0.75em);
  padding: 0.15em 0.4em;
  border-radius: 0.3em;
  font-weight: 500;
  transition: all 0.25s;
  opacity: calc(var(--carousel-opacity, 1) * 0.9);
}

.installation-version {
  background: color-mix(in srgb, var(--tertiary), 10%, transparent);
  color: var(--tertiary);
  
  .installation-item.selected & {
  background: #{'color-mix(in srgb, var(--loader-color, var(--tertiary)), 15%, transparent)'};
     color: var(--loader-color, var(--tertiary));
   }
}

// --- Right content: Search and mods ---
.mods-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.mods-header {
  background: linear-gradient(135deg, var(--card) 0%, color-mix(in srgb, var(--primary), 2%, transparent) 100%);
  border-bottom: 1px solid color-mix(in srgb, var(--primary), 8%, transparent);
  padding: 1.2rem 1.5rem;
  
  .mods-title-section {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 1rem 0 0.5rem 0;
    
    h3 {
      margin: 0;
      color: var(--text);
      font-weight: 500;
      font-size: 1.1em;
    }
  }
}

.mods-count-badge {
  display: flex;
  align-items: center;
  gap: 0.3em;
  background: linear-gradient(135deg, #{'color-mix(in srgb, var(--primary), 8%, transparent)'} 0%, #{'color-mix(in srgb, var(--secondary), 4%, transparent)'} 100%);
  border: 1px solid #{'color-mix(in srgb, var(--primary), 15%, transparent)'};
  border-radius: 1rem;
  padding: 0.4em 0.8em;
  font-size: 0.85em;
  font-weight: 500;
  backdrop-filter: blur(6px);
  -webkit-backdrop-filter: blur(6px);
  box-shadow: 0 1px 4px #{'color-mix(in srgb, var(--dark-900), 6%, transparent)'};
  
  .filtered-count {
    color: var(--primary);
    font-weight: 600;
  }
  
  .count-separator {
    color: var(--placeholder);
    font-size: 0.9em;
  }
  
  .total-count {
    color: var(--text);
    font-weight: 600;
  }
  
  .count-label {
    color: var(--placeholder);
    font-size: 0.9em;
  }
}

.search-controls {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.search-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  
  .search-icon {
    position: absolute;
    left: 0.85rem;
    top: 50%;
    transform: translateY(-50%);
    color: var(--placeholder);
    z-index: 1;
  }
  
  .search-input {
    flex: 1;
    padding: 0.75rem 1rem 0.75rem 2.5rem;
    border: 1px solid var(--dark-600);
    border-radius: 0.75rem;
    background: var(--input);
    color: var(--text);
    font-size: 1rem;
    
    &:focus {
      outline: none;
      border-color: var(--primary);
    }
    
    &::placeholder {
      color: var(--placeholder);
    }
  }
  
  .clear-btn {
    position: absolute;
    right: 0.75rem;
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    color: var(--red);
    font-size: 1.1em;
    cursor: pointer;
    padding: 0.25em;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.1s;
    z-index: 1;
    
    &:hover {
      color: var(--red-600);
      transform: translateY(-50%) scale(1.1);
    }
  }
}

.mods-content {
  flex: 1;
  padding: 1.5rem;
  background: var(--container);
  overflow-y: auto;
  overflow-x: hidden;
}


// --- Compact icon grid for mods with enhanced hover effects ---
.mods-icon-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
  padding: 0.3rem 0.1rem 0.3rem 0.1rem;
  align-items: flex-start;
  justify-content: flex-start;
}

.mod-icon-link {
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  background: var(--card);
  border-radius: 0.5rem;
  border: 1px solid color-mix(in srgb, var(--primary), 10%, transparent);
  width: 48px;
  height: 48px;
  min-width: 48px;
  min-height: 48px;
  box-shadow: 0 1px 6px 0 color-mix(in srgb, var(--dark-900), 6%, transparent), inset 0 1px 0 rgba(255, 255, 255, 0.08);
  transition: all 0.12s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: pointer;
  overflow: visible;
  backdrop-filter: blur(6px);
  -webkit-backdrop-filter: blur(6px);
  
  &:hover {
    transform: translateY(-1px) scale(1.04);
    box-shadow: 
      0 6px 20px 0 color-mix(in srgb, var(--primary), 18%, transparent),
      0 2px 8px 0 color-mix(in srgb, var(--dark-900), 12%, transparent),
      inset 0 1px 0 color-mix(in srgb, #fff, 15%, transparent);
    border-color: var(--primary);
    background: linear-gradient(135deg, var(--card) 0%, #{'color-mix(in srgb, var(--primary), 6%, transparent)'} 100%);
    z-index: 15;
    
    .mod-tooltip {
      opacity: 1;
      pointer-events: none;
      transform: translateY(-6px) scale(1.01);
      transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    }
    
    .mod-icon {
      transform: scale(1.08);
      filter: brightness(1.08) contrast(1.08) saturate(1.15);
    }
  }
  
  &.loading {
  background: linear-gradient(135deg, var(--card) 0%, #{'color-mix(in srgb, var(--placeholder), 4%, transparent)'} 100%);
    color: var(--placeholder);
    animation: pulse 1.8s ease-in-out infinite;
  }
  
  &.clickable {
    cursor: pointer;
    
    &:hover {
      .mod-tooltip-link {
        opacity: 1;
      }
    }
  }
  
  &:not(.clickable) {
    cursor: default;
  }
}

@keyframes pulse {
  0%, 100% { opacity: 0.6; }
  50% { opacity: 1; }
}

.mod-icon {
  width: 38px;
  height: 38px;
  border-radius: 0.4rem;
  object-fit: cover;
  background: linear-gradient(45deg, var(--background) 0%, #{'color-mix(in srgb, var(--primary), 2%, transparent)'} 100%);
  box-shadow: 0 1px 4px color-mix(in srgb, var(--dark-900), 8%, transparent), inset 0 1px 0 rgba(255, 255, 255, 0.08);
  transition: all 0.12s cubic-bezier(0.4, 0, 0.2, 1);
  border: 1px solid color-mix(in srgb, var(--primary), 8%, transparent);
}

.mod-tooltip {
  opacity: 0;
  pointer-events: none;
  position: absolute;
  background: var(--card);
  color: var(--text);
  border: 1px solid color-mix(in srgb, var(--primary), 25%, transparent);
  border-radius: 0.5rem;
  padding: 0.5em 0.8em;
  font-size: 0.85em;
  font-weight: 500;
  min-width: 160px;
  max-width: 280px;
  box-shadow: 
  0 8px 24px 0 color-mix(in srgb, var(--dark-900), 25%, transparent),
  0 2px 6px 0 color-mix(in srgb, var(--primary), 15%, transparent),
    inset 0 1px 0 rgba(255, 255, 255, 0.15);
  z-index: 100;
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  
  // Default positioning (right)
  left: 105%;
  top: 0%;
  transform: translateY(-8%) scale(0.96);
  margin-top: 0.2em;
  
  &::before {
    content: '';
    position: absolute;
    width: 10px;
    height: 10px;
  background: linear-gradient(135deg, #{'color-mix(in srgb, var(--container), 98%, transparent)'} 0%, #{'color-mix(in srgb, var(--primary), 100%, transparent)'} 100%);
  border-left: 1px solid color-mix(in srgb, var(--primary), 25%, transparent);
  border-bottom: 1px solid color-mix(in srgb, var(--primary), 25%, transparent);
    transform: rotate(45deg);
    // Default arrow position (left side, pointing left)
    left: -5px;
    top: 18%;
  }
  
  .mod-tooltip-title {
    font-weight: 600;
    font-size: 1em;
    background: linear-gradient(135deg, var(--primary) 0%, var(--secondary) 100%);
    background-clip: text;
    -webkit-background-clip: text;
    color: transparent;
    margin-bottom: 0.25em;
    text-overflow: ellipsis;
    overflow: hidden;
    white-space: nowrap;
  }
  .mod-tooltip-version {
    font-size: 0.8em;
    color: var(--tertiary);
  background: color-mix(in srgb, var(--tertiary), 8%, transparent);
    padding: 0.15em 0.4em;
    border-radius: 0.25em;
    display: inline-block;
    margin-bottom: 0.3em;
    font-weight: 500;
  }
  .mod-tooltip-desc {
    font-size: 0.78em;
    color: var(--placeholder);
    margin-top: 0.2em;
    line-height: 1.35;
    max-width: 260px;
    word-break: break-word;
  }
  
  .mod-tooltip-link {
    font-size: 0.75em;
    color: var(--primary);
    margin-top: 0.4em;
    padding: 0.2em 0.4em;
  background: color-mix(in srgb, var(--primary), 8%, transparent);
    border-radius: 0.25em;
    text-align: center;
    font-weight: 500;
    opacity: 0.8;
    transition: opacity 0.15s;
  }
}

// Global tooltip positioning classes to prevent Svelte from removing them
:global(.mod-tooltip.tooltip-right) {
  left: 105%;
  top: 0%;
  transform: translateY(-8%) scale(0.96);
  
  &::before {
    left: -5px;
    top: 18%;
  border-left: 1px solid color-mix(in srgb, var(--primary), 25%, transparent);
  border-bottom: 1px solid color-mix(in srgb, var(--primary), 25%, transparent);
    border-right: none;
    border-top: none;
    transform: rotate(45deg);
  }
}

:global(.mod-tooltip.tooltip-left) {
  right: 105%;
  left: auto;
  top: 0%;
  transform: translateY(-8%) scale(0.96);
  
  &::before {
    right: -5px;
    left: auto;
    top: 18%;
  border-right: 1px solid color-mix(in srgb, var(--primary), 25%, transparent);
  border-top: 1px solid color-mix(in srgb, var(--primary), 25%, transparent);
    border-left: none;
    border-bottom: none;
    transform: rotate(45deg);
  }
}

:global(.mod-tooltip.tooltip-top) {
  left: 50%;
  top: auto;
  bottom: 105%;
  transform: translateX(-50%) translateY(8%) scale(0.96);
  
  &::before {
    left: 50%;
    top: auto;
    bottom: -5px;
    transform: translateX(-50%) rotate(45deg);
  border-bottom: 1px solid color-mix(in srgb, var(--primary), 25%, transparent);
  border-right: 1px solid color-mix(in srgb, var(--primary), 25%, transparent);
    border-left: none;
    border-top: none;
  }
}

:global(.mod-tooltip.tooltip-bottom) {
  left: 50%;
  top: 105%;
  transform: translateX(-50%) translateY(-8%) scale(0.96);
  
  &::before {
    left: 50%;
    top: -5px;
    bottom: auto;
    transform: translateX(-50%) rotate(45deg);
  border-top: 1px solid color-mix(in srgb, var(--primary), 25%, transparent);
  border-left: 1px solid color-mix(in srgb, var(--primary), 25%, transparent);
    border-right: none;
    border-bottom: none;
  }
}

.loading-state {
  display: flex;
  align-items: center;
  gap: 0.7em;
  color: var(--placeholder);
  font-size: 1.1em;
  padding: 2.5rem 0;
  justify-content: center;
}
.error-message {
  color: var(--red);
  background: color-mix(in srgb, var(--red), 8%, transparent);
  border: 1px solid var(--red);
  border-radius: 0.5rem;
  padding: 0.7em 1em;
  margin: 1em 0;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 0.5em;
}
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--placeholder);
  font-size: 1.1em;
  padding: 2.5rem 0 1.5rem 0;
  gap: 0.5em;
}
@media (max-width: 900px) {
  .main-layout {
    flex-direction: column;
    height: auto;
  }
  
  .installation-sidebar {
    width: 100%;
    min-width: auto;
    
    h2 {
      padding: 1rem 1.5rem 0.5rem 1.5rem;
    }
  }
  
  .installation-carousel {
    display: flex;
    overflow-x: auto;
    overflow-y: hidden;
    padding: 0.5rem 1rem 1rem 1rem;
    
    .installation-item {
      min-width: 220px;
      margin: 0 0.3rem;
      flex-shrink: 0;
    }
  }
  
  .mods-section {
    border-left: none;
  border-top: 1px solid color-mix(in srgb, var(--primary), 8%, transparent);
  }
}

@media (max-width: 600px) {
  .installation-mods {
    padding: 0.5rem;
    height: auto;
  }
  
  .mods-icon-grid {
    gap: 0.3rem;
  }
  .mod-icon-link {
    width: 44px;
    height: 44px;
    min-width: 44px;
    min-height: 44px;
  }
  .mod-icon {
    width: 34px;
    height: 34px;
  }
  .mod-tooltip {
    left: auto;
    right: 105%;
    min-width: 140px;
    font-size: 0.8em;
  }
}
</style>
