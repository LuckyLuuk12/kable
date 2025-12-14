<script lang="ts">
import { onMount, onDestroy, tick } from "svelte";
import { page } from "$app/stores";
import {
  Icon,
  logsService,
  gameInstances,
  currentLogs,
  selectedInstanceId,
  type GameInstance,
  settings,
} from "$lib";

// String pool for common values to reduce memory
const STRING_POOL = {
  levels: {
    error: "error",
    warn: "warn",
    info: "info",
    debug: "debug",
  },
  icons: {
    error: "alert",
    warn: "warning",
    info: "info",
    debug: "bug",
  },
  classes: {
    error: "danger",
    warn: "warning",
    info: "info",
    debug: "muted",
  },
} as const;

let selectedLogType: "launcher" | "game" = "launcher";
let logContainer: HTMLElement;
let autoScroll = true; // Ensure auto-scroll is enabled by default
let searchTerm = "";
let searchMode: "normal" | "regex" | "fuzzy" = "fuzzy";

// Virtual scrolling variables with dynamic heights
const MIN_ITEM_HEIGHT = 28; // Minimum height for a single-line log
const BUFFER_SIZE = 20; // Number of extra items to render above/below viewport
let scrollTop = 0;
let containerHeight = 0;
let visibleStartIndex = 0;
let visibleEndIndex = 50;
let isAutoScrolling = false;

// Track measured heights for each log entry (only visible range)
let logHeights = new Map<string, number>();
let logElements: Record<string, HTMLElement> = {};
let heightsNeedRecalculation = false;

// Track last rendered keys to clean up old elements
let lastRenderedKeys = new Set<string>();
// Maximum heights to keep in memory (visible range + buffer * 2)
const MAX_TRACKED_HEIGHTS = 100;

// Debounce reactive calculations
let recalculationPending = false;

// Track when heights actually change to trigger recalculation
let heightsVersion = 0;

// Log level filters - use a regular variable instead of reactive to allow manual updates
let logLevelFilters = {
  error: true,
  warn: true,
  info: true,
  debug: true,
};

// Initialize from settings
$: if ($settings) {
  logLevelFilters = {
    error: $settings.logging.default_log_levels.indexOf("error") !== -1,
    warn: $settings.logging.default_log_levels.indexOf("warn") !== -1,
    info: $settings.logging.default_log_levels.indexOf("info") !== -1,
    debug: $settings.logging.default_log_levels.indexOf("debug") !== -1,
  };
}

let showLogLevelDropdown = false;
let showCopyNotification = false;
let cleanupIntervalId: number | undefined;

onMount(() => {
  // Logs service is already initialized in the layout
  // No need to initialize again here

  // Check for instance parameter in URL
  const instanceParam = $page.url.searchParams.get("instance");
  if (instanceParam) {
    // Set the selected instance from the URL parameter
    selectedInstanceId.set(instanceParam);
    // When navigating via URL parameter (e.g., from game launch), show game logs
    selectedLogType = "game";
  }

  // Initialize container dimensions
  if (logContainer) {
    containerHeight = logContainer.clientHeight;
    visibleEndIndex = Math.min(
      50,
      Math.ceil(containerHeight / MIN_ITEM_HEIGHT) + BUFFER_SIZE,
    );
    // Initialize visible range
    updateVisibleRange();
  }

  // Add event listener for clicking outside dropdown
  document.addEventListener("click", handleClickOutside);
  // Add event listener for Ctrl+A override
  document.addEventListener("keydown", handleKeyDown);
  // Add window resize listener to recalculate heights when width changes
  window.addEventListener("resize", handleResize);

  // Setup periodic cleanup to prevent memory leaks
  cleanupIntervalId = window.setInterval(() => {
    cleanupStaleElements();
  }, 30000) as any; // Every 30 seconds
});

onDestroy(() => {
  // Keep logs service running since it's used globally
  // Remove event listeners
  document.removeEventListener("click", handleClickOutside);
  document.removeEventListener("keydown", handleKeyDown);
  window.removeEventListener("resize", handleResize);

  // Clear intervals
  if (cleanupIntervalId) {
    clearInterval(cleanupIntervalId);
  }
  if (resizeTimeout) {
    clearTimeout(resizeTimeout);
  }

  // Aggressive cleanup of all element references and maps
  logElements = {};
  logHeights.clear();
  lastRenderedKeys.clear();

  // Null out container reference
  logContainer = null as any;
});

function selectInstance(instanceId: string | "global") {
  selectedInstanceId.set(instanceId);
  // Default to game logs for installations, launcher for global
  selectedLogType = instanceId === "global" ? "launcher" : "game";
}

function getInstanceDisplayName(instance: GameInstance): string {
  if (!instance || !instance.launchedAt) return "Unknown";

  const launchedAt =
    instance.launchedAt instanceof Date
      ? instance.launchedAt
      : new Date(instance.launchedAt);

  const duration = Math.floor((Date.now() - launchedAt.getTime()) / 1000);
  const durationStr =
    duration < 60 ? `${duration}s` : `${Math.floor(duration / 60)}m`;
  return `${instance.profileName || "Unknown"} (${durationStr})`;
}

function getStatusIcon(status: GameInstance["status"]): string {
  switch (status) {
    case "launching":
      return "rocket";
    case "running":
      return "play";
    case "closed":
      return "check";
    case "crashed":
      return "alert";
    case "stopped":
      return "square"; // Better icon for stopped
    default:
      return "help";
  }
}

function getStatusColor(status: GameInstance["status"]): string {
  switch (status) {
    case "launching":
      return "warning";
    case "running":
      return "success";
    case "closed":
      return "info";
    case "crashed":
      return "danger";
    case "stopped":
      return "secondary"; // Changed from 'muted' for better visibility
    default:
      return "muted";
  }
}

