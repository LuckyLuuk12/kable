<script lang="ts">
import { Icon } from "$lib";
import type { LogEvent } from "$lib/services/logs.svelte";
import { createVirtualizer } from "@tanstack/svelte-virtual";
import { onDestroy, tick, untrack } from "svelte";
import { get } from "svelte/store";

let {
  logs,
  searchTerm = "",
  searchMode = "fuzzy",
  autoScroll = $bindable(true),
  logLevelFilters = $bindable({
    error: true,
    warn: true,
    info: true,
    debug: true,
  }),
}: {
  logs: LogEvent[];
  searchTerm?: string;
  searchMode?: "normal" | "regex" | "fuzzy";
  autoScroll?: boolean;
  logLevelFilters?: Record<LogEvent["level"], boolean>;
} = $props();

const ESTIMATED_ROW_HEIGHT = 28;
const OVERSCAN = 8;
const BOTTOM_THRESHOLD = 50;

let container = $state<HTMLElement>();
let copyNotification = $state(false);

let previousFilteredCount = 0;
let updateVersion = 0;
let isAutoScrolling = false;
let autoScrollTimeout: ReturnType<typeof setTimeout> | undefined;
let copyNotificationTimeout: ReturnType<typeof setTimeout> | undefined;

const logKeys = new WeakMap<LogEvent, number>();
let nextLogKey = 0;

function getLogKey(log: LogEvent): number {
  let key = logKeys.get(log);

  if (key === undefined) {
    key = nextLogKey++;
    logKeys.set(log, key);
  }

  return key;
}

function fuzzyMatch(needle: string, haystack: string): boolean {
  if (!needle || !haystack) return true;
  if (haystack.includes(needle)) return true;

  let searchIndex = 0;

  for (let i = 0; i < haystack.length && searchIndex < needle.length; i++) {
    if (haystack[i] === needle[searchIndex]) {
      searchIndex++;
    }
  }

  return searchIndex / needle.length >= 0.7;
}

function createSearchMatcher(search: string, mode: "normal" | "regex" | "fuzzy"): (message: string) => boolean {
  if (!search) return () => true;

  const needle = search.toLowerCase();

  if (mode === "regex") {
    try {
      const regex = new RegExp(search, "i");
      return (message) => {
        regex.lastIndex = 0;
        return regex.test(message);
      };
    } catch {
      return (message) => message.toLowerCase().includes(needle);
    }
  }

  if (mode === "fuzzy") {
    return (message) => fuzzyMatch(needle, message.toLowerCase());
  }

  return (message) => message.toLowerCase().includes(needle);
}

const searchMatcher = $derived(createSearchMatcher(searchTerm, searchMode));

const filteredLogs = $derived.by(() => {
  const matcher = searchMatcher;
  const filters = logLevelFilters;
  const result: LogEvent[] = [];

  for (const log of logs) {
    if (!filters[log.level]) continue;
    if (!matcher(log.message)) continue;

    result.push(log);
  }

  return result;
});

const rowVirtualizer = createVirtualizer({
  count: 0,
  getScrollElement: () => container ?? null,
  estimateSize: () => ESTIMATED_ROW_HEIGHT,
  overscan: OVERSCAN,
  getItemKey: (index) => {
    const log = filteredLogs[index];
    return log ? getLogKey(log) : index;
  },
});

const virtualItems = $derived($rowVirtualizer.getVirtualItems());
const totalHeight = $derived($rowVirtualizer.getTotalSize());

const hasActiveFilters = $derived(Boolean(searchTerm) || Object.values(logLevelFilters).some((enabled) => !enabled));

function syncVirtualizerCount(count: number): void {
  const virtualizer = get(rowVirtualizer);

  if (virtualizer.options.count === count) return;

  virtualizer.setOptions({
    ...virtualizer.options,
    count,
  });
}

function measureElement(node: HTMLElement) {
  const virtualizer = get(rowVirtualizer);

  virtualizer.measureElement(node);

  return {
    destroy() {
      // TanStack Virtual owns the measurement observer.
    },
  };
}

function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleTimeString("en-US", {
    hour12: false,
  });
}

function getLevelIcon(level: LogEvent["level"]): string {
  switch (level) {
    case "error":
      return "alert";
    case "warn":
      return "warning";
    case "info":
      return "info";
    case "debug":
      return "bug";
  }
}

function getLevelClass(level: LogEvent["level"]): string {
  switch (level) {
    case "error":
      return "danger";
    case "warn":
      return "warning";
    case "info":
      return "info";
    case "debug":
      return "muted";
  }
}

function formatLogText(log: LogEvent): string {
  return `[${formatTime(log.timestamp)}] ${log.level.toUpperCase()} ${log.message}`;
}

async function copyLog(log: LogEvent): Promise<void> {
  await navigator.clipboard.writeText(formatLogText(log));
}

function showCopyNotification(): void {
  copyNotification = true;

  if (copyNotificationTimeout) {
    clearTimeout(copyNotificationTimeout);
  }

  copyNotificationTimeout = setTimeout(() => {
    copyNotification = false;
    copyNotificationTimeout = undefined;
  }, 2000);
}

