<!-- @component
InstallationShaders - Manage shader packs for installations

Features:
- Installation carousel selector
- Fuzzy search for shaders
- Enable/disable/remove shaders
- Visit shader pages
-->
<script lang="ts">
import { onMount } from "svelte";
import { get } from "svelte/store";
import { Icon, NotificationService, InstallationService } from "$lib";
import type { KableInstallation } from "$lib";
import { installations, selectedInstallation } from "$lib/stores";
import InstalledShaderPackCard from "./InstalledShaderPackCard.svelte";
import * as installationsApi from "$lib/api/installations";

export let selectedId: string = "";
let currentInstallation: KableInstallation | null = null;
let shaders: any[] = [];
let extendedShaderInfo: Record<string, any> = {};
let attemptedExtendedInfo = new Set<string>();
let loading = false;
let error: string | null = null;
let searchQuery = "";
let installationListContainer: HTMLDivElement | null = null;

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
    const nextIndex = (selectedIndex + 1) % sortedInstallations.length;
    selectInstallation(sortedInstallations[nextIndex]);
  } else if (delta < 0) {
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

// Reactively update currentInstallation and shaders
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
    loadShaderPacks(currentInstallation);
  } else if (!currentInstallation) {
    shaders = [];
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

// Filter shaders
$: filteredShaders = shaders.filter((shader) => {
  const info = extendedShaderInfo[shader.file_name];
  if (searchQuery) {
    const name = info?.name || shader.name || "";
    const desc = info?.description || "";
    const file = shader.file_name;

    return (
      fuzzyMatch(name, searchQuery) ||
      fuzzyMatch(desc, searchQuery) ||
      fuzzyMatch(file, searchQuery)
    );
  }
  return true;
});

// Handle shader changed event
function handleShaderChanged() {
  if (currentInstallation) {
    loadShaderPacks(currentInstallation);
  }
}

async function loadShaderPacks(installation: KableInstallation) {
  loading = true;
  error = null;
  try {
    if (installation.id === "global") {
      // Load global shaderpacks from .minecraft/shaderpacks
      shaders = await installationsApi.getGlobalShaderPacks();
    } else {
      // Load shaderpacks for specific installation
      shaders = await installationsApi.getShaderPackInfo(installation);
    }
    loadedInstallationId = installation.id;
    attemptedExtendedInfo.clear();
  } catch (e: any) {
    error = e?.message || e || "Failed to load shader packs info";
    shaders = [];
    attemptedExtendedInfo.clear();
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
    const firstInstallation = sortedInstallations[0];
    selectedId = firstInstallation.id;
    selectedInstallation.set(firstInstallation);
  }
});
</script>

<div class="installation-shaders">
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
        role="listbox"
      >
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
                role="button"
              >
                <div class="installation-icon">
                  <Icon name={loaderIcons[installation.id]} size="md" />
                </div>
                <div class="installation-meta">
                  <div class="installation-name">{installation.name}</div>
                  <div class="installation-details">
                    <span class="installation-version"
                      >{InstallationService.getVersionData(installation)
                        .version_id}</span
                    >
                  </div>
                </div>
              </div>
            {/if}
          {/each}
        </div>
      </div>
    </div>

    <!-- Right content: Search and shaders -->
    <div class="shaders-section">
      <div class="shaders-header">
        <div class="search-controls">
          <div class="search-input-wrapper">
            <span class="search-icon">🔍</span>
            <input
              type="text"
              placeholder="Search shader packs (fuzzy search enabled)..."
              bind:value={searchQuery}
              class="search-input"
            />
            {#if searchQuery}
              <button
                class="clear-btn"
                on:click={() => (searchQuery = "")}
                title="Clear search">✕</button
              >
            {/if}
          </div>
        </div>

        {#if currentInstallation}
          <div class="shaders-title-section">
            <div class="title-and-info">
              <h3>Shader Packs for {currentInstallation.name}</h3>
            </div>

            {#if shaders.length > 0}
              <div class="shaders-count-badge">
                {#if searchQuery}
                  <span class="filtered-count">{filteredShaders.length}</span>
                  <span class="count-separator">of</span>
                  <span class="total-count">{shaders.length}</span>
                  <span class="count-label">shaders</span>
                {:else}
                  <span class="total-count">{shaders.length}</span>
                  <span class="count-label"
                    >{shaders.length === 1 ? "shader" : "shaders"}</span
                  >
                {/if}
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <div class="shaders-content">
        {#if currentInstallation}
          {#if loading}
            <div class="loading-state">
              <Icon name="refresh" size="md" className="spin" />
              <span>Loading shader packs...</span>
            </div>
          {:else if error}
            <div class="error-message">
              <Icon name="alert" size="sm" />
              {error}
            </div>
          {:else if shaders.length > 0}
            <div class="shaders-card-grid">
              {#each filteredShaders as shader (shader.file_name)}
                <InstalledShaderPackCard
                  {shader}
                  installation={currentInstallation}
                  extendedInfo={extendedShaderInfo[shader.file_name]}
                  onshaderchanged={handleShaderChanged}
                />
              {/each}
            </div>
          {:else}
            <div class="empty-state">
              <Icon name="image" size="xl" />
              <span>No shader packs installed for this installation.</span>
            </div>
          {/if}
        {:else}
          <div class="empty-state">
            <Icon name="package" size="xl" />
            <span>Select an installation to view shader packs.</span>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;
.installation-shaders {
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

.shaders-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.shaders-header {
  background: linear-gradient(
    135deg,
    var(--card) 0%,
    color-mix(in srgb, var(--primary), 2%, transparent) 100%
  );
  border-bottom: 1px solid color-mix(in srgb, var(--primary), 8%, transparent);
  padding: 1.2rem 1.5rem;

  .shaders-title-section {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 1rem 0 0.5rem 0;

    .title-and-info {
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
}

.shaders-count-badge {
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

.shaders-content {
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

.shaders-card-grid {
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

  .shaders-section {
    border-left: none;
    border-top: 1px solid color-mix(in srgb, var(--primary), 8%, transparent);
  }
}

@media (max-width: 600px) {
  .installation-shaders {
    padding: 0.5rem;
    height: auto;
  }
}
</style>
