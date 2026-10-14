<script lang="ts">
import { app, Icon, type CacheEntryInfo } from "$lib";
import { onMount } from "svelte";

let searchQuery = $state("");
let copiedPath = $state<string | null>(null);
let copyTimeout: ReturnType<typeof setTimeout> | null = null;
let expandedNamespaces = $state<Set<string>>(new Set());

const entries = $derived(app.cacheService.entries);

// Calculate directly from reactive entries instead of relying on a service getter.
const totalSizeBytes = $derived(
  entries.reduce((total, entry) => {
    const size = Number(entry.size_bytes);
    return total + (Number.isFinite(size) && size > 0 ? size : 0);
  }, 0),
);

const namespaces = $derived.by(() => {
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  const stats = new Map<string, { count: number; sizeBytes: number }>();

  for (const entry of entries) {
    const current = stats.get(entry.namespace) ?? {
      count: 0,
      sizeBytes: 0,
    };

    const size = Number(entry.size_bytes);

    current.count++;
    current.sizeBytes += Number.isFinite(size) && size > 0 ? size : 0;

    stats.set(entry.namespace, current);
  }

  return [...stats.entries()]
    .map(([name, { count, sizeBytes }]) => ({
      name,
      count,
      sizeBytes,
    }))
    .sort((a, b) => a.name.localeCompare(b.name));
});

const filteredEntries = $derived.by(() => {
  const query = searchQuery.trim().toLowerCase();

  if (!query) return entries;

  return entries.filter((entry) => entry.path.toLowerCase().includes(query) || entry.namespace.toLowerCase().includes(query));
});

onMount(() => {
  void app.cacheService.loadEntries().catch((error) => {
    console.error("Failed to load cache entries:", error);
  });

  return () => {
    if (copyTimeout) {
      clearTimeout(copyTimeout);
      copyTimeout = null;
    }
  };
});

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
  if (bytes < 1024) return `${Math.round(bytes)} B`;

  const units = ["KB", "MB", "GB", "TB", "PB"];
  let size = bytes / 1024;
  let unit = 0;

  while (size >= 1024 && unit < units.length - 1) {
    size /= 1024;
    unit++;
  }

  return `${size.toFixed(2)} ${units[unit]}`;
}

function toggleNamespace(namespace: string): void {
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  const next = new Set(expandedNamespaces);

  if (next.has(namespace)) {
    next.delete(namespace);
  } else {
    next.add(namespace);
  }

  expandedNamespaces = next;
}

function isNamespaceExpanded(namespace: string): boolean {
  return expandedNamespaces.has(namespace);
}

async function copyPath(path: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(path);
    copiedPath = path;

    if (copyTimeout) clearTimeout(copyTimeout);

    copyTimeout = setTimeout(() => {
      copiedPath = null;
      copyTimeout = null;
    }, 2000);
  } catch (error) {
    console.error("Failed to copy cache path:", error);
  }
}

async function refresh(): Promise<void> {
  try {
    await app.cacheService.refreshEntries();
  } catch (error) {
    console.error("Failed to refresh cache entries:", error);
  }
}

async function invalidateEntry(entry: CacheEntryInfo): Promise<void> {
  if (app.cacheService.invalidating) return;

  const confirmed = window.confirm(
    `Invalidate this cache entry?\n\nPath: ${entry.path}\nSize: ${formatBytes(Number(entry.size_bytes))}\n\nThe cached value will be recomputed when needed.`,
  );

  if (!confirmed) return;

  try {
    await app.cacheService.invalidateEntry(entry.path);
  } catch (error) {
    console.error("Failed to invalidate cache entry:", error);
  }
}

async function invalidateNamespace(namespace: string): Promise<void> {
  if (app.cacheService.invalidating) return;

  // The backend rejects an empty namespace. Do not offer an invalid operation
  // for entries stored directly in the cache root.
  if (!namespace) return;

  const count = entries.filter((entry) => entry.namespace === namespace || entry.namespace.startsWith(`${namespace}/`)).length;

  const confirmed = window.confirm(
    `Invalidate this cache namespace?\n\nNamespace: ${namespace}\nEntries affected: ${count}\n\nCached values will be recomputed when needed.`,
  );

  if (!confirmed) return;

  try {
    await app.cacheService.invalidateNamespace(namespace);
  } catch (error) {
    console.error("Failed to invalidate cache namespace:", error);
  }
}

async function invalidateMemory(): Promise<void> {
  if (app.cacheService.invalidating) return;

  const confirmed = window.confirm("Clear the in-memory cache?\n\nPersistent cache files will remain untouched. Values may be loaded from disk again.");

  if (!confirmed) return;

  try {
    await app.cacheService.invalidateMemory();
  } catch (error) {
    console.error("Failed to clear memory cache:", error);
  }
}

