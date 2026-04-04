<!-- @component
InstallationMods - Manages and displays mods for a specific installation

Shows installed mods with options to enable/disable, delete, and view mod information.
Includes installation carousel for quick switching and semantic search filtering.

@example
```svelte
◄InstallationMods /►
```
-->
<script lang="ts">
import type { KableInstallation, ModInfoKind, ModJarInfo } from "$lib";
import {
  extendedModInfo,
  Icon,
  installations,
  InstallationService,
  ModsService,
  ProviderKind,
  selectedInstallation,
} from "$lib";
import * as installationsApi from "$lib/api/installations";
import * as modsApi from "$lib/api/mods";
import { openUrl } from "$lib/api/system";
import { onMount } from "svelte";
import { get } from "svelte/store";
import InstalledModCard from "./InstalledModCard.svelte";

let currentInstallation: KableInstallation | null = null;
let selectedId: string = "";
let mods: ModJarInfo[] = [];
let loading = false;
let error: string | null = null;
let installationListContainer: HTMLElement;

// Track which mods we've attempted to fetch extended info for to prevent infinite loops
let attemptedExtendedInfo = new Set<string>();

// Semantic search/filter state
let searchQuery = "";

// Sorting/grouping controls for installed mods.
type ModSortMode = "name_asc" | "name_desc" | "date_desc" | "date_asc";
type DisabledGroupMode = "end" | "start" | "none";
let modSortMode: ModSortMode = "name_asc"; // Default to alphabetical
let disabledGroupMode: DisabledGroupMode = "end";
let sourceViewEnabled = false;

// Cached mod metadata by jar filename (used for date sorting + stable project lookup).
let modMetadataMap = new Map<string, modsApi.ModMetadata | null>();
let modpackSources: modsApi.ModpackSourceRecord[] = [];
let modpackProjectMap = new Map<string, ModInfoKind>();
const nameCollator = new Intl.Collator(undefined, {
  sensitivity: "base",
  numeric: true,
});

function metadataKey(fileName: string): string {
  return `${selectedId}:${fileName}`;
}

// Track mods with available updates
let modsWithUpdates = new Map<
  string,
  { mod: ModJarInfo; latestVersion: string; versionId: string }
>();
let updatingAll = false;
let loadingModpackSources = false;

