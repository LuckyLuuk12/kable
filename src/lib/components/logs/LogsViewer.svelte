<script lang="ts">
import { Icon } from "$lib";
import type { LogEvent } from "$lib/services/logs.svelte";
import { tick } from "svelte";
import { SvelteMap } from "svelte/reactivity";

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

const MIN_ITEM_HEIGHT = 28;
const BUFFER_SIZE = 10;

let container = $state<HTMLElement>();
let scrollTop = $state(0);
let containerHeight = $state(0);

let visibleStartIndex = $state(0);
let visibleEndIndex = $state(50);

let heights = new SvelteMap<string, number>();
let elements = new SvelteMap<string, HTMLElement>();

let copyNotification = $state(false);
let previousLogCount = 0;
let isAutoScrolling = false;
let updateVersion = 0;

function getLogKey(log: LogEvent, index: number): string {
  return `${log.timestamp}-${index}`;
}

function fuzzyMatch(needle: string, haystack: string): boolean {
  if (!needle || !haystack) {
    return false;
  }

  const search = needle.toLowerCase();
  const value = haystack.toLowerCase();

  if (value.includes(search)) {
    return true;
  }

  let searchIndex = 0;

  for (let i = 0; i < value.length && searchIndex < search.length; i++) {
    if (value[i] === search[searchIndex]) {
      searchIndex++;
    }
  }

  return searchIndex / search.length >= 0.7;
}

function matchesSearch(message: string, search: string, mode: "normal" | "regex" | "fuzzy"): boolean {
  if (!search) {
    return true;
  }

  switch (mode) {
    case "regex":
      try {
        return new RegExp(search, "i").test(message);
      } catch {
        return message.toLowerCase().includes(search.toLowerCase());
      }

    case "fuzzy":
      return fuzzyMatch(search, message);

    case "normal":
      return message.toLowerCase().includes(search.toLowerCase());
  }
}

const filteredLogs = $derived(
  logs.filter((log) => {
    if (!logLevelFilters[log.level]) {
      return false;
    }

    return matchesSearch(log.message, searchTerm, searchMode);
  }),
);

const visibleLogs = $derived(filteredLogs.slice(visibleStartIndex, visibleEndIndex));

const totalHeight = $derived.by(() => {
  let height = 0;

  for (let index = 0; index < filteredLogs.length; index++) {
    const log = filteredLogs[index];
    height += heights.get(getLogKey(log, index)) ?? MIN_ITEM_HEIGHT;
  }

  return height;
});

const offsetY = $derived.by(() => {
  let height = 0;

  for (let index = 0; index < visibleStartIndex; index++) {
    const log = filteredLogs[index];
    height += heights.get(getLogKey(log, index)) ?? MIN_ITEM_HEIGHT;
  }

  return height;
});

const hasActiveFilters = $derived(Boolean(searchTerm) || Object.values(logLevelFilters).some((enabled) => !enabled));

function updateVisibleRange(): void {
  if (!container) {
    return;
  }

  scrollTop = container.scrollTop;
  containerHeight = container.clientHeight;

  const bufferHeight = BUFFER_SIZE * MIN_ITEM_HEIGHT;

  let accumulated = 0;
  let start = 0;

  for (let index = 0; index < filteredLogs.length; index++) {
    const log = filteredLogs[index];
    const height = heights.get(getLogKey(log, index)) ?? MIN_ITEM_HEIGHT;

    if (accumulated + height > scrollTop - bufferHeight) {
      start = index;
      break;
    }

    accumulated += height;
  }

  accumulated = 0;

  let end = filteredLogs.length;

  for (let index = 0; index < filteredLogs.length; index++) {
    const log = filteredLogs[index];
    const height = heights.get(getLogKey(log, index)) ?? MIN_ITEM_HEIGHT;

    accumulated += height;

    if (accumulated > scrollTop + containerHeight + bufferHeight) {
      end = index + 1;
      break;
    }
  }

  visibleStartIndex = Math.max(0, start);
  visibleEndIndex = Math.max(visibleStartIndex, end);
}

