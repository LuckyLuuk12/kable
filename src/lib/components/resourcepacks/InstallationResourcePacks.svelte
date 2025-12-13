<!-- @component
InstallationResourcePacks - Manage resource packs for installations

Features:
- Installation carousel selector
- Fuzzy search for packs
- Enable/disable/remove packs
- Drag-drop ordering (when merging enabled)
- Pack merging toggle
- Visit pack pages
-->
<script lang="ts">
import { onMount } from "svelte";
import { get } from "svelte/store";
import { Icon, NotificationService, InstallationService } from "$lib";
import type { KableInstallation } from "$lib";
import { installations, selectedInstallation } from "$lib/stores";
import InstalledResourcePackCard from "./InstalledResourcePackCard.svelte";
import * as installationsApi from "$lib/api/installations";
import { openUrl } from "$lib/api/system";
import {
  dndzone,
  TRIGGERS,
  SHADOW_ITEM_MARKER_PROPERTY_NAME,
} from "svelte-dnd-action";
import type { DndEvent } from "svelte-dnd-action";

let selectedId: string = "";
let currentInstallation: KableInstallation | null = null;
let packs: any[] = [];
let extendedPackInfo: Record<string, any> = {};
let attemptedExtendedInfo = new Set<string>();
let loading = false;
let error: string | null = null;
let searchQuery = "";
let installationListContainer: HTMLDivElement | null = null;

// Pack ordering and merging state
let packMergingEnabled = false;
let originalPackOrder: string[] = [];
let currentPackOrder: string[] = [];
let hasOrderChanged = false;
let savingOrder = false;
let orderedPacks: any[] = [];

// Drag-drop state
let dragDisabled = true;
const flipDurationMs = 200;

// Scroll tracking for fade indicator
let dragListElement: HTMLElement | null = null;
let showScrollFade = false;

function checkScrollFade() {
  if (!dragListElement) return;
  const { scrollTop, scrollHeight, clientHeight } = dragListElement;
  showScrollFade = scrollTop + clientHeight < scrollHeight - 10;
}

function selectInstallation(installation: KableInstallation) {
  selectedId = installation.id;
  currentInstallation = installation;
  selectedInstallation.set(installation);
}

// Handle mouse wheel for carousel
function handleWheel(event: WheelEvent) {
  if (sortedInstallations.length === 0) return;

  const delta = event.deltaY;
  const selectedIndex = sortedInstallations.findIndex(
    (inst) => inst.id === selectedId,
  );

  if (delta > 0) {
    // Scroll down - select next installation
    const nextIndex = (selectedIndex + 1) % sortedInstallations.length;
    selectInstallation(sortedInstallations[nextIndex]);
  } else if (delta < 0) {
    // Scroll up - select previous installation
    const prevIndex =
      (selectedIndex - 1 + sortedInstallations.length) %
      sortedInstallations.length;
    selectInstallation(sortedInstallations[prevIndex]);
  }

  event.preventDefault();
}

// Handle keyboard navigation
function handleKeydown(event: KeyboardEvent) {
  if (event.key === "ArrowUp" || event.key === "ArrowDown") {
    event.preventDefault();

    const selectedIndex = sortedInstallations.findIndex(
      (inst) => inst.id === selectedId,
    );
    let newIndex = selectedIndex;

    if (event.key === "ArrowDown") {
      newIndex = (selectedIndex + 1) % sortedInstallations.length;
    } else if (event.key === "ArrowUp") {
      newIndex =
        (selectedIndex - 1 + sortedInstallations.length) %
        sortedInstallations.length;
    }

    if (newIndex !== selectedIndex) {
      const installation = sortedInstallations[newIndex];
      selectInstallation(installation);
    }
  }
}