// Installation carousel logic
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
    const selectedIndex = sortedInstallations.findIndex(
      (inst) => inst.id === selectedId,
    );
    let newIndex = selectedIndex;

    if (scrollOffset > 0) {
      // Scroll down - select next installation (with wrapping)
      newIndex = (selectedIndex + 1) % sortedInstallations.length;
    } else if (scrollOffset < 0) {
      // Scroll up - select previous installation (with wrapping)
      newIndex =
        (selectedIndex - 1 + sortedInstallations.length) %
        sortedInstallations.length;
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
  if (event.key === "ArrowUp" || event.key === "ArrowDown") {
    event.preventDefault();

    const selectedIndex = sortedInstallations.findIndex(
      (inst) => inst.id === selectedId,
    );
    let newIndex = selectedIndex;

    if (event.key === "ArrowDown") {
      // Select next installation (with wrapping)
      newIndex = (selectedIndex + 1) % sortedInstallations.length;
    } else if (event.key === "ArrowUp") {
      // Select previous installation (with wrapping)
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

// Calculate carousel scaling and positioning for centered layout with wrapping
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
  // Calculate wrapped distance (shortest path around the carousel)
  const directDistance = Math.abs(currentIndex - selectedIndex);
  const wrapDistance = totalItems - directDistance;
  const distance = Math.min(directDistance, wrapDistance);

  // Determine relative position considering wrapping
  let relativePosition = currentIndex - selectedIndex;
  if (Math.abs(relativePosition) > totalItems / 2) {
    // Use wrapping path
    relativePosition =
      relativePosition > 0
        ? relativePosition - totalItems
        : relativePosition + totalItems;
  }

  // Only show items within a certain distance from the selected item
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

  // More dramatic scaling for centered layout, but adapt based on how well items fit
  // Compute fit ratio using the container height and total required height
  const containerHeight = installationListContainer
    ? installationListContainer.clientHeight
    : totalItems * 120;
  const baseItemHeight = 120;
  const fitRatio = Math.min(
    1,
    containerHeight / Math.max(1, totalItems * baseItemHeight),
  ); // 0..1

  // When items fit well (fitRatio ~ 1) we want smaller spacing and less aggressive scale shrink
  const spacing = 20 * (1 - fitRatio) + 8; // ranges ~8..28

  const baseScaleFactors = [1.0, 0.85, 0.7, 0.55, 0.4];
  // scaleReduction closer to 0 means less shrink (when fitRatio=1), when fitRatio=0 keep original
  const scaleReduction = 1 - fitRatio * 0.3; // between 0.7 and 1
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

  // Calculate vertical offset to center the selected item
  const itemHeight = baseItemHeight; // base height used for spacing calc
  // Compress spacing for near neighbors so selected item appears closer
  const distanceNorm = Math.min(distance, 4) / 4; // 0..1
  // Use a stronger compression floor so nearest items sit noticeably closer.
  // compressionFloor controls how much spacing nearest neighbors keep (0.0..1.0)
  const compressionFloor = 0.5; // previously ~0.6, lower -> tighter grouping
  const compression = compressionFloor + (1 - compressionFloor) * distanceNorm; // ranges compressionFloor..1.0
  const translateY =
    relativePosition * (itemHeight * scale + spacing * compression);

  // Z-index for layering (selected item on top)
  const zIndex = 100 - distance;

  return { scale, opacity, fontSize, translateY, zIndex, visible: true };
}

//  Loader styling helpers (inspired by InstallationsList)
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

//  Sort installations by favorite and date (same as InstallationsList)
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
  .filter((i) => InstallationService.getVersionData(i).loader !== "Vanilla");

// Track which installation we've loaded mods for to prevent infinite reactive loops
let loadedInstallationId: string | null = null;

// Reactively update currentInstallation and mods when selectedId changes
$: {
  const inst = get(installations).find((i) => i.id === selectedId) || null;
  currentInstallation = inst;
  selectedInstallation.set(inst);

  // Only load mods if we haven't already loaded for this installation and we're not currently loading
  if (
    currentInstallation &&
    currentInstallation.id !== loadedInstallationId &&
    !loading
  ) {
    loadedInstallationId = currentInstallation.id;
    // Clear attempted info when switching installations to allow refetch
    attemptedExtendedInfo.clear();
    // Clear update tracking when switching installations
    modsWithUpdates.clear();
    modsWithUpdates = modsWithUpdates; // Trigger reactivity
    loadMods(currentInstallation);
  } else if (!currentInstallation) {
    mods = [];
    loadedInstallationId = null;
    // Clear attempted info when no installation
    attemptedExtendedInfo.clear();
    // Clear update tracking when no installation
    modsWithUpdates.clear();
    modsWithUpdates = modsWithUpdates; // Trigger reactivity
    modMetadataMap = new Map();
    modpackSources = [];
    modpackProjectMap = new Map();
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

// Only trigger backend fetch for extended mod info when the actual mods list changes (not on sort/group/filter changes)
let lastFetchedModsKey = "";
$: {
  if (mods && mods.length > 0) {
    // Create a key based on the sorted file names to detect real changes
    const modsKey = mods
      .map((m) => m.file_name)
      .sort()
      .join("|");
    if (modsKey !== lastFetchedModsKey) {
      lastFetchedModsKey = modsKey;
      // Only fetch for mods that are missing in the store (undefined means not attempted, null means failed)
      const missing = mods.filter((mod) => {
        return (
          $extendedModInfo[mod.file_name] === undefined &&
          !attemptedExtendedInfo.has(mod.file_name)
        );
      });
      if (missing.length > 0) {
        // Mark these mods as attempted to prevent infinite loops
        missing.forEach((mod) => attemptedExtendedInfo.add(mod.file_name));
        Promise.all(missing.map((mod) => ModsService.getExtendedModInfo(mod)));
      }
    }
  } else {
    lastFetchedModsKey = "";
  }
}

// Fetch metadata lazily for mods that are missing it.
$: if (selectedId && mods && mods.length > 0) {
  const installationForMetadata =
    get(installations).find((i) => i.id === selectedId) || null;
  if (installationForMetadata) {
    const missingMeta = mods.filter(
      (mod) => !modMetadataMap.has(metadataKey(mod.file_name)),
    );
    if (missingMeta.length > 0) {
      Promise.all(
        missingMeta.map(async (mod) => {
          try {
            const meta = await modsApi.getModMetadata(
              installationForMetadata,
              mod.file_name,
            );
            return [metadataKey(mod.file_name), meta] as const;
          } catch {
            return [metadataKey(mod.file_name), null] as const;
          }
        }),
      ).then((pairs) => {
        if (pairs.length === 0) return;
        const next = new Map(modMetadataMap);
        for (const [fileName, meta] of pairs) {
          next.set(fileName, meta);
        }
        modMetadataMap = next;
      });
    }
  }
}

//  Fuzzy search helper function
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

//  Semantic search/filter logic with fuzzy matching
$: filteredMods = mods.filter((mod) => {
  const info = $extendedModInfo[mod.file_name];
  if (searchQuery) {
    const name = info?.mod_jar_info?.mod_name || mod.mod_name || "";
    const desc = info?.description || "";
    const file = mod.file_name;
    return (
      fuzzyMatch(name, searchQuery) ||
      fuzzyMatch(desc, searchQuery) ||
      fuzzyMatch(file, searchQuery)
    );
  }
  return true;
});

function getModDisplayName(mod: ModJarInfo): string {
  return (mod.mod_name || mod.file_name.replace(/\.jar$/i, "")).trim();
}

function getModDate(mod: ModJarInfo): number {
  const metadata = modMetadataMap.get(metadataKey(mod.file_name));
  if (!metadata?.download_time) return 0;
  const ts = Date.parse(metadata.download_time);
  return Number.isNaN(ts) ? 0 : ts;
}

function sortMods(a: ModJarInfo, b: ModJarInfo): number {
  const nameA = getModDisplayName(a);
  const nameB = getModDisplayName(b);

  switch (modSortMode) {
    case "name_asc":
      return nameCollator.compare(nameA, nameB);
    case "name_desc":
      return nameCollator.compare(nameB, nameA);
    case "date_asc": {
      const da = getModDate(a);
      const db = getModDate(b);
      if (da !== db) return da - db;
      return nameCollator.compare(nameA, nameB);
    }
    case "date_desc":
    default: {
      const da = getModDate(a);
      const db = getModDate(b);
      if (da !== db) return db - da;
      return nameCollator.compare(nameA, nameB);
    }
  }
}

$: sortNameStamp = mods.map((mod) => mod.mod_name || mod.file_name).join("|");

$: sortedFilteredMods = (() => {
  sortNameStamp;
  const currentSortMode = modSortMode;
  const currentDisabledGrouping = disabledGroupMode;

  const list = filteredMods.slice().sort(sortMods);
  if (currentSortMode === "name_asc") {
    // no-op; establishes explicit reactive dependency for Svelte
  }

  if (currentDisabledGrouping === "none") {
    return list;
  }

  const enabled = list.filter((mod) => !mod.disabled);
  const disabled = list.filter((mod) => mod.disabled);
  return currentDisabledGrouping === "start"
    ? [...disabled, ...enabled]
    : [...enabled, ...disabled];
})();

$: managedProjectIds = new Set(
  modpackSources.flatMap((source) => source.managed_project_ids),
);

$: standaloneFilteredMods = sortedFilteredMods.filter((mod) => {
  const metadata = modMetadataMap.get(metadataKey(mod.file_name));
  const projectId = metadata?.project_id;
  if (!projectId) return true;
  return !managedProjectIds.has(projectId);
});

$: if (currentInstallation && modpackSources.length > 0 && sourceViewEnabled) {
  const missingProjectIds = modpackSources
    .filter((source) => source.provider === ProviderKind.Modrinth)
    .map((source) => source.mod_id)
    .filter((projectId) => !modpackProjectMap.has(projectId));

  if (missingProjectIds.length > 0) {
    const uniqueProjectIds = Array.from(new Set(missingProjectIds));
    modsApi
      .getProjects(ProviderKind.Modrinth, uniqueProjectIds)
      .then((projects) => {
        const next = new Map(modpackProjectMap);
        for (const project of projects) {
          if ("Modrinth" in project) {
            next.set(project.Modrinth.project_id, project);
          }
        }
        modpackProjectMap = next;
      })
      .catch((err) => {
        console.warn("Failed to load modpack project cards:", err);
      });
  }
}

function getModpackCardTitle(source: modsApi.ModpackSourceRecord): string {
  const project = modpackProjectMap.get(source.mod_id);
  if (project && "Modrinth" in project) {
    return project.Modrinth.title || source.modpack_name || source.mod_id;
  }
  return source.modpack_name || source.mod_id;
}

function getModpackCardDescription(
  source: modsApi.ModpackSourceRecord,
): string {
  const project = modpackProjectMap.get(source.mod_id);
  if (project && "Modrinth" in project) {
    return project.Modrinth.description || "";
  }
  return "";
}

function getModpackCardIcon(
  source: modsApi.ModpackSourceRecord,
): string | null {
  const project = modpackProjectMap.get(source.mod_id);
  if (project && "Modrinth" in project) {
    return project.Modrinth.icon_url || null;
  }
  return null;
}

function getModpackCardUrl(source: modsApi.ModpackSourceRecord): string | null {
  if (source.provider === ProviderKind.Modrinth) {
    return `https://modrinth.com/modpack/${source.mod_id}`;
  }
  return null;
}

async function openModpackSource(source: modsApi.ModpackSourceRecord) {
  const url = getModpackCardUrl(source);
  if (!url) return;
  try {
    await openUrl(url);
  } catch (err) {
    console.error("Failed to open modpack page:", err);
  }
}

// Handle mod changed event (toggle, delete, etc.)
function handleModChanged() {
  // Reload mods to reflect changes
  if (currentInstallation) {
    loadMods(currentInstallation, { silent: true });
  }
}

async function loadModpackSources(installation: KableInstallation) {
  loadingModpackSources = true;
  try {
    modpackSources = await modsApi.getModpackSourceRecords(installation);
    const sourceIds = new Set(modpackSources.map((source) => source.mod_id));
    modpackProjectMap = new Map(
      Array.from(modpackProjectMap.entries()).filter(([id]) =>
        sourceIds.has(id),
      ),
    );
  } catch (err) {
    console.warn("Failed to load modpack source records:", err);
    modpackSources = [];
    modpackProjectMap = new Map();
  } finally {
    loadingModpackSources = false;
  }
}

// Handle update reports from individual mod cards
function handleUpdateReport(event: {
  fileName: string;
  hasUpdate: boolean;
  latestVersion?: string;
  versionId?: string;
  mod?: ModJarInfo;
}) {
  if (event.hasUpdate && event.latestVersion && event.versionId && event.mod) {
    modsWithUpdates.set(event.fileName, {
      mod: event.mod,
      latestVersion: event.latestVersion,
      versionId: event.versionId,
    });
  } else {
    modsWithUpdates.delete(event.fileName);
  }
  modsWithUpdates = modsWithUpdates; // Trigger reactivity
}

// Update all mods that have updates available
async function handleUpdateAll() {
  if (updatingAll || modsWithUpdates.size === 0 || !currentInstallation) return;

  updatingAll = true;
  const updates = Array.from(modsWithUpdates.values());
  let successCount = 0;
  let failCount = 0;

  for (const { mod, versionId } of updates) {
    try {
      const extendedInfo = $extendedModInfo[mod.file_name];
      if (!extendedInfo) continue;

      let projectId: string | null = null;

      // Prefer metadata because it is the most reliable identifier source.
      try {
        const metadata = await modsApi.getModMetadata(
          currentInstallation,
          mod.file_name,
        );
        projectId = metadata.project_id;
      } catch (e) {
        if (extendedInfo.page_uri) {
          const match = extendedInfo.page_uri.match(/\/mod\/([\w-]+)/);
          if (match) projectId = match[1];
        }
      }

      if (!projectId) {
        console.warn(`Could not get project ID for ${mod.file_name}`);
        failCount++;
        continue;
      }

      await modsApi.downloadMod(
        ProviderKind.Modrinth,
        projectId,
        versionId,
        currentInstallation,
      );
      successCount++;
    } catch (error) {
      console.error(`Failed to update ${mod.file_name}:`, error);
      failCount++;
    }
  }

  updatingAll = false;

  // Clear the updates map since we just processed them
  modsWithUpdates.clear();
  modsWithUpdates = modsWithUpdates; // Trigger reactivity

  // Show result notification
  if (successCount > 0) {
    import("$lib/services/NotificationService").then(
      ({ NotificationService }) => {
        NotificationService.success(
          `Updated ${successCount} mod${successCount !== 1 ? "s" : ""}${failCount > 0 ? `. ${failCount} failed.` : ""}`,
        );
      },
    );
  } else if (failCount > 0) {
    import("$lib/services/NotificationService").then(
      ({ NotificationService }) => {
        NotificationService.error(
          `Failed to update ${failCount} mod${failCount !== 1 ? "s" : ""}`,
        );
      },
    );
  }

  // Reload mods to show updated state
  handleModChanged();
}

async function handleModClick(mod: ModJarInfo) {
  const extendedInfo = $extendedModInfo[mod.file_name];
  if (extendedInfo?.page_uri) {
    try {
      await openUrl(extendedInfo.page_uri);
    } catch (error) {
      console.error("Failed to open mod page:", error);
    }
  }
}

// Toggle disabled state via the backend API. If Ctrl/Cmd is held when activating,
// we open the mod page instead (preserves previous behavior).
async function toggleModDisabledAction(mod: ModJarInfo) {
  if (!currentInstallation) return;
  try {
    const newDisabled = await installationsApi.toggleModDisabled(
      currentInstallation,
      mod.file_name,
    );
    // Update local list optimistically so UI reacts immediately
    mods = mods.map((m) =>
      m.file_name === mod.file_name ? { ...m, disabled: newDisabled } : m,
    );
  } catch (err) {
    console.error("Failed to toggle disabled state for", mod.file_name, err);
    // Try reloading mods to resync state
    try {
      await loadMods(currentInstallation);
    } catch (_) {}
  }
}

async function loadMods(
  installation: KableInstallation,
  options?: { silent?: boolean },
) {
  const silent = options?.silent === true;
  if (!silent) {
    loading = true;
  }
  error = null;
  try {
    const newMods = await InstallationService.getModInfo(installation);

    // Update in place to preserve component identity and prevent flashing
    // Create a map of new mods by file_name for quick lookup
    const newModsMap = new Map(newMods.map((m) => [m.file_name, m]));

    // Update existing mods if they still exist, remove those that don't
    mods = mods.filter((existingMod) => {
      const newMod = newModsMap.get(existingMod.file_name);
      if (newMod) {
        // Update the existing mod object properties
        Object.assign(existingMod, newMod);
        newModsMap.delete(existingMod.file_name);
        return true;
      }
      return false;
    });

    // Add any new mods that weren't in the previous list
    mods = [...mods, ...Array.from(newModsMap.values())];

    await loadModpackSources(installation);

    // Don't clear attemptedExtendedInfo - let it persist so we don't refetch
    // Only clear it when switching installations (handled in reactive statement)
  } catch (e: any) {
    error = e?.message || e || "Failed to load mods info";
    mods = [];
    modpackSources = [];
    modpackProjectMap = new Map();
    attemptedExtendedInfo.clear();
    // Reset the loaded installation ID so we can retry if user switches away and back
    loadedInstallationId = null;
  } finally {
    if (!silent) {
      loading = false;
    }
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
              <button
                class="clear-btn"
                on:click={() => (searchQuery = "")}
                title="Clear search">✕</button
              >
            {/if}
          </div>
        </div>

        {#if currentInstallation}
          <div class="mods-title-section">
            <div class="mods-title-left">
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
                    <span class="count-label"
                      >{mods.length === 1 ? "mod" : "mods"}</span
                    >
                  {/if}
                </div>
              {/if}
            </div>

            <div class="mods-title-right">
              <div class="mod-list-controls">
                {#if modsWithUpdates.size > 0}
                  <button
                    class="update-all-btn"
                    on:click={handleUpdateAll}
                    disabled={updatingAll}
                    title="Update {modsWithUpdates.size} mod{modsWithUpdates.size !==
                    1
                      ? 's'
                      : ''}"
                  >
                    <Icon name="arrow-up" size="sm" forceType="svg" />
                    <span>
                      {updatingAll
                        ? "Updating..."
                        : `Update All (${modsWithUpdates.size})`}
                    </span>
                  </button>
                {/if}

                <label class="control-field">
                  <span>Source View</span>
                  <button
                    class="source-toggle-btn"
                    class:active={sourceViewEnabled}
                    on:click={() => (sourceViewEnabled = !sourceViewEnabled)}
                    title={sourceViewEnabled
                      ? "Showing modpacks + standalone mods"
                      : "Showing all mods"}
                  >
                    <Icon name="layers" size="sm" />
                    <span>{sourceViewEnabled ? "Source On" : "Source Off"}</span
                    >
                  </button>
                </label>

                <label class="control-field">
                  <span class="control-label-with-icon">
                    Sort
                    {#if modSortMode === "date_asc" || modSortMode === "name_asc"}
                      <Icon name="arrow-up" size="sm" />
                    {:else}
                      <Icon name="arrow-down" size="sm" />
                    {/if}
                  </span>
                  <select bind:value={modSortMode} class="control-select">
                    <option value="date_desc">Date (newest first)</option>
                    <option value="date_asc">Date (oldest first)</option>
                    <option value="name_asc">Name (A to Z)</option>
                    <option value="name_desc">Name (Z to A)</option>
                  </select>
                </label>

                <label class="control-field">
                  <span>Disabled Grouping</span>
                  <select bind:value={disabledGroupMode} class="control-select">
                    <option value="end">Group Disabled End</option>
                    <option value="start">Group Disabled Start</option>
                    <option value="none">Do Not Group Disabled</option>
                  </select>
                </label>
              </div>
            </div>
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
            {#if sourceViewEnabled}
              {#if loadingModpackSources}
                <div class="loading-state compact-loading">
                  <Icon name="refresh" size="sm" className="spin" />
                  <span>Loading source mapping...</span>
                </div>
              {/if}

              {#if modpackSources.length > 0}
                <div class="source-section">
                  <h4>Installed Modpacks</h4>
                  <div class="modpack-grid">
                    {#each modpackSources as source (`${source.provider}:${source.mod_id}:${source.version_id || "latest"}`)}
                      <button
                        class="modpack-card"
                        on:click={() => openModpackSource(source)}
                        title="Open modpack page"
                      >
                        <div class="modpack-icon">
                          {#if getModpackCardIcon(source)}
                            <img
                              src={getModpackCardIcon(source) || ""}
                              alt={getModpackCardTitle(source)}
                            />
                          {:else}
                            <Icon name="package" size="md" />
                          {/if}
                        </div>
                        <div class="modpack-meta">
                          <div class="modpack-title">
                            {getModpackCardTitle(source)}
                          </div>
                          <div class="modpack-subtitle">
                            v{source.modpack_version ||
                              source.version_id ||
                              "unknown"}
                          </div>
                          <div class="modpack-count">
                            {source.managed_project_ids.length} managed mod{source
                              .managed_project_ids.length === 1
                              ? ""
                              : "s"}
                          </div>
                          {#if getModpackCardDescription(source)}
                            <div class="modpack-description">
                              {getModpackCardDescription(source)}
                            </div>
                          {/if}
                        </div>
                      </button>
                    {/each}
                  </div>
                </div>
              {/if}

              <div class="source-section">
                <h4>Standalone Mods</h4>
                {#if standaloneFilteredMods.length > 0}
                  <div class="mods-card-grid">
                    {#each standaloneFilteredMods as mod (mod.file_name)}
                      <InstalledModCard
                        {mod}
                        installation={currentInstallation}
                        extendedInfo={$extendedModInfo[mod.file_name]}
                        onmodchanged={handleModChanged}
                        onupdatereport={handleUpdateReport}
                      />
                    {/each}
                  </div>
                {:else}
                  <div class="empty-state inline-empty">
                    <Icon name="cube" size="md" />
                    <span>No standalone mods found in this view.</span>
                  </div>
                {/if}
              </div>
            {:else}
              <div class="mods-card-grid">
                {#each sortedFilteredMods as mod (mod.file_name)}
                  <InstalledModCard
                    {mod}
                    installation={currentInstallation}
                    extendedInfo={$extendedModInfo[mod.file_name]}
                    onmodchanged={handleModChanged}
                    onupdatereport={handleUpdateReport}
                  />
                {/each}
              </div>
            {/if}
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
  border: 1px solid #{"color-mix(in srgb, var(--primary), 8%, transparent)"};
  box-shadow: 0 2px 12px
    #{"color-mix(in srgb, var(--dark-900), 6%, transparent)"};
  overflow: hidden;
}

//  Left sidebar: Installation carousel
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
    border-color: var(
      --loader-color,
      #{"color-mix(in srgb, var(--primary), 15%, transparent)"}
    );
    // Properly maintain center position on hover
    box-shadow: 0 2px 8px
      #{"color-mix(in srgb, var(--loader-color, var(--primary)), 10%, transparent)"};
  }

  &.selected {
    border-color: var(--loader-color, var(--primary));
    box-shadow:
      0 4px 16px
        #{"color-mix(in srgb, var(--loader-color, var(--primary)), 15%, transparent)"},
      inset 0 1px 0 rgba(255, 255, 255, 0.1);
    z-index: 10; // Bring selected item to front

    // Add green selection indicator
    border: 2px solid var(--green-800);

    &:hover {
      // Keep the same transform as base state but with slight scale increase
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
  width: calc(48px * var(--carousel-scale, 1)); // Increased icon size
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

  // Default state
  color: var(--text);

  // Selected state - use solid color instead of gradient to avoid transparency issues
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

//  Right content: Search and mods
.mods-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.mods-header {
  background: linear-gradient(
    135deg,
    var(--card) 0%,
    color-mix(in srgb, var(--primary), 2%, transparent) 100%
  );
  border-bottom: 1px solid color-mix(in srgb, var(--primary), 8%, transparent);
  padding: 1.2rem 1.5rem;

  .mods-title-section {
    display: flex;
    align-items: center;
    justify-content: flex-start;
    margin: 1rem 0 0.5rem 0;
    gap: 1rem;

    h3 {
      margin: 0;
      color: var(--text);
      font-weight: 500;
      font-size: 1.1em;
    }
  }
}

.mods-title-left {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  min-width: 0;
}

.mods-title-right {
  margin-left: auto;
  display: flex;
  align-items: end;
  gap: 0.75rem;
}

.update-all-btn {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.5rem 1rem;
  border-radius: 0.25rem;
  // Use the same blue as .manage-btn in InstalledModCard, fallback to #3b82f6 (blue-500)
  $update-blue: #3b82f6;
  border: 1px solid rgba($update-blue, 0.3);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 0.95rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
  position: relative;

  &:hover:not(:disabled) {
    border-color: rgba($update-blue, 0.5);
    background: rgba($update-blue, 0.1);
    transform: translateY(-1px);
  }

  &:active:not(:disabled) {
    transform: translateY(0);
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  span {
    white-space: nowrap;
  }
}

.mods-count-badge {
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

.mod-list-controls {
  display: flex;
  flex-direction: row;
  align-items: flex-end;
  gap: 1.2rem;
  flex-wrap: wrap;
}

.control-field {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;

  span {
    font-size: 0.75rem;
    color: var(--placeholder);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
}

.control-label-with-icon {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
}

.control-select {
  padding: 0.55rem 0.7rem;
  border: 1px solid var(--dark-600);
  border-radius: 0.6rem;
  background: var(--input);
  color: var(--text);
  font-size: 0.9rem;

  &:focus {
    outline: none;
    border-color: color-mix(in srgb, var(--primary), 60%, transparent);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary), 15%, transparent);
  }
}

.source-toggle-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.55rem 0.75rem;
  border-radius: 0.6rem;
  border: 1px solid var(--dark-600);
  background: var(--input);
  color: var(--text);
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.15s ease;

  &:hover {
    border-color: color-mix(in srgb, var(--primary), 50%, transparent);
  }

  &.active {
    border-color: color-mix(in srgb, var(--green-700), 60%, transparent);
    background: color-mix(in srgb, var(--green-800), 12%, transparent);
    color: var(--green-800);
  }
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

.source-section {
  display: flex;
  flex-direction: column;
  gap: 0.7rem;
  margin-bottom: 1.2rem;

  h4 {
    margin: 0;
    font-size: 0.95rem;
    color: var(--placeholder);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }
}

.modpack-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 0.5rem;
}

.modpack-card {
  border: 1px solid color-mix(in srgb, var(--primary), 18%, transparent);
  border-radius: 0.65rem;
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--primary), 8%, transparent) 0%,
    var(--card) 100%
  );
  padding: 0.75rem;
  display: flex;
  align-items: flex-start;
  gap: 0.7rem;
  cursor: pointer;
  text-align: left;

  &:hover {
    border-color: color-mix(in srgb, var(--primary), 35%, transparent);
    transform: translateY(-1px);
  }
}

.modpack-icon {
  width: 48px;
  height: 48px;
  border-radius: 0.5rem;
  background: var(--bg-tertiary);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  flex-shrink: 0;

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
}

.modpack-meta {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.modpack-title {
  font-weight: 700;
  color: var(--text);
}

.modpack-subtitle {
  font-size: 0.8rem;
  color: var(--placeholder);
}

.modpack-count {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--green-800);
}

.modpack-description {
  color: var(--text);
  font-size: 0.8rem;
  opacity: 0.8;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  line-clamp: 2;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.compact-loading {
  padding: 0.25rem 0;
  justify-content: flex-start;
  font-size: 0.9rem;
}

.inline-empty {
  align-items: flex-start;
  padding: 0.7rem 0;
}

// Card grid for mods
.mods-card-grid {
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

@keyframes pulse {
  0%,
  100% {
    opacity: 0.6;
  }
  50% {
    opacity: 1;
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
    border-bottom: 1px solid
      color-mix(in srgb, var(--primary), 25%, transparent);
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
    border-bottom: 1px solid
      color-mix(in srgb, var(--primary), 25%, transparent);
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

  .mods-title-section {
    flex-direction: column;
    align-items: stretch;
  }

  .mods-title-right {
    margin-left: 0;
    align-items: stretch;
    flex-direction: column;
  }
}

@media (max-width: 700px) {
  .mod-list-controls {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 600px) {
  .installation-mods {
    padding: 0.5rem;
    height: auto;
  }
}
</style>
