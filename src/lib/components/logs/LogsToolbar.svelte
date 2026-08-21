<script lang="ts">
import { app, Icon } from "$lib";
import type { LogEvent } from "$lib/services/logs.svelte";

let {
  logs,
  searchTerm = $bindable(""),
  searchMode = $bindable<"normal" | "regex" | "fuzzy">("fuzzy"),
  autoScroll = $bindable(true),
}: {
  logs: LogEvent[];
  searchTerm?: string;
  searchMode?: "normal" | "regex" | "fuzzy";
  autoScroll?: boolean;
} = $props();

let showLogLevelDropdown = $state(false);

let logLevelFilters = $state({
  error: true,
  warn: true,
  info: true,
  debug: true,
});

const enabledLogLevelsCount = $derived(Object.values(logLevelFilters).filter(Boolean).length);

const logCounts = $derived.by(() => {
  const counts = {
    error: 0,
    warn: 0,
    info: 0,
    debug: 0,
  };

  for (const log of logs) {
    if (log.level in counts) {
      counts[log.level]++;
    }
  }

  return counts;
});

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

function getLogLevelIcon(level: string): string {
  switch (level) {
    case "error":
      return "alert";
    case "warn":
      return "warning";
    case "info":
      return "info";
    case "debug":
      return "bug";
    default:
      return "message";
  }
}

function toggleLogLevel(level: keyof typeof logLevelFilters): void {
  logLevelFilters[level] = !logLevelFilters[level];
}

async function clearLogs(): Promise<void> {
  await app.logsService.clearLogs();
}

function handleDocumentClick(event: MouseEvent): void {
  const target = event.target as HTMLElement;

  if (!target.closest(".log-level-dropdown")) {
    showLogLevelDropdown = false;
  }
}

$effect(() => {
  document.addEventListener("click", handleDocumentClick);

  return () => {
    document.removeEventListener("click", handleDocumentClick);
  };
});
</script>

<div class="filters-section">
  <div class="search-container">
    <Icon name="search" size="sm" />

    <input
      type="text"
      bind:value={searchTerm}
      class="search-input"
      placeholder={searchMode === "regex" ? "Search with regex..." : searchMode === "fuzzy" ? 'Fuzzy search (try "frge" for "forge")...' : "Search logs..."} />

    <div class="search-mode-selector">
      <button class:active={searchMode === "normal"} class="search-mode-button" title="Normal text search" onclick={() => (searchMode = "normal")}>
        <Icon name="text" size="sm" />
      </button>

      <button class:active={searchMode === "fuzzy"} class="search-mode-button" title="Fuzzy search" onclick={() => (searchMode = "fuzzy")}>
        <Icon name="zap" size="sm" />
      </button>

      <button class:active={searchMode === "regex"} class="search-mode-button" title="Regular expression search" onclick={() => (searchMode = "regex")}>
        <Icon name="code" size="sm" />
      </button>
    </div>
  </div>

  <div class="filter-controls">
    <div class="log-level-dropdown">
      <button type="button" class="dropdown-trigger" onclick={() => (showLogLevelDropdown = !showLogLevelDropdown)}>
        <span>Log Levels ({enabledLogLevelsCount}/4)</span>
        <Icon name={showLogLevelDropdown ? "chevron-up" : "chevron-down"} size="sm" />
      </button>

      {#if showLogLevelDropdown}
        <div class="dropdown-menu">
          <div class="dropdown-header">
            <span>Select log levels to display</span>
          </div>

          {#each Object.keys(logLevelFilters) as level (level)}
            {@const typedLevel = level as keyof typeof logLevelFilters}

            <label class="dropdown-item">
              <input type="checkbox" checked={logLevelFilters[typedLevel]} onchange={() => toggleLogLevel(typedLevel)} />

              <Icon name={getLogLevelIcon(level)} size="sm" />

              <span>{getLogLevelDisplayName(level)}</span>

              <span class="log-level-count">
                ({logCounts[typedLevel]})
              </span>
            </label>
          {/each}
        </div>
      {/if}
    </div>

    <label class="checkbox-label">
      <input type="checkbox" bind:checked={autoScroll} />
      <span>Auto-scroll</span>
    </label>

    <button class="btn btn-danger btn-sm" onclick={clearLogs} title="Clear current logs">
      <Icon name="trash" size="sm" />
      Clear
    </button>
  </div>
</div>

<style lang="scss">
.filters-section {
  display: flex;
  gap: 1rem;
  align-items: center;
  padding: 1rem;
  background: $color-surface-1;
  border-radius: $radius-md;
  border: 1px solid $color-border;

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
        border-color: $color-accent;
      }

      &::placeholder {
        color: var(--placeholder);
      }
    }

    .search-mode-selector {
      display: flex;
      background: $color-surface-2;
      border-radius: $radius-sm;
      overflow: hidden;

      .search-mode-button {
        display: flex;
        align-items: center;
        justify-content: center;
        min-width: 2.5rem;
        padding: 0.5rem;
        background: transparent;
        border: none;
        border-right: 1px solid $color-border;
        color: var(--placeholder);
        cursor: pointer;
        transition: all 0.2s ease;

        &:last-child {
          border-right: none;
        }

        &:hover {
          background: $color-border;
          color: $color-text;
        }

        &.active {
          background: $color-accent;
          color: $color-text-muted;
        }

        &:focus {
          outline: none;
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
        background: $color-surface-3;
        border: 1px solid $color-border;
        border-radius: $radius-sm;
        color: $color-text;
        font-size: 0.9rem;
        font-weight: 500;
        cursor: pointer;
        white-space: nowrap;

        &:hover,
        &:focus {
          border-color: $color-accent;
        }
      }

      .dropdown-menu {
        position: absolute;
        top: calc(100% + 0.25rem);
        right: 0;
        min-width: 180px;
        background: $color-surface-1;
        border: 1px solid $color-border;
        border-radius: $radius-md;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
        z-index: 1000;
        overflow: hidden;

        .dropdown-header {
          padding: 0.75rem;
          background: $color-surface-2;
          border-bottom: 1px solid $color-border;
          color: var(--placeholder);
          font-size: 0.85rem;
          font-weight: 600;
        }

        .dropdown-item {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.5rem 0.75rem;
          font-size: 0.9rem;
          cursor: pointer;

          &:hover {
            background: $color-surface-2;
          }

          input[type="checkbox"] {
            accent-color: $color-accent;
          }

          .log-level-count {
            min-width: 2.5em;
            margin-left: auto;
            color: $color-placeholder;
            text-align: right;
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
        accent-color: $color-accent;
      }
    }
  }
}
</style>