function measureVisibleLogs(): void {
  let changed = false;
  const measured = new SvelteMap(heights);

  for (const [key, element] of elements) {
    const height = element.offsetHeight;

    if (height > 0 && measured.get(key) !== height) {
      measured.set(key, height);
      changed = true;
    }
  }

  if (changed) {
    heights = measured;
  }
}

function scrollToBottom(): void {
  if (!container) {
    return;
  }

  isAutoScrolling = true;
  autoScroll = true;

  container.scrollTop = container.scrollHeight;

  updateVisibleRange();

  window.setTimeout(() => {
    isAutoScrolling = false;
  }, 50);
}

function handleScroll(): void {
  if (!container || isAutoScrolling) {
    return;
  }

  updateVisibleRange();

  const maxScroll = Math.max(0, container.scrollHeight - container.clientHeight);

  autoScroll = scrollTop >= maxScroll - 50;
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

function registerElement(node: HTMLElement, key: string) {
  elements.set(key, node);

  return {
    destroy() {
      elements.delete(key);
    },
  };
}

function formatLogText(log: LogEvent): string {
  return `[${formatTime(log.timestamp)}] ${log.level.toUpperCase()} ${log.message}`;
}

async function copyLog(log: LogEvent): Promise<void> {
  await navigator.clipboard.writeText(formatLogText(log));
}

async function copyAllLogs(): Promise<void> {
  const text = filteredLogs.map(formatLogText).join("\n");

  if (!text) {
    return;
  }

  await navigator.clipboard.writeText(text);

  copyNotification = true;

  window.setTimeout(() => {
    copyNotification = false;
  }, 2000);
}

async function copyAllCurrentLogs(): Promise<void> {
  const text = logs.map(formatLogText).join("\n");

  if (!text) {
    return;
  }

  await navigator.clipboard.writeText(text);

  copyNotification = true;

  window.setTimeout(() => {
    copyNotification = false;
  }, 2000);
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

$effect(() => {
  window.addEventListener("keydown", handleKeyDown);

  return () => {
    window.removeEventListener("keydown", handleKeyDown);
  };
});

$effect(() => {
  const count = filteredLogs.length;

  if (!autoScroll || count <= previousLogCount) {
    previousLogCount = count;
    return;
  }

  const version = ++updateVersion;

  tick().then(async () => {
    if (version !== updateVersion || !autoScroll || filteredLogs.length !== count || !container) {
      return;
    }

    await tick();

    if (version !== updateVersion || !container) {
      return;
    }

    measureVisibleLogs();
    updateVisibleRange();
    scrollToBottom();
  });

  previousLogCount = count;
});

$effect(() => {
  filteredLogs;
  searchTerm;
  searchMode;
  logLevelFilters;

  const version = ++updateVersion;

  heights = new SvelteMap();
  elements.clear();

  visibleStartIndex = 0;
  visibleEndIndex = Math.min(50, filteredLogs.length);

  tick().then(async () => {
    if (version !== updateVersion || !container) {
      return;
    }

    updateVisibleRange();

    await tick();

    if (version !== updateVersion || !container) {
      return;
    }

    measureVisibleLogs();
    updateVisibleRange();

    if (autoScroll) {
      scrollToBottom();
    }
  });
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
      <div class="log-entries-wrapper" style={`height: ${totalHeight}px`}>
        <div class="log-entries" style={`transform: translateY(${offsetY}px)`}>
          {#each visibleLogs as log, index (getLogKey(log, visibleStartIndex + index))}
            {@const absoluteIndex = visibleStartIndex + index}
            {@const logKey = getLogKey(log, absoluteIndex)}

            <div class="log-entry" use:registerElement={logKey}>
              <button class="log-copy-icon" title="Copy log entry" onclick={() => copyLog(log)}>
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
    top: 0;
    left: 0;
    right: 0;
    padding: 0.25rem;
    will-change: transform;
  }

  .log-entry {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    min-height: 28px;
    padding: 0.25rem 0.5rem;
    box-sizing: border-box;
    border-radius: $radius-md;
    transition: background-color 0.2s ease;

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