function formatTime(date: Date | string): string {
  const dateObj = date instanceof Date ? date : new Date(date);
  return dateObj.toLocaleTimeString("en-US", { hour12: false });
}

function formatLogLevel(level: string): string {
  return level.toUpperCase();
}

function getLogLevelIcon(level: string): string {
  const lowerLevel = level.toLowerCase();
  return (
    STRING_POOL.icons[lowerLevel as keyof typeof STRING_POOL.icons] || "message"
  );
}

function getLogLevelClass(level: string): string {
  const lowerLevel = level.toLowerCase();
  return (
    STRING_POOL.classes[lowerLevel as keyof typeof STRING_POOL.classes] ||
    "secondary"
  );
}

async function clearCurrentLogs() {
  const instanceId = $selectedInstanceId;
  await logsService.clearLogs(instanceId === "global" ? undefined : instanceId);
}

async function exportCurrentLogs() {
  const instanceId = $selectedInstanceId;
  await logsService.exportLogs(
    instanceId === "global" ? undefined : instanceId,
  );
}

async function copyLogEntry(logEntry: any) {
  if (!logEntry) return;

  const timestamp =
    logEntry.timestamp instanceof Date
      ? logEntry.timestamp
      : new Date(logEntry.timestamp);

  // Clean the message for copy (remove duplicate timestamps)
  const cleanMessage = removeDuplicateTimestamp(
    logEntry.message || "",
    timestamp,
  );
  const logText = `[${formatTime(timestamp)}] ${formatLogLevel(logEntry.level || "info")} ${cleanMessage}`;
  try {
    await navigator.clipboard.writeText(logText);
    // Could add a toast notification here if desired
  } catch (err) {
    console.error("Failed to copy log entry:", err);
  }
}

// Function to detect and remove duplicate timestamps from log messages
function removeDuplicateTimestamp(
  message: string,
  logTimestamp: Date | string,
): string {
  if (!message) return message;

  // Remove timestamp from the beginning of the message since we display it separately
  // Pattern to match timestamps like [HH:MM:SS] at the start
  const timestampPattern = /^\[\d{2}:\d{2}:\d{2}\]\s*/;

  let result = message.replace(timestampPattern, "");

  // Also remove log level if it appears right after the timestamp
  // This handles cases like "[22:54:42] INFO [Render thread/INFO]: message"
  if (result.match(/^(INFO|ERROR|WARN|DEBUG)\s+/)) {
    result = result.replace(/^(INFO|ERROR|WARN|DEBUG)\s+/, "");
  }

  return result.trim();
}

// Function to get display message (with duplicate timestamps removed)
function getDisplayMessage(logEntry: any): string {
  if (!logEntry || !logEntry.message) return "";

  const timestamp =
    logEntry.timestamp instanceof Date
      ? logEntry.timestamp
      : new Date(logEntry.timestamp);

  return removeDuplicateTimestamp(logEntry.message, timestamp);
}

// Fuzzy search implementation
function fuzzyMatch(needle: string, haystack: string): boolean {
  if (!needle || !haystack) return false;

  const needleLower = needle.toLowerCase();
  const haystackLower = haystack.toLowerCase();

  // Simple fuzzy matching: allow for missing characters and transpositions
  let needleIndex = 0;
  let score = 0;

  for (
    let i = 0;
    i < haystackLower.length && needleIndex < needleLower.length;
    i++
  ) {
    if (haystackLower[i] === needleLower[needleIndex]) {
      needleIndex++;
      score++;
    }
  }

  // Calculate match ratio (how much of the needle was found)
  const matchRatio = score / needleLower.length;

  // Also check for substring match (higher weight)
  const substringMatch = haystackLower.includes(needleLower);

  // Consider it a match if:
  // 1. It's a direct substring match, OR
  // 2. At least 70% of characters are found in order
  return substringMatch || matchRatio >= 0.7;
}

// Advanced search function
function matchesSearch(
  message: string,
  searchTerm: string,
  mode: string,
): boolean {
  if (!searchTerm) return true;
  if (!message) return false;

  switch (mode) {
    case "regex":
      try {
        const regex = new RegExp(searchTerm, "i");
        return regex.test(message);
      } catch (error) {
        // If regex is invalid, fall back to normal search
        return message.toLowerCase().includes(searchTerm.toLowerCase());
      }

    case "fuzzy":
      return fuzzyMatch(searchTerm, message);

    case "normal":
    default:
      return message.toLowerCase().includes(searchTerm.toLowerCase());
  }
}

function getLogKey(logEntry: any, index: number): string {
  if (!logEntry) return `unknown-${index}`;
  const timestamp =
    logEntry.timestamp instanceof Date
      ? logEntry.timestamp.getTime()
      : new Date(logEntry.timestamp).getTime();
  return `${timestamp}-${index}`;
}

function getLogHeight(key: string): number {
  return logHeights.get(key) || MIN_ITEM_HEIGHT;
}

function cleanupStaleElements() {
  // Remove element references that are no longer visible
  const currentKeys = new Set(lastRenderedKeys);
  const keysToDelete: string[] = [];

  for (const key in logElements) {
    if (!currentKeys.has(key)) {
      keysToDelete.push(key);
    }
  }

  // Delete stale references
  for (const key of keysToDelete) {
    delete logElements[key];
  }

  // More aggressive cleanup: only keep heights for visible range + small buffer
  if (logHeights.size > MAX_TRACKED_HEIGHTS) {
    const keys = Array.from(logHeights.keys());
    // Keep only the most recent measurements (likely to be in visible range)
    const toDelete = keys.slice(0, keys.length - MAX_TRACKED_HEIGHTS);
    for (const key of toDelete) {
      logHeights.delete(key);
    }
  }

  // Force cleanup if too many element refs
  const elementCount = Object.keys(logElements).length;
  if (elementCount > MAX_TRACKED_HEIGHTS * 2) {
    // Keep only the last MAX_TRACKED_HEIGHTS elements
    const allKeys = Object.keys(logElements);
    const keysToRemove = allKeys.slice(0, allKeys.length - MAX_TRACKED_HEIGHTS);
    keysToRemove.forEach((key) => delete logElements[key]);
  }
}