async function copyAllLogs(): Promise<void> {
  const text = filteredLogs.map(formatLogText).join("\n");

  if (!text) return;

  await navigator.clipboard.writeText(text);
  showCopyNotification();
}

async function copyAllCurrentLogs(): Promise<void> {
  const text = logs.map(formatLogText).join("\n");

  if (!text) return;

  await navigator.clipboard.writeText(text);
  showCopyNotification();
}

function scrollToBottom(): void {
  if (!container || filteredLogs.length === 0) return;

  autoScroll = true;
  isAutoScrolling = true;

  get(rowVirtualizer).scrollToIndex(filteredLogs.length - 1, {
    align: "end",
  });

  if (autoScrollTimeout) {
    clearTimeout(autoScrollTimeout);
  }

  autoScrollTimeout = setTimeout(() => {
    isAutoScrolling = false;
    autoScrollTimeout = undefined;
  }, 150);
}

function handleScroll(): void {
  if (!container || isAutoScrolling) return;

  const distanceFromBottom = container.scrollHeight - container.scrollTop - container.clientHeight;

  const shouldAutoScroll = distanceFromBottom <= BOTTOM_THRESHOLD;

  // Avoid invalidating reactive state when the value has not changed.
  if (autoScroll !== shouldAutoScroll) {
    autoScroll = shouldAutoScroll;
  }
}

function handleKeyDown(event: KeyboardEvent): void {
  if (!(event.ctrlKey || event.metaKey) || event.key.toLowerCase() !== "a") {
    return;
  }

  const target = event.target;

  if (target instanceof HTMLElement) {
    const tagName = target.tagName.toLowerCase();

    if (tagName === "input" || tagName === "textarea" || tagName === "select" || target.isContentEditable) {
      return;
    }
  }

  event.preventDefault();
  event.stopPropagation();

  void copyAllCurrentLogs();
}

// Only log-count changes update the virtualizer's item count.
// Scrolling and toggling auto-scroll cannot trigger this effect.
$effect(() => {
  const count = filteredLogs.length;

  syncVirtualizerCount(count);

  const previousCount = previousFilteredCount;
  previousFilteredCount = count;

  if (count <= previousCount || !untrack(() => autoScroll) || count === 0) {
    return;
  }

  const version = ++updateVersion;

  void tick().then(() => {
    if (version !== updateVersion || !untrack(() => autoScroll) || !container || filteredLogs.length !== count) {
      return;
    }

    scrollToBottom();
  });
});

// Only search/filter changes reset measurements and scroll position.
// Reading the filtered result count untracked prevents new log arrivals
// from repeatedly triggering this effect.
$effect(() => {
  searchTerm;
  searchMode;
  logLevelFilters.error;
  logLevelFilters.warn;
  logLevelFilters.info;
  logLevelFilters.debug;

  const count = untrack(() => filteredLogs.length);
  const version = ++updateVersion;

  previousFilteredCount = count;

  get(rowVirtualizer).measure();

  if (container) {
    container.scrollTop = 0;
  }

  void tick().then(() => {
    if (version !== updateVersion || !container) return;

    if (untrack(() => autoScroll) && filteredLogs.length > 0) {
      scrollToBottom();
    }
  });
});

$effect(() => {
  window.addEventListener("keydown", handleKeyDown);

  return () => {
    window.removeEventListener("keydown", handleKeyDown);
  };
});

onDestroy(() => {
  updateVersion++;

  if (autoScrollTimeout) clearTimeout(autoScrollTimeout);
  if (copyNotificationTimeout) clearTimeout(copyNotificationTimeout);
});
</script>