// Calculate carousel scaling and positioning
function getCarouselScale(
  currentIndex: number,
  selectedIndex: number,
  totalItems: number,
): {
  scale: number;
  opacity: number;
  fontSize: number;
  translateY: number;
  zIndex: number;
  visible: boolean;
} {
  const directDistance = Math.abs(currentIndex - selectedIndex);
  const wrapDistance = totalItems - directDistance;
  const distance = Math.min(directDistance, wrapDistance);

  let relativePosition = currentIndex - selectedIndex;
  if (Math.abs(relativePosition) > totalItems / 2) {
    relativePosition =
      relativePosition > 0
        ? relativePosition - totalItems
        : relativePosition + totalItems;
  }

  const maxVisibleDistance = Math.min(4, Math.ceil(totalItems / 2));
  const visible = distance <= maxVisibleDistance;

  if (!visible) {
    return {
      scale: 0,
      opacity: 0,
      fontSize: 0,
      translateY: 0,
      zIndex: 0,
      visible: false,
    };
  }

  const containerHeight = installationListContainer
    ? installationListContainer.clientHeight
    : totalItems * 120;
  const baseItemHeight = 120;
  const fitRatio = Math.min(
    1,
    containerHeight / Math.max(1, totalItems * baseItemHeight),
  );

  const spacing = 20 * (1 - fitRatio) + 8;

  const baseScaleFactors = [1.0, 0.85, 0.7, 0.55, 0.4];
  const scaleReduction = 1 - fitRatio * 0.3;
  const scaleFactors = baseScaleFactors.map(
    (s) => 1 - (1 - s) * scaleReduction,
  );

  const opacityFactors = [1.0, 0.85, 0.7, 0.55, 0.4].map(
    (o) => o * (0.9 + 0.1 * fitRatio),
  );
  const fontFactors = [1.0, 0.95, 0.9, 0.85, 0.8];

  const scale = scaleFactors[Math.min(distance, scaleFactors.length - 1)];
  const opacity = opacityFactors[Math.min(distance, opacityFactors.length - 1)];
  const fontSize = fontFactors[Math.min(distance, fontFactors.length - 1)];

  const itemHeight = baseItemHeight;
  const distanceNorm = Math.min(distance, 4) / 4;
  const compressionFloor = 0.5;
  const compression = compressionFloor + (1 - compressionFloor) * distanceNorm;
  const translateY =
    relativePosition * (itemHeight * scale + spacing * compression);

  const zIndex = 100 - distance;

  return { scale, opacity, fontSize, translateY, zIndex, visible: true };
}

// Loader styling helpers
$: loaderIcons = Object.fromEntries(
  $installations.map((installation) => [
    installation.id,
    InstallationService.getLoaderIcon(
      InstallationService.getVersionData(installation).loader,
    ),
  ]),
);
$: loaderColors = Object.fromEntries(
  $installations.map((installation) => [
    installation.id,
    InstallationService.getLoaderColor(
      InstallationService.getVersionData(installation).loader,
    ),
  ]),
);

// Create Global pseudo-installation
const globalInstallation: KableInstallation = {
  id: "global",
  name: "Global (All Installations)",
  icon: null,
  version_id: "global",
  created: new Date().toISOString(),
  last_used: new Date().toISOString(),
  java_args: [],
  dedicated_mods_folder: null,
  dedicated_resource_pack_folder: null,
  dedicated_shaders_folder: null,
  dedicated_config_folder: null,
  favorite: false,
  total_time_played_ms: 0,
  parameters_map: {},
  description: null,
  times_launched: 0,
  enable_pack_merging: false,
  pack_order: [],
};

// Sort installations and prepend Global
$: sortedInstallations = [
  globalInstallation,
  ...$installations
    .slice()
    .sort((a, b) => {
      if ((a.favorite ? 1 : 0) !== (b.favorite ? 1 : 0)) {
        return (b.favorite ? 1 : 0) - (a.favorite ? 1 : 0);
      }
      const aTime = a.last_used ? new Date(a.last_used).getTime() : 0;
      const bTime = b.last_used ? new Date(b.last_used).getTime() : 0;
      return bTime - aTime;
    })
    .filter((i) => InstallationService.getVersionData(i).loader !== "Vanilla"),
];

let loadedInstallationId: string | null = null;