function measureLogHeights() {
  // Measure all currently rendered log elements
  const currentKeys = new Set<string>();
  let heightsChanged = false;

  Object.entries(logElements).forEach(([key, element]) => {
    if (element) {
      const height = element.offsetHeight;
      if (height > 0) {
        const oldHeight = logHeights.get(key);
        if (oldHeight !== height) {
          heightsChanged = true;
        }
        logHeights.set(key, height);
        currentKeys.add(key);
      }
    }
  });

  lastRenderedKeys = currentKeys;
  heightsNeedRecalculation = false;

  // Only increment version if heights actually changed
  if (heightsChanged) {
    heightsVersion++;
  }

  // Clean up stale elements periodically
  cleanupStaleElements();
}

function scrollToBottom() {
  if (logContainer) {
    isAutoScrolling = true;
    // For virtual scrolling, scroll to the maximum scrollable position
    // scrollTop can't exceed scrollHeight - clientHeight
    const maxScroll = Math.max(0, totalHeight - logContainer.clientHeight);
    logContainer.scrollTop = maxScroll;

    // Update visible range immediately for virtual scrolling
    updateVisibleRange();

    // Update autoScroll flag
    autoScroll = true;

    // Reset flag after a short delay
    setTimeout(() => {
      isAutoScrolling = false;
    }, 100);
  }
}

function updateVisibleRange() {
  if (!logContainer || filteredIndices.length === 0) return;

  const { scrollTop: st, clientHeight } = logContainer;
  scrollTop = st;
  containerHeight = clientHeight;

  // Calculate visible range using accumulated heights
  let accumulatedHeight = 0;
  let startIndex = 0;
  let endIndex = filteredIndices.length;

  // Find start index
  for (let i = 0; i < filteredIndices.length; i++) {
    const origIndex = filteredIndices[i];
    const log = activeLogEntries[origIndex];
    const key = getLogKey(log, i);
    const height = getLogHeight(key);

    if (
      accumulatedHeight + height >
      scrollTop - BUFFER_SIZE * MIN_ITEM_HEIGHT
    ) {
      startIndex = Math.max(0, i);
      break;
    }
    accumulatedHeight += height;
  }

  // Find end index
  accumulatedHeight = 0;
  for (let i = 0; i < filteredIndices.length; i++) {
    const origIndex = filteredIndices[i];
    const log = activeLogEntries[origIndex];
    const key = getLogKey(log, i);
    const height = getLogHeight(key);
    accumulatedHeight += height;

    if (
      accumulatedHeight >
      scrollTop + containerHeight + BUFFER_SIZE * MIN_ITEM_HEIGHT
    ) {
      endIndex = Math.min(filteredIndices.length, i + 1);
      break;
    }
  }

  visibleStartIndex = startIndex;
  visibleEndIndex = endIndex;
}

function handleScroll() {
  if (logContainer && !isAutoScrolling) {
    updateVisibleRange();

    // Check if user is at bottom (with small threshold)
    // For virtual scrolling, check against totalHeight instead of scrollHeight
    const maxScroll = Math.max(0, totalHeight - containerHeight);
    autoScroll = scrollTop >= maxScroll - 50;
  }
}

function toggleLogLevelDropdown() {
  showLogLevelDropdown = !showLogLevelDropdown;
}

function getLogLevelDisplayName(level: string): string {
  switch (level) {
    case "error":
      return "Errors";
    case "warn":
      return "Warnings";
    case "info":
      return "Info";
    case "debug":
      return "Debug";
    default:
      return level;
  }
}

// Reactive variable for enabled log levels count
$: enabledLogLevelsCount =
  Object.values(logLevelFilters).filter(Boolean).length;

// Close dropdown when clicking outside
function handleClickOutside(event: MouseEvent) {
  const target = event.target as HTMLElement;
  if (!target.closest(".log-level-dropdown")) {
    showLogLevelDropdown = false;
  }
}

// Handle keyboard shortcuts
function handleKeyDown(event: KeyboardEvent) {
  // Override Ctrl+A to select all logs in the current tab
  if (event.ctrlKey && event.key === "a") {
    // Only override if we're not in an input field
    const target = event.target as HTMLElement;
    if (target.tagName === "INPUT" || target.tagName === "TEXTAREA") {
      return; // Let default behavior happen in input fields
    }

    event.preventDefault();
    selectAllCurrentLogs();
  }
}

// Handle window resize - recalculate heights when width changes
let resizeTimeout: number;
function handleResize() {
  // Debounce resize to avoid excessive recalculations
  clearTimeout(resizeTimeout);
  resizeTimeout = window.setTimeout(() => {
    // Clear all measured heights to force remeasurement
    logHeights.clear();
    heightsNeedRecalculation = true;
    tick().then(() => {
      measureLogHeights();
      updateVisibleRange();
    });
  }, 150);
}