<div class="log-content">
  <div bind:this={container} class:copy-notification-active={copyNotification} class="log-container" onscroll={handleScroll}>
    {#if filteredLogs.length === 0}
      <div class="empty-state">
        <div class="empty-icon">
          <Icon name={hasActiveFilters ? "search" : "archive"} size="xl" />
        </div>

        <h3>
          {hasActiveFilters ? "No logs match your filters" : "No logs yet"}
        </h3>

        <p>
          {hasActiveFilters ? "Try adjusting your search or filter settings" : "Launch an installation to see logs here"}
        </p>
      </div>
    {:else}
      <div class="log-entries-wrapper" style={`height: ${totalHeight + 8}px`}>
        <div class="log-entries">
          {#each virtualItems as virtualItem (virtualItem.key)}
            {@const index = virtualItem.index}
            {@const log = filteredLogs[index]}

            {#if log}
              <div class="log-entry" data-index={index} use:measureElement style={`transform: translateY(${virtualItem.start}px)`}>
                <button class="log-copy-icon" title="Copy log entry" aria-label="Copy log entry" onclick={() => copyLog(log)}>
                  <Icon name="clipboard" size="sm" />
                </button>

                <div class="log-timestamp">
                  {formatTime(log.timestamp)}
                </div>

                <div class="log-level badge {getLevelClass(log.level)}">
                  <Icon name={getLevelIcon(log.level)} size="sm" />
                  {log.level.toUpperCase()}
                </div>

                <div class="log-message">
                  <pre class="log-message-content"><code>{log.message}</code></pre>
                </div>
              </div>
            {/if}
          {/each}
        </div>
      </div>
    {/if}
  </div>

  {#if copyNotification}
    <div class="overlay-copy-notification" role="status" aria-live="polite">
      <div class="overlay-copy-content">
        <Icon name="clipboard" size="md" />
        <span>
          Copied {filteredLogs.length}
          {filteredLogs.length === 1 ? "log entry" : "log entries"}
        </span>
      </div>
    </div>
  {/if}

  <div class="status-bar">
    <div class="status-left">
      <span class="status-text">
        {filteredLogs.length === logs.length ? `${logs.length} entries` : `${filteredLogs.length} / ${logs.length} entries`}
      </span>

      {#if searchTerm && searchMode !== "normal"}
        <span>• {searchMode}</span>
      {/if}

      <button class="btn btn-secondary btn-sm" onclick={copyAllLogs} disabled={filteredLogs.length === 0}>
        <Icon name="clipboard" size="sm" />
        Copy
      </button>

      {#if !autoScroll}
        <button class="btn btn-link btn-sm" onclick={scrollToBottom}>
          <Icon name="arrow-down" size="sm" />
          Jump to bottom
        </button>
      {/if}
    </div>

    <div class="status-right">
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
.log-content {
  position: relative;
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 100%;
  max-height: 100%;

  .log-container {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    position: relative;
    background: $color-surface-1;
    border: 1px solid $color-border;
    border-radius: 0 0 $radius-md $radius-md;

    &.copy-notification-active {
      .log-entry {
        background: color-mix(in srgb, $color-accent, 10%, transparent);
      }
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
    color: $color-placeholder;

    .empty-icon {
      margin-bottom: 1rem;
      opacity: 0.5;
    }

    h3 {
      margin: 0 0 0.5rem;
      font-size: 1.25rem;
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
    position: absolute;
    top: 4px;
    left: 0;
    right: 0;
    width: 100%;
  }

  .log-entry {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    box-sizing: border-box;
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    min-height: 28px;
    padding: 0.25rem 0.5rem;
    border-radius: $radius-md;

    &:hover {
      background: $color-surface-2;

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
      padding: 0;
      border: none;
      background: transparent;
      color: $color-placeholder;
      cursor: pointer;
      opacity: 0.3;
      border-radius: calc($radius-md * 0.5);

      &:hover {
        opacity: 1;
        color: $color-accent;
      }
    }

    .log-timestamp {
      flex-shrink: 0;
      min-width: 3rem;
      padding-top: 0.15rem;
      color: $color-placeholder;
      font-size: 0.75rem;
    }

    .log-level {
      flex-shrink: 0;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.25rem;
      min-width: 5.5rem;
      max-width: 5.5rem;
      margin-top: 0.05rem;
      font-size: 0.75rem;
      font-weight: 1000;
      border-radius: $radius-sm;
    }

    .log-message {
      flex: 1;
      min-width: 0;
      font-size: 0.9rem;
      line-height: 1.3;
      user-select: text;
      word-break: break-all;

      .log-message-content {
        margin: 0;
        padding: 0;
        font-family: "JetBrains Mono", "Fira Code", "Consolas", "Monaco", monospace;
        font-size: inherit;
        line-height: inherit;
        color: inherit;
        background: transparent;
        white-space: pre-wrap;
        overflow-wrap: break-word;
      }
    }
  }
}

.overlay-copy-notification {
  position: absolute;
  inset: 0;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
  background: rgba(0, 0, 0, 0.28);
  backdrop-filter: blur(6px);
}

.overlay-copy-content {
  display: inline-flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem 1.25rem;
  background: $color-surface-2;
  color: $color-text;
  border: 1px solid $color-border;
  border-radius: calc($radius-md * 0.7);
  box-shadow: 0 14px 40px rgba(0, 0, 0, 0.45);
  font-weight: 800;
}

.status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem;
  background: $color-surface-2;
  border-radius: $radius-sm;
  color: $color-placeholder;
  font-size: 0.75rem;

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

.badge {
  &.danger {
    background: color-mix(in srgb, $color-error, 10%, transparent);
    color: $color-error;
    border: 1px solid color-mix(in srgb, $color-error, 20%, transparent);
  }

  &.warning {
    background: color-mix(in srgb, $color-warning, 10%, transparent);
    color: $color-warning;
    border: 1px solid color-mix(in srgb, $color-warning, 20%, transparent);
  }

  &.info {
    background: color-mix(in srgb, $color-info, 10%, transparent);
    color: $color-info;
    border: 1px solid color-mix(in srgb, $color-info, 20%, transparent);
  }

  &.muted {
    background: color-mix(in srgb, $color-text-muted, 10%, transparent);
    color: $color-text-muted;
    border: 1px solid color-mix(in srgb, $color-text-muted, 20%, transparent);
  }
}
</style>