async function invalidateAll(): Promise<void> {
  if (app.cacheService.invalidating) return;

  const confirmed = window.confirm(
    `Clear the entire cache?\n\nPersistent entries: ${entries.length}\nPersistent size: ${formatBytes(totalSizeBytes)}\n\nAll persistent cache files and in-memory entries will be removed. Cached values will be recomputed when needed.`,
  );

  if (!confirmed) return;

  try {
    await app.cacheService.invalidateAll();
    searchQuery = "";
    expandedNamespaces = new Set();
  } catch (error) {
    console.error("Failed to clear entire cache:", error);
  }
}

function namespaceEntries(namespace: string): CacheEntryInfo[] {
  return filteredEntries.filter((entry) => entry.namespace === namespace);
}
</script>

<section class="cache-manager">
  <div class="overview">
    <div class="stat">
      <Icon name="database" forceType="svg" size="sm" />
      <span class="stat-label">Persistent entries</span>
      <strong>{entries.length.toLocaleString()}</strong>
    </div>

    <span class="stat-divider"></span>

    <div class="stat">
      <Icon name="file" forceType="svg" size="sm" />
      <span class="stat-label">Persistent size</span>
      <strong>{formatBytes(totalSizeBytes)}</strong>
    </div>

    <span class="stat-divider"></span>

    <div class="stat">
      <Icon name="folder" forceType="svg" size="sm" />
      <span class="stat-label">Namespaces</span>
      <strong>{namespaces.length.toLocaleString()}</strong>
    </div>

    <span class="overview-spacer"></span>
    <div class="header-actions">
      {#if app.cacheService.loading || app.cacheService.invalidating}
        <span class="busy-indicator" role="status">
          <Icon name="load" forceType="svg" size="sm" />
          {app.cacheService.loading ? "Loading…" : "Updating…"}
        </span>
      {/if}

      <button
        class="action-btn"
        type="button"
        onclick={refresh}
        disabled={app.cacheService.loading || app.cacheService.invalidating}
        title="Refresh cache listing"
        aria-label="Refresh cache listing">
        <Icon name="refresh" forceType="svg" size="sm" />
      </button>

      <button class="danger-btn" type="button" onclick={invalidateAll} disabled={app.cacheService.loading || app.cacheService.invalidating || entries.length === 0}>
        <Icon name="delete" forceType="svg" size="sm" />
        Clear all
      </button>
    </div>
    <button
      class="secondary-btn"
      type="button"
      onclick={invalidateMemory}
      disabled={app.cacheService.loading || app.cacheService.invalidating}
      title="Clear in-memory entries without deleting persistent files">
      <Icon name="trash" forceType="svg" size="sm" />
      Clear memory
    </button>
  </div>

  <div class="list-toolbar">
    <div class="search-field">
      <Icon name="search" forceType="svg" size="sm" />

      <input type="search" placeholder="Search paths or namespaces…" bind:value={searchQuery} aria-label="Search cache entries" />

      {#if searchQuery}
        <button class="clear-search" type="button" onclick={() => (searchQuery = "")} aria-label="Clear search">
          <Icon name="close" forceType="svg" size="sm" />
        </button>
      {/if}
    </div>

    <span class="result-count">
      {filteredEntries.length.toLocaleString()} of {entries.length.toLocaleString()} entries
    </span>
  </div>

  {#if app.cacheService.loading && entries.length === 0}
    <div class="state-panel">
      <Icon name="load" forceType="svg" />
      <p>Loading cache entries…</p>
    </div>
  {:else if entries.length === 0}
    <div class="state-panel">
      <Icon name="database" size="xl" />
      <h2>Cache is empty</h2>
      <p>Persistent cache entries will appear here when generated.</p>
    </div>
  {:else if filteredEntries.length === 0}
    <div class="state-panel">
      <Icon name="search" size="xl" />
      <h2>No matching entries</h2>
      <p>Try a different path or namespace.</p>
    </div>
  {:else}
    <div class="namespace-list">
      {#each namespaces as namespace (namespace.name)}
        {@const visibleEntries = namespaceEntries(namespace.name)}
        {@const expanded = isNamespaceExpanded(namespace.name)}

        {#if visibleEntries.length > 0}
          <section class="namespace-card">
            <header class="namespace-header">
              <button class="namespace-toggle" type="button" onclick={() => toggleNamespace(namespace.name)} aria-expanded={expanded}>
                <Icon name={expanded ? "chevron-down" : "chevron-right"} forceType="svg" size="sm" />
                <Icon name="folder" forceType="svg" size="sm" />

                <span class="namespace-name">{namespace.name || "(root)"}</span>

                <span class="entry-count">
                  {visibleEntries.length}
                  {visibleEntries.length === 1 ? "entry" : "entries"}
                </span>
              </button>

              {#if namespace.name}
                <span class="namespace-size" title="Total persistent size of this namespace">
                  {formatBytes(namespace.sizeBytes)}
                </span>

                <button
                  class="icon-btn danger"
                  type="button"
                  onclick={() => invalidateNamespace(namespace.name)}
                  disabled={app.cacheService.loading || app.cacheService.invalidating}
                  title={`Invalidate namespace ${namespace.name}`}
                  aria-label={`Invalidate namespace ${namespace.name}`}>
                  <Icon name="delete" forceType="svg" size="sm" />
                </button>
              {/if}
            </header>

            {#if expanded}
              <div class="entry-list">
                {#each visibleEntries as entry (entry.path)}
                  <div class="entry-row">
                    <button class="path-btn" type="button" onclick={() => copyPath(entry.path)} title={`Copy path: ${entry.path}`}>
                      <span class="entry-path">{entry.path}</span>
                      <Icon name={copiedPath === entry.path ? "check" : "copy"} forceType="svg" size="sm" />
                    </button>

                    <span class="entry-size">
                      {formatBytes(Number(entry.size_bytes))}
                    </span>

                    <button
                      class="icon-btn danger"
                      type="button"
                      onclick={() => invalidateEntry(entry)}
                      disabled={app.cacheService.loading || app.cacheService.invalidating}
                      title="Invalidate entry"
                      aria-label={`Invalidate ${entry.path}`}>
                      <Icon name="delete" forceType="svg" size="sm" />
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </section>
        {/if}
      {/each}
    </div>
  {/if}

  <p class="footer-hint">Cache entries are regenerated automatically when required. Clearing memory does not remove persistent files.</p>
</section>

{#if copiedPath}
  <div class="copy-notification" role="status">
    <Icon name="check" forceType="svg" size="sm" />
    Cache path copied
  </div>
{/if}

<style lang="scss">
.cache-manager {
  display: flex;
  flex-direction: column;
  gap: $space-md;
  min-width: 0;
  color: $color-text;
}

.header-actions {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  gap: $space-sm;
}

.busy-indicator {
  display: inline-flex;
  align-items: center;
  gap: $space-xs;
  color: $color-text-muted;
  font-size: 0.75rem;
}

.overview {
  display: flex;
  align-items: center;
  gap: $space-lg;
  min-width: 0;
  padding: $space-sm $space-md;
  border: 1px solid $color-border-muted;
  border-radius: $radius-md;
  background: $color-surface-1;
}

.namespace-size {
  flex-shrink: 0;
  color: $color-text-muted;
  font-size: 0.73rem;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.stat {
  display: flex;
  align-items: center;
  gap: $space-sm;
  min-width: 0;
  color: $color-text-muted;
  font-size: 0.78rem;

  strong {
    color: $color-text;
    font-size: 0.85rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
}

.stat-label {
  white-space: nowrap;
}

.stat-divider {
  width: 1px;
  align-self: stretch;
  background: $color-border-muted;
}

.overview-spacer {
  flex: 1;
}

.secondary-btn,
.danger-btn,
.action-btn,
.icon-btn,
.namespace-toggle,
.path-btn,
.clear-search {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: $space-sm;
  border: 1px solid transparent;
  border-radius: $radius-md;
  font: inherit;
  cursor: pointer;
  transition:
    background 120ms ease,
    color 120ms ease,
    border-color 120ms ease;

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }

  &:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
}

.secondary-btn,
.danger-btn {
  padding: $space-sm $space-md;
  font-size: 0.78rem;
  font-weight: 600;
  white-space: nowrap;
}

.secondary-btn {
  border-color: $color-border;
  background: $color-surface-2;
  color: $color-text;

  &:hover:not(:disabled) {
    background: $color-hover;
  }
}

.danger-btn {
  background: $color-error;
  color: white;

  &:hover:not(:disabled) {
    filter: brightness(1.08);
  }
}

.action-btn,
.icon-btn {
  flex-shrink: 0;
  padding: $space-sm;
  background: transparent;
  color: $color-text-muted;

  &:hover:not(:disabled) {
    border-color: $color-border;
    background: $color-hover;
    color: $color-text;
  }
}

.icon-btn.danger {
  color: $color-error;
}

.list-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-md;
}

.search-field {
  display: flex;
  align-items: center;
  gap: $space-sm;
  width: min(100%, 460px);
  padding: $space-sm $space-md;
  border: 1px solid $color-border-muted;
  border-radius: $radius-md;
  background: $color-surface-1;
  color: $color-text-muted;

  &:focus-within {
    border-color: $color-focus;
  }

  input {
    flex: 1;
    min-width: 0;
    padding: 0;
    border: 0;
    outline: 0;
    background: transparent;
    color: $color-text;
    font: inherit;
    font-size: 0.82rem;

    &::placeholder {
      color: $color-text-muted;
    }
  }
}

.clear-search {
  padding: $space-xs;
  background: transparent;
  color: $color-text-muted;
}

.result-count {
  flex-shrink: 0;
  color: $color-text-muted;
  font-size: 0.75rem;
  font-variant-numeric: tabular-nums;
}

.namespace-list {
  display: flex;
  flex-direction: column;
  gap: $space-sm;
}

.namespace-card {
  min-width: 0;
  overflow: hidden;
  border: 1px solid $color-border-muted;
  border-radius: $radius-md;
  background: $color-surface-1;
}

.namespace-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-sm;
  padding: $space-xs $space-sm;
  background: $color-surface-2;
}

.namespace-toggle {
  flex: 1;
  justify-content: flex-start;
  min-width: 0;
  padding: $space-xs;
  background: transparent;
  color: $color-text;
  text-align: left;

  &:hover {
    color: $color-accent;
  }
}

.namespace-name {
  overflow: hidden;
  font-size: 0.82rem;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.entry-count {
  flex-shrink: 0;
  padding: 2px 7px;
  border-radius: $radius-sm;
  background: $color-surface-1;
  color: $color-text-muted;
  font-size: 0.68rem;
}

.entry-list {
  display: flex;
  flex-direction: column;
}

.entry-row {
  display: flex;
  align-items: center;
  gap: $space-md;
  min-width: 0;
  padding: $space-xs $space-md;
  border-top: 1px solid $color-border-muted;

  &:hover {
    background: $color-hover;
  }
}

.path-btn {
  flex: 1;
  justify-content: flex-start;
  min-width: 0;
  gap: $space-sm;
  padding: $space-xs;
  background: transparent;
  color: $color-text;
  font-family: monospace;
  font-size: 0.76rem;
  text-align: left;
  cursor: copy;

  &:hover {
    color: $color-accent;
  }
}

.entry-path {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.entry-size {
  flex-shrink: 0;
  color: $color-text-muted;
  font-size: 0.73rem;
  font-variant-numeric: tabular-nums;
  text-align: right;
  white-space: nowrap;
}

.state-panel {
  display: flex;
  min-height: 180px;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: $space-sm;
  padding: $space-lg;
  border: 1px solid $color-border-muted;
  border-radius: $radius-md;
  background: $color-surface-1;
  color: $color-text-muted;
  text-align: center;

  p,
  h2 {
    margin: 0;
  }

  h2 {
    color: $color-text;
    font-size: 0.95rem;
  }
}

.footer-hint {
  margin: 0;
  color: $color-text-muted;
  font-size: 0.72rem;
}

.copy-notification {
  position: fixed;
  right: $space-xl;
  bottom: $space-xl;
  z-index: 1001;
  display: flex;
  align-items: center;
  gap: $space-sm;
  padding: $space-sm $space-md;
  border: 1px solid color-mix(in srgb, $color-success, transparent 65%);
  border-radius: $radius-md;
  background: $color-surface-2;
  color: $color-success;
  box-shadow: 0 4px 16px rgb(0 0 0 / 25%);
  font-size: 0.82rem;
}

@media (max-width: 900px) {
  .overview {
    flex-wrap: wrap;
    gap: $space-sm $space-md;
  }

  .overview-spacer {
    display: none;
  }

  .overview .secondary-btn {
    margin-left: auto;
  }
}

@media (max-width: 600px) {
  .header-actions {
    justify-content: flex-end;
  }

  .overview {
    align-items: flex-start;
  }

  .stat {
    flex-wrap: wrap;
    gap: $space-xs;
  }

  .stat-divider {
    display: none;
  }

  .overview .secondary-btn {
    margin-left: 0;
  }

  .list-toolbar {
    align-items: stretch;
    flex-direction: column;
  }

  .search-field {
    box-sizing: border-box;
    width: 100%;
  }

  .entry-row {
    gap: $space-sm;
    padding-right: $space-sm;
    padding-left: $space-sm;
  }

  .entry-size {
    font-size: 0.68rem;
  }
}
</style>