// Function to select all logs in the current tab
async function selectAllCurrentLogs() {
  if (filteredCount === 0) return;

  // Show visual feedback
  showCopyNotification = true;

  // Generate the text for all logs using on-demand access
  const allLogsText = filteredIndices
    .map((origIndex) => {
      const logEntry = activeLogEntries[origIndex];
      if (!logEntry) return "";

      const timestamp =
        logEntry.timestamp instanceof Date
          ? logEntry.timestamp
          : new Date(logEntry.timestamp);

      // Clean the message for copy (remove duplicate timestamps)
      const cleanMessage = removeDuplicateTimestamp(
        logEntry.message || "",
        timestamp,
      );
      return `[${formatTime(timestamp)}] ${formatLogLevel(logEntry.level || STRING_POOL.levels.info)} ${cleanMessage}`;
    })
    .filter(Boolean)
    .join("\n");

  try {
    await navigator.clipboard.writeText(allLogsText);
  } catch (err) {
    console.error("Failed to copy all logs:", err);
  }

  // Hide notification after a slightly longer delay so users notice it
  setTimeout(() => {
    showCopyNotification = false;
  }, 3000);
}

$: sortedInstances = $gameInstances
  ? Array.from($gameInstances.values()).sort(
      (a: GameInstance, b: GameInstance) => {
        const aTime =
          a.launchedAt instanceof Date
            ? a.launchedAt.getTime()
            : new Date(a.launchedAt).getTime();
        const bTime =
          b.launchedAt instanceof Date
            ? b.launchedAt.getTime()
            : new Date(b.launchedAt).getTime();
        return bTime - aTime;
      },
    )
  : [];

// Reactive: Ensure we're on the game subtab when a valid game instance is selected
// This handles the case where URL navigation happens before the instance is added to the store
$: {
  const currentInstanceId = $selectedInstanceId;
  if (currentInstanceId !== "global" && $gameInstances) {
    const instanceExists = $gameInstances.has(currentInstanceId);
    if (instanceExists && selectedLogType === "launcher") {
      // Switch to game subtab if we're on an instance but still showing launcher logs
      selectedLogType = "game";
    }
  }
}

$: currentLogsData = $currentLogs || { launcherLogs: [], gameLogs: [] };
$: activeLogEntries =
  selectedLogType === "launcher"
    ? currentLogsData.launcherLogs || []
    : currentLogsData.gameLogs || [];

// Compute filtered count and indices with proper reactivity
// This reactive statement depends on: activeLogEntries, searchTerm, searchMode, logLevelFilters
$: filteredIndices = (() => {
  const indices: number[] = [];

  // Explicitly reference reactive dependencies
  const logs = activeLogEntries;
  const search = searchTerm;
  const mode = searchMode;
  const filters = logLevelFilters;

  for (let i = 0; i < logs.length; i++) {
    const log = logs[i];
    if (!log) continue;

    // Use cleaned message for search to avoid searching duplicate timestamps
    const cleanMessage = getDisplayMessage(log);
    const matchesSearchTerm = matchesSearch(cleanMessage, search, mode);

    // Check if the log level is enabled in filters
    const logLevel = (log.level || STRING_POOL.levels.info).toLowerCase();
    const matchesLevelFilter =
      logLevel in filters ? filters[logLevel as keyof typeof filters] : true; // Show unknown log levels by default

    if (matchesSearchTerm && matchesLevelFilter) {
      indices.push(i);
    }
  }

  return indices;
})();

$: filteredCount = filteredIndices.length;

// Virtual scrolling: only render visible logs (on-demand access)
$: visibleLogs = filteredIndices
  .slice(visibleStartIndex, visibleEndIndex)
  .map((i) => activeLogEntries[i]);

// Calculate total height and offset using measured heights
// Use heightsVersion as dependency to only recalculate when heights change
let totalHeight = 0;
let offsetY = 0;

$: if (heightsVersion >= 0 && filteredIndices) {
  totalHeight = filteredIndices.reduce((sum, origIndex, filteredIndex) => {
    const log = activeLogEntries[origIndex];
    const key = getLogKey(log, filteredIndex);
    return sum + getLogHeight(key);
  }, 0);
}

$: if (heightsVersion >= 0 && filteredIndices && visibleStartIndex >= 0) {
  offsetY = filteredIndices
    .slice(0, visibleStartIndex)
    .reduce((sum, origIndex, filteredIndex) => {
      const log = activeLogEntries[origIndex];
      const key = getLogKey(log, filteredIndex);
      return sum + getLogHeight(key);
    }, 0);
}

// Update visible range when filteredIndices changes (tab switch, filters, etc.)
// Only trigger when filteredIndices length changes or container becomes ready
let lastFilteredLogsLength = 0;
let recalculationTimer: number | undefined;
$: if (
  filteredIndices &&
  logContainer &&
  filteredCount !== lastFilteredLogsLength &&
  !recalculationPending
) {
  lastFilteredLogsLength = filteredCount;
  recalculationPending = true;
  heightsNeedRecalculation = true;

  // Debounce recalculation to prevent excessive updates
  if (recalculationTimer) {
    clearTimeout(recalculationTimer);
  }

  recalculationTimer = window.setTimeout(() => {
    tick().then(() => {
      if (heightsNeedRecalculation) {
        measureLogHeights();
        updateVisibleRange();
      }
      recalculationPending = false;
    });
  }, 100) as any; // 100ms debounce
}

// Track previous log count to detect new logs
let previousLogCount = 0;

// Auto-scroll when NEW logs arrive (not on every reactive update)
$: {
  if (filteredIndices && filteredCount > previousLogCount && autoScroll) {
    const currentCount = filteredCount;
    tick().then(() => {
      // Only scroll if count still matches and autoScroll is still enabled
      if (filteredCount === currentCount && autoScroll && logContainer) {
        scrollToBottom();
      }
    });
    previousLogCount = currentCount;
  } else if (filteredIndices) {
    previousLogCount = filteredCount;
  }
}

// Check if any filters are active
$: hasActiveFilters = searchTerm || enabledLogLevelsCount < 4;
</script>