// Reactively update currentInstallation and packs
$: {
  const inst =
    selectedId === "global"
      ? globalInstallation
      : get(installations).find((i) => i.id === selectedId) || null;
  currentInstallation = inst;
  selectedInstallation.set(inst);

  if (
    currentInstallation &&
    currentInstallation.id !== loadedInstallationId &&
    !loading
  ) {
    loadedInstallationId = currentInstallation.id;
    loadResourcePacks(currentInstallation);
  } else if (!currentInstallation) {
    packs = [];
    loadedInstallationId = null;
  }
}

// Auto-select Global by default
$: {
  if (!selectedId && sortedInstallations.length > 0) {
    const firstInstallation = sortedInstallations[0];
    selectedId = firstInstallation.id;
    currentInstallation = firstInstallation;
    selectedInstallation.set(firstInstallation);
  }
}

// Fuzzy search helper
function fuzzyMatch(text: string, query: string): boolean {
  if (!query) return true;
  const textLower = text.toLowerCase();
  const queryLower = query.toLowerCase();

  if (textLower.includes(queryLower)) return true;

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

// Order packs according to currentPackOrder when merging is enabled
// Add 'id' field for dnd library
$: orderedPacks =
  packMergingEnabled && currentPackOrder.length > 0
    ? (() => {
        const ordered = [];
        const packMap = new Map(
          packs.map((p) => [p.file_name, { ...p, id: p.file_name }]),
        );

        // Add packs in specified order
        for (const fileName of currentPackOrder) {
          const pack = packMap.get(fileName);
          if (pack) {
            ordered.push(pack);
            packMap.delete(fileName);
          }
        }

        // Add any remaining packs not in order
        ordered.push(...Array.from(packMap.values()));

        return ordered;
      })()
    : packs.map((p) => ({ ...p, id: p.file_name }));

// Check scroll fade when orderedPacks changes
$: if (orderedPacks.length > 0 && dragListElement) {
  setTimeout(checkScrollFade, 100);
}

// Filter packs
$: filteredPacks = orderedPacks.filter((pack) => {
  const info = extendedPackInfo[pack.file_name];
  if (searchQuery) {
    const name = info?.name || pack.name || "";
    const desc = info?.description || "";
    const file = pack.file_name;

    return (
      fuzzyMatch(name, searchQuery) ||
      fuzzyMatch(desc, searchQuery) ||
      fuzzyMatch(file, searchQuery)
    );
  }
  return true;
});

// Handle pack changed event
function handlePackChanged() {
  if (currentInstallation) {
    loadResourcePacks(currentInstallation);
  }
}

async function loadResourcePacks(installation: KableInstallation) {
  loading = true;
  error = null;
  try {
    if (installation.id === "global") {
      // Load global resourcepacks from .minecraft/resourcepacks
      packs = await installationsApi.getGlobalResourcePacks();
      packMergingEnabled = false; // Global doesn't support merging
      originalPackOrder = [];
      currentPackOrder = [];
      dragDisabled = true;
    } else {
      // Load resourcepacks for specific installation
      packs = await installationsApi.getResourcePackInfo(installation);

      // Load merging settings
      packMergingEnabled = installation.enable_pack_merging || false;
      originalPackOrder = installation.pack_order || [];
      currentPackOrder = [...originalPackOrder];
      dragDisabled = !packMergingEnabled;

      // Initialize order with current packs if not set
      if (currentPackOrder.length === 0 && packs.length > 0) {
        currentPackOrder = packs.map((p) => p.file_name);
        originalPackOrder = [...currentPackOrder];
      }
    }

    loadedInstallationId = installation.id;
    attemptedExtendedInfo.clear();
    hasOrderChanged = false;
  } catch (e: any) {
    error = e?.message || e || "Failed to load resource packs info";
    packs = [];
    attemptedExtendedInfo.clear();
    loadedInstallationId = null;
  } finally {
    loading = false;
  }
}

async function togglePackMerging() {
  if (!currentInstallation || currentInstallation.id === "global") return;

  packMergingEnabled = !packMergingEnabled;
  dragDisabled = !packMergingEnabled;

  // Initialize order if enabling merging
  if (packMergingEnabled && currentPackOrder.length === 0) {
    currentPackOrder = packs.map((p) => p.file_name);
    originalPackOrder = [...currentPackOrder];
  }

  // Auto-save when toggling
  try {
    await installationsApi.updateResourcePackSettings(
      currentInstallation.id,
      packMergingEnabled,
      currentPackOrder,
    );

    NotificationService.success(
      packMergingEnabled ? "Pack merging enabled" : "Pack merging disabled",
    );

    // Refresh installations to update state
    await installationsApi.refreshInstallations();
    hasOrderChanged = false;
  } catch (e: any) {
    NotificationService.error(`Failed to update settings: ${e}`);
    packMergingEnabled = !packMergingEnabled; // Revert
    dragDisabled = !packMergingEnabled;
  }
}

async function confirmOrder() {
  if (!currentInstallation || currentInstallation.id === "global") return;

  savingOrder = true;
  try {
    await installationsApi.updateResourcePackSettings(
      currentInstallation.id,
      packMergingEnabled,
      currentPackOrder,
    );

    originalPackOrder = [...currentPackOrder];
    hasOrderChanged = false;

    NotificationService.success("Pack order saved and applied");

    // Refresh installations
    await installationsApi.refreshInstallations();
  } catch (e: any) {
    NotificationService.error(`Failed to save order: ${e}`);
  } finally {
    savingOrder = false;
  }
}

function handleDndConsider(e: CustomEvent<DndEvent>) {
  const { items } = e.detail;
  orderedPacks = items as any[];
}

function handleDndFinalize(e: CustomEvent<DndEvent>) {
  const { items } = e.detail;
  orderedPacks = items as any[];
  currentPackOrder = orderedPacks.map((p: any) => p.file_name);

  // Check if order changed
  const orderChanged =
    currentPackOrder.length !== originalPackOrder.length ||
    currentPackOrder.some((name, idx) => name !== originalPackOrder[idx]);

  hasOrderChanged = orderChanged;
}

onMount(() => {
  const inst = get(selectedInstallation);
  if (inst) {
    selectedId = inst.id;
  } else if (sortedInstallations.length > 0) {
    const firstInstallation = sortedInstallations[0];
    selectedId = firstInstallation.id;
    selectedInstallation.set(firstInstallation);
  }
});
</script>

<div class="installation-resourcepacks">
  <div class="main-layout">
    <!-- Left sidebar: Installation carousel -->
    <div class="installation-sidebar">
      <h2>Installations</h2>
      <div
        class="installation-carousel"
        bind:this={installationListContainer}
        on:wheel={handleWheel}
        on:keydown={handleKeydown}
        tabindex="-1"
        role="listbox">
        <div class="carousel-container">
          {#each sortedInstallations as installation, index}
            {@const selectedIndex = sortedInstallations.findIndex(
              (inst) => inst.id === selectedId,
            )}
            {@const carouselEffects = getCarouselScale(
              index,
              selectedIndex >= 0 ? selectedIndex : 0,
              sortedInstallations.length,
            )}
            {#if carouselEffects.visible}
              <div
                class="installation-item"
                class:selected={installation.id === selectedId}
                data-installation-id={installation.id}
                style="
              background: linear-gradient(135deg, {loaderColors[
                  installation.id
                ]}22 0%, {loaderColors[installation.id]}08 40%); 
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
                on:keydown={(e) =>
                  e.key === "Enter" && selectInstallation(installation)}
                tabindex="0"
                role="button">
                <div class="installation-icon">
                  <Icon name={loaderIcons[installation.id]} size="md" />
                </div>
                <div class="installation-meta">
                  <div class="installation-name">{installation.name}</div>
                  <div class="installation-details">
                    <span class="installation-version"
                      >{InstallationService.getVersionData(installation)
                        .version_id}</span>
                  </div>
                </div>
              </div>
            {/if}
          {/each}
        </div>
      </div>
    </div>

    <!-- Right content: Search and packs -->
    <div class="packs-section">
      <div class="packs-header">
        <div class="search-controls">
          <div class="search-input-wrapper">
            <span class="search-icon">🔍</span>
            <input
              type="text"
              placeholder="Search resource packs (fuzzy search enabled)..."
              bind:value={searchQuery}
              class="search-input" />
            {#if searchQuery}
              <button
                class="clear-btn"
                on:click={() => (searchQuery = "")}
                title="Clear search">✕</button>
            {/if}
          </div>
        </div>

        {#if currentInstallation}
          <div class="packs-title-section">
            <div class="title-and-toggle">
              <h3>Resource Packs for {currentInstallation.name}</h3>

              {#if currentInstallation.id !== "global"}
                <button
                  class="merge-toggle"
                  class:enabled={packMergingEnabled}
                  on:click={togglePackMerging}
                  title={packMergingEnabled
                    ? "Pack merging enabled - packs will be merged into one"
                    : "Pack merging disabled - packs loaded individually"}>
                  <Icon
                    name={packMergingEnabled ? "layers" : "package"}
                    size="sm" />
                  <span
                    >{packMergingEnabled
                      ? "Merging Enabled"
                      : "Individual Packs"}</span>
                </button>
              {/if}
            </div>

            {#if packs.length > 0}
              <div class="packs-count-badge">
                {#if searchQuery}
                  <span class="filtered-count">{filteredPacks.length}</span>
                  <span class="count-separator">of</span>
                  <span class="total-count">{packs.length}</span>
                  <span class="count-label">packs</span>
                {:else}
                  <span class="total-count">{packs.length}</span>
                  <span class="count-label"
                    >{packs.length === 1 ? "pack" : "packs"}</span>
                {/if}
              </div>
            {/if}
          </div>

          {#if packMergingEnabled && hasOrderChanged}
            <div class="order-actions">
              <div class="order-hint">
                <Icon name="info" size="sm" />
                <span>Drag packs to reorder (top = highest priority)</span>
              </div>
              <button
                class="confirm-order-btn"
                on:click={confirmOrder}
                disabled={savingOrder}>
                {#if savingOrder}
                  <Icon name="refresh" size="sm" className="spin" />
                  <span>Saving...</span>
                {:else}
                  <Icon name="check" size="sm" />
                  <span>Apply Order</span>
                {/if}
              </button>
            </div>
          {/if}
        {/if}
      </div>

      <div class="packs-content">
        {#if currentInstallation}
          {#if loading}
            <div class="loading-state">
              <Icon name="refresh" size="md" className="spin" />
              <span>Loading resource packs...</span>
            </div>
          {:else if error}
            <div class="error-message">
              <Icon name="alert" size="sm" />
              {error}
            </div>
          {:else if packs.length > 0}
            {#if packMergingEnabled && !searchQuery}
              <div class="packs-list-container">
                <div
                  class="packs-list-draggable"
                  bind:this={dragListElement}
                  on:scroll={checkScrollFade}
                  use:dndzone={{
                    items: orderedPacks,
                    flipDurationMs,
                    dragDisabled: false,
                    dropTargetStyle: {},
                  }}
                  on:consider={handleDndConsider}
                  on:finalize={handleDndFinalize}>
                  {#each orderedPacks as pack (pack.id)}
                    <div class="drag-item-wrapper">
                      <div class="drag-handle">
                        <Icon name="menu" size="md" />
                      </div>
                      <InstalledResourcePackCard
                        {pack}
                        installation={currentInstallation}
                        extendedInfo={extendedPackInfo[pack.file_name]}
                        onpackchanged={handlePackChanged} />
                    </div>
                  {/each}
                </div>
                {#if showScrollFade}
                  <div class="scroll-fade-indicator">
                    <Icon name="chevron-down" size="sm" />
                    <span>Scroll for more</span>
                  </div>
                {/if}
              </div>
            {:else}
              <div class="packs-card-grid">
                {#each filteredPacks as pack (pack.file_name)}
                  <InstalledResourcePackCard
                    {pack}
                    installation={currentInstallation}
                    extendedInfo={extendedPackInfo[pack.file_name]}
                    onpackchanged={handlePackChanged} />
                {/each}
              </div>
            {/if}
          {:else}
            <div class="empty-state">
              <Icon name="image" size="xl" />
              <span>No resource packs installed for this installation.</span>
            </div>
          {/if}
        {:else}
          <div class="empty-state">
            <Icon name="package" size="xl" />
            <span>Select an installation to view resource packs.</span>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;
.installation-resourcepacks {
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
  border: 1px solid #{"color-mix(in srgb, var(--primary), 8%, transparent)"};
  box-shadow: 0 2px 12px
    #{"color-mix(in srgb, var(--dark-900), 6%, transparent)"};
  overflow: hidden;
}

.installation-sidebar {
  width: 320px;
  min-width: 320px;
  border-right: 1px solid color-mix(in srgb, var(--primary), 8%, transparent);
  display: flex;
  flex-direction: column;

  h2 {
    margin: 0;
    padding: 1.5rem 1.5rem 1rem 1.5rem;
    background: linear-gradient(
      135deg,
      var(--primary) 0%,
      var(--secondary) 100%
    );
    background-clip: text;
    -webkit-background-clip: text;
    color: transparent;
    font-weight: 700;
    font-size: 1.4em;
    border-bottom: 1px solid
      #{"color-mix(in srgb, var(--primary), 8%, transparent)"};
  }
}

.installation-carousel {
  flex: 1;
  overflow: hidden;
  padding: 0.5rem;
  position: relative;
  display: flex;
  align-items: center;

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
  pointer-events: none;
}

.installation-item {
  padding: 1rem 1.2rem;
  margin: 0;
  border-radius: 0.6rem;
  cursor: pointer;
  border: 1px solid transparent;
  position: absolute;
  top: 35%;
  left: 50%;
  transform-origin: center center;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  width: 100%;
  pointer-events: auto;

  transition: all 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94);

  &:hover {
    border-color: var(
      --loader-color,
      #{"color-mix(in srgb, var(--primary), 15%, transparent)"}
    );
    box-shadow: 0 2px 8px
      #{"color-mix(in srgb, var(--loader-color, var(--primary)), 10%, transparent)"};
  }

  &.selected {
    border-color: var(--loader-color, var(--primary));
    box-shadow:
      0 4px 16px
        #{"color-mix(in srgb, var(--loader-color, var(--primary)), 15%, transparent)"},
      inset 0 1px 0 rgba(255, 255, 255, 0.1);
    z-index: 10;

    border: 2px solid var(--green-800);

    &:hover {
      box-shadow:
        0 6px 20px
          #{"color-mix(in srgb, var(--loader-color, var(--primary)), 20%, transparent)"},
        0 0 0 3px #{"color-mix(in srgb, var(--green-800), 30%, transparent)"},
        inset 0 1px 0 #{"color-mix(in srgb, #fff, 15%, transparent)"};
    }

    &::before {
      content: "";
      position: absolute;
      left: -0.4rem;
      top: 50%;
      transform: translateY(-50%);
      width: 4px;
      height: 60%;
      background: linear-gradient(
        to bottom,
        var(--green-700),
        var(--green-900)
      );
      box-shadow: 0 0 8px
        #{"color-mix(in srgb, var(--green-800), 40%, transparent)"};
    }
  }

  &:focus {
    outline: none;
    box-shadow: 0 0 0 2px
      #{"color-mix(in srgb, var(--loader-color, var(--primary)), 30%, transparent)"};
  }
}

.installation-icon {
  width: calc(48px * var(--carousel-scale, 1));
  height: calc(48px * var(--carousel-scale, 1));
  border-radius: 0.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--container);
  color: var(--loader-color, var(--primary));
  box-shadow: 0 2px 6px
    #{"color-mix(in srgb, var(--dark-900), 8%, transparent)"};
  flex-shrink: 0;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);

  .installation-item.selected & {
    background: linear-gradient(
      135deg,
      var(--loader-color, var(--primary)) 0%,
      #{"color-mix(in srgb, var(--loader-color, var(--secondary)), 80%, transparent)"}
        100%
    );
    color: white;
    box-shadow: 0 3px 12px
      #{"color-mix(in srgb, var(--loader-color, var(--primary)), 30%, transparent)"};
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

  color: var(--text);

  .installation-item.selected & {
    color: var(--loader-color, var(--primary));
    font-weight: 700;
    text-shadow: 0 0 8px
      #{"color-mix(in srgb, var(--loader-color, var(--primary)), 30%, transparent)"};
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
    background: #{"color-mix(in srgb, var(--loader-color, var(--tertiary)), 15%, transparent)"};
    color: var(--loader-color, var(--tertiary));
  }
}

.packs-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.packs-header {
  background: linear-gradient(
    135deg,
    var(--card) 0%,
    color-mix(in srgb, var(--primary), 2%, transparent) 100%
  );
  border-bottom: 1px solid color-mix(in srgb, var(--primary), 8%, transparent);
  padding: 1.2rem 1.5rem;

  .packs-title-section {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 1rem 0 0.5rem 0;

    .title-and-toggle {
      display: flex;
      align-items: center;
      gap: 1rem;
      flex: 1;
    }

    h3 {
      margin: 0;
      color: var(--text);
      font-weight: 500;
      font-size: 1.1em;
    }
  }

  .merge-toggle {
    display: flex;
    align-items: center;
    gap: 0.4em;
    padding: 0.5em 1em;
    border-radius: 0.5rem;
    border: 1px solid var(--dark-600);
    background: var(--input);
    color: var(--text-secondary);
    font-size: 0.85em;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;

    &:hover {
      border-color: var(--primary);
      background: var(--card);
      color: var(--text);
    }

    &.enabled {
      border-color: var(--green-700);
      background: linear-gradient(
        135deg,
        #{"color-mix(in srgb, var(--green-700), 12%, transparent)"} 0%,
        #{"color-mix(in srgb, var(--green-800), 8%, transparent)"} 100%
      );
      color: var(--green-600);
      box-shadow: 0 1px 6px
        #{"color-mix(in srgb, var(--green-700), 15%, transparent)"};

      &:hover {
        border-color: var(--green-600);
        box-shadow: 0 2px 10px
          #{"color-mix(in srgb, var(--green-700), 20%, transparent)"};
      }
    }
  }

  .order-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 0.75rem 0 0.5rem 0;
    padding: 0.75rem 1rem;
    background: linear-gradient(
      135deg,
      #{"color-mix(in srgb, var(--primary), 6%, transparent)"} 0%,
      #{"color-mix(in srgb, var(--secondary), 3%, transparent)"} 100%
    );
    border: 1px solid #{"color-mix(in srgb, var(--primary), 12%, transparent)"};
    border-radius: 0.5rem;

    .order-hint {
      display: flex;
      align-items: center;
      gap: 0.4em;
      color: var(--text-secondary);
      font-size: 0.85em;
    }
  }

  .confirm-order-btn {
    display: flex;
    align-items: center;
    gap: 0.4em;
    padding: 0.5em 1.2em;
    border-radius: 0.5rem;
    border: 1px solid var(--green-700);
    background: linear-gradient(
      135deg,
      var(--green-700) 0%,
      var(--green-800) 100%
    );
    color: white;
    font-size: 0.9em;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    box-shadow: 0 2px 8px
      #{"color-mix(in srgb, var(--green-700), 25%, transparent)"};

    &:hover:not(:disabled) {
      transform: translateY(-1px);
      box-shadow: 0 4px 12px
        #{"color-mix(in srgb, var(--green-700), 35%, transparent)"};
    }

    &:active:not(:disabled) {
      transform: translateY(0);
    }

    &:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }
  }
}

.packs-count-badge {
  display: flex;
  align-items: center;
  gap: 0.3em;
  background: linear-gradient(
    135deg,
    #{"color-mix(in srgb, var(--primary), 8%, transparent)"} 0%,
    #{"color-mix(in srgb, var(--secondary), 4%, transparent)"} 100%
  );
  border: 1px solid #{"color-mix(in srgb, var(--primary), 15%, transparent)"};
  border-radius: 1rem;
  padding: 0.4em 0.8em;
  font-size: 0.85em;
  font-weight: 500;
  backdrop-filter: blur(6px);
  -webkit-backdrop-filter: blur(6px);
  box-shadow: 0 1px 4px
    #{"color-mix(in srgb, var(--dark-900), 6%, transparent)"};

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

.packs-content {
  flex: 1;
  padding: 1.5rem;
  background: var(--container);
  overflow-y: auto;
  overflow-x: hidden;

  &::-webkit-scrollbar {
    width: 8px;
  }

  &::-webkit-scrollbar-track {
    background: var(--bg-tertiary);
    border-radius: 4px;
  }

  &::-webkit-scrollbar-thumb {
    background: var(--dark-600);
    border-radius: 4px;

    &:hover {
      background: var(--dark-500);
    }
  }
}

.packs-card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 0.5rem;
  padding: 0.5rem;

  @media (max-width: 900px) {
    grid-template-columns: 1fr;
  }

  @media (min-width: 1400px) {
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  }
}

.packs-list-container {
  position: relative;
}

.packs-list-draggable {
  display: flex;
  flex-direction: column;
  padding: 0 0.5rem;
  max-height: calc(100vh - 400px);
  overflow-y: auto;
  overflow-x: hidden;
  scrollbar-width: thin;
  scrollbar-color: var(--primary) transparent;

  &::-webkit-scrollbar {
    width: 8px;
  }

  &::-webkit-scrollbar-track {
    background: transparent;
  }

  &::-webkit-scrollbar-thumb {
    background: #{"color-mix(in srgb, var(--primary), 60%, transparent)"};
    border-radius: 4px;

    &:hover {
      background: #{"color-mix(in srgb, var(--primary), 80%, transparent)"};
    }
  }
}

.scroll-fade-indicator {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 60px;
  background: linear-gradient(
    to bottom,
    transparent 0%,
    #{"color-mix(in srgb, var(--container), 95%, transparent)"} 50%,
    var(--container) 100%
  );
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-end;
  padding-bottom: 0.75rem;
  gap: 0.25rem;
  color: var(--text-secondary);
  font-size: 0.85em;
  pointer-events: none;
  animation: pulse 2s ease-in-out infinite;

  @keyframes pulse {
    0%,
    100% {
      opacity: 0.6;
    }
    50% {
      opacity: 1;
    }
  }
}

.drag-item-wrapper {
  display: flex;
  align-items: stretch;
  gap: 0.5rem;
  background: transparent;
  border-radius: 0.5rem;
  padding: 0.25rem;
  transition: all 0.2s ease;
  user-select: none;
  margin-bottom: 0.35rem;

  &:hover {
    background: color-mix(in srgb, var(--primary), 3%, transparent);
  }
}

.drag-handle {
  flex-shrink: 0;
  width: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  cursor: grab;
  transition: all 0.2s ease;
  background: var(--bg-secondary);
  border-radius: 0.4rem;
  border: 1px solid color-mix(in srgb, var(--primary), 8%, transparent);
  margin-right: 0.25rem;

  &:hover {
    color: var(--primary);
    background: var(--bg-tertiary);
    border-color: color-mix(in srgb, var(--primary), 20%, transparent);
  }

  &:active {
    cursor: grabbing;
    background: var(--bg-tertiary);
    border-color: var(--primary);
  }
}

:global(.drag-item-wrapper .installed-pack-card) {
  flex: 1;
  min-width: 0;
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

  .packs-section {
    border-left: none;
    border-top: 1px solid color-mix(in srgb, var(--primary), 8%, transparent);
  }
}

@media (max-width: 600px) {
  .installation-resourcepacks {
    padding: 0.5rem;
    height: auto;
  }
}
</style>