<div class="logs-page">
  <!-- Page Header -->
  <div class="page-header">
    <div class="header-content">
      <h1>
        <Icon name="archive" size="md" />
        Logs
      </h1>
      <p>Monitor launcher activity and game processes in real-time</p>
      {#if (sortedInstances || []).length > 0}
        <div class="stat-badge">
          <Icon name="activity" size="sm" />
          {(sortedInstances || []).length} active instance{(
            sortedInstances || []
          ).length !== 1
            ? "s"
            : ""}
        </div>
      {/if}
    </div>
    <div class="header-actions">
      <button
        class="btn btn-danger btn-sm"
        on:click={clearCurrentLogs}
        title="Clear current logs"
      >
        <Icon name="trash" size="sm" />
        Clear
      </button>
      <button
        class="btn btn-secondary btn-sm"
        on:click={exportCurrentLogs}
        title="Export logs to file"
      >
        <Icon name="download" size="sm" />
        Export
      </button>
    </div>
  </div>

  <!-- Filters Section -->
  <div class="filters-section">
    <div class="search-container">
      <Icon name="search" size="sm" />
      <input
        type="text"
        placeholder={searchMode === "regex"
          ? "Search with regex..."
          : searchMode === "fuzzy"
            ? 'Fuzzy search (try "frge" for "forge")...'
            : "Search logs..."}
        bind:value={searchTerm}
        class="search-input"
      />
      <div class="search-mode-selector">
        <button
          class="search-mode-button {searchMode === 'normal' ? 'active' : ''}"
          on:click={() => (searchMode = "normal")}
          title="Normal text search"
        >
          <Icon name="text" size="sm" />
        </button>
        <button
          class="search-mode-button {searchMode === 'fuzzy' ? 'active' : ''}"
          on:click={() => (searchMode = "fuzzy")}
          title="Fuzzy search (handles typos)"
        >
          <Icon name="zap" size="sm" />
        </button>
        <button
          class="search-mode-button {searchMode === 'regex' ? 'active' : ''}"
          on:click={() => (searchMode = "regex")}
          title="Regular expression search"
        >
          <Icon name="code" size="sm" />
        </button>
      </div>
    </div>
    <div class="filter-controls">
      <div class="log-level-dropdown">
        <button
          class="dropdown-trigger"
          on:click={toggleLogLevelDropdown}
          type="button"
        >
          <span>Log Levels ({enabledLogLevelsCount}/4)</span>
          <Icon
            name={showLogLevelDropdown ? "chevron-up" : "chevron-down"}
            size="sm"
          />
        </button>

        {#if showLogLevelDropdown}
          <div class="dropdown-menu">
            <div class="dropdown-header">
              <span>Select log levels to display</span>
            </div>
            {#each Object.entries(logLevelFilters) as [level, enabled]}
              <label class="dropdown-item">
                <input
                  type="checkbox"
                  checked={enabled}
                  on:change={(e) => {
                    const target = e.target as HTMLInputElement;
                    // Create a new object to trigger reactivity
                    logLevelFilters = {
                      ...logLevelFilters,
                      [level]: target.checked,
                    };
                  }}
                />
                <Icon name={getLogLevelIcon(level)} size="sm" />
                <span>{getLogLevelDisplayName(level)}</span>
                <span class="log-level-count"
                  >({(activeLogEntries || []).filter(
                    (log) => (log.level || "info").toLowerCase() === level,
                  ).length})</span
                >
              </label>
            {/each}
          </div>
        {/if}
      </div>

      <label class="checkbox-label">
        <input
          type="checkbox"
          bind:checked={autoScroll}
          on:change={(e) => {
            const target = e.target as HTMLInputElement;
            if (target.checked) {
              // When auto-scroll is enabled, immediately jump to bottom
              scrollToBottom();
            }
          }}
        />
        <span>Auto-scroll</span>
      </label>
    </div>
  </div>

  <!-- Instance Tabs -->
  <div class="tabs-container">
    <div class="tab-list">
      <!-- Global Tab -->
      <button
        class="tab-button {$selectedInstanceId === 'global' ? 'active' : ''}"
        on:click={() => selectInstance("global")}
      >
        <Icon name="globe" size="sm" />
        <span>Launcher</span>
      </button>

      <!-- Instance Tabs -->
      {#each sortedInstances as instance (instance.id)}
        <button
          class="tab-button {$selectedInstanceId === instance.id
            ? 'active'
            : ''}"
          on:click={() => selectInstance(instance.id)}
        >
          <Icon name={getStatusIcon(instance.status)} size="sm" />
          <span>{getInstanceDisplayName(instance)}</span>
          <span class="status-badge {getStatusColor(instance.status)}">
            {instance.status}
          </span>
        </button>
      {/each}
    </div>
  </div>

  <!-- Sub-tabs for Log Types (when instance is selected) -->
  {#if $selectedInstanceId !== "global"}
    <div class="sub-tabs-container">
      <div class="sub-tab-list">
        <button
          class="sub-tab-button {selectedLogType === 'launcher'
            ? 'active'
            : ''}"
          on:click={() => (selectedLogType = "launcher")}
        >
          <Icon name="rocket" size="sm" />
          <span>Launcher</span>
          <span class="count-badge"
            >{(currentLogsData.launcherLogs || []).length}</span
          >
        </button>
        <button
          class="sub-tab-button {selectedLogType === 'game' ? 'active' : ''}"
          on:click={() => (selectedLogType = "game")}
        >
          <Icon name="gamepad" size="sm" />
          <span>Game</span>
          <span class="count-badge"
            >{(currentLogsData.gameLogs || []).length}</span
          >
        </button>
      </div>
    </div>
  {/if}

  <!-- Log Content -->
  <div class="log-content">
    <div
      bind:this={logContainer}
      on:scroll={handleScroll}
      class="log-container {showCopyNotification
        ? 'copy-notification-active'
        : ''}"
    >
      {#if filteredCount === 0}
        <div class="empty-state">
          <div class="empty-icon">
            {#if hasActiveFilters}
              <Icon name="search" size="xl" />
            {:else}
              <Icon name="archive" size="xl" />
            {/if}
          </div>
          <h3>
            {#if hasActiveFilters}
              No logs match your filters
            {:else}
              No logs yet
            {/if}
          </h3>
          <p>
            {#if hasActiveFilters}
              Try adjusting your search or filter settings
            {:else}
              Launch an installation to see logs here
            {/if}
          </p>
        </div>
      {:else}
        <div class="log-entries-wrapper" style="height: {totalHeight}px;">
          <div class="log-entries" style="transform: translateY({offsetY}px);">
            {#each visibleLogs as logEntry, index (getLogKey(logEntry, visibleStartIndex + index))}
              {#if logEntry}
                {@const logKey = getLogKey(logEntry, visibleStartIndex + index)}
                <div class="log-entry" bind:this={logElements[logKey]}>
                  <div
                    class="log-copy-icon"
                    on:click={() => copyLogEntry(logEntry)}
                    title="Copy log entry"
                    role="button"
                    tabindex="0"
                    on:keydown={(e) =>
                      e.key === "Enter" && copyLogEntry(logEntry)}
                  >
                    <Icon name="clipboard" size="sm" />
                  </div>
                  <div class="log-timestamp">
                    {formatTime(logEntry.timestamp)}
                  </div>
                  <div
                    class="log-level badge {getLogLevelClass(
                      logEntry.level || 'info',
                    )}"
                  >
                    <Icon
                      name={getLogLevelIcon(logEntry.level || "info")}
                      size="sm"
                    />
                    {formatLogLevel(logEntry.level || "info")}
                  </div>
                  <div class="log-message">
                    <pre class="log-message-content"><code
                        >{getDisplayMessage(logEntry)}</code
                      ></pre>
                  </div>
                </div>
              {/if}
            {/each}
          </div>
        </div>
      {/if}
    </div>
    <!-- Overlay copy notification placed over the visible log area -->
    {#if showCopyNotification}
      <div class="overlay-copy-notification" role="status" aria-live="polite">
        <div class="overlay-copy-content">
          <Icon name="clipboard" size="md" />
          <span
            >Copied {filteredCount} log entr{filteredCount === 1
              ? "y"
              : "ies"}</span
          >
        </div>
      </div>
    {/if}
  </div>

  <!-- Status Bar -->
  <div class="status-bar">
    <div class="status-left">
      <span class="status-text">
        {#if $selectedInstanceId === "global"}
          {#if filteredCount === (currentLogsData.launcherLogs || []).length}
            Global logs: {(currentLogsData.launcherLogs || []).length} entries
          {:else}
            Global logs: {filteredCount} / {(currentLogsData.launcherLogs || [])
              .length} entries
          {/if}
        {:else if filteredCount === (activeLogEntries || []).length}
          {selectedLogType === "launcher" ? "Launcher" : "Game"} logs: {(
            activeLogEntries || []
          ).length} entries
        {:else}
          {selectedLogType === "launcher" ? "Launcher" : "Game"} logs: {filteredCount}
          / {(activeLogEntries || []).length} entries
        {/if}
        {#if searchTerm && searchMode !== "normal"}
          • {searchMode}
        {/if}
      </span>
      {#if !autoScroll}
        <button class="btn btn-link btn-sm" on:click={scrollToBottom}>
          <Icon name="arrow-down" size="sm" />
          Jump to bottom
        </button>
      {/if}
    </div>
    <div class="status-right">
      <span class="status-text"
        >{(sortedInstances || []).length} active instances</span
      >
      {#if autoScroll}
        <span class="auto-scroll-indicator">
          <Icon name="refresh" size="sm" />
          Auto-scroll
        </span>
      {/if}
    </div>
  </div>
</div>

<style lang="scss">
.logs-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  user-select: none; // Disable text selection for the entire page
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 1.5rem;

  .header-content {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 0 1rem;

    h1 {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      margin: 0 0 0.5rem 0;
      font-size: 2rem;
      font-weight: 700;
    }

    p {
      margin: 0 0 1rem 0;
      color: var(--placeholder);
      font-size: 1.1rem;
    }

    .stat-badge {
      display: inline-flex;
      align-items: center;
      gap: 0.25rem;
      padding: 0.25rem 0.75rem;
      background: color-mix(in srgb, var(--primary), 10%, transparent);
      color: var(--primary);
      border-radius: var(--border-radius);
      font-size: 0.875rem;
      font-weight: 500;
    }
  }

  .header-actions {
    display: flex;
    gap: 0.5rem;
  }
}

.filters-section {
  display: flex;
  gap: 1rem;
  align-items: center;
  margin-bottom: 0.5rem;
  padding: 1rem;
  background: var(--container);
  border-radius: var(--border-radius);
  border: 1px solid var(--dark-200);

  .search-container {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
    gap: 0.5rem;

    .search-input {
      flex: 1;
      padding: 0.5rem 0.75rem;
      font-size: 0.9rem;

      &:focus {
        outline: none;
        border-color: var(--primary);
      }

      &::placeholder {
        color: var(--placeholder);
      }
    }

    .search-mode-selector {
      display: flex;
      background: var(--card);
      // border: 1px solid var(--dark-200);
      border-radius: var(--border-radius-tiny);
      overflow: hidden;

      .search-mode-button {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0.5rem;
        background: transparent;
        border: none;
        color: var(--placeholder);
        cursor: pointer;
        transition: all 0.2s ease;
        min-width: 2.5rem;
        border-right: 1px solid var(--dark-200);

        &:last-child {
          border-right: none;
        }

        &:hover {
          background: var(--dark-200);
          color: var(--text);
        }

        &.active {
          background: var(--primary);
          color: var(--text-white);
        }

        &:focus {
          outline: none;
          box-shadow: inset 0 0 0 2px
            color-mix(in srgb, var(--primary), 30%, transparent);
        }
      }
    }
  }

  .filter-controls {
    display: flex;
    gap: 1rem;
    align-items: center;

    .log-level-dropdown {
      position: relative;

      .dropdown-trigger {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.5rem 0.75rem;
        background: var(--card);
        border: 1px solid var(--dark-200);
        border-radius: var(--border-radius-small);
        color: var(--text);
        font-size: 0.9rem;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s ease;
        white-space: nowrap;

        &:hover {
          background: var(--dark-200);
          border-color: var(--primary);
        }

        &:focus {
          outline: none;
          border-color: var(--primary);
        }
      }

      .dropdown-menu {
        position: absolute;
        top: calc(100% + 0.25rem);
        right: 0;
        min-width: 180px;
        background: var(--container);
        border: 1px solid var(--dark-200);
        border-radius: var(--border-radius);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
        z-index: 1000;
        overflow: hidden;

        .dropdown-header {
          padding: 0.75rem;
          background: var(--card);
          border-bottom: 1px solid var(--dark-200);
          font-size: 0.85rem;
          font-weight: 600;
          color: var(--placeholder);
        }

        .dropdown-item {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.5rem 0.75rem;
          font-size: 0.9rem;
          cursor: pointer;
          transition: background-color 0.2s ease;

          &:hover {
            background: var(--card);
          }

          input[type="checkbox"] {
            accent-color: var(--primary);
          }
          .log-level-count {
            margin-left: auto;
            color: var(--placeholder);
            font-size: 0.85em;
            font-weight: 500;
            letter-spacing: 0.02em;
            text-align: right;
            min-width: 2.5em;
            display: flex;
            align-items: center;
            justify-content: flex-end;
          }
        }
      }
    }

    .checkbox-label {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      font-size: 0.9rem;
      cursor: pointer;

      input[type="checkbox"] {
        accent-color: var(--primary);
      }
    }
  }
}

.tabs-container {
  .tab-list {
    display: flex;
    background: var(--container);
    border-radius: var(--border-radius) var(--border-radius) 0 0;
    border: 1px solid var(--dark-200);
    overflow-x: auto;
    gap: 0.05rem;

    .tab-button {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      padding: 0.75rem 1rem;
      background: transparent;
      border: none;
      color: var(--placeholder);
      font-size: 0.9rem;
      font-weight: 500;
      cursor: pointer;
      white-space: nowrap;
      transition: all 0.2s ease;
      border-right: 1px solid var(--dark-200);
      border-radius: var(--border-radius) var(--border-radius) 0 0;

      &:last-child {
        border-right: none;
      }

      &:hover {
        // background: var(--hover);
        color: var(--text);
      }

      &.active {
        background: var(--primary);
        color: var(--text);
      }

      .status-badge {
        padding: 0.125rem 0.375rem;
        border-radius: calc(var(--border-radius) * 0.5);
        font-size: 0.85rem;
        font-weight: 600;

        &.success {
          background: color-mix(in srgb, var(--green), 10%, transparent);
          color: var(--green);
        }
        &.warning {
          background: color-mix(in srgb, var(--yellow), 10%, transparent);
          color: var(--yellow);
        }
        &.danger {
          background: color-mix(in srgb, var(--red), 10%, transparent);
          color: var(--red);
        }
        &.info {
          background: color-mix(in srgb, var(--green), 10%, transparent);
          color: var(--green-900);
        }
        &.secondary {
          background: color-mix(in srgb, var(--text), 20%, transparent);
          color: var(--text);
        }
        &.muted {
          background: color-mix(in srgb, var(--dark-300), 10%, transparent);
          color: var(--text-white);
        }
      }
    }
  }
}

.sub-tabs-container {
  // margin-bottom: 1rem;

  .sub-tab-list {
    display: flex;
    background: var(--container);
    border-left: 1px solid var(--dark-200);
    border-right: 1px solid var(--dark-200);
    gap: 0.05rem;

    .sub-tab-button {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      padding: 0.25rem 0.75rem;
      background: transparent;
      border: none;
      color: var(--placeholder);
      font-size: 0.875rem;
      font-weight: 500;
      cursor: pointer;
      transition: all 0.2s ease;
      border-right: 1px solid var(--dark-200);
      border-radius: var(--border-radius) var(--border-radius) 0 0;

      &:last-child {
        border-right: none;
      }

      &:hover {
        background: var(--card);
        color: var(--text);
      }

      &.active {
        background: var(--primary);
        color: var(--text-white);
      }

      .count-badge {
        padding: 0.125rem 0.25rem;
        background: rgba(255, 255, 255, 0.2);
        border-radius: calc(var(--border-radius) * 0.5);
        font-size: 0.75rem;
        min-width: 1.25rem;
        text-align: center;
      }
    }
  }
}

.log-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;

  .log-container {
    flex: 1;
    overflow-y: auto;
    background: var(--container);
    border: 1px solid var(--dark-200);
    border-radius: 0 0 var(--border-radius) var(--border-radius);
    margin-bottom: 0.5rem;
    position: relative;

    &.copy-notification-active {
      .log-entries-wrapper .log-entries {
        .log-entry {
          background: color-mix(in srgb, var(--primary), 10%, transparent);
          border: 1px solid color-mix(in srgb, var(--primary), 20%, transparent);
        }
      }
    }

    .copy-notification {
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background: color-mix(in srgb, var(--dark-100), 35%, transparent);
      backdrop-filter: blur(4px);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 1000;
      animation: fadeInOut 1s ease-in-out;
    }

    @keyframes fadeInOut {
      0% {
        opacity: 0;
        transform: scale(0.9);
      }
      20% {
        opacity: 1;
        transform: scale(1);
      }
      80% {
        opacity: 1;
        transform: scale(1);
      }
      100% {
        opacity: 0;
        transform: scale(0.9);
      }
    }

    .empty-state {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 100%;
      padding: 2rem;
      text-align: center;
      color: var(--placeholder);

      .empty-icon {
        margin-bottom: 1rem;
        opacity: 0.5;
      }

      h3 {
        margin: 0 0 0.5rem 0;
        font-size: 1.25rem;
        font-weight: 600;
      }

      p {
        margin: 0;
        font-size: 0.9rem;
      }
    }

    .log-entries-wrapper {
      position: relative;
      width: 100%;
    }

    .log-entries {
      padding: 0.25rem;
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      will-change: transform;

      .log-entry {
        display: flex;
        align-items: flex-start;
        gap: 0.5rem;
        padding: 0.25rem 0.5rem;
        margin-bottom: 0;
        border-radius: var(--border-radius);
        transition: background-color 0.2s ease;
        box-sizing: border-box;
        min-height: 28px;

        &:hover {
          background: var(--card);

          .log-copy-icon {
            opacity: 1;
          }
        }

        .log-copy-icon {
          flex-shrink: 0;
          display: flex;
          align-items: center;
          justify-content: center;
          width: 1.5rem;
          height: 1.5rem;
          color: var(--placeholder);
          cursor: pointer;
          opacity: 0.3;
          transition: all 0.2s ease;
          border-radius: calc(var(--border-radius) * 0.5);

          &:hover {
            opacity: 1;
            color: var(--primary);
          }

          &:active {
            transform: scale(0.95);
          }
        }

        .log-timestamp {
          flex-shrink: 0;
          font-size: 0.75rem;
          color: var(--placeholder);
          min-width: 3rem;
          padding-top: 0.15rem;
        }

        .log-level {
          flex-shrink: 0;
          display: flex;
          justify-content: center;
          align-items: center;
          gap: 0.25rem;
          font-size: 0.75rem;
          font-weight: 1000;
          min-width: 5.5rem;
          max-width: 5.5rem;
          border-radius: var(--border-radius-small);
          align-self: flex-start;
          margin-top: 0.05rem;
        }

        .log-message {
          flex: 1;
          font-size: 0.9rem;
          line-height: 1.3;
          word-break: break-all;
          user-select: text; // Re-enable text selection for log content

          .log-message-content {
            margin: 0;
            padding: 0;
            font-family:
              "JetBrains Mono", "Fira Code", "Consolas", "Monaco", monospace;
            font-size: inherit;
            line-height: inherit;
            color: inherit;
            background: transparent;
            border: none;
            white-space: pre-wrap;
            word-wrap: break-word;
            overflow-wrap: break-word;
          }
        }
      }
    }
  }
}

/* Full-bleed overlay that blurs the visible log area and centers a message */
.overlay-copy-notification {
  position: absolute;
  inset: 0; /* fill the log-content area */
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none; /* don't block interactions with the rest of the UI */
  z-index: 1500; /* above log content but below UI chrome */
  animation: fadeInScale 0.12s ease-out;
  /* translucent dark layer with blur to visually separate logs */
  background: rgba(0, 0, 0, 0.28);
  backdrop-filter: blur(6px) saturate(1.05);
}

.overlay-copy-content {
  pointer-events: auto; /* allow interaction only on the small content box if needed */
  display: inline-flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem 1.25rem;
  background: linear-gradient(
    180deg,
    rgba(255, 255, 255, 0.06),
    rgba(255, 255, 255, 0.03)
  );
  color: var(--text-white);
  border-radius: calc(var(--border-radius) * 0.7);
  box-shadow: 0 14px 40px rgba(0, 0, 0, 0.45);
  font-weight: 800;
  border: 1px solid rgba(255, 255, 255, 0.06);
  transform-origin: center;
  transform: translateY(0);
  white-space: nowrap;
  font-size: 1rem;
}

@keyframes fadeInScale {
  from {
    opacity: 0;
    transform: scale(0.96);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem;
  background: var(--card);
  font-size: 0.75rem;
  color: var(--placeholder);
  border-radius: var(--border-radius-small);

  .status-left,
  .status-right {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .status-text {
    font-weight: 500;
  }

  .auto-scroll-indicator {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    color: var(--green);
    font-weight: 500;
  }
}

// Badge variants
.badge {
  &.danger {
    background: color-mix(in srgb, var(--red), 10%, transparent);
    color: var(--red);
    border: 1px solid color-mix(in srgb, var(--red), 20%, transparent);
  }

  &.warning {
    background: color-mix(in srgb, var(--yellow), 10%, transparent);
    color: var(--yellow);
    border: 1px solid color-mix(in srgb, var(--yellow), 20%, transparent);
  }

  &.info {
    background: color-mix(in srgb, var(--blue), 10%, transparent);
    color: var(--blue);
    border: 1px solid color-mix(in srgb, var(--blue), 20%, transparent);
  }

  &.muted {
    background: color-mix(in srgb, var(--dark-300), 10%, transparent);
    color: var(--dark-300);
    border: 1px solid color-mix(in srgb, var(--dark-300), 20%, transparent);
  }

  &.secondary {
    background: color-mix(in srgb, var(--text), 10%, transparent);
    color: var(--text);
    border: 1px solid color-mix(in srgb, var(--text), 20%, transparent);
  }
}
</style>
